const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const PUSH_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_push_argument_expansion.js");

#[test]
fn focused_push_control_covers_more_than_eight_and_spread_arguments() {
    for marker in [
        "let target = [0];",
        "record(8),",
        "...spread,",
        "record(12),",
        "if (target.length !== 1) throw \"push started before spread expansion\";",
        "let correct = length === 13 && target.length === 13",
        "correct = correct && target[index] === index;",
    ] {
        assert!(
            PUSH_FIXTURE.contains(marker),
            "missing Push marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_expands_all_array_push_arguments_before_appending()"));
}
