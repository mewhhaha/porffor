const HEAP: &str = include_str!("../src/heap.rs");
const BOOTSTRAP: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST: &str = include_str!("../src/builtins/host.rs");
const CREATED: &str = include_str!("../src/builtins/host/created_realm_temporal_intrinsics.rs");
const ENTRY: &str = include_str!("../src/intrinsics/temporal.rs");
const MEMBERS: &str = include_str!("../src/intrinsics/temporal/members.rs");
const REALM: &str = include_str!("../src/intrinsics/temporal/realm.rs");

#[test]
fn both_realms_install_the_same_instant_and_duration_members() {
    assert_eq!(
        ENTRY
            .matches("self.emit_install_temporal_intrinsic_members(")
            .count(),
        2
    );
    assert_eq!(
        CREATED
            .matches("self.emit_install_temporal_intrinsic_members(")
            .count(),
        1
    );
    assert!(CREATED.contains("TemporalIntrinsicFamily::ALL"));
    assert!(!CREATED.contains("continue;"));
    assert!(!CREATED.contains("GlobalSet("));
    assert!(MEMBERS.contains("family.constructor_methods()"));
    assert!(MEMBERS.contains("family.getters()"));
    assert!(MEMBERS.contains("family.methods()"));
    assert!(MEMBERS.contains("TemporalIntrinsicRealm::Created(context)"));
    assert!(MEMBERS.contains("emit_function_value_payload_in_realm("));
    assert!(MEMBERS.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
    for family in ["Instant", "Duration"] {
        assert!(MEMBERS.contains(&format!(
            "Self::{family} => NonArrayRealmIntrinsicSlot::Temporal{family}Prototype"
        )));
        assert!(HEAP.contains(&format!("name: \"%Temporal.{family}.prototype%\"")));
        assert!(BOOTSTRAP.contains(&format!(
            "NonArrayRealmIntrinsicSlot::Temporal{family}Prototype"
        )));
    }
}

#[test]
fn result_prototypes_are_required_active_realm_intrinsics() {
    for required in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "family.prototype_slot().offset()",
        "Instruction::Unreachable",
    ] {
        assert!(REALM.contains(required));
    }
    assert!(!REALM.contains("emit_object_read("));
    assert!(!REALM.contains("emit_load_realm_intrinsic_prototype_or_global("));
    assert!(CREATED.contains("#[must_use"));
    assert!(CREATED.contains("intrinsics: CreatedRealmTemporalIntrinsics,"));
    let materialize = HOST.find("let created_realm_temporal =").unwrap();
    let publish = HOST
        .find("self.emit_publish_created_realm_temporal_intrinsics(")
        .unwrap();
    let intl_materialize = HOST.find("let created_realm_intl =").unwrap();
    let intl_publish = HOST
        .find("self.emit_publish_created_realm_intl_intrinsics(")
        .unwrap();
    assert!(
        materialize < intl_materialize && intl_materialize < intl_publish && intl_publish < publish
    );
}
