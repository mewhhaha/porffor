const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/language_errors.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_htmldda_host_hook.js");
const GOLDEN_SOURCE: &str = include_str!("emit_golden.rs");

#[test]
fn host_html_dda_fixture_and_golden_corpus_cover_the_owned_pair() {
    assert_eq!(
        CLI_SOURCE
            .matches("fn run_wasm_backend_succeeds_for_htmldda_host_hook_fixture()")
            .count(),
        1
    );
    assert_eq!(CLI_SOURCE.matches("wasm_htmldda_host_hook.js").count(), 1);
    for witness in [
        "IsHTMLDDA: __lilaCreateHTMLDDA()",
        "typeof $262.IsHTMLDDA !== \"undefined\"",
        "!!$262.IsHTMLDDA !== false",
        "$262.IsHTMLDDA == null",
        "$262.IsHTMLDDA() !== null",
        "new $262.IsHTMLDDA()",
        "Reflect.construct($262.IsHTMLDDA, [])",
        "items[Symbol.iterator] = $262.IsHTMLDDA",
        "class C extends $262.IsHTMLDDA {}",
        "if (prototypeGetterCalled)",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(witness),
            "HTMLDDA fixture must retain `{witness}`"
        );
    }

    assert!(GOLDEN_SOURCE.contains(".join(\"../lila-cli/tests/fixtures\")"));
    assert!(GOLDEN_SOURCE.contains("path.extension().is_some_and(|ext| ext == \"js\")"));
}
