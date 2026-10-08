const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/intl-locale-string-slot-dispatch.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T23: &str = include_str!("../../../tasks/23-intl402.md");

#[test]
fn locale_string_slot_contract_records_exact_witnesses_and_nonclaims() {
    for marker in [
        "private, non-derived domain",
        "five fixed entries",
        "00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a",
        "4b346dcd2c819c503603ed7c08842e577d4b893dc98aa1f33c5f7d2c864cd134",
        "no new Intl behavior",
        "does not close T23",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
    for task in [TASK_T02, TASK_T23] {
        assert!(task.contains("intl-locale-string-slot-dispatch.md"));
        assert!(task.contains("00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a"));
        assert!(task.contains("4/4"));
        assert!(task.contains("no new Intl behavior"));
    }
}
