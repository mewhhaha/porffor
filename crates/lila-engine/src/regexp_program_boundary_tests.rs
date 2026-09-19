use super::*;
use lila_ir::{
    RegExpProgram, RegExpProgramWord, ValidatedRegExpProgram, REGEXP_INSTRUCTION_WIDTH,
    REGEXP_OPCODE_NAMED_BACKREFERENCE, REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
    REGEXP_OPCODE_UNICODE_PROPERTY, REGEXP_PROGRAM_HEADER_SIZE,
};

fn run_bytes(engine: &Engine, bytes: &[u8]) -> RunOutcome {
    run_on_sized_stack(|| {
        engine.run_with_wasm_bytes_inner(
            bytes,
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
        let outcome = run_bytes(&engine, &bytes);
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
            let outcome = run_bytes(&engine, &bytes);
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            assert!(
                outcome.note.contains("boolean(true)"),
                "/{pattern}/ operand {operand}: {}",
                outcome.note
            );
        }
    }
}
