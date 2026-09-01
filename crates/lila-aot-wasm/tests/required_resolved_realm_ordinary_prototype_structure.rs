use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OWNER_SOURCE: &str =
    include_str!("../src/functions/required_resolved_realm_ordinary_prototype.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn recursive_rust_source_count(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return recursive_rust_source_count(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn required_resolved_realm_ordinary_prototype_has_one_private_owner() {
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod required_resolved_realm_ordinary_prototype;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("\npub mod required_resolved_realm_ordinary_prototype;"));
    assert!(
        !FUNCTIONS_SOURCE.contains("\npub(crate) mod required_resolved_realm_ordinary_prototype;")
    );
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("required_resolved_realm_ordinary_prototype::")
            .count(),
        1
    );
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches(concat!(
                "pub(crate) use required_resolved_realm_ordinary_prototype::",
                "OrdinaryDefaultPrototype;"
            ))
            .count(),
        1
    );

    assert_eq!(
        OWNER_SOURCE
            .matches("pub(crate) enum OrdinaryDefaultPrototype {")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("enum OrdinaryDefaultPrototype {"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) struct ResolvedRealmOrdinaryPrototypeLocal(u32);")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("ResolvedRealmOrdinaryPrototypeLocal"));
    assert!(OWNER_SOURCE.contains(concat!(
        "#[must_use = \"the resolved-realm prototype must be installed with its ",
        "representation tag\"]\npub(super) struct ResolvedRealmOrdinaryPrototypeLocal(u32);"
    )));
    assert!(!OWNER_SOURCE.contains("impl Copy for ResolvedRealmOrdinaryPrototypeLocal"));
    assert_eq!(
        OWNER_SOURCE
            .matches("ResolvedRealmOrdinaryPrototypeLocal(prototype_local)")
            .count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("prototype.0").count(), 2);

    for (method, visibility) in [
        (
            "emit_load_required_resolved_realm_ordinary_prototype",
            "pub(super)",
        ),
        (
            "emit_required_new_target_realm_ordinary_prototype",
            "pub(crate)",
        ),
        (
            "emit_install_resolved_realm_ordinary_prototype",
            "pub(super)",
        ),
    ] {
        let definition = format!("{visibility} fn {method}(");
        assert_eq!(OWNER_SOURCE.matches(&definition).count(), 1, "{method}");
        assert!(!FUNCTIONS_SOURCE.contains(&definition), "{method}");
    }
}

#[test]
fn ordinary_default_prototype_domain_exhaustively_owns_every_offset() {
    let domain = bounded(
        OWNER_SOURCE,
        "pub(crate) enum OrdinaryDefaultPrototype {",
        "\n}\n\nimpl OrdinaryDefaultPrototype",
    );
    let offsets = bounded(
        OWNER_SOURCE,
        "impl OrdinaryDefaultPrototype {",
        "\n}\n\n/// A populated ordinary-object prototype",
    );
    assert_eq!(
        domain
            .lines()
            .filter(|line| line.trim_end().ends_with(','))
            .count(),
        9
    );
    for (variant, offset) in [
        ("Object", "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET"),
        ("MessageError(kind)", "kind.prototype_slot().offset()"),
        ("String", "HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET"),
        ("Number", "HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET"),
        ("Boolean", "HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET"),
        ("Date", "HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET"),
        (
            "Iterator",
            "HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET",
        ),
        ("RegExp", "HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET"),
        ("Promise", "HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET"),
    ] {
        assert_eq!(
            offsets
                .matches(&format!("Self::{variant} => {offset}"))
                .count(),
            1,
            "{variant}"
        );
    }
    assert!(!domain.contains("Array"));
    assert!(!offsets.contains("_ =>"));
}

#[test]
fn every_resolved_ordinary_prototype_is_loaded_and_installed_as_one_witness() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (call, expected) in [
        (".emit_load_required_resolved_realm_ordinary_prototype(", 5),
        (".emit_install_resolved_realm_ordinary_prototype(", 5),
        (".emit_required_new_target_realm_ordinary_prototype(", 3),
    ] {
        assert_eq!(
            recursive_rust_source_count(&source_root, call),
            expected,
            "unexpected recursive caller census for {call}"
        );
    }

    let construct = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_handle_construct_with_argv(",
        "    pub(crate) fn copy_function_realm_typed_array_prototypes(",
    );
    assert_eq!(
        construct
            .matches("self.emit_load_required_resolved_realm_ordinary_prototype(")
            .count(),
        4
    );
    assert_eq!(
        construct
            .matches("self.emit_install_resolved_realm_ordinary_prototype(")
            .count(),
        4
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("self.emit_required_new_target_realm_ordinary_prototype(")
            .count(),
        2
    );

    let required_new_target = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn emit_required_new_target_realm_ordinary_prototype(",
        "    /// Consume a required ordinary-object prototype",
    );
    for marker in [
        "self.emit_get_function_realm(",
        "self.emit_route_function_realm_result(",
        "FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn",
        "self.emit_load_required_resolved_realm_ordinary_prototype(",
        "self.emit_install_resolved_realm_ordinary_prototype(",
        "self.release_resolved_function_realm_local(realm)",
    ] {
        assert_eq!(required_new_target.matches(marker).count(), 1, "{marker}");
    }
}
