use lila_front::{parse, ParseOptions};
use lila_ir::{lower, StatementIr};

fn contains_for_in(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::ForInArray { .. }
        | StatementIr::ForInString { .. }
        | StatementIr::ForInObject { .. } => true,
        StatementIr::Block(block) => block.statements.iter().any(contains_for_in),
        StatementIr::LexicalBlock(statements) => statements.iter().any(contains_for_in),
        StatementIr::Labelled { statement, .. } => contains_for_in(statement),
        _ => false,
    }
}

fn assert_runtime_loop(source: &str) {
    let parsed = parse(source, ParseOptions::script()).expect("for-in fixture parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script IR");
    assert!(
        script.body.statements.iter().any(contains_for_in),
        "for-in must retain runtime enumeration: {source}"
    );
}

#[test]
fn every_non_nullish_primitive_head_retains_runtime_enumeration() {
    for target in ["7", "false", "7n", "Symbol()", "'text'"] {
        assert_runtime_loop(&format!("for (var key in {target}) {{}}"));
    }
}

#[test]
fn builtin_targets_and_body_shapes_cannot_remove_enumeration() {
    for source in [
        "for (var key in TypeError) {}",
        "var absent = true; for (var key in this) if (key === 'parseInt') absent = false;",
        "var assert = { notSameValue(a, b) {} }; for (var key in Number) assert.notSameValue(key, 'MAX_VALUE');",
        "var assert = { notSameValue(a, b) {} }; for (var key in Boolean) assert.notSameValue(key, 'prototype');",
    ] {
        assert_runtime_loop(source);
    }
}
