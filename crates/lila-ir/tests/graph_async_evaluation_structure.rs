const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_async_evaluation.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn graph_async_evaluation_has_one_private_owner_with_the_public_inherent_api() {
    assert_eq!(
        MODULES_SOURCE
            .matches("\nmod graph_async_evaluation;\n")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_async_evaluation;\n"));
    assert!(!MODULES_SOURCE.contains("use graph_async_evaluation::"));
    assert_eq!(OWNER_SOURCE.matches("impl ModuleGraphIr {").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("pub fn async_evaluation(").count(), 1);
    assert_eq!(
        OWNER_SOURCE
            .matches("pub fn pending_async_dependencies(")
            .count(),
        1
    );
    assert!(!OWNER_SOURCE.contains("pub(crate) fn"));
    assert!(!OWNER_SOURCE.contains("pub(super) fn"));
    assert!(!GRAPH_SOURCE.contains("pub fn async_evaluation("));
    assert!(!GRAPH_SOURCE.contains("pub fn pending_async_dependencies("));
    assert_eq!(LINK_SOURCE.matches("async_evaluation(").count(), 2);
    assert_eq!(
        LINK_SOURCE.matches("pending_async_dependencies(").count(),
        1
    );
}

#[test]
fn async_evaluation_propagates_over_eager_components_in_evaluation_order() {
    let propagation = bounded(
        OWNER_SOURCE,
        "pub fn async_evaluation(&self) -> Vec<bool> {",
        "/// `[[PendingAsyncDependencies]]`",
    );
    assert!(propagation.contains("let components = self.component_of_unit();"));
    assert!(propagation.contains("let mut component_async = vec![false; component_count];"));
    assert!(propagation.contains("for member in &self.evaluation_order"));
    assert!(propagation.contains("self.evaluation_mode(*member) != ModuleEvaluationModeIr::Eager"));
    assert!(propagation.contains("if self.has_tla(*member)"));
    assert!(propagation.contains("for dependency in self.evaluation_dependencies_of(*member)"));
    assert!(propagation.contains("dependency_component != component"));
    assert!(propagation.contains("component_async[dependency_component]"));
    assert!(propagation.contains("self.evaluation_mode(module) == ModuleEvaluationModeIr::Eager"));
}

#[test]
fn pending_async_dependencies_counts_distinct_external_async_components() {
    let pending = OWNER_SOURCE
        .split_once("pub fn pending_async_dependencies(")
        .expect("pending async dependency query")
        .1;
    assert!(pending.contains("self.evaluation_mode(module) != ModuleEvaluationModeIr::Eager"));
    assert!(pending.contains("let asynchronous = self.async_evaluation();"));
    assert!(pending.contains("let Some(&own) = components.get(module as usize)"));
    assert!(pending.contains("let mut counted: BTreeSet<usize> = BTreeSet::new();"));
    assert!(pending.contains("for dependency in self.evaluation_dependencies_of(module)"));
    assert!(pending.contains("component != own && asynchronous[dependency as usize]"));
    assert!(pending.contains("counted.insert(component);"));
    assert!(pending.contains("counted.len()"));
}
