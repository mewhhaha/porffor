use lila_front::{parse, ParseOptions};
use lila_ir::{lower, StandardBuiltinId};

#[test]
fn float16_constructor_and_inherited_methods_lower_through_standard_builtins() {
    for source in [
        "new Float16Array(2);",
        "Float16Array.from([1.5]);",
        "Float16Array.of(1.5);",
        "new Float16Array([1.5]).map(x => x * 2);",
        "new Float16Array([1.5]).slice();",
        "class Half extends Float16Array {} new Half(1);",
        "new Float16Array(new ArrayBuffer(4), 2, 1);",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn float16_constructor_has_a_distinct_constructable_global_identity() {
    let builtin = StandardBuiltinId::Float16ArrayConstructor;
    assert_eq!(builtin.global_name(), Some("Float16Array"));
    assert_eq!(builtin.debug_name(), "Float16Array");
    assert_ne!(
        builtin.function_id(),
        StandardBuiltinId::Float32ArrayConstructor.function_id()
    );
    assert!(builtin.constructable());
}
