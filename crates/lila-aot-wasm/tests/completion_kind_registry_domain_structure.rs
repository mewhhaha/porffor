const ABI_SOURCE: &str = include_str!("../src/abi.rs");

fn completion_kind_registry_source() -> &'static str {
    let start = ABI_SOURCE
        .find("pub(crate) const COMPLETION_KIND_REGISTRY")
        .expect("completion-kind registry should exist");
    let end = ABI_SOURCE[start..]
        .find(';')
        .map(|offset| start + offset + 1)
        .expect("completion-kind registry should be a bounded declaration");
    &ABI_SOURCE[start..end]
}

#[test]
fn completion_kind_registry_contains_only_the_ordered_closed_domain() {
    let registry = completion_kind_registry_source();

    assert_eq!(
        registry,
        "pub(crate) const COMPLETION_KIND_REGISTRY: &[CompletionKindIr] = CompletionKindIr::ALL;"
    );
    assert!(!ABI_SOURCE.contains("struct CompletionKindSlot"));
    assert!(!registry.contains("name:"));
    assert!(!registry.contains("value:"));
}

#[test]
fn completion_kind_registry_tests_use_enum_projections() {
    let tests = ABI_SOURCE
        .split_once("#[cfg(test)]")
        .map(|(_, tests)| tests)
        .expect("ABI unit tests should exist");

    assert!(tests.contains("kind.abi_code()"));
    assert!(tests.contains("kind.name()"));
    assert!(tests.contains("backend.name()"));
    assert!(tests.contains("backend.abi_code()"));
    assert!(!tests.contains("slot.name"));
    assert!(!tests.contains("slot.value"));
    assert!(!tests.contains("backend.name,"));
    assert!(!tests.contains("backend.value,"));
}
