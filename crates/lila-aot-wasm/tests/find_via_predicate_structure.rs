const REGRESSIONS: &str = include_str!("../../lila-engine/tests/aot_array_find.rs");

#[test]
fn complete_regression_inventory_targets_the_product_backend() {
    assert_eq!(REGRESSIONS.matches("#[test]").count(), 24);
    assert!(!REGRESSIONS.contains("#[ignore"));
    assert!(REGRESSIONS.contains("backend: ExecutionBackend::WasmAot"));
    for scenario in [
        "borrowed_typed_array_observes_own_and_inherited_length",
        "arguments_length_getter_and_coercion_are_observable",
        "global_object_is_a_valid_generic_receiver",
        "proxy_receivers_get_every_index_without_has_property",
        "all_eight_methods_keep_predicates_alive_and_return_no_match_sentinels",
        "huge_lengths_do_not_trap_or_truncate_reverse_indices",
        "strict_typed_array_validation_remains_separate_from_generic_length",
    ] {
        assert!(REGRESSIONS.contains(scenario));
    }
}
