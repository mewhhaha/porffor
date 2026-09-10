use lila_front::{parse, ParseOptions};
use lila_ir::{lower, HeapShape, KindSet, StatementIr, ValueKind};

#[test]
fn unknown_object_constructor_input_retains_all_object_result_kinds() {
    let source = parse(
        "function box(value) { return Object(value); } box(1n);",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let function = script.functions.iter().find(|f| f.name == "box").unwrap();
    let result = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .expect("Object return expression");
    let object_kinds = KindSet::from_kind(ValueKind::Object)
        .union(KindSet::from_kind(ValueKind::Array))
        .union(KindSet::from_kind(ValueKind::Function))
        .union(KindSet::from_kind(ValueKind::Arguments));
    assert_eq!(result.possible_kinds, object_kinds);
    assert!(result.heap_shape.is_none());
}

#[test]
fn known_primitive_boxing_keeps_its_primitive_and_prototype_shape() {
    for (expression, kind) in [
        ("17", ValueKind::Number),
        ("false", ValueKind::Boolean),
        ("'value'", ValueKind::String),
        ("1n", ValueKind::BigInt),
        ("Symbol()", ValueKind::Symbol),
    ] {
        let source = parse(
            format!("function box() {{ return Object({expression}); }} box();"),
            ParseOptions::script(),
        )
        .unwrap();
        let program = lower(&source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let function = script.functions.iter().find(|f| f.name == "box").unwrap();
        let result = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(value) => Some(value),
                _ => None,
            })
            .expect("Object return expression");
        assert_eq!(result.possible_kinds, KindSet::from_kind(ValueKind::Object));
        let Some(HeapShape::Object(shape)) = result.heap_shape.as_deref() else {
            panic!("{expression}: missing boxed object shape");
        };
        assert_eq!(shape.boxed_primitive.as_ref().unwrap().kind, kind);
        assert!(shape.prototype.is_some(), "{expression}");
    }
}
