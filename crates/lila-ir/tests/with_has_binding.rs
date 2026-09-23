use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, SpecOperationIr, StatementIr, ValueKind};

#[test]
fn with_entry_converts_before_publishing_the_binding_object() {
    let program = lower(&parse("with ('abc') { length; }", ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let StatementIr::LexicalBlock(statements) = &script.body.statements[0] else {
        panic!("With entry must retain its converted head outside the new environment");
    };
    let StatementIr::Lexical { name, init, .. } = &statements[0] else {
        panic!("the first step must materialize ToObject");
    };
    let ExprIr::SpecOperation {
        operation,
        operands,
    } = &init.expr
    else {
        panic!("With entry must use the canonical ToObject operation");
    };
    assert_eq!(*operation, SpecOperationIr::ToObject);
    assert_eq!(operands.len(), 1);
    assert!(matches!(&operands[0].expr, ExprIr::String(value) if value == "abc"));
    assert!(!init.possible_kinds.contains(ValueKind::String));
    assert!(!init.possible_kinds.contains(ValueKind::Null));
    let StatementIr::Block(body) = &statements[1] else {
        panic!("the new environment follows head conversion");
    };
    let StatementIr::Lexical { init: binding, .. } = &body.statements[0] else {
        panic!("the hidden binding publishes the converted value");
    };
    assert!(matches!(&binding.expr, ExprIr::Identifier(value) if value == name));
    assert_eq!(binding.possible_kinds, init.possible_kinds);
}
