const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/typed-array-witness-use-ownership.md");

#[test]
fn contract_records_the_owned_witness_use() {
    for marker in [
        "move-only witness-use",
        "final consuming projection",
        "typed_array_witness_use_ownership_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(CONTRACT.contains(marker), "task marker `{marker}`");
    }
}
