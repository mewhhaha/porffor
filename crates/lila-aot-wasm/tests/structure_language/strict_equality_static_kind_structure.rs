const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/strict-equality-static-kind-domain.md");
const TASK: &str = include_str!("../../../../tasks/04-spec-operations-and-completion-abi.md");

#[test]
fn contract_and_task_record_total_strict_equality_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("ValueKind"));
        assert!(source.contains("Dynamic"));
        assert!(source.contains("tagged equality"));
        assert!(source.contains("raw-payload equality"));
    }
}
