const HEAP: &str = include_str!("../src/heap.rs");
const FUNCTIONS: &str = include_str!("../src/functions.rs");
const REQUIRED: &str =
    include_str!("../src/functions/required_resolved_realm_ordinary_prototype.rs");
const BOOTSTRAP: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST: &str = include_str!("../src/builtins/host.rs");
const CREATED: &str = include_str!("../src/builtins/host/created_realm_intl_intrinsics.rs");
const PROPERTIES: &str = include_str!("../src/intrinsics/intl.rs");
const LOCALE: &str = include_str!("../src/builtins/intl/construction_lifecycle.rs");
const NUMBER_FORMATTER: &str =
    include_str!("../src/builtins/intl_numberformat/construction_lifecycle.rs");
const FORMATTER: &str =
    include_str!("../src/builtins/intl_datetimeformat/construction_lifecycle.rs");

#[test]
fn intl_default_prototypes_are_rooted_realm_intrinsics() {
    for (variant, constant, name) in [
        ("IntlLocale", "INTL_LOCALE", "Intl.Locale"),
        (
            "IntlDateTimeFormat",
            "INTL_DATE_TIME_FORMAT",
            "Intl.DateTimeFormat",
        ),
        (
            "IntlNumberFormat",
            "INTL_NUMBER_FORMAT",
            "Intl.NumberFormat",
        ),
    ] {
        let offset = format!("HEAP_REALM_INTRINSICS_{constant}_PROTOTYPE_OFFSET");
        assert!(HEAP.contains(&format!("name: \"%{name}.prototype%\",\n        offset: {offset},\n        width: 8,\n        pointer: true,")));
        for (source, projection) in [
            (FUNCTIONS, format!("Self::{variant}Prototype")),
            (REQUIRED, format!("Self::{variant}")),
        ] {
            let compact = source.split_whitespace().collect::<String>();
            assert!(
                compact.contains(&format!("{projection}=>{offset}"))
                    || compact.contains(&format!("{projection}=>{{{offset}}}"))
            );
        }
        assert!(BOOTSTRAP.contains(&format!("{constant}_PROTOTYPE_GLOBAL_INDEX,\n            NonArrayRealmIntrinsicSlot::{variant}Prototype,")));
        assert!(PROPERTIES.contains(&format!(
            "prototype_slot: NonArrayRealmIntrinsicSlot::{variant}Prototype,"
        )));
    }
    assert!(CREATED.contains("self.emit_store_non_array_realm_intrinsic("));
    assert!(CREATED.contains("properties.prototype_slot,"));
    assert!(!CREATED.contains("GlobalSet("));
    for constructor in [LOCALE, FORMATTER, NUMBER_FORMATTER] {
        assert!(constructor.contains("NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary("));
        assert!(!constructor.contains("NewTargetPrototypeFallback::CurrentGlobal"));
        assert!(!constructor.contains("NewTargetPrototypeFallback::RealmIntrinsic"));
    }
}

#[test]
fn created_intl_uses_complete_membership_and_shared_property_definitions() {
    assert_eq!(
        HOST.matches("mod created_realm_intl_intrinsics;").count(),
        1
    );
    assert_eq!(
        HOST.matches("self.emit_materialize_created_realm_intl_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        HOST.matches("self.emit_publish_created_realm_intl_intrinsics(")
            .count(),
        1
    );
    for source in [BOOTSTRAP, CREATED] {
        assert!(source.contains("members: IntlNamespaceMembers,"));
        assert!(source.contains("members.in_installation_order()"));
    }
    assert!(CREATED.contains("intl_constructor_properties(builtin)"));
    assert!(PROPERTIES.contains("intl_constructor_properties(context.builtin)"));
    for source in [PROPERTIES, CREATED] {
        assert!(source.contains("properties.constructor"));
        assert!(source.contains("properties.prototype"));
        assert!(source.contains("self.emit_define_intl_intrinsic_function_property("));
    }
    assert!(!CREATED.contains("should_initialize_standard_builtin"));
    assert!(!CREATED.contains("continue;"));
    assert_eq!(
        CREATED
            .matches("emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert!(!CREATED.contains("self.emit_function_value_payload("));
    assert!(CREATED.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
}

#[test]
fn created_intl_namespace_local_is_published_before_older_retained_locals() {
    let created = HOST
        .find("let created_realm_intl =")
        .expect("Intl allocation");
    let weak_created = HOST
        .find("let created_realm_weak_collections =")
        .expect("older allocation");
    let publish = HOST
        .find("self.emit_publish_created_realm_intl_intrinsics(")
        .expect("Intl publication");
    let weak_publish = HOST
        .find("self.emit_publish_created_realm_weak_collection_intrinsics(")
        .expect("older publication");
    assert!(weak_created < created && created < publish && publish < weak_publish);
    assert!(CREATED.contains("#[must_use = \"created-Realm Intl intrinsics must be published\"]"));
    assert!(CREATED.contains("struct CreatedRealmIntlIntrinsics(u32);"));
    assert!(CREATED.contains("intrinsics: CreatedRealmIntlIntrinsics,"));
}
