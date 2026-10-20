const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/regexp-flag-getter.md");

#[test]
fn contract_records_the_private_dispatcher_boundary() {
    for evidence in [CONTRACT] {
        assert!(evidence.contains("Batch AZ"));
        assert!(evidence.contains("eight fixed RegExp flag-getter entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new RegExp behavior"));
    }
}
