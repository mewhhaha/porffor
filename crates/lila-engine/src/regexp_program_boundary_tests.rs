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
    let positions = artifact
        .bytes
        .windows(descriptor.bytes().len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == descriptor.bytes()).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        1,
        "one deduplicated descriptor must be embedded"
    );
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
        let mut bytes = artifact.bytes.clone();
        let offset = positions[0] + offset;
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
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
        let source = format!("let expression = /{pattern}/i; expression.lastIndex = 3; let rejected = false; try {{ expression.test(''); }} catch (error) {{ rejected = error.message === 'RegExp compiled program matcher failed'; }} rejected && expression.lastIndex === 3;");
        let unit = engine
            .compile_script(&source, CompileOptions::default())
            .unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        let program = RegExpProgram::compile(pattern, "i").unwrap();
        let descriptor = ValidatedRegExpProgram::from_program(&program).unwrap();
        let positions = artifact
            .bytes
            .windows(descriptor.bytes().len())
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == descriptor.bytes()).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(
            positions.len(),
            1,
            "one deduplicated descriptor must be embedded"
        );
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
        let offset = positions[0] + REGEXP_PROGRAM_HEADER_SIZE + pc * REGEXP_INSTRUCTION_WIDTH + 16;
        for operand in operands {
            // Corrupt the encoded allocation after the producer's validation;
            // mutating compiler IR would only test its construction boundary.
            let mut bytes = artifact.bytes.clone();
            bytes[offset..offset + 8].copy_from_slice(&operand.to_le_bytes());
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
    let positions = artifact
        .bytes
        .windows(original.bytes().len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == original.bytes()).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        1,
        "one real immutable descriptor allocation"
    );

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
        let mut bytes = artifact.bytes.clone();
        bytes[positions[0]..positions[0] + descriptor.bytes().len()]
            .copy_from_slice(descriptor.bytes());
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
    let positions = artifact
        .bytes
        .windows(original.bytes().len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == original.bytes()).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        1,
        "one real immutable descriptor allocation"
    );
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
        let mut bytes = artifact.bytes.clone();
        bytes[positions[0]..positions[0] + descriptor.bytes().len()]
            .copy_from_slice(descriptor.bytes());
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
        let positions = artifact
            .bytes
            .windows(original.bytes().len())
            .enumerate()
            .filter_map(|(offset, bytes)| (bytes == original.bytes()).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(
            positions.len(),
            1,
            "one real immutable descriptor allocation"
        );
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
            let mut bytes = artifact.bytes.clone();
            bytes[positions[0]..positions[0] + descriptor.bytes().len()]
                .copy_from_slice(descriptor.bytes());
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
