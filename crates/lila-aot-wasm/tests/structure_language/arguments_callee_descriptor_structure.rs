const OBJECTS: &str = include_str!("../../src/objects.rs");
const DEFINE: &str = include_str!("../../src/objects/define_property.rs");
const BUILTIN: &str = include_str!("../../src/builtins/object/define_property.rs");
const ARGUMENTS: &str = include_str!("../../src/functions/arguments_object.rs");
fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}
fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}
fn ordered(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        cursor += source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing {marker}"))
            + marker.len();
    }
}

#[test]
fn arguments_callee_uses_the_same_six_field_validated_gc_descriptor() {
    let owner = compact(between(
        OBJECTS,
        "struct DescriptorFieldLocals {",
        "#[must_use = \"a completed descriptor",
    ));
    assert!(owner.contains("present:I32Local,value:ValueLocals,"));
    assert!(owner.contains("fields:[DescriptorFieldLocals;6],"));
    for (field, index) in [
        ("Value", 0),
        ("Writable", 1),
        ("Get", 2),
        ("Set", 3),
        ("Enumerable", 4),
        ("Configurable", 5),
    ] {
        assert!(owner.contains(&format!("DescriptorField::{field}=>{index},")));
    }
    assert!(owner.contains("fndefinition_descriptor(&self)->WasmDescriptor<'_>"));
    for field in ["Value", "Get", "Set"] {
        assert!(owner.contains(&format!(
            "{}:value(DescriptorField::{field})",
            field.to_ascii_lowercase()
        )));
    }
    for field in ["Writable", "Enumerable", "Configurable"] {
        let name = field.to_ascii_lowercase();
        assert!(owner.contains(&format!(
            "{name}:flag(DescriptorField::{field},self.{name})"
        )));
    }
    assert_eq!(owner.matches("Presence::Runtime{").count(), 2);
    assert!(owner.contains("value:&field.value,"));
    assert!(owner.contains("schema.release_i32_local(field.present,function)"));
    assert_eq!(owner.matches(".from_runtime_checked()").count(), 1);
    assert!(owner.contains("fnclear(self,"));
    assert!(owner.contains("field.value.clear(function);"));
    assert!(!owner.contains("derive("));
}

#[test]
fn arguments_callee_define_boundary_accepts_only_one_validated_descriptor() {
    let consumer = compact(between(
        DEFINE,
        "fn emit_arguments_define_own_property(",
        "pub(crate) fn emit_object_define_entry_validated(",
    ));
    assert!(consumer.contains("arguments:&GcLocal<crate::gc_types::ArgumentsObject>,key:&PropertyKeyLocals,incoming:&WasmDescriptor<'_>,result:&CompletionLocals,"));
    // A named callee goes through OrdinaryDefineOwnProperty; only indexed mapped
    // arguments need parameter-map synchronization.
    assert!(consumer.contains(
        "self.emit_ordinary_define_own_property(&header,key,incoming,result,function)?;"
    ));
    assert!(!consumer.contains("callee"));
    assert!(!DEFINE.contains("fn emit_arguments_define_callee("));
    let dispatch = compact(between(
        DEFINE,
        "fn emit_non_proxy_define_own_property(",
        "\n}\n",
    ));
    assert_eq!(dispatch.matches("self.emit_arguments_define_own_property(&arguments,key,incoming,result,function)?;").count(), 1);
}

#[test]
fn arguments_callee_definition_keeps_the_sole_complete_conversion_producer() {
    let convert = compact(between(
        OBJECTS,
        "pub(crate) fn emit_to_property_descriptor(",
        "fn emit_descriptor_conversion_abrupt_exit(",
    ));
    ordered(
        &convert,
        &[
            "ReservedPropertyDescriptorLocals{",
            "forfieldinTO_PROPERTY_DESCRIPTOR_ORDER{",
            "ObjectHasPropertyArguments::new(",
            "self.emit_descriptor_conversion_abrupt_exit(&pending,function);",
            "ObjectReadArguments::new(",
            "self.emit_descriptor_conversion_abrupt_exit(&pending,function);",
            "PROPERTY_DESCRIPTOR_GETTER_SETTER_MUST_BE_CALLABLE_OR_UNDEFINED",
            "PROPERTY_DESCRIPTOR_CANNOT_BE_BOTH_ACCESSOR_AND_DATA",
            "Ok(converted)",
        ],
    );
    let builtin = compact(BUILTIN);
    ordered(
        &builtin,
        &[
            "self.emit_value_to_property_key_locals(",
            "self.emit_to_property_descriptor(",
            "self.emit_object_define_entry_validated(",
            "&descriptor.definition_descriptor(),",
            "descriptor.clear(schema,function);",
        ],
    );
    assert_eq!(
        builtin.matches("self.emit_to_property_descriptor(").count(),
        1
    );
    assert_eq!(
        builtin
            .matches("self.emit_object_define_entry_validated(")
            .count(),
        1
    );
}

#[test]
fn initial_callee_kind_and_updates_share_canonical_protocol_and_classification() {
    let initial = compact(between(
        ARGUMENTS,
        "let callee_key =",
        "callee_key.clear(function);",
    ));
    ordered(
        &initial,
        &[
            "matchprotocol{",
            "PresentArgumentsObjectProtocol::Mapped(_)=>",
            "property.set_reference(&callable,schema,function);",
            "self.emit_object_append_data_property_with_flags(",
            "true,false,true,function,",
            "PresentArgumentsObjectProtocol::Unmapped(_)=>",
            "self.emit_load_required_function_realm_throw_type_error(",
            "self.emit_object_append_accessor_property_with_flags(",
            "false,false,function,",
        ],
    );
    assert!(!initial.contains("_=>"));
    let validation = compact(between(
        DEFINE,
        "pub(crate) fn emit_validate_stored_descriptor(",
        "fn emit_descriptor_reject_if_true(",
    ));
    assert_eq!(validation.matches("classify(incoming)").count(), 1);
    assert_eq!(
        validation
            .matches("classification.terms(DescriptorSide::Data)")
            .count(),
        1
    );
    assert_eq!(
        validation
            .matches("classification.terms(DescriptorSide::Accessor)")
            .count(),
        1
    );
    assert!(validation.contains("(DescriptorField::Get,&descriptor.get)"));
    assert!(validation.contains("(DescriptorField::Set,&descriptor.set)"));
}
