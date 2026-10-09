const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/private-element-entry-protocol.md");
const TASK: &str = include_str!("../../../../tasks/09-functions-classes-private-elements.md");

#[test]
fn private_element_entry_contract_and_t09_checkpoint_name_the_closed_row_owner() {
    assert!(CONTRACT.contains("Private Name list publication"));
    assert!(
        TASK.contains("Realm-list publication"),
        "historical T09 checkpoint"
    );
    for marker in [
        "PrivateElementEntryLocals",
        "one owned row",
        "borrowed exhaustive projections",
        "13 lexical mentions",
        "five product producers",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(TASK.contains(marker), "missing T09 marker: {marker}");
    }
}
