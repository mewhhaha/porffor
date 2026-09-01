const ENGINE_SOURCE: &str = include_str!("../src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

#[test]
fn engine_error_stores_one_optional_wasmtime_policy_authority() {
    let error = bounded(
        ENGINE_SOURCE,
        "pub struct EngineError {",
        "/// Closed pre-instantiation failures",
    );
    assert_eq!(
        error
            .matches("wasmtime_policy: Option<WasmtimeRuntimePolicy>,")
            .count(),
        1
    );
    assert!(!error.contains("wasm_gc_capability:"));
    assert!(!error.contains("wasm_weak_reachability_capability:"));
}

#[test]
fn only_wasmtime_setup_errors_retain_the_policy() {
    let constructors = bounded(
        ENGINE_SOURCE,
        "impl EngineError {",
        "impl core::fmt::Display for EngineError",
    );
    assert_eq!(
        constructors
            .matches("wasmtime_policy: Some(err.policy),")
            .count(),
        1
    );
    assert_eq!(constructors.matches("wasmtime_policy: None,").count(), 4);

    let setup = bounded(
        constructors,
        "fn from_wasmtime_setup(",
        "fn from_intl_artifact_identity(",
    );
    assert!(setup.contains("wasmtime_policy: Some(err.policy),"));
}

#[test]
fn public_capabilities_project_exhaustively_from_the_retained_policy() {
    let gc = bounded(
        ENGINE_SOURCE,
        "pub const fn wasm_gc_capability(",
        "/// Required weak-reachability capability",
    );
    let weak = bounded(
        ENGINE_SOURCE,
        "pub const fn wasm_weak_reachability_capability(",
        "impl core::fmt::Display for EngineError",
    );

    for (projection, capability) in [
        (gc, "policy.gc_capability()"),
        (weak, "policy.weak_reachability_capability()"),
    ] {
        assert!(projection.contains("match self.wasmtime_policy {"));
        assert!(projection.contains(&format!("Some(policy) => Some({capability})")));
        assert!(projection.contains("None => None"));
        assert!(!projection.contains("_ =>"));
    }
}
