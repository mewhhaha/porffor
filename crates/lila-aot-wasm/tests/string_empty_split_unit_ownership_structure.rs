const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/string-empty-split-code-unit-walk.md");
const TASK: &str = include_str!("../../../tasks/18-strings-unicode.md");

#[test]
fn contract_and_task_record_the_ownership_boundary_without_a_runtime_claim() {
    for marker in [
        "None of the three local domains implements `Clone` or `Copy`.",
        "`emit_one_unit_payload` borrows the index and one-unit width",
        "source-equivalent and the existing empty-separator fixture was not rerun",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
    }
    for marker in [
        "private index, length and one-unit local domains now derive",
        "no incidental capabilities",
        "source-equivalent ownership closure",
        "no runtime or conformance claim",
    ] {
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
