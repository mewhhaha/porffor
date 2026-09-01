const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_build.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const LOADED_SOURCES_SOURCE: &str = include_str!("../src/modules/loaded_sources.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");

#[test]
fn graph_build_has_one_private_owner_and_narrow_crate_reexport() {
    assert_eq!(MODULES_SOURCE.matches("\nmod graph_build;\n").count(), 1);
    assert_eq!(
        MODULES_SOURCE
            .matches("pub(crate) use graph_build::build_graph;")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_build;\n"));
    assert!(!MODULES_SOURCE.contains("\npub use graph_build::build_graph;\n"));
    assert!(!MODULES_SOURCE.contains("pub(crate) use graph::{build_graph, link};"));
    assert_eq!(
        OWNER_SOURCE.matches("pub(crate) fn build_graph(").count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("pub(crate) fn build_graph("));
    assert_eq!(
        GRAPH_TESTS_SOURCE
            .matches("use super::super::graph_build::build_graph;")
            .count(),
        1
    );
    assert!(OWNER_SOURCE.contains("use super::graph::ModuleGraphIr;"));
    assert!(!GRAPH_SOURCE.contains("use super::graph_build::build_graph;"));
    assert!(!GRAPH_TESTS_SOURCE.contains("use super::graph_build::build_graph;"));
}

#[test]
fn graph_build_keeps_parse_once_unit_minting_and_the_closed_parse_dispatch() {
    assert!(OWNER_SOURCE.contains("let mut graph = ModuleGraphIr::default();"));
    assert_eq!(OWNER_SOURCE.matches("ModuleParse::Module(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("ModuleParse::ScriptEntry(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("ModuleParse::Rejected {").count(), 1);
    assert!(OWNER_SOURCE.contains("parse_module_record(unit_source, id, source.key().clone())"));
    assert!(OWNER_SOURCE.contains("super::record::script_entry_record("));
    assert!(OWNER_SOURCE.contains("super::early::module_parse_failure_diagnostic(error)"));
    assert!(OWNER_SOURCE.contains(".filter(|id| *id <= MAX_LINKABLE_MODULE_UNIT_ID)"));
    assert!(OWNER_SOURCE.contains("graph.units.push(ModuleUnitIr {"));
    assert!(!OWNER_SOURCE.contains(
        "let id = ModuleUnitId::try_from(graph.units.len()).unwrap_or(ModuleUnitId::MAX);"
    ));
    assert_eq!(
        LOADED_SOURCES_SOURCE
            .matches("pub(super) enum ModuleParse")
            .count(),
        1
    );
}

#[test]
fn graph_build_keeps_duplicate_identity_and_inconsistent_resolution_without_a_winner() {
    assert!(OWNER_SOURCE.contains("if let Some(&existing) = graph.keys.get(source.key())"));
    assert!(OWNER_SOURCE.contains("ModuleLinkErrorIr::InconsistentLoad { key }"));
    assert!(OWNER_SOURCE.contains("graph.resolutions.remove(&identity);"));
    assert!(OWNER_SOURCE.contains("inconsistent_resolutions.insert(identity);"));
    assert!(
        OWNER_SOURCE.contains("ModuleLinkErrorIr::InconsistentResolution { referrer, request }")
    );
}

#[test]
fn build_graph_callers_keep_the_existing_crate_boundary() {
    assert_eq!(OWNER_SOURCE.matches("build_graph").count(), 1);
    assert_eq!(GRAPH_SOURCE.matches("build_graph(").count(), 0);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("build_graph(").count(), 33);
    assert_eq!(LOWERING_SOURCE.matches("modules::build_graph(").count(), 1);
    assert_eq!(
        DYNAMIC_SOURCE
            .matches("crate::modules::build_graph(")
            .count(),
        1
    );
    assert_eq!(
        LINK_SOURCE.matches("crate::modules::build_graph(").count(),
        1
    );
    assert_eq!(
        NAMESPACE_SOURCE
            .matches("super::super::build_graph(")
            .count(),
        2
    );
}
