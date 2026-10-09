const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/regexp-flag-getter.md");
const TASK: &str = include_str!("../../../../tasks/19-regexp.md");

#[test]
fn contract_and_task_record_the_private_dispatcher_boundary() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("Batch AZ"));
        assert!(evidence.contains("eight fixed RegExp flag-getter entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new RegExp behavior"));
    }
}
