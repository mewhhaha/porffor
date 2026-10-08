const METHODS_SOURCE: &str = include_str!("../src/builtins/temporal_zoned_date_time_methods.rs");
const MOD_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/temporal-zoned-date-time-direction-dispatch.md"
);
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T22: &str = include_str!("../../../tasks/22-date-temporal.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn zoned_date_time_catalog_routes_use_four_fixed_family_entries() {
    for (method, domain, variant, raw_emitter) in [
        (
            "add",
            "TemporalZonedArithmeticOperation",
            "Add",
            "emit_temporal_zoned_date_time_add_or_subtract",
        ),
        (
            "subtract",
            "TemporalZonedArithmeticOperation",
            "Subtract",
            "emit_temporal_zoned_date_time_add_or_subtract",
        ),
        (
            "until",
            "TemporalZonedDifferenceOperation",
            "Until",
            "emit_temporal_zoned_date_time_until_or_since",
        ),
        (
            "since",
            "TemporalZonedDifferenceOperation",
            "Since",
            "emit_temporal_zoned_date_time_until_or_since",
        ),
    ] {
        let fixed_entry = format!("pub(super) fn emit_temporal_zoned_date_time_{method}_builtin(");
        let fixed_entry_body = METHODS_SOURCE
            .split_once(&fixed_entry)
            .unwrap_or_else(|| panic!("missing fixed {method} entry"))
            .1
            .split_once("\n    }")
            .expect("fixed entry end")
            .0;
        let standard_route =
            format!("self.emit_temporal_zoned_date_time_{method}_builtin(function)?;");
        assert_eq!(METHODS_SOURCE.matches(&fixed_entry).count(), 1);
        assert_eq!(
            fixed_entry_body
                .matches(&format!("{domain}::{variant}"))
                .count(),
            1
        );
        assert_eq!(STANDARD_SOURCE.matches(&standard_route).count(), 1);
        assert!(!STANDARD_SOURCE.contains(raw_emitter));
    }

    assert_eq!(
        METHODS_SOURCE
            .matches("fn emit_temporal_zoned_date_time_add_or_subtract(")
            .count(),
        1
    );
    assert_eq!(
        METHODS_SOURCE
            .matches("fn emit_temporal_zoned_date_time_until_or_since(")
            .count(),
        1
    );
    assert!(
        !METHODS_SOURCE.contains("pub(crate) fn emit_temporal_zoned_date_time_add_or_subtract(")
    );
    assert!(!METHODS_SOURCE.contains("pub(crate) fn emit_temporal_zoned_date_time_until_or_since("));
    assert!(!STANDARD_SOURCE.contains("TemporalZonedArithmeticOperation"));
    assert!(!STANDARD_SOURCE.contains("TemporalZonedDifferenceOperation"));
    assert!(!MOD_SOURCE.contains("pub(crate) use temporal_zoned_date_time_methods"));
}

#[test]
fn zoned_date_time_dispatch_contract_records_exact_witnesses_and_nonclaims() {
    for marker in [
        "shared closed direction domains",
        "four fixed entries",
        "82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0",
        "0df4c7b1b768c8520b30f505c8d5c5f6e18d1a8dbee0dff7b08149f2aa3bbde2",
        "8c95229bd602e45445a7c6ad5e2a89b3d120b903be74b73ac185782859d73cdf",
        "historical closure introduced no new Temporal behavior",
        "staged and unexecuted",
        "does not close T22",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
    for task in [TASK_T02, TASK_T22] {
        assert!(task.contains("temporal-zoned-date-time-direction-dispatch.md"));
        assert!(task.contains("82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0"));
        assert!(task.contains("3/3"));
        assert!(task.contains("no new Temporal behavior"));
    }
}
