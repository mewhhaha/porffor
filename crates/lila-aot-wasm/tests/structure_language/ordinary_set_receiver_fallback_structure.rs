const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_ordinary_set_outlined_receiver.js");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/ordinary-set-receiver-fallback.md");
const TASK: &str = include_str!("../../../../tasks/10-object-model-descriptors-exotics.md");

#[test]
fn contract_and_existing_product_witness_pin_the_closed_seam() {
    assert!(CONTRACT.contains("OrdinarySetReceiverFallback"));
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test structure_language -- ordinary_set_receiver_fallback_structure::"));
    assert!(TASK.contains("OrdinarySetReceiverFallback"));
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_preserves_outlined_ordinary_set_receiver_semantics()"));
    assert!(CLI_TESTS.contains("wasm_ordinary_set_outlined_receiver.js"));
    for marker in [
        "throw \"inherited setter receiver\"",
        "Reflect.set({}, symbol, 8, receiver)",
        "throw \"mapped arguments receiver write\"",
        "otherGlobal.Reflect.set([], \"length\", -1)",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
