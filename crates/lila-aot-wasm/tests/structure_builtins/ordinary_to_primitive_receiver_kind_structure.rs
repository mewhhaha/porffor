const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/ordinary-to-primitive-receiver-kind.md");

#[test]
fn contract_records_the_closed_receiver_boundary() {
    for evidence in [CONTRACT] {
        assert!(evidence.contains("OrdinaryToPrimitiveReceiverKind"));
        assert!(evidence.contains("Object"));
        assert!(evidence.contains("Function"));
        assert!(evidence.contains("Array"));
        assert!(evidence.contains("Arguments"));
        assert!(evidence.contains("unrepresentable"));
    }
}
