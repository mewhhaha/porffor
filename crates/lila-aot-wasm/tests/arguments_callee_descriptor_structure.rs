const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const PROPERTY_DESCRIPTOR_SOURCE: &str = include_str!("../../lila-ir/src/property_descriptor.rs");

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
fn arguments_callee_descriptor_has_one_exact_runtime_presence_shape() {
    let domain = bounded(
        DEFINE_PROPERTY_SOURCE,
        "struct ArgumentsCalleeDescriptorLocals {",
        "impl<'a> FunctionBuilder<'a> {",
    );

    for field in ["value", "get", "set"] {
        assert!(domain.contains(&format!("{field}: RuntimeDescriptorField<TaggedLocals>,")));
    }
    for field in ["writable", "enumerable", "configurable"] {
        assert!(domain.contains(&format!("{field}: RuntimeDescriptorField<u32>,")));
    }
    assert_eq!(domain.matches("Presence::Runtime {").count(), 6);
    assert_eq!(domain.matches(".from_runtime_checked()").count(), 1);
    assert!(!DEFINE_PROPERTY_SOURCE.contains("pub struct RuntimeDescriptorField"));
    assert!(!domain.contains("pub struct ArgumentsCalleeDescriptorLocals"));
    assert!(PROPERTY_DESCRIPTOR_SOURCE
        .contains("`ArgumentsCalleeDescriptorLocals::validated_descriptor` — its sole"));
}

#[test]
fn arguments_callee_define_boundary_accepts_one_validated_descriptor() {
    let consumer = bounded(
        DEFINE_PROPERTY_SOURCE,
        "    fn emit_arguments_define_callee(",
        "    fn emit_store_arguments_length_descriptor_kind(",
    );
    let signature = consumer
        .split_once(") -> Result<(), EmitError> {")
        .expect("arguments callee descriptor signature")
        .0;

    assert!(signature.contains("arguments_local: u32,"));
    assert!(signature.contains("descriptor: ArgumentsCalleeDescriptorLocals,"));
    assert!(signature.contains("function: &mut Function,"));
    for forbidden in [
        "value_payload_local: u32,",
        "getter_payload_local: u32,",
        "setter_payload_local: u32,",
        "writable_payload_local: u32,",
        "value_present_local: u32,",
        "getter_present_local: u32,",
        "setter_present_local: u32,",
        "writable_present_local: u32,",
        "enumerable_present_local: u32,",
        "configurable_present_local: u32,",
    ] {
        assert!(
            !signature.contains(forbidden),
            "legacy positional callee descriptor parameter `{forbidden}` remains"
        );
    }
}

#[test]
fn arguments_callee_define_has_one_complete_descriptor_producer() {
    assert_eq!(
        DEFINE_PROPERTY_SOURCE
            .matches("self.emit_arguments_define_callee(")
            .count(),
        1
    );

    let define_property = bounded(
        DEFINE_PROPERTY_SOURCE,
        "    pub(in crate::builtins) fn compile_object_define_property_builtin(",
        "\n}",
    );
    let producer = bounded(
        define_property,
        "        function.instruction(&Instruction::I64Const(self.strings.payload(\"callee\")));",
        "        function.instruction(&Instruction::Else);",
    );
    assert_eq!(
        producer
            .matches("let descriptor = ArgumentsCalleeDescriptorLocals {")
            .count(),
        1
    );
    assert_eq!(producer.matches("RuntimeDescriptorField {").count(), 6);
    assert!(producer.contains("value: TaggedLocals::new(value_payload_local, value_tag_local),"));
    assert!(producer.contains("value: TaggedLocals::new(getter_payload_local, getter_tag_local),"));
    assert!(producer.contains("value: TaggedLocals::new(setter_payload_local, setter_tag_local),"));
    assert!(producer.contains(
        "self.emit_arguments_define_callee(target_payload_local, descriptor, function)?;"
    ));
}

#[test]
fn arguments_callee_kind_selection_comes_only_from_descriptor_classification() {
    let consumer = bounded(
        DEFINE_PROPERTY_SOURCE,
        "    fn emit_arguments_define_callee(",
        "    fn emit_store_arguments_length_descriptor_kind(",
    );

    assert_eq!(
        consumer
            .matches("let validated_descriptor = descriptor.validated_descriptor();")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("let classification = classify(&validated_descriptor);")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("classification.terms(DescriptorSide::Data)")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("classification.terms(DescriptorSide::Accessor)")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("Self::emit_array_descriptor_side_present_to_local(")
            .count(),
        2
    );
    assert!(!consumer.contains("descriptor.into_partial()"));
    assert!(!consumer.contains("unreachable!"));

    let before_classification = consumer
        .split_once("let value_present_local = descriptor.value.present;")
        .expect("exact-shape descriptor field projection")
        .0;
    for forbidden in [
        "LocalGet(getter_present_local)",
        "LocalGet(setter_present_local)",
        "LocalGet(value_present_local)",
        "LocalGet(writable_present_local)",
    ] {
        assert!(
            !before_classification.contains(forbidden),
            "callee kind was re-derived from `{forbidden}` before canonical classification"
        );
    }
}
