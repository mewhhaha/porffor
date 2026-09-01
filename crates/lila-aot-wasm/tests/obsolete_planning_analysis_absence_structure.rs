const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const MODULE_SOURCE: &str = include_str!("../src/module.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/obsolete-planning-analysis-removal.md");
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

#[test]
fn unreachable_planning_analysis_is_absent() {
    for name in [
        "is_large_deferred_standard_builtin",
        "script_uses_env",
        "script_uses_calls",
        "script_uses_function_heap",
        "script_uses_function_table",
        "block_uses_function_table",
        "block_uses_calls",
        "statement_uses_calls",
        "for_init_uses_calls",
        "statement_uses_function_table",
        "for_init_uses_function_table",
        "expr_uses_function_table",
        "expr_uses_calls",
    ] {
        assert!(!PLANNING_SOURCE.contains(name), "`{name}`");
    }
}

#[test]
fn live_planning_and_ir_authorities_remain() {
    for declaration in [
        "pub(crate) fn count_param_binding_locals(",
        "pub(crate) fn count_block_lexicals(",
        "pub(crate) fn block_references_function(",
        "pub(crate) fn should_stub_standard_builtin(",
        "pub(crate) fn values(&self)",
        "pub(crate) fn metas(&self)",
    ] {
        assert!(PLANNING_SOURCE.contains(declaration), "`{declaration}`");
    }
    assert!(!PLANNING_SOURCE.contains("super_constructor_target"));
    assert!(!PLANNING_SOURCE.contains("pub(crate) fn iter(&self)"));
    assert!(MODULE_SOURCE.contains("pub(crate) fn is_typed_array_constructor("));
    assert!(DATA_SOURCE.contains(
        "function.super_constructor_target.as_deref() == Some(BUILTIN_REGEXP_FUNCTION_ID)"
    ));
}

#[test]
fn removal_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "be7c5a1e0e9fe6fefc2c8a5db187f192c1e5f55764eeee29d940dc26ad94a177",
            "17b31c1feb5348b2f1e2dc0cdf24a618519ddebf55d39105288c6b898d8fb88f",
            "4050159124cc94d7b65ee22e7bd566c9b600bf5bbb55b5815ca6f4ef537e3ea8",
            "34679124b57a9e0716f4a604d29f5383ffd4c91ce3d1fdb8aa509c65951df238",
            "c99ecf4f2aca412f218f8e5a6be29cacb0fe51d34635533114bc0d74e698bba5",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
