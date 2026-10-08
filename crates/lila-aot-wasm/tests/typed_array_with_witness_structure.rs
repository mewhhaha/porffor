const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_with_buffer_witness.js");

#[test]
fn focused_cli_fixture_pins_with_entry_witness_behavior() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_validates_typedarray_with_entry_buffer_witness()")
        .expect("missing focused TypedArray.prototype.with witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray.prototype.with witness CLI test")
        .0;

    assert!(test.contains("wasm_typedarray_with_buffer_witness.js"));
    assert!(test.contains("boolean(true)"));
    for marker in [
        "detached receiver error realm",
        "detached receiver skips coercion",
        "out-of-bounds receiver error realm",
        "odd-byte tracking length floor",
        "borrowed with out-of-range error realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing TypedArray.prototype.with CLI control: {marker}"
        );
    }
}
