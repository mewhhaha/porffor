const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/strict-equality-static-kind-domain.md");

#[test]
fn contract_records_total_strict_equality_ownership() {
    for source in [CONTRACT] {
        assert!(source.contains("ValueKind"));
        assert!(source.contains("Dynamic"));
        assert!(source.contains("tagged equality"));
        assert!(source.contains("raw-payload equality"));
    }
}
