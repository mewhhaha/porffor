const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/temporal-plain-time-field-authority.md");
const TASK: &str = include_str!("../../../../tasks/22-date-temporal.md");

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn contract_and_task_record_the_invariant_and_non_claim() {
    let normalized_contract = normalized(CONTRACT);
    let normalized_task = normalized(TASK);
    for evidence in [
        "TemporalTimeUnit",
        "record offset",
        "valid maximum",
        "property-bag read order",
        "does not change emitted Wasm",
    ] {
        let normalized_evidence = normalized(evidence);
        assert!(
            normalized_contract.contains(&normalized_evidence),
            "contract evidence `{evidence}`"
        );
        assert!(
            normalized_task.contains(&normalized_evidence),
            "task evidence `{evidence}`"
        );
    }
}
