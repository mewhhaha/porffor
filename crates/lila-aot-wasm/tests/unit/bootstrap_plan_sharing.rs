use super::*;

#[test]
fn full_plans_share_the_same_closed_catalogue_across_threads() {
    let first = RuntimeBootstrapPlan::full();
    let second = std::thread::spawn(RuntimeBootstrapPlan::full)
        .join()
        .unwrap();
    assert!(Arc::ptr_eq(&first.shared, &second.shared));
    let uncached = RuntimeBootstrapPlan::build_full();
    assert_eq!(first.standard_roots, uncached.standard_roots);
    assert_eq!(first.walked, uncached.walked);
    assert!(first.full_standard_globals);
    assert!(first.intl_namespace_members().is_some());
    assert!(first.temporal_namespace_members().is_some());
}

#[test]
fn partial_plan_mutation_is_independent_of_aliases_and_the_full_cache() {
    let mut original = RuntimeBootstrapPlan::default();
    original.require_standard_builtin(StandardBuiltinId::MathSin);
    let mut changed = original.clone();
    assert!(Arc::ptr_eq(&original.shared, &changed.shared));
    changed.require_standard_builtin(StandardBuiltinId::MathCos);
    assert!(original
        .standard_roots
        .contains(&StandardBuiltinId::MathSin));
    assert!(!original
        .standard_roots
        .contains(&StandardBuiltinId::MathCos));
    assert!(changed.standard_roots.contains(&StandardBuiltinId::MathCos));
    assert!(!Arc::ptr_eq(&original.shared, &changed.shared));
    let full = RuntimeBootstrapPlan::full();
    let mut independent = full.clone();
    independent.full_standard_globals = false;
    assert!(full.full_standard_globals);
    assert!(RuntimeBootstrapPlan::full().full_standard_globals);
    assert!(!independent.full_standard_globals);
}
