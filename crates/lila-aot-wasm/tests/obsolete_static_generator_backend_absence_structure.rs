const IR_NAMES: &str = include_str!("../../lila-ir/src/names.rs");
const WASM_ROOT: &str = include_str!("../src/lib.rs");
const DATA: &str = include_str!("../src/data.rs");
const FUNCTIONS: &str = include_str!("../src/functions.rs");
const CONTROL_FLOW: &str = include_str!("../src/control_flow.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");

#[test]
fn retired_static_generator_protocol_has_no_producer_or_consumer() {
    for source in [IR_NAMES, WASM_ROOT, DATA, FUNCTIONS, CONTROL_FLOW, STANDARD] {
        for retired_spelling in [
            "LILA_STATIC_GENERATOR_",
            "$LilaStaticGenerator",
            "StaticGeneratorValues",
            "emit_exhaust_static_generator_iterator_if_marked",
        ] {
            assert!(
                !source.contains(retired_spelling),
                "retired static-generator backend spelling `{retired_spelling}` survived"
            );
        }
    }
}
