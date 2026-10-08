const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/coercive-number-arithmetic-operation.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

#[test]
fn contract_and_task_record_total_number_arithmetic_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("ArithmeticBinaryOp"));
        assert!(source.contains("Add"));
        assert!(source.contains("Mod"));
        assert!(source.contains("Exp"));
    }
}
