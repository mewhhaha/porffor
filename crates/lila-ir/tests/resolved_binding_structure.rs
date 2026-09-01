const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/resolved_binding.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_BUILD_SOURCE: &str = include_str!("../src/modules/graph_build.rs");
const GRAPH_RESOLUTION_SOURCE: &str = include_str!("../src/modules/graph_resolution.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const RECORD_SOURCE: &str = include_str!("../src/modules/record.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn resolved_binding_has_one_private_owner_and_narrow_public_facade() {
    assert_eq!(
        MODULES_SOURCE.matches("\nmod resolved_binding;\n").count(),
        1
    );
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use resolved_binding::{ModuleBindingNameIr, ResolvedBindingIr};")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod resolved_binding;\n"));
    assert!(!MODULES_SOURCE.contains("\nmod resolved_binding {\n"));
    assert!(!GRAPH_SOURCE.contains("enum ModuleBindingNameIr"));
    assert!(!GRAPH_SOURCE.contains("enum ResolvedBindingIr"));
    assert!(!GRAPH_TESTS_SOURCE.contains("enum ModuleBindingNameIr"));
    assert!(!GRAPH_TESTS_SOURCE.contains("enum ResolvedBindingIr"));
    assert_eq!(
        OWNER_SOURCE.matches("pub enum ModuleBindingNameIr").count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE.matches("pub enum ResolvedBindingIr").count(),
        1
    );
    assert_eq!(LIB_SOURCE.matches("ModuleBindingNameIr").count(), 1);
    assert_eq!(LIB_SOURCE.matches("ResolvedBindingIr").count(), 1);
}

#[test]
fn resolved_binding_preserves_the_closed_binding_and_resolution_domains() {
    let binding_domain = bounded(
        OWNER_SOURCE,
        "pub enum ModuleBindingNameIr {",
        "/// Result of `ResolveExport` (16.2.1.6.3).",
    );
    assert_eq!(
        code_without_whitespace(binding_domain),
        "Namespace,Name(LocalName),ModuleSource,}"
    );

    let resolution_domain = OWNER_SOURCE
        .split_once("pub enum ResolvedBindingIr {")
        .expect("resolved binding domain")
        .1;
    assert_eq!(
        code_without_whitespace(resolution_domain),
        "Resolved{module:ModuleUnitId,binding:ModuleBindingNameIr,},Ambiguous,NotFound,}"
    );
    assert!(!OWNER_SOURCE.contains("_ =>"));
}

#[test]
fn resolution_algorithms_and_existing_consumers_keep_their_owners() {
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE
            .matches("pub fn resolve_export(")
            .count(),
        1
    );
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE
            .matches("fn resolve_export_inner(")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("fn resolve_export"));
    assert!(!GRAPH_TESTS_SOURCE.contains("fn resolve_export"));
    assert!(!OWNER_SOURCE.contains("ModuleLinkErrorIr"));
    assert!(!OWNER_SOURCE.contains("fn resolve_export"));

    assert_eq!(GRAPH_SOURCE.matches("ModuleBindingNameIr").count(), 2);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ModuleBindingNameIr").count(), 7);
    assert_eq!(GRAPH_SOURCE.matches("ResolvedBindingIr").count(), 10);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ResolvedBindingIr").count(), 9);
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE
            .matches("ModuleBindingNameIr")
            .count(),
        3
    );
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE.matches("ResolvedBindingIr").count(),
        15
    );
    assert_eq!(GRAPH_BUILD_SOURCE.matches("ResolvedBindingIr").count(), 3);
    assert_eq!(LINK_SOURCE.matches("ModuleBindingNameIr").count(), 3);
    assert_eq!(LINK_SOURCE.matches("ResolvedBindingIr").count(), 2);
    assert_eq!(NAMESPACE_SOURCE.matches("ModuleBindingNameIr").count(), 8);
    assert_eq!(NAMESPACE_SOURCE.matches("ResolvedBindingIr").count(), 20);
    assert_eq!(DYNAMIC_SOURCE.matches("ModuleBindingNameIr").count(), 1);
    assert_eq!(DYNAMIC_SOURCE.matches("ResolvedBindingIr").count(), 4);
    assert!(!RECORD_SOURCE.contains("ModuleBindingNameIr"));
    assert_eq!(RECORD_SOURCE.matches("ResolvedBindingIr").count(), 1);
}
