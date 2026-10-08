const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/conversion-error-realm-source-lifecycle.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

#[test]
fn contract_and_t04_record_the_non_copy_phase_authority() {
    for marker in [
        "type-owned current-function Realm proof",
        "payload and tag locals only",
        "two fixed boundary selections",
        "helper ABI parameter 2",
        "does not claim a conversion-semantics change",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("conversion-error-realm-source-lifecycle.md"));
}
