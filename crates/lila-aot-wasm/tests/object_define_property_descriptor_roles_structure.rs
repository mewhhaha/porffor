const OBJECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/object.rs");
const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const PROPERTY_DESCRIPTOR_SOURCE: &str = include_str!("../../lila-ir/src/property_descriptor.rs");
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_descriptor_core.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-define-property-descriptor-roles.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

#[test]
fn object_define_property_branches_form_one_closed_descriptor_domain() {
    let domain = bounded(
        DEFINE_PROPERTY_SOURCE,
        "enum ObjectDefinePropertyDescriptorLocals {",
        "struct ArgumentsCalleeDescriptorLocals {",
    );
    let declaration_prefix = DEFINE_PROPERTY_SOURCE
        .split_once("enum ObjectDefinePropertyDescriptorLocals {")
        .expect("descriptor role domain")
        .0
        .rsplit("\n\n")
        .next()
        .unwrap_or_default();

    for field in ["value", "getter", "setter"] {
        assert!(domain.contains(&format!("{field}: RuntimeDescriptorField<TaggedLocals>,")));
    }
    for field in ["writable", "enumerable", "configurable"] {
        assert!(domain.contains(&format!("{field}: RuntimeDescriptorField<u32>,")));
    }
    assert_eq!(domain.matches("Self::Data {").count(), 1);
    assert_eq!(domain.matches("Self::Accessor {").count(), 1);
    assert_eq!(domain.matches("Presence::Runtime {").count(), 8);
    assert_eq!(domain.matches("Presence::Absent").count(), 4);
    assert_eq!(domain.matches(".validate()").count(), 1);
    assert!(!domain.contains(".from_runtime_checked()"));
    assert!(!domain.contains("Option<"));
    assert!(!domain.contains(": bool"));
    assert!(!domain.contains("_ =>"));
    assert!(!declaration_prefix.contains("#[derive"));
}

#[test]
fn both_define_property_producers_name_their_exact_descriptor_kind() {
    let define_property = bounded(
        DEFINE_PROPERTY_SOURCE,
        "    pub(in crate::builtins) fn compile_object_define_property_builtin(",
        "\n}",
    );
    let accessor = bounded(
        define_property,
        "let descriptor = ObjectDefinePropertyDescriptorLocals::Accessor {",
        "self.emit_object_define_entry_validated(",
    );
    let data = bounded(
        define_property,
        "let descriptor = ObjectDefinePropertyDescriptorLocals::Data {",
        "self.emit_object_define_entry_validated(",
    );

    for field in ["getter", "setter", "enumerable", "configurable"] {
        assert!(accessor.contains(&format!("{field}: RuntimeDescriptorField {{")));
    }
    assert!(!accessor.contains("value: RuntimeDescriptorField {"));
    assert!(!accessor.contains("writable: RuntimeDescriptorField {"));
    for field in ["value", "writable", "enumerable", "configurable"] {
        assert!(data.contains(&format!("{field}: RuntimeDescriptorField {{")));
    }
    assert!(!data.contains("getter: RuntimeDescriptorField {"));
    assert!(!data.contains("setter: RuntimeDescriptorField {"));
    assert_eq!(
        define_property.matches(".validated_descriptor();").count(),
        2
    );
    assert_eq!(
        define_property
            .matches("self.emit_object_define_entry_validated(")
            .count(),
        2
    );
}

#[test]
fn positional_descriptor_adapter_and_runtime_checked_escape_are_retired() {
    assert!(!OBJECTS_SOURCE.contains("pub(crate) fn emit_object_define_entry("));
    assert!(!OBJECTS_SOURCE.contains("presence_from_positional"));
    assert!(!DEFINE_PROPERTY_SOURCE.contains("self.emit_object_define_entry("));
    assert_eq!(
        DEFINE_PROPERTY_SOURCE
            .matches(".from_runtime_checked()")
            .count(),
        1
    );
    assert!(!PROPERTY_DESCRIPTOR_SOURCE.contains("`emit_object_define_entry` — the"));
    assert!(!PROPERTY_DESCRIPTOR_SOURCE.contains("two `Object.defineProperty` sites"));
}

#[test]
fn define_property_family_has_one_private_module_owner() {
    assert_eq!(
        OBJECT_BUILTIN_SOURCE
            .matches("mod define_property;")
            .count(),
        1
    );
    assert!(!OBJECT_BUILTIN_SOURCE.contains("pub mod define_property;"));
    for marker in [
        "enum ObjectDefinePropertyDescriptorLocals {",
        "struct ArgumentsCalleeDescriptorLocals {",
        "fn emit_arguments_define_index_descriptor(",
        "fn emit_arguments_define_callee(",
        "fn compile_object_define_property_builtin(",
    ] {
        assert!(!OBJECT_BUILTIN_SOURCE.contains(marker));
        assert_eq!(DEFINE_PROPERTY_SOURCE.matches(marker).count(), 1);
    }
}

#[test]
fn focused_descriptor_behavior_and_evidence_remain_in_inventory() {
    assert!(
        CLI_SOURCE.contains("run_wasm_backend_succeeds_for_supported_object_descriptor_fixture")
    );
    for witness in [
        "mixedRejected",
        "undefDataDesc",
        "undefGetterDesc",
        "redefineDataDesc",
        "redefineAccessorDesc",
        "rejectAccessorToData",
    ] {
        assert!(
            CLI_FIXTURE.contains(witness),
            "missing fixture witness `{witness}`"
        );
    }
    assert!(CONTRACT.contains("ObjectDefinePropertyDescriptorLocals::{Data, Accessor}"));
    assert!(TASK.contains("ObjectDefinePropertyDescriptorLocals::{Data, Accessor}"));
}
