const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/coercive-number-arithmetic-operation.md");

#[test]
fn contract_records_total_number_arithmetic_ownership() {
    for source in [CONTRACT] {
        assert!(source.contains("ArithmeticBinaryOp"));
        assert!(source.contains("Add"));
        assert!(source.contains("Mod"));
        assert!(source.contains("Exp"));
    }
}
