const MODULE_SOURCE: &str = include_str!("../src/module.rs");
const OWNER_SOURCE: &str = include_str!("../src/module/error_prototype_location.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOUND_ALLOCATION_SOURCE: &str = include_str!("../src/functions/bound_function_allocation.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn error_prototype_locations_have_one_private_file_owner() {
    assert_eq!(
        MODULE_SOURCE
            .matches("\nmod error_prototype_location;\n")
            .count(),
        1
    );
    assert!(!MODULE_SOURCE.contains("\npub mod error_prototype_location;\n"));
    assert!(!MODULE_SOURCE.contains("\nmod error_prototype_location {\n"));
    assert!(OWNER_SOURCE.starts_with("use super::*;\n\n"));

    let reexport = bounded(
        MODULE_SOURCE,
        "pub(crate) use error_prototype_location::{",
        "};",
    );
    for accessor in [
        "error_prototype_global_index",
        "error_realm_prototype_entries",
        "error_realm_prototype_offset",
    ] {
        assert_eq!(reexport.matches(accessor).count(), 1, "{accessor}");
    }

    assert!(!MODULE_SOURCE.contains("struct ErrorPrototypeLocation"));
    assert!(!MODULE_SOURCE.contains("const fn error_prototype_location("));
    assert_eq!(OWNER_SOURCE.matches("pub(crate) ").count(), 3);
    assert!(!OWNER_SOURCE.contains("pub struct ErrorPrototypeLocation"));
    assert!(!OWNER_SOURCE.contains("pub(crate) struct ErrorPrototypeLocation"));
}

#[test]
fn error_prototype_location_authority_is_exhaustive_and_single_sourced() {
    let authority = bounded(
        OWNER_SOURCE,
        "const fn error_prototype_location(kind: NativeErrorKind)",
        "pub(crate) const fn error_prototype_global_index",
    );
    for kind in [
        "Error",
        "EvalError",
        "RangeError",
        "ReferenceError",
        "SyntaxError",
        "TypeError",
        "URIError",
        "AggregateError",
        "SuppressedError",
    ] {
        assert_eq!(
            authority
                .matches(&format!("NativeErrorKind::{kind} =>"))
                .count(),
            1,
            "{kind}"
        );
    }
    assert!(!authority.contains("_ =>"));
    assert_eq!(authority.matches("global_index,").count(), 2);
    assert_eq!(authority.matches("realm_offset,").count(), 1);

    let projections = bounded(
        OWNER_SOURCE,
        "pub(crate) const fn error_prototype_global_index",
        "pub(crate) fn error_realm_prototype_entries",
    );
    assert_eq!(
        projections
            .matches("error_prototype_location(kind)")
            .count(),
        2
    );
    assert!(projections.contains(".global_index"));
    assert!(projections.contains(".realm_offset"));

    let entries = OWNER_SOURCE
        .split_once("pub(crate) fn error_realm_prototype_entries")
        .expect("realm prototype entries")
        .1;
    assert!(entries.contains("[(NativeErrorKind, u32, u64); 9]"));
    assert_eq!(entries.matches("NativeErrorKind::ALL.map").count(), 1);
    assert_eq!(entries.matches("error_prototype_location(kind)").count(), 1);
}

#[test]
fn error_prototype_location_accessors_keep_the_reviewed_caller_census() {
    assert_eq!(
        ERRORS_SOURCE
            .matches("error_prototype_global_index(")
            .count(),
        5
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("error_realm_prototype_offset(")
            .count(),
        2
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("error_realm_prototype_entries(")
            .count(),
        0
    );

    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("error_realm_prototype_entries(")
            .count(),
        1
    );
    assert_eq!(
        BOUND_ALLOCATION_SOURCE
            .matches("error_realm_prototype_entries(")
            .count(),
        1
    );
    for source in [FUNCTIONS_SOURCE, BOUND_ALLOCATION_SOURCE] {
        assert!(!source.contains("error_prototype_global_index("));
        assert!(!source.contains("error_realm_prototype_offset("));
        assert!(!source.contains("struct ErrorPrototypeLocation"));
    }
}
