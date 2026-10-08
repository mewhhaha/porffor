const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-field-offset-table-privacy.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

#[test]
fn field_offset_table_frozen_history_remains_documented() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TEMPORAL_DURATION_FIELD_OFFSETS`"));
        assert!(evidence.contains("owner-private `TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS`"));
        assert!(
            evidence.contains("b47f9d79e4e1dc65b91a4ac7a2663a20b54cb5b6aea099266b381e6380e06ab1")
        );
        assert!(
            evidence.contains("f7047424c3fe0d3837f3d5db310d41d2c7a61740badcb97ec606c89c65746123")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
