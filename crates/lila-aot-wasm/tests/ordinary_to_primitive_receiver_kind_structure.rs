const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/ordinary-to-primitive-receiver-kind.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

#[test]
fn contract_and_task_record_the_closed_receiver_boundary() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("OrdinaryToPrimitiveReceiverKind"));
        assert!(evidence.contains("Object"));
        assert!(evidence.contains("Function"));
        assert!(evidence.contains("Array"));
        assert!(evidence.contains("Arguments"));
        assert!(evidence.contains("unrepresentable"));
    }
}
