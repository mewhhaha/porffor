const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const CLASSIFICATION_SOURCE: &str =
    include_str!("../src/modules/graph_evaluation_classification.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const TASK: &str = include_str!("../../../tasks/12-modules-linking-loading.md");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/dynamic-component-authority.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn code_without_comments_or_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn dynamic_component_exposes_observations_without_construction_fields() {
    let declaration = bounded(
        DYNAMIC_SOURCE,
        "pub struct DynamicComponentIr {",
        "impl DynamicComponentIr {",
    );
    assert_eq!(
        code_without_comments_or_whitespace(declaration),
        "key:ModuleKey,request:ModuleRequestIr,referrer:ModuleUnitId,module:ModuleUnitId,}"
    );
    for public_field in ["pub key:", "pub request:", "pub referrer:", "pub module:"] {
        assert!(!declaration.contains(public_field), "{public_field}");
    }

    let observations = bounded(
        DYNAMIC_SOURCE,
        "impl DynamicComponentIr {",
        "/// Discovers every statically knowable `import()` target",
    );
    for signature in [
        "pub const fn target_key(&self) -> &ModuleKey",
        "pub const fn request(&self) -> &ModuleRequestIr",
        "pub const fn referrer(&self) -> ModuleUnitId",
        "pub const fn target(&self) -> ModuleUnitId",
    ] {
        assert_eq!(observations.matches(signature).count(), 1, "{signature}");
    }
    assert!(!observations.contains("&mut self"));
}

#[test]
fn discovery_is_the_only_dynamic_component_constructor() {
    assert_eq!(
        DYNAMIC_SOURCE
            .matches("components.push(DynamicComponentIr {")
            .count(),
        1
    );
    let discovery = bounded(
        DYNAMIC_SOURCE,
        "pub(super) fn discover_components(",
        "impl ModuleGraphIr {",
    );
    assert!(discovery.contains("let key = graph.unit(module).record.key.clone();"));
    assert!(discovery.contains("key,"));
    assert!(discovery.contains("request,"));
    assert!(discovery.contains("referrer,"));
    assert!(discovery.contains("module,"));
}

#[test]
fn linked_graph_owns_mutation_and_publishes_only_a_slice() {
    let graph_declaration = bounded(
        GRAPH_SOURCE,
        "pub struct ModuleGraphIr {",
        "impl ModuleGraphIr {",
    );
    assert_eq!(
        graph_declaration
            .matches("components: Vec<DynamicComponentIr>,")
            .count(),
        1
    );
    assert!(!graph_declaration.contains("pub components:"));

    let observations = bounded(
        GRAPH_SOURCE,
        "impl ModuleGraphIr {",
        "/// Resolves every import entry",
    );
    assert_eq!(
        observations
            .matches("pub fn dynamic_components(&self)")
            .count(),
        1
    );
    assert!(observations.contains("-> &[DynamicComponentIr]"));
    assert!(!observations.contains("&mut [DynamicComponentIr]"));
    assert!(!observations.contains("&mut Vec<DynamicComponentIr>"));
    assert_eq!(
        GRAPH_SOURCE
            .matches("graph.components = components;")
            .count(),
        1
    );
}

#[test]
fn sibling_consumers_use_the_read_only_component_boundary() {
    for source in [CLASSIFICATION_SOURCE, NAMESPACE_SOURCE, LINK_SOURCE] {
        assert!(!source.contains("graph.components"));
        assert!(!source.contains("component.module"));
        assert!(!source.contains("component.referrer,"));
        assert!(!source.contains("component.request."));
    }
    assert!(CLASSIFICATION_SOURCE.contains("component.referrer()"));
    assert!(CLASSIFICATION_SOURCE.contains("component.target()"));
    assert!(NAMESPACE_SOURCE.contains("graph.dynamic_components()"));
    assert!(LINK_SOURCE.contains("graph.dynamic_components()"));
    assert!(TASK.contains("DynamicComponentIr"));
    assert!(TASK.contains("read-only component slice"));
    assert!(CONTRACT.contains("construction authority"));
}
