const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_creation_execution_realm.js");

#[test]
fn created_realm_publication_and_behavior_witness_cover_all_products() {
    assert!(CLI_TESTS.contains("fn proxy_creation_uses_the_builtin_execution_realm()"));
    assert!(CLI_TESTS.contains("wasm_proxy_creation_execution_realm.js"));
    assert!(
        !CLI_FIXTURE.contains("evalScript"),
        "the Proxy fixture must not seed host-owned createRealm property names"
    );
    for marker in [
        "new OtherProxy(0, {})",
        "new OtherProxy({}, 0)",
        "otherRevocable(0, {})",
        "otherRevocable({}, 0)",
        "Object.getPrototypeOf(revocable) === other.Object.prototype",
        "Object.getPrototypeOf(revocable.revoke) === other.Function.prototype",
        "revocable.revoke();\nrevocable.revoke();",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
