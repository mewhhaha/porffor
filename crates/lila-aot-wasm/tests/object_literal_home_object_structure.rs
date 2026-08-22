const FUNCTION_PROTOCOL_SOURCE: &str = include_str!("../../lila-ir/src/function_protocol.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const LOWERING_HELPERS_SOURCE: &str = include_str!("../../lila-ir/src/lowering_helpers.rs");
const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_literal_home_object.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-literal-home-object.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 5] = [
    witness!("language/expressions/object/method.js"),
    witness!("language/expressions/object/method-definition/name-super-prop-body.js"),
    witness!("language/expressions/object/method-definition/name-super-prop-param.js"),
    witness!("language/expressions/object/getter-super-prop.js"),
    witness!("language/expressions/object/setter-super-prop.js"),
];

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_index = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier marker: {earlier}"));
    let later_index = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later marker: {later}"));
    assert!(
        earlier_index < later_index,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn object_method_protocol_and_private_carrier_close_the_ir_domain() {
    let protocol = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "pub enum FunctionProtocolIr {",
        "impl FunctionProtocolIr {",
    );
    for marker in [
        "ObjectMethod(FunctionExecutionKind)",
        "ObjectGetter",
        "ObjectSetter",
    ] {
        assert!(
            protocol.contains(marker),
            "missing protocol variant: {marker}"
        );
    }

    let flavor = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub const fn flavor(self)",
        "    pub const fn execution_kind(self)",
    );
    let execution = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub const fn execution_kind(self)",
        "    pub const fn is_constructable(self)",
    );
    let constructability = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub const fn is_constructable(self)",
        "    pub const fn class_kind(self)",
    );
    let class = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub const fn class_kind(self)",
        "    pub const fn is_object_literal_method(self)",
    );
    for query in [flavor, class] {
        for marker in [
            "Self::ObjectMethod(_)",
            "Self::ObjectGetter",
            "Self::ObjectSetter",
        ] {
            assert!(query.contains(marker), "missing protocol query: {marker}");
        }
        assert!(!query.contains("_ =>"));
    }
    for marker in [
        "Self::ObjectMethod(kind) => kind",
        "Self::ObjectGetter",
        "Self::ObjectSetter",
    ] {
        assert!(
            execution.contains(marker),
            "missing execution-kind query: {marker}"
        );
    }
    assert!(!execution.contains("_ =>"));
    assert!(!constructability.contains("Self::ObjectMethod"));
    assert!(!constructability.contains("Self::ObjectGetter"));
    assert!(!constructability.contains("Self::ObjectSetter"));
    let object_role = bounded(
        FUNCTION_PROTOCOL_SOURCE,
        "    pub const fn is_object_literal_method(self)",
        "\n    }\n}",
    );
    for marker in [
        "Self::ObjectMethod(_)",
        "Self::ObjectGetter",
        "Self::ObjectSetter",
    ] {
        assert!(
            object_role.contains(marker),
            "missing object-role query: {marker}"
        );
    }

    let carrier = bounded(
        IR_SOURCE,
        "/// Exact function identity for an object-literal method",
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum ObjectPropertyIr",
    );
    for marker in [
        "#[must_use = \"an object-method function must be materialized with its HomeObject\"]",
        "pub struct ObjectMethodFunctionIr {",
        "function_id: FunctionId",
        "protocol: FunctionProtocolIr",
        "pub(crate) enum ObjectMethodProtocolIr {",
        "Method(FunctionExecutionKind)",
        "pub(crate) fn new(function_id: FunctionId, protocol: ObjectMethodProtocolIr)",
        "pub fn function_id(&self) -> &FunctionId",
        "pub const fn protocol(&self) -> FunctionProtocolIr",
    ] {
        assert!(carrier.contains(marker), "missing carrier marker: {marker}");
    }
    assert!(!carrier.contains("pub function_id:"));
    assert!(!carrier.contains("pub protocol:"));
    assert!(!carrier.contains("pub fn new("));

    let properties = bounded(
        IR_SOURCE,
        "pub enum ObjectPropertyIr {",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum PrivateElementKindIr",
    );
    assert_eq!(
        properties
            .matches("function: ObjectMethodFunctionIr")
            .count(),
        6
    );
    assert!(!properties.contains("function: TypedExpr"));

    let mapper = bounded(
        LOWERING_HELPERS_SOURCE,
        "pub(crate) const fn object_method_protocol(",
        "pub(crate) fn for_in_loop_binding_storage_name(",
    );
    for marker in [
        "MethodDefinitionKind::Ordinary",
        "MethodDefinitionKind::Generator",
        "MethodDefinitionKind::Async",
        "MethodDefinitionKind::AsyncGenerator",
        "MethodDefinitionKind::Get",
        "MethodDefinitionKind::Set",
        "ObjectMethodProtocolIr::Method(FunctionExecutionKind::Ordinary)",
        "ObjectMethodProtocolIr::Getter",
        "ObjectMethodProtocolIr::Setter",
    ] {
        assert!(
            mapper.contains(marker),
            "missing exhaustive mapper: {marker}"
        );
    }
    assert!(!mapper.contains("_ =>"));

    let analysis = bounded(
        ANALYSIS_SOURCE,
        "PropertyDefinition::MethodDefinition(method) => {",
        "PropertyDefinition::IdentifierReference(identifier) =>",
    );
    assert!(analysis.contains("protocol: object_method_protocol(method.kind())"));
    assert!(analysis.contains(".function_protocol()"));

    let producer = bounded(
        LOWERING_SOURCE,
        "    fn lower_object_method_function(",
        "    fn observe_proxy_handler_trap_expression_hints(",
    );
    assert!(producer.contains("ObjectMethodFunctionIr::new("));
    assert!(producer.contains("object_method_protocol(method.kind())"));
    assert!(!producer.contains("ExprIr::FunctionValue"));
}

#[test]
fn super_references_carry_receiver_and_parameter_initializers_gain_context_first() {
    let expressions = bounded(IR_SOURCE, "    SuperPropertyRead {", "    PrivateRead {");
    assert_eq!(expressions.matches("receiver: Box<TypedExpr>").count(), 2);

    let reference = bounded(
        REFERENCE_SOURCE,
        "pub(crate) enum ReferenceBase {",
        "impl ReferenceBase {",
    );
    assert!(
        reference.contains("Super {\n        key: PropertyKeyIr,\n        receiver: TypedExpr,")
    );

    let read_write = bounded(
        REFERENCE_SOURCE,
        "impl ReferenceBase {",
        "/// Why a lowered read is not usable as a Reference.",
    );
    assert!(read_write.contains("Self::Super { key, receiver } => ExprIr::SuperPropertyRead"));
    assert!(read_write.contains("Self::Super { key, receiver } => ExprIr::SuperPropertyWrite"));
    assert!(read_write.contains("receiver: Box::new(receiver.clone())"));
    assert!(read_write.contains("receiver: Box::new(receiver)"));

    let function_lowering = bounded(
        LOWERING_SOURCE,
        "        let lexical_derived_activation =",
        "        if let Some(self_binding_name) = function.self_binding_name.as_ref() {",
    );
    assert_before(
        function_lowering,
        "function.protocol.is_object_literal_method()",
        "lowerer.lower_function_parameters(",
    );

    let super_read = bounded(
        LOWERING_SOURCE,
        "    fn lower_super_property_access(",
        "    fn lower_private_property_access(",
    );
    assert_before(super_read, "lower_super_property_key", "lower_current_this");
    assert!(super_read.contains("receiver: Box::new(receiver)"));

    let super_write = bounded(
        LOWERING_SOURCE,
        "            PropertyAccess::Super(access) => {",
        "        }\n    }\n\n    /// 13.4 `++`/`--` on a property Reference.",
    );
    assert_before(
        super_write,
        "lower_super_property_key",
        "lower_current_this",
    );
    assert_before(super_write, "lower_current_this", "lower_expression(rhs)");
    assert!(super_write.contains("receiver: Box::new(receiver)"));
}

#[test]
fn durable_fixture_and_exact_current_failure_inventory_bound_the_claim() {
    assert_eq!(SELECTED_WITNESSES.len(), 5);
    assert!(SELECTED_WITNESSES
        .iter()
        .all(|(_, source)| source.contains("super")));
    assert!(SELECTED_WITNESSES
        .iter()
        .all(|(_, source)| !source.contains("flags:")));
    assert!(SELECTED_WITNESSES[0]
        .1
        .contains("Object.setPrototypeOf(object, proto)"));
    assert!(SELECTED_WITNESSES[2]
        .1
        .contains("method(x = super.toString)"));
    assert!(SELECTED_WITNESSES[3].1.contains("get x()"));
    assert!(SELECTED_WITNESSES[4].1.contains("set x(v)"));

    for marker in [
        "method(suffix)",
        "parameterMethod(value = super.parameterValue)",
        "get namedAccessor()",
        "set namedAccessor(value)",
        "[computedKey(\"m\", \"computedMethod\")]",
        "get [computedKey(\"g\", \"computedAccessor\")]()",
        "set [computedKey(\"s\", \"computedAccessor\")](value)",
        "keyTrace === \"mgs\"",
        "method.call(alien, \"first\")",
        "Object.setPrototypeOf(literal, prototypeB)",
        "secondMethod === \"alien:B:second\"",
        "isNonConstructable(method)",
    ] {
        assert!(
            FIXTURE.contains(marker),
            "missing fixture contract: {marker}"
        );
    }
    assert!(!FIXTURE.contains("async "));
    assert!(!FIXTURE.contains("function*"));
    assert!(!FIXTURE.contains("=>"));

    for marker in [
        "At clean commit `304e4bbad3`",
        "physical files and ten sloppy/strict Script executions",
        "The existing Wasm binary reports `0/10`",
        "unsupported in lila wasm-aot first slice: object literal method",
        "Generator, async, and async-generator object methods remain explicit protocol",
        "nested arrows using an enclosing object method's `super`",
        "keys share the closed IR carrier",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing evidence boundary: {marker}"
        );
    }
}

#[test]
fn backend_home_object_lifecycle_is_typed_and_ordered() {
    assert!(OBJECTS_SOURCE.contains(
        "#[must_use = \"an object-method function must be attached to its literal before publication\"]\nstruct ObjectMethodHomeObjectMaterialization<'a> {"
    ));
    let request = bounded(
        OBJECTS_SOURCE,
        "struct ObjectMethodHomeObjectMaterialization<'a> {",
        "impl PrivateElementEntryLocals {",
    );
    for marker in [
        "method: &'a ObjectMethodFunctionIr",
        "home_object_local: u32",
        "fn new(method: &'a ObjectMethodFunctionIr, home_object_local: u32) -> Self",
    ] {
        assert!(request.contains(marker), "missing request marker: {marker}");
    }
    assert!(!request.contains("Clone"));
    assert!(!request.contains("Copy"));

    let materialize = bounded(
        OBJECTS_SOURCE,
        "    fn emit_object_method_value_to_locals(",
        "    pub(crate) fn compile_object_literal_payload(",
    );
    for marker in [
        "request: ObjectMethodHomeObjectMaterialization<'_>",
        ".get(request.method.function_id())",
        "if meta.protocol != request.method.protocol()",
        "self.emit_function_value_payload(&meta, function)?",
        "self.store_function_home_object(",
        "request.home_object_local",
        "ValueKind::Object",
    ] {
        assert!(
            materialize.contains(marker),
            "missing materialization marker: {marker}"
        );
    }
    assert_before(
        materialize,
        "self.emit_function_value_payload",
        "self.store_function_home_object",
    );

    let literal = bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn compile_object_literal_payload(",
        "    pub(crate) fn compile_property_read_to_locals(",
    );
    assert_before(
        literal,
        "let object_local = self.reserve_temp_local()",
        "while property_index < properties.len()",
    );
    assert_before(
        literal,
        "self.compile_expr_to_locals(key, key_payload, key_tag, function)?",
        "function: method, ..",
    );
    for marker in [
        "ObjectPropertyIr::Method",
        "ObjectPropertyIr::ComputedMethod",
        "ObjectPropertyIr::Getter",
        "ObjectPropertyIr::ComputedGetter",
        "ObjectPropertyIr::Setter",
        "ObjectPropertyIr::ComputedSetter",
        "ObjectMethodHomeObjectMaterialization::new",
    ] {
        assert!(literal.contains(marker), "missing literal arm: {marker}");
    }
    let method_arm = bounded(
        literal,
        "                ObjectPropertyIr::Method {\n                    function: method, ..",
        "                ObjectPropertyIr::Getter {",
    );
    assert_before(
        method_arm,
        "self.emit_object_method_value_to_locals(",
        "self.emit_object_define_enumerable_data(",
    );

    let meta = bounded(PLANNING_SOURCE, "impl WasmFunctionMeta {", "#[cfg(test)]");
    assert!(meta.contains("pub(crate) const fn has_home_object_execution_context"));
    assert!(meta.contains("self.protocol.is_object_literal_method()"));
    assert!(meta.contains("|| self.has_home_object_execution_context()"));

    let planner = bounded(
        PLANNING_SOURCE,
        "        ExprIr::ObjectLiteral(properties) => {\n            let child = properties",
        "        ExprIr::ArrayLiteral(elements) => {",
    );
    for marker in [
        "ObjectPropertyIr::ComputedMethod { key, .. }",
        "ObjectPropertyIr::ComputedGetter { key, .. }",
        "ObjectPropertyIr::ComputedSetter { key, .. }",
        "ObjectPropertyIr::Method { .. }",
        "ObjectPropertyIr::Getter { .. }",
        "ObjectPropertyIr::Setter { .. }",
        "child.max(13)",
    ] {
        assert!(planner.contains(marker), "missing planner marker: {marker}");
    }

    let storage = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn store_function_home_object(",
        "    pub(crate) fn emit_alloc_realm_record(",
    );
    assert!(storage.contains("HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET"));
    assert!(storage.contains("HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET"));
}

#[test]
fn super_emission_preserves_receiver_base_and_rhs_order() {
    let read = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_super_property_read_to_locals(",
        "    fn compile_super_property_write_to_locals(",
    );
    for marker in [
        "receiver: &TypedExpr",
        "compile_super_property_key_expression_to_locals",
        "self.compile_expr_to_locals(\n            receiver,",
        "self.emit_load_super_base(",
        "self.emit_object_read_with_key_tag(",
        "receiver_payload_local",
        "receiver_tag_local",
    ] {
        assert!(read.contains(marker), "missing super-read marker: {marker}");
    }
    assert_before(
        read,
        "self.compile_expr_to_locals(\n            receiver,",
        "compile_super_property_key_expression_to_locals",
    );
    assert_before(
        read,
        "compile_super_property_key_expression_to_locals",
        "self.emit_load_super_base(",
    );
    assert_before(
        read,
        "self.emit_load_super_base(",
        "self.emit_object_read_with_key_tag(",
    );

    let write = bounded(
        EXPRESSIONS_SOURCE,
        "    fn compile_super_property_write_to_locals(",
        "    pub(crate) fn compile_expr_payload(",
    );
    for marker in [
        "receiver: &TypedExpr",
        "value: &TypedExpr",
        "strictness: Strictness",
        "let key_local = self.reserve_temp_local()",
        "self.compile_expr_to_locals(\n            receiver,",
        "self.compile_super_property_key_expression_to_locals(",
        "self.emit_load_super_base(",
        "self.compile_expr_to_locals(value, payload_local, tag_local, function)?",
        "self.emit_value_to_property_key_locals(key_local, key_tag_local, function)?",
        "self.emit_ordinary_set_result_via_helper(",
        "receiver_payload_local",
        "receiver_tag_local",
        "self.with_reference_strictness(strictness",
    ] {
        assert!(
            write.contains(marker),
            "missing super-write marker: {marker}"
        );
    }
    assert_before(
        write,
        "self.compile_expr_to_locals(\n            receiver,",
        "self.compile_super_property_key_expression_to_locals(",
    );
    assert_before(
        write,
        "self.compile_super_property_key_expression_to_locals(",
        "self.emit_load_super_base(",
    );
    assert_before(
        write,
        "self.emit_load_super_base(",
        "self.compile_expr_to_locals(value, payload_local, tag_local, function)?",
    );
    assert_before(
        write,
        "self.compile_expr_to_locals(value, payload_local, tag_local, function)?",
        "self.emit_value_to_property_key_locals(key_local, key_tag_local, function)?",
    );
    assert_before(
        write,
        "self.emit_value_to_property_key_locals(key_local, key_tag_local, function)?",
        "self.emit_ordinary_set_result_via_helper(",
    );
}
