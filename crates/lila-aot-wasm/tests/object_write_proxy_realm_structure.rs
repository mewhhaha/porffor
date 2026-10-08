const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_set_error_realm.js");

#[test]
fn borrowed_created_realm_builtins_pin_proxy_set_error_realms() {
    for marker in [
        "other.Array.prototype.fill.call(revocable.proxy, 1)",
        "other.Array.prototype.fill.call(nonCallableProxy, 1)",
        "other.Array.prototype.fill.call(falseResultProxy, 1)",
        "other.Reflect.set(incompatibleProxy, \"fixed\", 2)",
        "other.Reflect.set(directReflectRevocable.proxy, \"value\", 1)",
        "other.Reflect.set(directReflectNonCallable, \"value\", 1)",
        "other.Array.prototype.fill.call(fillPrototypeReceiver, 2)",
        "other.Array.prototype.fill.call(falsePrototypeReceiver, 2)",
        "other.Array.prototype.push.call(pushFalsePrototypeReceiver, 2)",
        "other.Reflect.set(reflectPrototypeReceiver, \"value\", 1)",
        "Object.getPrototypeOf(error) === other.TypeError.prototype",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(CLI_TESTS.contains("fn proxy_set_errors_use_the_borrowed_builtin_realm()"));
    assert!(CLI_TESTS.contains("wasm_proxy_set_error_realm.js"));
}
