const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/proxy-revocation-route-ownership.md");
const TASK: &str = include_str!("../../../../tasks/10-object-model-descriptors-exotics.md");

#[test]
fn contract_and_task_record_the_bounded_ownership_law() {
    for phrase in [
        "ten exact producers",
        "one consuming exhaustive router",
        "CurrentFunctionRealm",
        "ProxyExecutionRealmToActiveHandler",
        "ObjectMutationRealmToActiveHandler",
        "Realm-correcting",
    ] {
        assert!(CONTRACT.contains(phrase), "contract missing `{phrase}`");
    }
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test structure_builtins -- proxy_revocation_route_ownership_structure::"));
    assert!(TASK.contains("`ProxyRevocationRoute` is now a crate-private, capability-free"));
}
