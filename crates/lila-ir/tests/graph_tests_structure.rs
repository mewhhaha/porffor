const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");

#[test]
fn graph_tests_have_one_private_adjacent_owner_with_the_existing_namespace() {
    assert_eq!(GRAPH_SOURCE.matches("\n#[cfg(test)]\n").count(), 2);
    assert_eq!(
        GRAPH_SOURCE
            .matches("#[path = \"graph_tests.rs\"]\nmod tests;")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("mod tests {"));
    assert!(!GRAPH_SOURCE.contains("#[test]"));
    assert!(!OWNER_SOURCE.contains("mod tests"));
}

#[test]
fn graph_tests_keep_the_inherited_graph_namespace_and_exact_test_census() {
    assert!(OWNER_SOURCE.starts_with("use super::*;\n\n"));
    assert_eq!(
        OWNER_SOURCE
            .matches("use super::super::graph_build::build_graph;")
            .count(),
        1
    );
    // The added bytes test rejects equal JavaScript spelling with different
    // host provenance while the source-phase re-export coverage remains here.
    assert_eq!(OWNER_SOURCE.matches("#[test]").count(), 62);
    assert!(
        OWNER_SOURCE.contains("fn identical_source_text_cannot_replace_bytes_module_provenance()")
    );
    assert_eq!(OWNER_SOURCE.matches("fn sources_of(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("fn linked(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("fn unit_of(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("fn components(").count(), 1);
}
