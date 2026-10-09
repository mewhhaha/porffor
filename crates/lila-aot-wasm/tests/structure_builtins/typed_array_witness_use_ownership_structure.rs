const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/typed-array-witness-use-ownership.md");
const TASK: &str = include_str!("../../../../tasks/17-typedarrays-binary-data-atomics.md");

#[test]
fn contract_and_task_record_the_owned_witness_use() {
    for marker in [
        "move-only witness-use",
        "final consuming projection",
        "typed_array_witness_use_ownership_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
