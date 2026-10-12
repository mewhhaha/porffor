const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/prepared-destructuring-property-key-ownership.md"
);

#[test]
fn contract_task_and_existing_semantic_witness_name_the_boundary() {
    for text in [CONTRACT] {
        assert!(text.contains("PreparedDestructuringPropertyKey"));
        assert!(text.contains("prepared_destructuring_property_key_ownership_structure"));
    }
    assert!(CONTRACT
        .contains("run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions"));
    assert!(CONTRACT.contains("source-equivalent"));
}
