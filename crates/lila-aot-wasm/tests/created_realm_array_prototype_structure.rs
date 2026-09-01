use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OWNER_SOURCE: &str = include_str!("../src/functions/created_realm_array_prototype.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");

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
fn created_realm_array_prototype_lifecycle_has_one_private_owner() {
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod created_realm_array_prototype;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("\npub mod created_realm_array_prototype;\n"));
    assert!(!FUNCTIONS_SOURCE.contains("created_realm_array_prototype::"));

    for state in [
        "ReservedRealmArrayPrototypeLocal",
        "RealmArrayPrototypeLocal",
    ] {
        let declaration = format!("pub(crate) struct {state}");
        assert_eq!(OWNER_SOURCE.matches(&declaration).count(), 1);
        assert!(!FUNCTIONS_SOURCE.contains(&declaration));
        for capability in ["Clone", "Copy"] {
            assert!(!OWNER_SOURCE.contains(&format!("impl {capability} for {state}")));
        }
    }
    assert!(!OWNER_SOURCE.contains("#[derive("));
    assert_eq!(
        OWNER_SOURCE
            .matches("ReservedRealmArrayPrototypeLocal(self.reserve_temp_local())")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("RealmArrayPrototypeLocal(reserved.0)")
            .count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("reserved.0").count(), 4);
    assert_eq!(OWNER_SOURCE.matches("prototype.0").count(), 5);

    for method in [
        "reserve_realm_array_prototype_local",
        "emit_initialize_realm_array_prototype",
        "emit_store_realm_array_prototype",
        "emit_define_realm_array_prototype_data_with_flags",
        "emit_bind_realm_array_constructor_prototype",
        "release_realm_array_prototype_local",
    ] {
        let definition = format!("pub(crate) fn {method}(");
        assert_eq!(OWNER_SOURCE.matches(&definition).count(), 1, "{method}");
        assert!(!FUNCTIONS_SOURCE.contains(&definition), "{method}");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (call, expected) in [
        ("self.reserve_realm_array_prototype_local(", 1),
        ("self.emit_initialize_realm_array_prototype(", 1),
        ("self.emit_store_realm_array_prototype(", 1),
        ("self.emit_define_realm_array_prototype_data_with_flags(", 3),
        ("self.emit_bind_realm_array_constructor_prototype(", 1),
        ("self.release_realm_array_prototype_local(", 1),
    ] {
        assert_eq!(
            recursive_rust_source_count(&source_root, call),
            expected,
            "unexpected recursive caller census for {call}"
        );
    }
}

#[test]
fn initialization_and_constructor_linking_consume_only_typed_states() {
    let initialize = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn emit_initialize_realm_array_prototype(",
        "    pub(crate) fn emit_store_realm_array_prototype(",
    );
    for marker in [
        "Instruction::I64Const(0)",
        "emit_alloc_array_payload_with_length(length_local, reserved.0, function)?",
        "HEAP_PROTOTYPE_OFFSET",
        "HEAP_ARRAY_PROTOTYPE_TAG_OFFSET",
        "ValueKind::Object.tag() as u64",
        "Ok(RealmArrayPrototypeLocal(reserved.0))",
    ] {
        assert!(
            initialize.contains(marker),
            "missing initializer marker: {marker}"
        );
    }

    let bind = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn emit_bind_realm_array_constructor_prototype(",
        "    pub(crate) fn release_realm_array_prototype_local(",
    );
    assert!(bind.contains("HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET"));
    assert!(bind.contains("HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET"));
    assert!(bind.contains("ValueKind::Array.tag() as i64"));
    assert!(bind.contains("ValueKind::Function.tag() as i64"));
    assert!(bind.contains(concat!(
        "constructor_local,\n            key_local,\n            prototype.0,\n            tag_local,",
        "\n            false,\n            false,\n            false,"
    )));
    assert!(bind.contains(concat!(
        "prototype,\n            \"constructor\",\n            constructor_local,\n            tag_local,",
        "\n            true,\n            false,\n            true,"
    )));
}

#[test]
fn created_realm_bootstrap_owns_the_only_complete_lifecycle() {
    let create_realm = bounded(
        HOST_SOURCE,
        "    pub(crate) fn compile_host_create_realm_builtin(",
        "    pub(crate) fn compile_host_realm_eval_script_builtin(",
    );
    let reserve = create_realm
        .find("let array_prototype_slot = self.reserve_realm_array_prototype_local()")
        .expect("reserved Array prototype storage");
    let initialize = create_realm
        .find("let array_prototype = self.emit_initialize_realm_array_prototype(")
        .expect("initialized Array prototype");
    let store = create_realm
        .find("self.emit_store_realm_array_prototype(")
        .expect("published Array prototype intrinsic");
    let bind = create_realm
        .find("self.emit_bind_realm_array_constructor_prototype(")
        .expect("bound Array constructor/prototype links");
    let release = create_realm
        .find("self.release_realm_array_prototype_local(array_prototype)")
        .expect("released Array prototype witness");
    assert!(reserve < initialize);
    assert!(initialize < store);
    assert!(store < bind);
    assert!(bind < release);
    assert_eq!(
        create_realm
            .matches("self.emit_define_realm_array_prototype_data_with_flags(")
            .count(),
        2
    );
}
