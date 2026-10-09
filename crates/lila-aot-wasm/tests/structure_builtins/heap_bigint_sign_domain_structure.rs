const LAYOUTS: &str = include_str!("../../src/gc_types/layouts.rs");
const SNAPSHOT: &str = include_str!("../../../lila-engine/src/wasm_rooted_snapshot.rs");
const DECIMAL: &str = include_str!("../../../lila-engine/src/wasm_rooted_snapshot/bigint.rs");
fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}
#[test]
fn bigint_sign_is_the_exact_gc_boolean_domain() {
    let layout = bounded(
        LAYOUTS,
        "struct BigIntValue => BigIntValueSchema {",
        "        }",
    );
    assert!(layout.contains("NEGATIVE: bool, Immutable, NonNullable;"));
    let read = bounded(SNAPSHOT, "    fn boolean(", "    fn reference(");
    assert!(read.contains("0 => Ok(false)"));
    assert!(read.contains("1 => Ok(true)"));
    assert!(read.contains(r#"_ => Err(invariant("noncanonical Boolean field"))"#));
}
#[test]
fn bigint_observation_reads_the_sign_once_and_rejects_noncanonical_zero() {
    let read = bounded(SNAPSHOT, "    fn bigint(", "    fn descriptor(");
    assert_eq!(
        read.matches("self.boolean(reference, F::BigIntSign)?")
            .count(),
        1
    );
    assert!(read.contains("bigint::decimal(negative, limbs, budget)"));
    let decimal = bounded(DECIMAL, "pub(super) fn decimal(", "#[cfg(test)]");
    assert!(decimal.contains("negative: bool,"));
    assert!(decimal.contains("limbs.last() == Some(&0) || (limbs.is_empty() && negative)"));
    assert!(decimal.contains(r#"return Err(invariant("noncanonical BigInt"))"#));
    assert!(decimal.contains("if negative {"));
    assert!(decimal.contains("decimal.push('-')"));
}
#[test]
fn bigint_observation_retains_bounded_complete_magnitude_and_rejection_controls() {
    for marker in [
        "budget.work(limbs.len())?",
        "max_digits.div_ceil(9)",
        "budget.take_bigint(decimal)",
    ] {
        assert!(DECIMAL.contains(marker), "lost {marker}");
    }
    for marker in [
        "fn sign_and_cross_limb_magnitude_fit_the_exact_decimal_budget()",
        "fn invalid_magnitude_and_budget_failure_cannot_publish_partial_decimal()",
        "decimal(true, vec![]",
        "decimal(false, vec![1, 0]",
    ] {
        assert!(DECIMAL.contains(marker), "lost {marker}");
    }
}
