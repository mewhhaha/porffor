const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/object-read-realm-projection-capability.md"
);
const TASK: &str = include_str!("../../../../tasks/10-object-model-descriptors-exotics.md");

#[test]
fn contract_and_t10_record_the_source_equivalent_capability_closure() {
    for marker in [
        "two distinct private, non-derived",
        "ABI argument receives",
        "does not claim an object-model redesign",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("object-read-realm-projection-capability.md"));
}
