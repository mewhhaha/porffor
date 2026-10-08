const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-calendar-carrier-privacy.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

#[test]
fn calendar_carrier_frozen_history_remains_documented() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TemporalCalendarCarrier`"));
        assert!(
            evidence.contains("1726881c45223f008814169edef8a3066c23b8733d86714d63570535ba3dd831")
        );
        assert!(
            evidence.contains("a74006922ea5018cd1d001421de4f83b70c23db9b73924ab24627415c642765c")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
