use super::*;
use lila_ir::{
    RegExpProgram, RegExpProgramWord, ValidatedRegExpProgram, REGEXP_INSTRUCTION_WIDTH,
    REGEXP_OPCODE_NAMED_BACKREFERENCE, REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
    REGEXP_OPCODE_UNICODE_PROPERTY, REGEXP_PROGRAM_HEADER_SIZE,
};

/// Runs `bytes` (a patched copy of `artifact.bytes`) against the runtime that
/// `artifact` links to.
fn run_bytes(engine: &Engine, artifact: &Artifact, bytes: &[u8]) -> RunOutcome {
    run_on_sized_stack(|| {
        engine.run_with_wasm_bytes_inner(
            WasmProgramRef::new(bytes, artifact.runtime.as_ref().map(|runtime| &runtime.0)),
            Some(10_000),
            false,
            WasmModuleMemoryCachePolicy::BypassRetention,
        )
    })
    .expect("descriptor fixture must complete through Wasm AOT")
}

/// Rewrite the actual GC byte-array initializer, after normal producer
/// admission. A matching Data segment is not executable program ownership.
/// Preserve every other operator and rebuild body/section lengths because
/// signed i32 immediates do not necessarily keep the same encoded width.
fn replace_gc_program(bytes: &[u8], original: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(original.len(), replacement.len(), "exact array footprint");
    fn uleb(output: &mut Vec<u8>, mut value: u32) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            output.push(byte | if value == 0 { 0 } else { 0x80 });
            if value == 0 {
                break;
            }
        }
    }
    fn read_uleb(bytes: &[u8], cursor: &mut usize) -> u32 {
        let mut result = 0;
        for shift in (0..35).step_by(7) {
            let byte = bytes[*cursor];
            *cursor += 1;
            result |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return result;
            }
        }
        panic!("test artifact length");
    }
    let mut constants = Vec::new();
    for &byte in replacement {
        constants.push(0x41); // i32.const
                              // Byte values are positive i32 values: preserve the signed-LEB sign bit.
        if byte < 64 {
            constants.push(byte);
        } else {
            constants.extend_from_slice(&[(byte & 0x7f) | 0x80, byte >> 7]);
        }
    }
    let mut byte_array_types = std::collections::BTreeSet::new();
    let mut type_index = 0;
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let WasmPayload::TypeSection(groups) = payload.unwrap() {
            for group in groups {
                for ty in group.unwrap().into_types() {
                    if let wasmparser::CompositeInnerType::Array(array) = ty.composite_type.inner {
                        if array.0.element_type == wasmparser::StorageType::I8 {
                            byte_array_types.insert(type_index);
                        }
                    }
                    type_index += 1;
                }
            }
        }
    }
    let mut bodies = std::collections::BTreeMap::new();
    let mut replacements = 0;
    for payload in WasmParser::new(0).parse_all(bytes) {
        let WasmPayload::CodeSectionEntry(body) = payload.unwrap() else {
            continue;
        };
        let mut reader = body.get_operators_reader().unwrap();
        let mut run = Vec::new();
        let mut spans = Vec::new();
        while !reader.eof() {
            let start = reader.original_position();
            match reader.read().unwrap() {
                wasmparser::Operator::I32Const { value } if (0..=255).contains(&value) => {
                    run.push((start, value as u8));
                }
                wasmparser::Operator::ArrayNewFixed {
                    array_type_index,
                    array_size,
                } => {
                    if byte_array_types.contains(&array_type_index)
                        && array_size as usize == original.len()
                        && run.len() >= original.len()
                    {
                        let tail = &run[run.len() - original.len()..];
                        if tail
                            .iter()
                            .map(|(_, byte)| *byte)
                            .eq(original.iter().copied())
                        {
                            spans.push(tail[0].0..start);
                        }
                    }
                    run.clear();
                }
                _ => run.clear(),
            }
        }
        if spans.is_empty() {
            continue;
        }
        let range = body.range();
        let mut changed = Vec::new();
        let mut start = range.start;
        for span in spans {
            changed.extend_from_slice(&bytes[start..span.start]);
            changed.extend_from_slice(&constants);
            start = span.end;
            replacements += 1;
        }
        changed.extend_from_slice(&bytes[start..range.end]);
        assert!(bodies.insert(range.start, changed).is_none());
    }
    assert!(
        replacements > 0,
        "descriptor must reach a physical GC initializer"
    );
    let mut rewritten = bytes[..8].to_vec();
    let mut cursor = 8;
    while cursor < bytes.len() {
        let section_start = cursor;
        let id = bytes[cursor];
        cursor += 1;
        let length = read_uleb(bytes, &mut cursor) as usize;
        let end = cursor + length;
        if id != 10 {
            rewritten.extend_from_slice(&bytes[section_start..end]);
            cursor = end;
            continue;
        }
        let count = read_uleb(bytes, &mut cursor);
        let mut code = Vec::new();
        uleb(&mut code, count);
        for _ in 0..count {
            let length_start = cursor;
            let length = read_uleb(bytes, &mut cursor) as usize;
            let body_end = cursor + length;
            if let Some(body) = bodies.remove(&cursor) {
                uleb(&mut code, body.len().try_into().unwrap());
                code.extend_from_slice(&body);
            } else {
                code.extend_from_slice(&bytes[length_start..body_end]);
            }
            cursor = body_end;
        }
        assert_eq!(cursor, end);
        rewritten.push(id);
        uleb(&mut rewritten, code.len().try_into().unwrap());
        rewritten.extend_from_slice(&code);
    }
    assert!(bodies.is_empty(), "every matched body was rewritten");
    rewritten
}

#[test]
fn corrupt_program_sections_throw_without_reading_neighboring_data_or_changing_last_index() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = engine.compile_script(r#"
var expression = /(?<x>[Ā-Ă])+/dg;
expression.lastIndex = 0;
var caught = false;
try { expression.exec('Ā'); }
catch (error) { caught = error instanceof Error && error.message === 'RegExp compiled program matcher failed'; }
caught && expression.lastIndex === 0;
"#, CompileOptions::default()).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    let program = RegExpProgram::compile(r"(?<x>[Ā-Ă])+", "dg").unwrap();
    let descriptor = ValidatedRegExpProgram::from_program(&program).unwrap();
    let names = descriptor.word(RegExpProgramWord::NamedGroupTableOffset) as usize;
    let class_pc = program
        .instructions
        .iter()
        .position(|instruction| instruction.opcode == REGEXP_OPCODE_UNICODE_PROPERTY)
        .unwrap();
    let corruptions = [
        (
            "allocation length",
            RegExpProgramWord::ByteLength.offset() as usize,
            descriptor.bytes().len() as u64 + 8,
        ),
        (
            "range into name section",
            REGEXP_PROGRAM_HEADER_SIZE + class_pc * REGEXP_INSTRUCTION_WIDTH + 8,
            descriptor.word(RegExpProgramWord::RangeCount),
        ),
        ("name into table header", names + 32, 1),
        (
            "candidate into name bytes",
            names + 40,
            descriptor.bytes().len() as u64,
        ),
    ];
    for (name, offset, value) in corruptions {
        let mut corrupted = descriptor.bytes().to_vec();
        corrupted[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        let bytes = replace_gc_program(&artifact.bytes, descriptor.bytes(), &corrupted);
        let outcome = run_bytes(&engine, &artifact, &bytes);
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            outcome.note.contains("boolean(true)"),
            "{name}: {}",
            outcome.note
        );
    }
}

#[test]
fn immutable_program_owners_survive_cloning_recompile_and_matcher_scratch_rewinds() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            r#"
var original = /(?<word>[Ā-Ă]+)/du;
var clone = new RegExp(original);
var before = clone.exec('xĀĂ');
var changed = /old/;
changed.compile(original);
var after = changed.exec('Ă');
var replacement = 'ĀĂ'.replace(original, '<$<word>>');
var matches = 'ĀĂ Ă'.matchAll(new RegExp(original, 'dgu'));
var first = matches.next();
var second = matches.next();
var end = matches.next();
var split = 'a,b'.split(/(?<separator>,)/);
var rejected = false;
original.lastIndex = 3;
try { original.compile('['); } catch (error) { rejected = error instanceof SyntaxError; }
var independent = clone.exec('Ā');
var omitted = /(?<outer>a(?<inner>b)){0}/d.exec('');
before.groups.word === 'ĀĂ' && before.index === 1 && before.indices.groups.word[1] === 3 &&
after.groups.word === 'Ă' && replacement === '<ĀĂ>' && !first.done && !second.done && end.done &&
first.value.groups.word === 'ĀĂ' && second.value.groups.word === 'Ă' && second.value.index === 3 &&
split.length === 3 && split[1] === ',' && rejected && original.lastIndex === 3 &&
independent.groups.word === 'Ā' && changed !== clone && clone !== original &&
omitted.length === 3 && omitted[0] === '' && omitted[1] === undefined && omitted[2] === undefined &&
omitted.groups.outer === undefined && omitted.groups.inner === undefined &&
omitted.indices.groups.outer === undefined && omitted.indices.groups.inner === undefined;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn malformed_backreference_operands_are_rejected_even_for_empty_captures() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    for (pattern, operands) in [
        (r"()\1", &[4, 1_u64 << 63][..]),
        (r"(?<a>)\k<a>", &[1, 3, 4][..]),
    ] {
        let source = format!(
            "let expression = /{pattern}/i; expression.lastIndex = 3; let rejected = false; try {{ expression.test(''); }} catch (error) {{ rejected = error.message === 'RegExp compiled program matcher failed'; }} rejected && expression.lastIndex === 3;"
        );
        let unit = engine
            .compile_script(&source, CompileOptions::default())
            .unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        let program = RegExpProgram::compile(pattern, "i").unwrap();
        let descriptor = ValidatedRegExpProgram::from_program(&program).unwrap();
        let pc = program
            .instructions
            .iter()
            .position(|instruction| {
                matches!(
                    instruction.opcode,
                    REGEXP_OPCODE_NAMED_BACKREFERENCE | REGEXP_OPCODE_NUMBERED_BACKREFERENCE
                )
            })
            .unwrap();
        let offset = REGEXP_PROGRAM_HEADER_SIZE + pc * REGEXP_INSTRUCTION_WIDTH + 16;
        for operand in operands {
            // Corrupt the encoded allocation after the producer's validation;
            // mutating compiler IR would only test its construction boundary.
            let mut corrupted = descriptor.bytes().to_vec();
            corrupted[offset..offset + 8].copy_from_slice(&operand.to_le_bytes());
            let bytes = replace_gc_program(&artifact.bytes, descriptor.bytes(), &corrupted);
            let outcome = run_bytes(&engine, &artifact, &bytes);
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            assert!(
                outcome.note.contains("boolean(true)"),
                "/{pattern}/ operand {operand}: {}",
                outcome.note
            );
        }
    }
}

#[test]
fn admitted_capture_only_body_preserves_a_later_replay_failure_and_last_index() {
    use lila_ir::{
        RegExpInstruction, RegExpNatural, RegExpOpcode, RegExpRepeatBounds, RegExpRepeatMaximum,
    };

    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = engine.compile_script(r#"
var expression = /(){2}/dg;
expression.lastIndex = 3;
var result = 0;
try {
    var match = expression.exec('abc');
    if (match !== null && match.index === 3 && match[0] === '' &&
        match[1] === undefined && match.indices[1] === undefined) result = 1;
} catch (error) {
    if (error instanceof Error && error.message === 'RegExp compiled program matcher failed') result = 2;
}

expression.lastIndex === 3 ? result : 0;
"#, CompileOptions::default()).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    let program = RegExpProgram::compile("(){2}", "dg").unwrap();
    let original = ValidatedRegExpProgram::from_program(&program).unwrap();

    let mut replacement = program.clone();
    replacement.instructions = vec![
        RegExpInstruction::capture_start(1),
        RegExpInstruction::repeat_begin(0),
        RegExpInstruction {
            opcode: RegExpOpcode::RepeatGuard as u64,
            operand0: 5,
            operand1: 0,
        },
        RegExpInstruction::capture_end(1),
        RegExpInstruction::clear_capture_range(1, 2),
        RegExpInstruction::repeat_end(1),
        RegExpInstruction::repeat_exit(1),
        RegExpInstruction::accept(),
    ];
    for iterations in [1_u64, 2] {
        replacement.repeat_bounds = vec![RegExpRepeatBounds::new(
            RegExpNatural::from_u64(iterations),
            RegExpRepeatMaximum::Finite(RegExpNatural::from_u64(iterations)),
        )
        .unwrap()];
        let descriptor = ValidatedRegExpProgram::from_program(&replacement)
            .expect("the paired, dense, non-consuming graph is structurally admitted");
        assert_eq!(
            ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
            descriptor
        );
        assert_eq!(
            descriptor.bytes().len(),
            original.bytes().len(),
            "preserve the exact allocation footprint"
        );
        let bytes = replace_gc_program(&artifact.bytes, original.bytes(), descriptor.bytes());
        let outcome = run_bytes(&engine, &artifact, &bytes);
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        // One iteration proves the real native byte gate accepts the graph.
        // Two iterations must reach ordinary replay: End then Clear leaves its
        // next End without a Start, so CorruptProgram preserves lastIndex.
        let expected = format!("number({iterations})");
        assert!(
            outcome.note.contains(&expected),
            "{iterations}: {}",
            outcome.note
        );
    }
}

#[test]
fn admitted_greedy_shape_cannot_hide_capture_writes_from_required_replay() {
    use lila_ir::{
        RegExpInstruction, RegExpNatural, RegExpOpcode, RegExpRepeatBounds, RegExpRepeatMaximum,
    };
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let pattern = "(?:()|()){2}";
    let source = r#"
var expression = /(?:()|()){2}/dg;
expression.lastIndex = 3;
var result = 0;
try {
    var match = expression.exec('abc');
    if (match !== null && match.index === 3 && match[0] === '' &&
        match[1] === undefined && match[2] === undefined &&
        match.indices[1] === undefined && match.indices[2] === undefined) result = 1;
} catch (error) {
    if (error instanceof Error && error.message === 'RegExp compiled program matcher failed') result = 2;
}
expression.lastIndex === 3 ? result : 0;
"#;
    let unit = engine
        .compile_script(source, CompileOptions::default())
        .unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    let program = RegExpProgram::compile(pattern, "dg").unwrap();
    let original = ValidatedRegExpProgram::from_program(&program).unwrap();
    let mut replacement = program.clone();
    replacement.instructions = vec![
        RegExpInstruction::capture_start(1),
        RegExpInstruction::repeat_begin(0),
        RegExpInstruction {
            opcode: RegExpOpcode::RepeatGuard as u64,
            operand0: 6,
            operand1: 0,
        },
        RegExpInstruction::split(4, 6),
        RegExpInstruction::capture_end(1),
        RegExpInstruction::clear_capture_range(1, 2),
        RegExpInstruction::repeat_end(1),
        RegExpInstruction::repeat_exit(1),
    ];
    assert!(program.instructions.len() > replacement.instructions.len());
    while replacement.instructions.len() + 1 < program.instructions.len() {
        replacement
            .instructions
            .push(RegExpInstruction::clear_capture_range(1, 3));
    }
    replacement.instructions.push(RegExpInstruction::accept());
    for iterations in [1_u64, 2] {
        replacement.repeat_bounds = vec![RegExpRepeatBounds::new(
            RegExpNatural::from_u64(iterations),
            RegExpRepeatMaximum::Finite(RegExpNatural::from_u64(iterations)),
        )
        .unwrap()];
        let descriptor = ValidatedRegExpProgram::from_program(&replacement).expect(
            "the dense paired greedy shape is byte-admitted even though its atom writes captures",
        );
        assert_eq!(
            ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
            descriptor
        );
        assert_eq!(
            descriptor.bytes().len(),
            original.bytes().len(),
            "the actual allocation remains exact"
        );
        let bytes = replace_gc_program(&artifact.bytes, original.bytes(), descriptor.bytes());
        let outcome = run_bytes(&engine, &artifact, &bytes);
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        // The greedy optional has succeeded without consuming, so its fallback
        // survives. End/Clear is not a pure failed attempt: the next real End
        // must read an unset Start and preserve the transactional lastIndex.
        assert!(
            outcome.note.contains(&format!("number({iterations})")),
            "{}",
            outcome.note
        );
    }
}

#[test]
fn admitted_named_reference_body_preserves_second_iteration_ambiguity_and_last_index() {
    use lila_ir::{
        RegExpCaseFolding, RegExpInstruction, RegExpNatural, RegExpOpcode, RegExpRepeatBounds,
        RegExpRepeatMaximum,
    };

    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    for nested in [false, true] {
        let pattern = if nested {
            "(?:(?:(?<x>)|(?<x>)){2}){2}"
        } else {
            "(?:(?<x>)|(?<x>)){2}"
        };
        let source = format!("var expression = /{pattern}/dg;\n");
        let source = source
            + r#"
expression.lastIndex = 3;
var result = 0;
try {
    var match = expression.exec('abc');
    if (match !== null && match.index === 3 && match[0] === '' &&
        match[1] === undefined && match[2] === undefined && match.groups.x === undefined) result = 1;
} catch (error) {
    if (error instanceof Error && error.message === 'RegExp compiled program matcher failed') result = 2;
}
expression.lastIndex === 3 ? result : 0;
"#;
        let unit = engine
            .compile_script(&source, CompileOptions::default())
            .unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        let program = RegExpProgram::compile(pattern, "dg").unwrap();
        let original = ValidatedRegExpProgram::from_program(&program).unwrap();
        let mut replacement = program.clone();
        replacement.instructions = vec![
            RegExpInstruction::repeat_begin(0),
            RegExpInstruction {
                opcode: RegExpOpcode::RepeatGuard as u64,
                operand0: 0,
                operand1: 0,
            },
        ];
        if nested {
            replacement.instructions.extend([
                RegExpInstruction::repeat_begin(1),
                RegExpInstruction {
                    opcode: RegExpOpcode::RepeatGuard as u64,
                    operand0: 9,
                    operand1: 2,
                },
            ]);
        }
        replacement.instructions.extend([
            RegExpInstruction::named_backreference(0, RegExpCaseFolding::Sensitive),
            RegExpInstruction::capture_start(1),
            RegExpInstruction::capture_end(1),
            RegExpInstruction::capture_start(2),
            RegExpInstruction::capture_end(2),
        ]);
        if nested {
            replacement.instructions.extend([
                RegExpInstruction::repeat_end(2),
                RegExpInstruction::repeat_exit(2),
            ]);
        }
        let end = replacement.instructions.len();
        replacement.instructions[1].operand0 = end as u64;
        replacement.instructions.extend([
            RegExpInstruction::repeat_end(0),
            RegExpInstruction::repeat_exit(0),
        ]);
        assert!(program.instructions.len() >= replacement.instructions.len() + 2);
        while replacement.instructions.len() + 1 < program.instructions.len() {
            replacement
                .instructions
                .push(RegExpInstruction::clear_capture_range(1, 3));
        }
        replacement.instructions.push(RegExpInstruction::accept());
        for iterations in [1_u64, 2] {
            replacement.repeat_bounds = vec![RegExpRepeatBounds::new(
                RegExpNatural::from_u64(iterations),
                RegExpRepeatMaximum::Finite(RegExpNatural::from_u64(iterations)),
            )
            .unwrap()];
            if nested {
                replacement.repeat_bounds.push(
                    RegExpRepeatBounds::new(
                        RegExpNatural::from_u64(1),
                        RegExpRepeatMaximum::Finite(RegExpNatural::from_u64(1)),
                    )
                    .unwrap(),
                );
            }
            let descriptor = ValidatedRegExpProgram::from_program(&replacement)
                .expect("the dense paired graph and original named candidate table are admitted");
            assert_eq!(
                ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
                descriptor
            );
            assert_eq!(
                descriptor.bytes().len(),
                original.bytes().len(),
                "same actual allocation footprint"
            );
            let bytes = replace_gc_program(&artifact.bytes, original.bytes(), descriptor.bytes());
            let outcome = run_bytes(&engine, &artifact, &bytes);
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            // Both aliases are initially absent. The first body makes both empty;
            // post-loop Clear permits one iteration to succeed. A second reference
            // must observe both participants and fail before that Clear. Empty
            // post-body pairs alone must never authorize skipping this real read.
            assert!(
                outcome.note.contains(&format!("number({iterations})")),
                "nested={nested}, {iterations}: {}",
                outcome.note
            );
        }
    }
}
