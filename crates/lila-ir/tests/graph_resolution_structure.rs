const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_resolution.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_CLASSIFICATION_SOURCE: &str =
    include_str!("../src/modules/graph_evaluation_classification.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");

#[test]
fn graph_resolution_has_one_private_owner_without_a_compatibility_reexport() {
    assert_eq!(
        MODULES_SOURCE.matches("\nmod graph_resolution;\n").count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_resolution;\n"));
    assert!(!MODULES_SOURCE.contains("use graph_resolution::"));
    assert!(OWNER_SOURCE.contains("use super::graph::ModuleGraphIr;"));
    assert!(!GRAPH_SOURCE.contains("use super::graph_resolution"));

    assert_eq!(OWNER_SOURCE.matches("impl ModuleGraphIr {").count(), 1);
    assert!(!GRAPH_SOURCE.contains("pub fn resolve_request("));
    assert!(!GRAPH_SOURCE.contains("pub fn resolve_request_key("));
    assert!(!GRAPH_SOURCE.contains("pub fn exported_names("));
    assert!(!GRAPH_SOURCE.contains("fn collect_exported_names("));
    assert!(!GRAPH_SOURCE.contains("pub fn resolve_export("));
    assert!(!GRAPH_SOURCE.contains("fn resolve_export_inner("));
}

#[test]
fn graph_resolution_preserves_the_exact_public_and_private_method_boundary() {
    assert_eq!(OWNER_SOURCE.matches("pub fn resolve_request(").count(), 1);
    assert_eq!(
        OWNER_SOURCE.matches("pub fn resolve_request_key(").count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("pub fn exported_names(").count(), 1);
    assert_eq!(
        OWNER_SOURCE.matches("fn collect_exported_names(").count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("pub fn resolve_export(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("fn resolve_export_inner(").count(), 1);
    assert!(!OWNER_SOURCE.contains("pub(crate) fn"));
    assert!(!OWNER_SOURCE.contains("pub(super) fn"));
}

#[test]
fn graph_resolution_keeps_request_identity_and_export_star_invariants() {
    assert!(OWNER_SOURCE.contains("self.resolve_request_key(referrer, request.key())"));
    assert!(OWNER_SOURCE.contains("self.resolutions.get(&(referrer, request.clone())).copied()"));
    assert!(OWNER_SOURCE.contains("if !export_star_set.insert(module)"));
    assert!(OWNER_SOURCE.contains("if !name.is_default()"));
    assert!(OWNER_SOURCE.contains("if export_name.is_default()"));
    assert!(OWNER_SOURCE.contains("if star_resolution != resolution"));
    assert!(OWNER_SOURCE.contains("return ResolvedBindingIr::Ambiguous;"));
    assert!(!OWNER_SOURCE.contains("self.keys.get("));
}

#[test]
fn graph_resolution_callers_keep_the_existing_inherent_api() {
    assert_eq!(OWNER_SOURCE.matches("resolve_request(").count(), 4);
    assert_eq!(GRAPH_SOURCE.matches("resolve_request(").count(), 3);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("resolve_request(").count(), 1);
    assert_eq!(
        GRAPH_CLASSIFICATION_SOURCE
            .matches("resolve_request(")
            .count(),
        1
    );
    assert_eq!(DYNAMIC_SOURCE.matches("resolve_request(").count(), 1);

    assert_eq!(OWNER_SOURCE.matches("resolve_request_key(").count(), 2);
    assert_eq!(GRAPH_SOURCE.matches("resolve_request_key(").count(), 1);
    assert_eq!(
        GRAPH_TESTS_SOURCE.matches("resolve_request_key(").count(),
        1
    );

    assert_eq!(OWNER_SOURCE.matches("pub fn exported_names(").count(), 1);
    assert_eq!(GRAPH_SOURCE.matches("exported_names(").count(), 0);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("exported_names(").count(), 2);
    assert_eq!(NAMESPACE_SOURCE.matches("exported_names(").count(), 1);

    assert_eq!(OWNER_SOURCE.matches("collect_exported_names(").count(), 3);
    assert!(!GRAPH_SOURCE.contains("collect_exported_names("));

    assert_eq!(OWNER_SOURCE.matches("resolve_export(").count(), 1);
    assert_eq!(GRAPH_SOURCE.matches("resolve_export(").count(), 2);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("resolve_export(").count(), 3);
    assert_eq!(NAMESPACE_SOURCE.matches("resolve_export(").count(), 1);

    assert_eq!(OWNER_SOURCE.matches("resolve_export_inner(").count(), 4);
    assert!(!GRAPH_SOURCE.contains("resolve_export_inner("));
}
