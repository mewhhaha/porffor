use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use lila_ir::{
    private_brand_key, private_data_key, AnnexBFunctionCopyTargetIr, ArithmeticBinaryOp,
    ArrayDestructuringElementIr, ArrayDestructuringPatternIr, BigIntBitwiseOp, BindingMode,
    BitwiseBinaryOp, BlockIr, CallableToStringRepresentation, ClassDefinitionIr,
    ClassElementDefinitionIr, ClassElementExecutionKind, ClassFieldKeyIr, ClassFunctionKind,
    ClassHeritageKind, ClassInstanceElementPlanIr, ClassMethodPlacementIr, ClassStaticElementIr,
    DeleteIdentifierKindIr, DestructuringPropertyKeyIr, DestructuringTargetIr, DynamicFunctionKind,
    DynamicSourceIntrinsic, EqualityBinaryOp, ExprIr, ForInOfEnvironmentIr, ForInitIr,
    ForLexicalEnvironmentIr, ForOfIteratorHeadIr, FunctionExecutionKind, FunctionFlavor,
    FunctionId, FunctionIr, FunctionParamIr, FunctionProtocolIr, GeneratorResumeModeIr,
    GeneratorTryPlanIr, GlobalBindingPlan, GlobalPropertyInitializerIr, HeapShape, HostBuiltinId,
    IdentifierWriteDisposition, JsonStaticValueIr, KindSet, LexicalEnvironmentIr, LogicalBinaryOp,
    NumericUpdateOp, ObjectPropertyIr, ObjectShapeProperty, OrdinaryPropertyAssignmentIr,
    OrdinaryPropertyEagerCompoundAssignmentIr, OrdinaryPropertyNumericUpdateIr, OwnedEnvBindingIr,
    PrivateNameId, PropertyKeyIr, RelationalBinaryOp, ScriptIr, SpecOperationIr, SpreadArgumentIr,
    StandardBuiltinId, StatementIr, Strictness, SuspendedPropertyReferenceIr,
    SuspendedPropertyReferenceUse, SwitchCaseIr, SyncDisposableResourcesIr, ToPrimitiveHint,
    TypedExpr, UnaryBitwiseOp, UnaryNumericOp, UpdateReturnMode, ValueInfo, ValueKind,
    VarDeclaratorIr, YieldForm, AGGREGATE_ERROR_NAME, ARRAY_BUFFER_NAME, ARRAY_NAME, ATOMICS_NAME,
    BIGINT64_ARRAY_NAME, BIGUINT64_ARRAY_NAME, BOOLEAN_NAME, DATA_VIEW_NAME, DATE_NAME,
    DATE_VALUE_SLOT, ERROR_NAME, EVAL_ERROR_NAME, FLOAT32_ARRAY_NAME, FLOAT64_ARRAY_NAME,
    FUNCTION_NAME, GLOBAL_THIS_NAME, HOST_PARSE_FLOAT_FUNCTION_ID, INT16_ARRAY_NAME,
    INT32_ARRAY_NAME, INT8_ARRAY_NAME, INTL_NAMESPACE_CONSTRUCTORS, IS_CONSTRUCTOR_NAME, JSON_NAME,
    JS_STRING_SURROGATE_SENTINEL, LEXICAL_ARGUMENTS_NAME, LEXICAL_HOME_OBJECT_NAME,
    LEXICAL_NEW_TARGET_NAME, LEXICAL_THIS_NAME, LILA_GENERATOR_THROW_SLOT,
    LILA_STATIC_GENERATOR_ITERATOR_SLOT, LILA_STATIC_GENERATOR_VALUES_METHOD, MAP_NAME, MATH_NAME,
    NUMBER_NAME, OBJECT_NAME, PRINT_NAME, PROXY_NAME, RANGE_ERROR_NAME, REFERENCE_ERROR_NAME,
    REFLECT_NAME, REGEXP_NAME, SET_NAME, SHARED_ARRAY_BUFFER_NAME, STRING_NAME,
    SUPPRESSED_ERROR_NAME, SYMBOL_NAME, SYNTAX_ERROR_NAME, TEMPORAL_DURATION_NAME,
    TEMPORAL_NOW_NAME, TEMPORAL_PLAIN_DATE_NAME, TEMPORAL_PLAIN_DATE_TIME_NAME,
    TEMPORAL_PLAIN_MONTH_DAY_NAME, TEMPORAL_PLAIN_TIME_NAME, TEMPORAL_PLAIN_YEAR_MONTH_NAME,
    TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_METHODS, TYPE_ERROR_NAME, UINT16_ARRAY_NAME,
    UINT32_ARRAY_NAME, UINT8_ARRAY_NAME, UINT8_CLAMPED_ARRAY_NAME, URI_ERROR_NAME,
};
use lila_ir::{SuperPropertyMutationIr, SuperPropertyMutationOperationIr};
// `Function` is deliberately absent from this list. The name is bound below to
// `code_sink::Function`, the wrapper that counts real Wasm label depth, and
// every submodule of this crate reaches `Function` through this one binding
// (`use super::*` / `use super::super::*`). That is what lets ~600 `&mut
// Function` signatures and ~77,000 `function.instruction(..)` calls keep their
// exact text while the branch arithmetic underneath them becomes correct.
// See `code_sink.rs`.
use wasm_encoder::{BlockType, Ieee64, Instruction, MemArg, ValType};

mod abi;
mod arguments_protocol;
mod bigint;
mod builtins;
mod code_sink;
mod control_flow;
mod data;
mod emission_sites;
mod emit;
mod emitted_function;
mod environments;
mod expressions;
mod functions;
pub(crate) use functions::RealmRecordLocal;
mod gc_types;
mod generator_delegation;
mod generator_reference;
mod heap;
mod intrinsics;
mod module;
mod modules;
mod objects;
mod operations;
mod planning;
mod runtime_abi;
mod runtime_helpers;
use abi::*;
use arguments_protocol::*;
use bigint::BigIntHelperOp;
use builtins::*;
use code_sink::{Function, LabelDepth};
use data::*;
pub use emit::emit;
pub(crate) use emit::{
    AccessorThrowRouting, BindingStorage, CompletionKind, ControlFrameKind, FunctionBuilder,
    IteratorCloseOnThrowLocals, LabelTargets, LoopTargets, OrdinarySetDataOnReceiverEmission,
    PropagateCallThrow, ReturnAbi,
};
// `FunctionBodySize` and `FunctionLocalCount` are part of the public face
// because `EmittedFunctionSummary` carries them: a `pub` struct whose fields
// name crate-private types is a `private_interfaces` warning, and flattening
// them back to `u32` at the boundary would give the two figures the same type
// again — which is exactly what this module's newtypes exist to prevent.
pub(crate) use emitted_function::{
    emit_size_report_requested, write_size_report_file_if_requested, EmittedFunction,
    FunctionBodyBudget, FunctionIdentity, ModuleCode,
};
pub use emitted_function::{EmittedFunctionSummary, FunctionBodySize, FunctionLocalCount};
use heap::*;
use intrinsics::*;
use module::*;
use modules::module_unit_guard_count;
pub(crate) use operations::ToPrimitiveAbruptRoute;
use planning::*;
pub use runtime_abi::{decode_heap_bigint_decimal, WasmRuntimeDecodeError, WasmRuntimeValueTag};
pub(crate) use runtime_helpers::{RuntimeHelperEmission, RuntimeHelperFact, RuntimeHelperId};

fn read_static_heap_shape_property(shape: &HeapShape, key: &str) -> Option<ObjectShapeProperty> {
    match shape {
        HeapShape::Object(object) => object.properties.get(key).cloned().or_else(|| {
            object
                .prototype
                .as_deref()
                .and_then(|prototype| read_static_heap_shape_property(prototype, key))
        }),
        HeapShape::Array(array) => array.properties.get(key).cloned().or_else(|| {
            array
                .prototype
                .as_deref()
                .and_then(|prototype| read_static_heap_shape_property(prototype, key))
        }),
    }
}

static EMPTY_BLOCK: LazyLock<BlockIr> = LazyLock::new(|| BlockIr {
    statements: Vec::new(),
    result_kind: ValueKind::Undefined,
    lexical_environment: None,
});

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::{lower, lower_with_host_surface_policy, BigIntLiteralIr, HostSurfacePolicy};
    use wasmparser::{Operator, Parser, Payload, Validator, WasmFeatures};

    fn emit_script(source: &str) -> Result<WasmArtifact, EmitError> {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        emit(&lower_with_host_surface_policy(
            &source,
            HostSurfacePolicy::Test262,
        ))
    }

    #[test]
    fn disposable_stack_construction_and_lifecycle_are_one_intrinsic_unit() {
        let constructor = include_str!("builtins/disposable_stack.rs");
        let functions = include_str!("functions.rs");
        let heap = include_str!("heap.rs");
        let installer = include_str!("intrinsics/resource_management.rs");
        let planning = include_str!("planning.rs");
        let standard = include_str!("builtins/standard.rs");
        let catalog = include_str!("../../lila-ir/src/builtins/catalog.rs");
        let names = include_str!("../../lila-ir/src/names.rs");

        assert!(constructor.contains(
            "#[must_use = \"a pending DisposableStack record must be consumed by the instance finalizer\"]\nstruct PendingDisposableStackRecordLocal(u32);"
        ));
        assert!(!constructor.contains("derive(Clone"));
        assert_eq!(
            constructor
                .matches("emit_alloc_pending_disposable_stack_record(function)?")
                .count(),
            2,
            "the constructor and move each allocate one fresh pending record"
        );
        assert_eq!(
            constructor
                .matches("emit_new_target_prototype_to_locals(")
                .count(),
            1,
            "the constructor body owns exactly one observable prototype Get"
        );
        assert_eq!(
            constructor
                .matches("emit_finalize_disposable_stack_instance(")
                .count(),
            3,
            "constructor and move consume one record each through one private finalizer"
        );
        let finalizer = constructor
            .split_once("fn emit_finalize_disposable_stack_instance(")
            .expect("DisposableStack consuming finalizer")
            .1
            .split_once("fn emit_take_disposable_stack_capability(")
            .expect("DisposableStack finalizer must be bounded")
            .0;
        assert_eq!(
            finalizer
                .matches("OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK")
                .count(),
            1,
            "only the consuming finalizer may install the sync brand"
        );
        let receiver_check = constructor
            .split_once("fn emit_disposable_stack_record_from_receiver(")
            .expect("DisposableStack receiver checker")
            .1
            .split_once("fn emit_disposable_stack_require_pending(")
            .expect("DisposableStack receiver checker must be bounded")
            .0;
        assert_eq!(
            receiver_check
                .matches("OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK")
                .count(),
            1,
            "every lifecycle operation checks the distinct sync brand"
        );
        assert!(!constructor.contains("OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK"));
        assert!(heap.contains("pub(crate) const OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK: u64 = 40;"));
        assert!(heap
            .contains("pub(crate) const OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK: u64 = 39;"));

        let direct_returning = functions
            .split_once("let direct_returning_constructor_table_indices: Vec<i64> = [")
            .expect("direct-returning constructor domain should exist")
            .1
            .split_once("]\n        .into_iter()")
            .expect("direct-returning constructor domain should be bounded")
            .0;
        assert_eq!(
            direct_returning
                .matches("StandardBuiltinId::DisposableStackConstructor,")
                .count(),
            1,
            "the constructor body must run before generic prototype Get/allocation"
        );

        for (builtin, function_id, emitter) in [
            (
                "DisposableStackPrototypeUse",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_USE_FUNCTION_ID",
                "emit_disposable_stack_use(function)?",
            ),
            (
                "DisposableStackPrototypeAdopt",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_ADOPT_FUNCTION_ID",
                "emit_disposable_stack_adopt(function)?",
            ),
            (
                "DisposableStackPrototypeDefer",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DEFER_FUNCTION_ID",
                "emit_disposable_stack_defer(function)?",
            ),
            (
                "DisposableStackPrototypeMove",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_MOVE_FUNCTION_ID",
                "emit_disposable_stack_move(function)?",
            ),
            (
                "DisposableStackPrototypeDispose",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DISPOSE_FUNCTION_ID",
                "emit_disposable_stack_dispose(function)?",
            ),
            (
                "DisposableStackPrototypeDisposedGetter",
                "BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DISPOSED_GETTER_FUNCTION_ID",
                "emit_disposable_stack_disposed_getter(function)?",
            ),
        ] {
            assert_eq!(
                catalog.matches(&format!("\n    {builtin} {{")).count(),
                1,
                "the lifecycle member must have exactly one catalog row"
            );
            assert_eq!(
                names.matches(&format!("pub const {function_id}:")).count(),
                1,
                "the lifecycle member must have exactly one function id"
            );
            assert_eq!(
                standard.matches(emitter).count(),
                1,
                "the lifecycle member must have exactly one dispatcher arm"
            );
        }

        let constructor_dependencies = planning
            .split_once("if builtin == StandardBuiltinId::DisposableStackConstructor {")
            .expect("DisposableStack constructor dependency closure")
            .1
            .split_once("if builtin == StandardBuiltinId::DisposableStackPrototypeDispose {")
            .expect("constructor dependency closure must be bounded")
            .0;
        for builtin in ["Use", "Adopt", "Defer", "Move", "Dispose", "DisposedGetter"] {
            assert_eq!(
                constructor_dependencies
                    .matches(&format!(
                        "StandardBuiltinId::DisposableStackPrototype{builtin},"
                    ))
                    .count(),
                1,
                "constructor installation must root {builtin} exactly once"
            );
        }

        assert_eq!(
            installer
                .matches("emit_object_define_function_data_with_aliases(")
                .count(),
            1,
            "dispose and Symbol.dispose must share one function value"
        );
        assert!(installer.contains("&[\"Symbol.dispose\"]"));
        assert!(installer.contains("Some((payload_local, tag_local)),\n            None,"));
    }

    #[test]
    fn typed_array_accessors_use_the_closed_buffer_witness() {
        let binary_data = include_str!("builtins/binary_data.rs");
        let standard = include_str!("builtins/standard.rs");
        let accessor_domain = binary_data
            .split_once("pub(super) enum TypedArrayAccessorKind {")
            .expect("typed-array accessor domain should exist")
            .1
            .split_once("}\n\n/// The closed set of observation points")
            .expect("typed-array accessor domain should be bounded")
            .0;
        let accessor_projection = binary_data
            .split_once("TypedArrayWitnessUse::Accessor { kind, result_local } => match kind {")
            .expect("typed-array accessor witness projection should exist")
            .1
            .split_once("        }\n\n        self.release_temp_local(data_ptr_local);")
            .expect("typed-array accessor witness projection should be bounded")
            .0;
        let accessor_compiler = binary_data
            .split_once("pub(super) fn compile_typed_array_accessor_builtin(")
            .expect("typed-array accessor compiler should exist")
            .1
            .split_once("pub(crate) fn emit_initialize_array_buffer_private_state(")
            .expect("typed-array accessor compiler should be bounded")
            .0;
        let delegates = standard
            .split_once("StandardBuiltinId::TypedArrayPrototypeByteLengthGetter => {")
            .expect("typed-array accessor delegates should exist")
            .1
            .split_once("StandardBuiltinId::TypedArrayPrototypeSubarray => {")
            .expect("typed-array accessor delegates should be bounded")
            .0;

        for variant in ["ByteLength", "ByteOffset", "Length"] {
            assert_eq!(
                accessor_domain.matches(&format!("    {variant},")).count(),
                1,
                "the accessor domain must contain {variant} exactly once"
            );
            assert_eq!(
                accessor_projection
                    .matches(&format!("TypedArrayAccessorKind::{variant} =>"))
                    .count(),
                1,
                "the witness must project {variant} exactly once"
            );
            assert_eq!(
                delegates
                    .matches(&format!("TypedArrayAccessorKind::{variant}"))
                    .count(),
                1,
                "the builtin dispatch must select {variant} explicitly"
            );
        }
        assert_eq!(
            accessor_domain
                .lines()
                .filter(|line| line.trim_end().ends_with(','))
                .count(),
            3,
            "the accessor result domain must stay closed"
        );
        assert_eq!(
            delegates
                .matches("compile_typed_array_accessor_builtin(")
                .count(),
            3,
            "all three accessors must delegate through the typed compiler"
        );
        assert_eq!(
            accessor_compiler
                .matches("emit_typed_array_witness(")
                .count(),
            1,
            "the accessor compiler must make exactly one live buffer witness"
        );
        for forbidden in [
            "emit_load_array_buffer_data(",
            "emit_load_array_buffer_byte_length(",
            "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
            "emit_typed_array_current_byte_length(",
        ] {
            assert!(
                !accessor_compiler.contains(forbidden),
                "the accessor compiler must not bypass its witness with {forbidden}"
            );
            assert!(
                !delegates.contains(forbidden),
                "the accessor delegates must not bypass their compiler with {forbidden}"
            );
        }
        assert!(accessor_projection.contains("witness.out_of_bounds_local"));
        assert!(accessor_projection.contains("view.byte_offset_local"));
        assert!(accessor_projection.contains("witness.element_length_local"));
        assert!(accessor_projection.contains("view.bytes_per_element_local"));
    }

    #[test]
    fn construct_fallback_requires_resolved_realm_intrinsics() {
        let source = include_str!("functions.rs");
        let domain = source
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let offsets = source
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;
        let construct = source
            .split_once("pub(crate) fn emit_function_handle_construct_with_argv(")
            .expect("shared construct path should exist")
            .1
            .split_once("pub(crate) fn copy_function_realm_typed_array_prototypes(")
            .expect("shared construct path should be bounded")
            .0;
        let required_load = source
            .split_once("fn emit_load_required_resolved_realm_ordinary_prototype(")
            .expect("required resolved-realm ordinary-prototype loader should exist")
            .1
            .split_once("fn emit_install_resolved_realm_ordinary_prototype(")
            .expect("required resolved-realm ordinary-prototype loader should be bounded")
            .0;
        let install = source
            .split_once("fn emit_install_resolved_realm_ordinary_prototype(")
            .expect("resolved-realm ordinary-prototype consumer should exist")
            .1
            .split_once("pub(crate) fn emit_load_required_resolved_realm_array_prototype(")
            .expect("resolved-realm ordinary-prototype consumer should be bounded")
            .0;

        for (variant, offset) in [
            ("Object", "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET"),
            ("String", "HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET"),
            ("Number", "HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET"),
            ("Boolean", "HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET"),
        ] {
            assert_eq!(
                domain.matches(&format!("    {variant},")).count(),
                1,
                "the closed ordinary default-prototype domain must contain {variant} exactly once"
            );
            assert_eq!(
                offsets
                    .matches(&format!("Self::{variant} => {offset}"))
                    .count(),
                1,
                "{variant} must map exhaustively to its realm-intrinsic slot"
            );
            assert_eq!(
                construct
                    .matches(&format!("OrdinaryDefaultPrototype::{variant}"))
                    .count(),
                1,
                "the construct path must select {variant} through the closed domain once"
            );
        }
        assert!(
            !domain.contains("Array"),
            "Array must retain its separate exotic-prototype typestate"
        );
        assert!(source.contains(
            "#[must_use = \"the resolved-realm prototype must be installed with its representation tag\"]\nstruct ResolvedRealmOrdinaryPrototypeLocal"
        ));
        assert_eq!(
            construct
                .matches("emit_load_required_resolved_realm_ordinary_prototype(")
                .count(),
            4
        );
        assert_eq!(
            construct
                .matches("emit_install_resolved_realm_ordinary_prototype(")
                .count(),
            4
        );
        assert_eq!(
            construct
                .matches("emit_load_required_resolved_realm_array_prototype(")
                .count(),
            1,
            "Array must keep its existing required realm slot and Array tag path"
        );
        assert!(
            !construct.contains("emit_load_realm_intrinsic_prototype_or_global("),
            "resolved GetPrototypeFromConstructor results must not select entry globals"
        );
        for global in [
            "OBJECT_PROTOTYPE_GLOBAL_INDEX",
            "STRING_PROTOTYPE_GLOBAL_INDEX",
            "NUMBER_PROTOTYPE_GLOBAL_INDEX",
            "BOOLEAN_PROTOTYPE_GLOBAL_INDEX",
        ] {
            assert!(
                !construct.contains(global),
                "the construct fallback must not retain entry-global prototype {global}"
            );
        }

        assert!(!required_load.contains("GlobalGet"));
        assert!(!required_load.contains("GLOBAL_INDEX"));
        assert_eq!(required_load.matches("Instruction::Unreachable").count(), 3);
        assert!(required_load.contains("intrinsic.offset()"));
        assert!(install.contains("prototype: ResolvedRealmOrdinaryPrototypeLocal"));
        assert_eq!(install.matches("prototype.0").count(), 2);
        assert_eq!(install.matches("ValueKind::Object.tag() as i64").count(), 1);
    }

    #[test]
    fn ordinary_default_prototype_structural_count_tracks_message_error_and_regexp() {
        let source = include_str!("functions.rs");
        let domain = source
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let offsets = source
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;

        assert_eq!(
            domain
                .matches("MessageError(ErrorMessageConstructorKind),")
                .count(),
            1,
            "the structural count must retain the shared message-Error variant"
        );
        assert_eq!(
            offsets
                .matches("Self::MessageError(kind) => kind.prototype_slot().offset()")
                .count(),
            1,
            "the message-Error variant must retain its typed prototype-slot map"
        );
        assert_eq!(domain.matches("    RegExp,").count(), 1);
        assert_eq!(
            offsets
                .matches("Self::RegExp => HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET")
                .count(),
            1
        );
        assert_eq!(
            domain
                .lines()
                .filter(|line| line.trim_end().ends_with(','))
                .count(),
            8,
            "the closed domain count must move with MessageError and RegExp"
        );
    }

    #[test]
    fn iterator_constructor_realm_prototype_is_required_tagged_and_published() {
        let functions = include_str!("functions.rs");
        let standard = include_str!("builtins/standard.rs");
        let bootstrap = include_str!("builtins/bootstrap.rs");
        let host = include_str!("builtins/host.rs");

        let domain = functions
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let offsets = functions
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;
        assert_eq!(domain.matches("    Iterator,").count(), 1);
        assert_eq!(
            offsets
                .matches("Self::Iterator => HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET")
                .count(),
            1
        );

        let constructor = standard
            .split_once("StandardBuiltinId::IteratorConstructor => {")
            .expect("Iterator constructor builtin should exist")
            .1
            .split_once("StandardBuiltinId::FunctionConstructor => {")
            .expect("Iterator constructor builtin should be bounded")
            .0;
        for (operation, count) in [
            ("emit_new_target_prototype_to_locals(", 1),
            (
                "NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(",
                1,
            ),
            ("OrdinaryDefaultPrototype::Iterator", 1),
            ("emit_alloc_plain_object_with_prototype_and_tag(", 1),
        ] {
            assert_eq!(
                constructor.matches(operation).count(),
                count,
                "Iterator construction must retain exactly {count} {operation} occurrence(s)"
            );
        }
        let prototype_resolution = constructor
            .find("emit_new_target_prototype_to_locals(")
            .unwrap();
        let tagged_allocation = "emit_alloc_plain_object_with_prototype_and_tag(\n                    Some(prototype_payload_local),\n                    Some(prototype_tag_local),\n                    None";
        assert_eq!(constructor.matches(tagged_allocation).count(), 1);
        assert!(prototype_resolution < constructor.find(tagged_allocation).unwrap());
        for forbidden in [
            "emit_error_new_target_prototype_to_local(",
            "NewTargetPrototypeFallback::CurrentGlobal",
            "emit_alloc_plain_object_with_prototype(",
        ] {
            assert!(
                !constructor.contains(forbidden),
                "Iterator construction must not retain {forbidden}"
            );
        }
        let temp_reservation = "let prototype_payload_local = self.reserve_temp_local();\n                let prototype_tag_local = self.reserve_temp_local();";
        let temp_release = "self.release_temp_local(prototype_tag_local);\n                self.release_temp_local(prototype_payload_local);";
        assert_eq!(constructor.matches(temp_reservation).count(), 1);
        assert_eq!(constructor.matches(temp_release).count(), 1);

        let entry_publication = "emit_store_current_realm_global_intrinsic(\n            ITERATOR_PROTOTYPE_GLOBAL_INDEX,\n            NonArrayRealmIntrinsicSlot::IteratorPrototype";
        assert_eq!(bootstrap.matches(entry_publication).count(), 1);
        assert_eq!(
            host.matches("self.emit_store_realm_iterator_prototype(")
                .count(),
            1
        );
    }

    #[test]
    fn iterator_constructor_active_function_is_realm_local_and_closed() {
        let functions = include_str!("functions.rs");
        let standard = include_str!("builtins/standard.rs");
        let bootstrap = include_str!("builtins/bootstrap.rs");
        let host = include_str!("builtins/host.rs");

        let domain = standard
            .split_once("enum ActiveStandardBuiltinFunction {")
            .expect("active standard-builtin domain should exist")
            .1
            .split_once("}\n\nimpl ActiveStandardBuiltinFunction")
            .expect("active standard-builtin domain should be bounded")
            .0;
        let mapping = standard
            .split_once("impl ActiveStandardBuiltinFunction {")
            .expect("active standard-builtin global map should exist")
            .1
            .split_once("enum ArrayBufferSliceKind")
            .expect("active standard-builtin global map should be bounded")
            .0;
        assert_eq!(domain.matches("    IteratorConstructor,").count(), 1);
        assert_eq!(
            domain
                .lines()
                .filter(|line| line.trim_end().ends_with(','))
                .count(),
            2
        );
        assert_eq!(
            mapping
                .matches("Self::IteratorConstructor => ITERATOR_CONSTRUCTOR_GLOBAL_INDEX")
                .count(),
            1
        );

        let emitter = standard
            .split_once("fn emit_active_standard_builtin_function_payload(")
            .expect("active standard-builtin emitter should exist")
            .1
            .split_once("fn emit_normalize_undefined_new_target_to_active_standard_builtin(")
            .expect("active standard-builtin emitter should be bounded")
            .0;
        for (operation, count) in [
            ("LocalGet(self.current_env_local)", 2),
            ("Instruction::I64Eqz", 1),
            ("Instruction::If(BlockType::Result(ValType::I64))", 1),
            ("Instruction::GlobalGet(active.entry_global_index())", 1),
        ] {
            assert_eq!(
                emitter.matches(operation).count(),
                count,
                "active standard-builtin emission must retain exactly {count} {operation} occurrence(s)"
            );
        }
        let environment_test = emitter
            .find("LocalGet(self.current_env_local)")
            .expect("active emitter must inspect its realm environment");
        let entry_fallback = emitter
            .find("Instruction::GlobalGet(active.entry_global_index())")
            .expect("active emitter must retain the typed entry fallback");
        let created_identity = emitter
            .rfind("LocalGet(self.current_env_local)")
            .expect("active emitter must select its created-realm identity");
        assert!(environment_test < entry_fallback && entry_fallback < created_identity);

        let constructor = standard
            .split_once("StandardBuiltinId::IteratorConstructor => {")
            .expect("Iterator constructor builtin should exist")
            .1
            .split_once("StandardBuiltinId::FunctionConstructor => {")
            .expect("Iterator constructor builtin should be bounded")
            .0;
        let active_call = "self.emit_active_standard_builtin_function_payload(\n                    ActiveStandardBuiltinFunction::IteratorConstructor,\n                    function,\n                );";
        assert_eq!(constructor.matches(active_call).count(), 1);
        assert!(
            !constructor.contains("Instruction::GlobalGet(ITERATOR_CONSTRUCTOR_GLOBAL_INDEX)"),
            "Iterator construction must not compare NewTarget with the entry global directly"
        );
        let active_test = constructor.find(active_call).unwrap();
        let active_throw = constructor
            .find("emit_throw_current_function_realm_type_error(")
            .unwrap();
        let prototype_resolution = constructor
            .find("emit_new_target_prototype_to_locals(")
            .unwrap();
        assert!(active_test < active_throw && active_throw < prototype_resolution);

        let entry_identity = "self.init_builtin_constructor_object(\n                StandardBuiltinId::IteratorConstructor,\n                ITERATOR_PROTOTYPE_GLOBAL_INDEX";
        assert_eq!(bootstrap.matches(entry_identity).count(), 1);
        let created_identity = "self.store_i64_local_at_offset(\n            iterator_constructor_local,\n            HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n            iterator_constructor_local";
        assert_eq!(host.matches(created_identity).count(), 1);
        let created_type_error = "self.store_i64_local_at_offset(\n            iterator_constructor_local,\n            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,\n            type_error_prototype_local";
        assert_eq!(host.matches(created_type_error).count(), 1);

        let construct = functions
            .split_once("pub(crate) fn emit_function_handle_construct_with_argv(")
            .expect("shared construct path should exist")
            .1
            .split_once("pub(crate) fn copy_function_realm_typed_array_prototypes(")
            .expect("shared construct path should be bounded")
            .0;
        let direct_returning_domain = construct
            .split_once("let direct_returning_constructor_table_indices: Vec<i64> = [")
            .expect("direct-returning constructor domain should exist")
            .1
            .split_once("]\n        .into_iter()")
            .expect("direct-returning constructor domain should be bounded")
            .0;
        assert_eq!(
            direct_returning_domain
                .matches("StandardBuiltinId::IteratorConstructor,")
                .count(),
            1,
            "Iterator must route to its body before generic construction"
        );
        let direct_dispatch = construct
            .find("for table_index in direct_returning_constructor_table_indices {")
            .expect("direct-returning constructor dispatch should exist");
        let generic_prototype_get = construct
            .find("function.instruction(&Instruction::I64Const(self.strings.payload(\"prototype\")));")
            .expect("generic construct path should read NewTarget.prototype");
        let generic_preallocation = construct
            .find("self.emit_alloc_plain_object_with_prototype_and_tag(")
            .expect("generic construct path should allocate its receiver");
        assert!(
            direct_dispatch < generic_prototype_get
                && generic_prototype_get < generic_preallocation,
            "Iterator's direct-returning body must run before generic prototype Get and allocation"
        );
        let direct_dispatch_body = &construct[direct_dispatch..generic_prototype_get];
        assert_eq!(
            direct_dispatch_body
                .matches("Instruction::CallIndirect {")
                .count(),
            1
        );
        assert_eq!(
            direct_dispatch_body
                .matches("function.instruction(&Instruction::Br(1));")
                .count(),
            1,
            "a direct-returning constructor must leave the generic construct block"
        );
    }

    #[test]
    fn regexp_constructor_realm_prototype_is_active_required_tagged_direct_and_published() {
        let functions = include_str!("functions.rs");
        let standard = include_str!("builtins/standard.rs");
        let bootstrap = include_str!("builtins/bootstrap.rs");
        let host = include_str!("builtins/host.rs");

        let ordinary_domain = functions
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let ordinary_offsets = functions
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;
        assert_eq!(ordinary_domain.matches("    RegExp,").count(), 1);
        assert_eq!(
            ordinary_offsets
                .matches("Self::RegExp => HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET")
                .count(),
            1
        );

        let active_domain = standard
            .split_once("enum ActiveStandardBuiltinFunction {")
            .expect("active standard-builtin domain should exist")
            .1
            .split_once("}\n\nimpl ActiveStandardBuiltinFunction")
            .expect("active standard-builtin domain should be bounded")
            .0;
        let active_mapping = standard
            .split_once("impl ActiveStandardBuiltinFunction {")
            .expect("active standard-builtin global map should exist")
            .1
            .split_once("enum ArrayBufferSliceKind")
            .expect("active standard-builtin global map should be bounded")
            .0;
        assert_eq!(active_domain.matches("    RegExpConstructor,").count(), 1);
        assert_eq!(
            active_mapping
                .matches("Self::RegExpConstructor => REGEXP_CONSTRUCTOR_GLOBAL_INDEX")
                .count(),
            1
        );

        let normalization = standard
            .split_once("fn emit_normalize_undefined_new_target_to_active_standard_builtin(")
            .expect("active new-target normalization should exist")
            .1
            .split_once("fn compile_typed_array_prototype_reverse_builtin(")
            .expect("active new-target normalization should be bounded")
            .0;
        for operation in [
            "active: ActiveStandardBuiltinFunction",
            "ValueKind::Undefined.tag() as i64",
            "self.emit_active_standard_builtin_function_payload(active, function)",
            "Instruction::LocalSet(new_target_payload_local)",
            "ValueKind::Function.tag() as i64",
            "Instruction::LocalSet(new_target_tag_local)",
        ] {
            assert_eq!(
                normalization.matches(operation).count(),
                1,
                "active new-target normalization must retain one {operation}"
            );
        }
        let undefined_test = normalization
            .find("ValueKind::Undefined.tag() as i64")
            .unwrap();
        let active_selection = normalization
            .find("self.emit_active_standard_builtin_function_payload(active, function)")
            .unwrap();
        let function_tag = normalization
            .find("ValueKind::Function.tag() as i64")
            .unwrap();
        assert!(undefined_test < active_selection && active_selection < function_tag);

        let constructor = standard
            .split_once("StandardBuiltinId::RegExpConstructor => {")
            .expect("RegExp constructor builtin should exist")
            .1
            .split_once("StandardBuiltinId::JsonParse =>")
            .expect("RegExp constructor builtin should be bounded")
            .0;
        let active_normalization = "self.emit_normalize_undefined_new_target_to_active_standard_builtin(\n                    ActiveStandardBuiltinFunction::RegExpConstructor,\n                    function,\n                );";
        for (operation, count) in [
            (active_normalization, 1),
            ("emit_new_target_prototype_to_locals(", 1),
            (
                "NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(",
                1,
            ),
            ("OrdinaryDefaultPrototype::RegExp", 1),
            ("emit_alloc_plain_object_with_prototype_and_tag(", 1),
            ("OBJECT_INTERNAL_BRAND_REGEXP", 1),
        ] {
            assert_eq!(
                constructor.matches(operation).count(),
                count,
                "RegExp construction must retain exactly {count} {operation} occurrence(s)"
            );
        }
        let prototype_resolution = constructor
            .find("emit_new_target_prototype_to_locals(")
            .unwrap();
        let tagged_allocation = "emit_alloc_plain_object_with_prototype_and_tag(\n                    Some(prototype_payload_local),\n                    Some(prototype_tag_local),\n                    None";
        assert_eq!(constructor.matches(tagged_allocation).count(), 1);
        assert!(
            constructor.find(active_normalization).unwrap() < prototype_resolution
                && prototype_resolution < constructor.find(tagged_allocation).unwrap()
        );
        for forbidden in [
            "emit_error_new_target_prototype_to_local(",
            "NewTargetPrototypeFallback::CurrentGlobal",
            "emit_alloc_plain_object_with_prototype(",
        ] {
            assert!(
                !constructor.contains(forbidden),
                "RegExp construction must not retain {forbidden}"
            );
        }
        let temp_reservation = "let prototype_payload_local = self.reserve_temp_local();\n                let prototype_tag_local = self.reserve_temp_local();";
        let temp_release = "self.release_temp_local(prototype_tag_local);\n                self.release_temp_local(prototype_payload_local);";
        assert_eq!(constructor.matches(temp_reservation).count(), 1);
        assert_eq!(constructor.matches(temp_release).count(), 1);

        let entry_identity = "self.init_builtin_constructor_object(\n                StandardBuiltinId::RegExpConstructor,\n                REGEXP_PROTOTYPE_GLOBAL_INDEX";
        assert_eq!(bootstrap.matches(entry_identity).count(), 1);
        assert_eq!(
            bootstrap
                .matches("Instruction::GlobalSet(REGEXP_PROTOTYPE_GLOBAL_INDEX)")
                .count(),
            1
        );
        assert_eq!(
            bootstrap
                .matches("NonArrayRealmIntrinsicSlot::RegExpPrototype")
                .count(),
            1
        );
        assert_eq!(
            host.matches("NonArrayRealmIntrinsicSlot::RegExpPrototype")
                .count(),
            1
        );
        let created_identity = "self.store_i64_local_at_offset(\n            regexp_constructor_local,\n            HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n            regexp_constructor_local";
        assert_eq!(host.matches(created_identity).count(), 1);

        let construct = functions
            .split_once("pub(crate) fn emit_function_handle_construct_with_argv(")
            .expect("shared construct path should exist")
            .1
            .split_once("pub(crate) fn copy_function_realm_typed_array_prototypes(")
            .expect("shared construct path should be bounded")
            .0;
        let direct_returning_domain = construct
            .split_once("let direct_returning_constructor_table_indices: Vec<i64> = [")
            .expect("direct-returning constructor domain should exist")
            .1
            .split_once("]\n        .into_iter()")
            .expect("direct-returning constructor domain should be bounded")
            .0;
        assert_eq!(
            direct_returning_domain
                .matches("StandardBuiltinId::RegExpConstructor,")
                .count(),
            1,
            "RegExp must route to its body before generic construction"
        );
        let direct_dispatch = construct
            .find("for table_index in direct_returning_constructor_table_indices {")
            .expect("direct-returning constructor dispatch should exist");
        let generic_prototype_get = construct
            .find("function.instruction(&Instruction::I64Const(self.strings.payload(\"prototype\")));")
            .expect("generic construct path should read NewTarget.prototype");
        let generic_preallocation = construct
            .find("self.emit_alloc_plain_object_with_prototype_and_tag(")
            .expect("generic construct path should allocate its receiver");
        assert!(
            direct_dispatch < generic_prototype_get
                && generic_prototype_get < generic_preallocation,
            "RegExp's direct-returning body must run before generic prototype Get and allocation"
        );
    }

    #[test]
    fn date_constructor_realm_prototype_is_required_and_published() {
        let heap = include_str!("heap.rs");
        let functions = include_str!("functions.rs");
        let errors = include_str!("builtins/errors.rs");
        let date = include_str!("builtins/date.rs");
        let standard = include_str!("builtins/standard.rs");
        let bootstrap = include_str!("builtins/bootstrap.rs");
        let host = include_str!("builtins/host.rs");

        let domain = functions
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let offsets = functions
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;
        assert_eq!(domain.matches("    Date,").count(), 1);
        assert_eq!(
            offsets
                .matches("Self::Date => HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET")
                .count(),
            1
        );

        for required in [
            "pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 400;",
            "pub(crate) const HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET: u64 = 344;",
            "name: \"%Date.prototype%\"",
            "offset: HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET",
        ] {
            assert!(
                heap.contains(required),
                "Date realm layout must contain {required}"
            );
        }

        let generic_new_target = errors
            .split_once("pub(crate) fn emit_new_target_prototype_to_locals(")
            .expect("generic new-target prototype operation should exist")
            .1
            .split_once("pub(crate) fn emit_aggregate_error_new_target_prototype_to_local(")
            .expect("generic new-target prototype operation should be bounded")
            .0;
        assert_eq!(
            generic_new_target.matches("self.emit_object_read(").count(),
            1
        );
        assert_eq!(
            generic_new_target
                .matches("NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(intrinsic)")
                .count(),
            1
        );
        assert!(
            generic_new_target.find("self.emit_object_read(").unwrap()
                < generic_new_target
                    .find("NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(intrinsic)")
                    .unwrap(),
            "the observable prototype Get must precede function-realm fallback"
        );
        let required_arm = generic_new_target
            .split_once("NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(intrinsic) => {")
            .expect("required resolved-realm policy arm should exist")
            .1
            .split_once(
                "NewTargetPrototypeFallback::RequiredResolvedRealmMessageErrorActive(kind) => {",
            )
            .expect("required resolved-realm policy arm should be bounded")
            .0;
        assert_eq!(
            required_arm
                .matches("emit_required_new_target_realm_ordinary_prototype(")
                .count(),
            1
        );
        assert!(!required_arm.contains("GlobalGet"));
        assert!(!required_arm.contains("GLOBAL_INDEX"));

        let required_helper = functions
            .split_once("pub(crate) fn emit_required_new_target_realm_ordinary_prototype(")
            .expect("required new-target realm helper should exist")
            .1
            .split_once("/// Consume a required ordinary-object prototype")
            .expect("required new-target realm helper should be bounded")
            .0;
        for call in [
            "emit_get_function_realm(",
            "FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn",
            "emit_load_required_resolved_realm_ordinary_prototype(",
            "emit_install_resolved_realm_ordinary_prototype(",
            "release_resolved_function_realm_local(",
        ] {
            assert_eq!(
                required_helper.matches(call).count(),
                1,
                "required Date fallback must use {call} exactly once"
            );
        }
        assert!(!required_helper.contains("GlobalGet"));
        assert!(!required_helper.contains("GLOBAL_INDEX"));

        let date_wrapper = date
            .split_once("pub(crate) fn emit_date_constructor_prototype_to_locals(")
            .expect("Date constructor prototype wrapper should exist")
            .1
            .split_once("fn emit_date_time_value_from_source(")
            .expect("Date constructor prototype wrapper should be bounded")
            .0;
        assert_eq!(
            date_wrapper
                .matches("NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(")
                .count(),
            1
        );
        assert_eq!(
            date_wrapper
                .matches("OrdinaryDefaultPrototype::Date")
                .count(),
            1
        );
        assert!(date_wrapper.contains("prototype_tag_local: u32"));
        assert!(!date_wrapper.contains("reserve_temp_local"));
        assert!(!date_wrapper.contains("NewTargetPrototypeFallback::CurrentGlobal"));

        let constructor = standard
            .split_once("StandardBuiltinId::DateConstructor => {")
            .expect("Date constructor builtin should exist")
            .1
            .split_once(
                "StandardBuiltinId::DatePrototypeGetTime | StandardBuiltinId::DatePrototypeValueOf",
            )
            .expect("Date constructor builtin should be bounded")
            .0;
        for (operation, count) in [
            ("emit_date_constructor_prototype_to_locals(", 1),
            ("emit_alloc_plain_object_with_prototype_and_tag(", 1),
            ("OBJECT_INTERNAL_BRAND_DATE", 1),
            ("ValueKind::Object.tag() as i64", 2),
        ] {
            assert_eq!(
                constructor.matches(operation).count(),
                count,
                "Date construction must retain exactly {count} {operation} occurrence(s)"
            );
        }
        let prototype_resolution = constructor
            .find("emit_date_constructor_prototype_to_locals(")
            .unwrap();
        for (computation, count) in [
            ("emit_date_current_time_payload(", 1),
            ("emit_tagged_to_primitive_locals(", 2),
            ("emit_value_to_number_payload(", 2),
            ("emit_date_parse_string(", 1),
            ("emit_date_make_day(", 1),
            ("emit_date_time_clip(", 2),
        ] {
            assert_eq!(constructor.matches(computation).count(), count);
            assert!(
                constructor.rfind(computation).unwrap() < prototype_resolution,
                "every Date {computation} emission must precede the observable prototype Get"
            );
        }
        let tagged_allocation = "emit_alloc_plain_object_with_prototype_and_tag(\n                    Some(prototype_payload_local),\n                    Some(prototype_tag_local),\n                    None";
        assert_eq!(constructor.matches(tagged_allocation).count(), 1);
        assert!(prototype_resolution < constructor.find(tagged_allocation).unwrap());
        assert!(!constructor.contains("emit_error_new_target_prototype_to_local("));

        let entry_publication = "emit_store_current_realm_global_intrinsic(\n            DATE_PROTOTYPE_GLOBAL_INDEX,\n            NonArrayRealmIntrinsicSlot::DatePrototype";
        assert_eq!(bootstrap.matches(entry_publication).count(), 1);
        assert_eq!(
            host.matches("self.emit_store_realm_date_prototype(")
                .count(),
            1
        );
    }

    #[test]
    fn error_message_constructors_are_realm_typed_direct_and_tagged() {
        let heap = include_str!("heap.rs");
        let functions = include_str!("functions.rs");
        let errors = include_str!("builtins/errors.rs");
        let error_constructor = include_str!("builtins/errors/constructor.rs");
        let bootstrap = include_str!("builtins/bootstrap.rs");
        let host = include_str!("builtins/host.rs");

        let kind_rows = functions
            .split_once("error_message_constructor_kinds! {")
            .expect("shared-message Error constructor rows should exist")
            .1
            .split_once("/// The fallback selected after")
            .expect("shared-message Error constructor rows should be bounded")
            .0;
        for (kind, constructor, constructor_global, prototype_global, slot) in [
            (
                "Error",
                "ErrorConstructor",
                "ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "ERROR_PROTOTYPE_GLOBAL_INDEX",
                "ErrorPrototype",
            ),
            (
                "EvalError",
                "EvalErrorConstructor",
                "EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "EvalErrorPrototype",
            ),
            (
                "RangeError",
                "RangeErrorConstructor",
                "RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "RangeErrorPrototype",
            ),
            (
                "ReferenceError",
                "ReferenceErrorConstructor",
                "REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "ReferenceErrorPrototype",
            ),
            (
                "SyntaxError",
                "SyntaxErrorConstructor",
                "SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "SyntaxErrorPrototype",
            ),
            (
                "TypeError",
                "TypeErrorConstructor",
                "TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "TypeErrorPrototype",
            ),
            (
                "URIError",
                "URIErrorConstructor",
                "URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX",
                "URI_ERROR_PROTOTYPE_GLOBAL_INDEX",
                "URIErrorPrototype",
            ),
        ] {
            let row_start = format!("    {kind} => {{");
            let row = kind_rows
                .split_once(row_start.as_str())
                .unwrap_or_else(|| panic!("{kind} row should exist"))
                .1
                .split_once("    };")
                .unwrap_or_else(|| panic!("{kind} row should be bounded"))
                .0;
            for value in [constructor, constructor_global, prototype_global, slot] {
                assert!(row.contains(value), "{kind} row must own {value}");
            }
        }
        assert!(functions.contains("pub(crate) const ALL: [Self; 7]"));
        assert!(functions.contains("#[repr(usize)]"));
        assert!(functions.contains("pub(crate) const fn index(self) -> usize"));

        let domain = functions
            .split_once("enum OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype domain should exist")
            .1
            .split_once("}\n\nimpl OrdinaryDefaultPrototype")
            .expect("ordinary default-prototype domain should be bounded")
            .0;
        let offsets = functions
            .split_once("impl OrdinaryDefaultPrototype {")
            .expect("ordinary default-prototype offset map should exist")
            .1
            .split_once("struct ResolvedRealmOrdinaryPrototypeLocal")
            .expect("ordinary default-prototype offset map should be bounded")
            .0;
        assert_eq!(
            domain
                .matches("MessageError(ErrorMessageConstructorKind),")
                .count(),
            1
        );
        assert_eq!(
            offsets
                .matches("Self::MessageError(kind) => kind.prototype_slot().offset()")
                .count(),
            1
        );

        for required in [
            "pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 400;",
            "pub(crate) const HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 0;",
            "pub(crate) const HEAP_REALM_INTRINSICS_ERROR_PROTOTYPE_OFFSET: u64 = 352;",
            "pub(crate) const HEAP_REALM_INTRINSICS_EVAL_ERROR_PROTOTYPE_OFFSET: u64 = 360;",
            "pub(crate) const HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET: u64 = 368;",
            "pub(crate) const HEAP_REALM_INTRINSICS_REFERENCE_ERROR_PROTOTYPE_OFFSET: u64 = 376;",
            "pub(crate) const HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET: u64 = 384;",
            "pub(crate) const HEAP_REALM_INTRINSICS_URI_ERROR_PROTOTYPE_OFFSET: u64 = 392;",
            "name: \"%Error.prototype%\"",
            "name: \"%TypeError.prototype%\"",
            "name: \"%EvalError.prototype%\"",
            "name: \"%RangeError.prototype%\"",
            "name: \"%ReferenceError.prototype%\"",
            "name: \"%SyntaxError.prototype%\"",
            "name: \"%URIError.prototype%\"",
        ] {
            assert!(
                heap.contains(required),
                "Error-family realm layout must contain {required}"
            );
        }

        let witness = error_constructor
            .split_once("struct ErrorConstructorPrototypeLocals {")
            .expect("Error constructor prototype witness should exist")
            .1
            .split_once("impl<'a> FunctionBuilder<'a> {")
            .expect("Error constructor prototype witness should be bounded")
            .0;
        assert!(witness.contains("payload: u32"));
        assert!(witness.contains("tag: u32"));
        assert!(!witness.contains("derive(Clone"));
        assert!(!witness.contains("derive(Copy"));
        assert!(error_constructor.contains(
            "#[must_use = \"the resolved Error-family prototype must be used for allocation and released\"]"
        ));

        let constructor = error_constructor
            .split_once("fn emit_error_message_constructor(")
            .expect("shared Error message constructor body should exist")
            .1
            .split_once("pub(super) fn emit_alloc_error_instance_from_locals(")
            .expect("shared Error message constructor body should be bounded")
            .0;
        assert_eq!(
            constructor
                .matches("emit_error_constructor_prototype(kind, function)")
                .count(),
            1
        );
        assert_eq!(constructor.matches("&prototype").count(), 2);
        assert_eq!(
            constructor
                .matches("emit_install_error_cause_from_arg")
                .count(),
            2
        );
        assert_eq!(
            constructor
                .matches("release_error_constructor_prototype(prototype)")
                .count(),
            1
        );
        assert!(!constructor.contains("emit_error_new_target_prototype_to_local("));

        let producer = error_constructor
            .split_once("fn emit_error_constructor_prototype(")
            .expect("Error prototype producer should exist")
            .1
            .split_once("fn release_error_constructor_prototype(")
            .expect("Error prototype producer should be bounded")
            .0;
        for required in [
            "emit_new_target_prototype_to_locals(",
            "kind.prototype_global_index()",
            "NewTargetPrototypeFallback::RequiredResolvedRealmMessageErrorActive(kind)",
        ] {
            assert_eq!(producer.matches(required).count(), 1, "{required}");
        }
        assert!(!producer.contains("FunctionSnapshot"));
        assert!(!producer.contains("CurrentGlobal"));

        let shared_resolution = errors
            .split_once("    pub(crate) fn emit_new_target_prototype_to_locals(")
            .expect("shared new-target prototype resolver should exist")
            .1
            .split_once("    pub(crate) fn emit_aggregate_error_new_target_prototype_to_local(")
            .expect("shared new-target prototype resolver should be bounded")
            .0;
        assert_eq!(shared_resolution.matches("emit_object_read(").count(), 1);
        let active_selection = shared_resolution
            .find("NewTargetPrototypeFallback::RequiredResolvedRealmMessageErrorActive(kind) =>")
            .expect("active Error-family fallback arm should exist");
        let guarded_get = shared_resolution
            .find("LocalGet(should_get_prototype_local)")
            .expect("the common prototype Get should be guarded");
        let common_get = shared_resolution
            .find("emit_object_read(")
            .expect("the common prototype Get should exist");
        assert!(active_selection < guarded_get && guarded_get < common_get);
        assert!(shared_resolution.contains("kind.constructor_global_index()"));
        let active_arm = shared_resolution[active_selection..]
            .split_once("            _ => {")
            .expect("active Error-family selection should end before the generic arm")
            .0;
        assert_eq!(
            active_arm
                .matches("LocalGet(self.current_env_local)")
                .count(),
            2
        );
        assert_eq!(
            active_arm
                .matches("kind.constructor_global_index()")
                .count(),
            1
        );
        assert!(shared_resolution.contains("OrdinaryDefaultPrototype::MessageError(kind)"));
        assert!(shared_resolution.contains("LocalSet(should_get_prototype_local)"));
        assert!(!errors.contains("emit_native_error_constructor_wrapper"));
        assert!(!errors.contains("Instruction::Call(error_wasm_index)"));

        let allocator = error_constructor
            .split_once("fn emit_alloc_error_instance_from_locals(")
            .expect("Error instance allocator should exist")
            .1
            .split_once("fn emit_error_constructor_prototype(")
            .expect("Error instance allocator should be bounded")
            .0;
        assert_eq!(
            allocator
                .matches("emit_alloc_plain_object_with_prototype_and_tag(")
                .count(),
            1
        );
        assert!(!allocator.contains("emit_alloc_plain_object_with_prototype("));
        assert!(allocator.contains("Some(prototype.payload)"));
        assert!(allocator.contains("Some(prototype.tag)"));

        let direct = functions
            .split_once("let direct_returning_constructor_table_indices: Vec<i64> = [")
            .expect("direct-returning constructor domain should exist")
            .1
            .split_once(".filter_map(|builtin|")
            .expect("direct-returning constructor domain should be bounded")
            .0;
        assert!(direct.contains("ErrorMessageConstructorKind::ALL"));
        assert!(direct.contains(".map(ErrorMessageConstructorKind::constructor)"));

        let realm_publication = functions
            .split_once("pub(crate) fn emit_store_realm_message_error_prototype(")
            .expect("typed created-realm Error prototype publisher should exist")
            .1
            .split_once("pub(crate) fn emit_store_current_realm_message_error_prototype(")
            .expect("typed created-realm Error prototype publisher should be bounded")
            .0;
        assert!(realm_publication.contains("kind.prototype_slot()"));
        assert!(realm_publication.contains("prototype_local"));
        let entry_publication = functions
            .split_once("pub(crate) fn emit_store_current_realm_message_error_prototype(")
            .expect("typed entry-realm Error prototype publisher should exist")
            .1
            .split_once("pub(crate) fn emit_store_non_array_realm_intrinsic(")
            .expect("typed entry-realm Error prototype publisher should be bounded")
            .0;
        assert!(entry_publication.contains("kind.prototype_global_index()"));
        assert!(entry_publication.contains("kind.prototype_slot()"));

        for kind in [
            "Error",
            "EvalError",
            "RangeError",
            "ReferenceError",
            "SyntaxError",
            "TypeError",
            "URIError",
        ] {
            let kind_path = format!("ErrorMessageConstructorKind::{kind}");
            assert_eq!(
                bootstrap.matches(kind_path.as_str()).count(),
                1,
                "entry bootstrap must publish {kind} exactly once"
            );
        }
        let created_publication = host
            .split_once("for (kind, prototype_local) in [")
            .expect("created-realm Error prototype publication should exist")
            .1
            .split_once("self.emit_store_realm_array_iterator_prototype(")
            .expect("created-realm Error prototype publication should be bounded")
            .0;
        for kind in [
            "Error",
            "EvalError",
            "RangeError",
            "ReferenceError",
            "SyntaxError",
            "TypeError",
            "URIError",
        ] {
            let kind_path = format!("ErrorMessageConstructorKind::{kind}");
            assert_eq!(
                created_publication.matches(kind_path.as_str()).count(),
                1,
                "created realm must publish {kind} exactly once"
            );
        }
        assert_eq!(
            created_publication
                .matches("emit_store_realm_message_error_prototype(")
                .count(),
            1
        );
        assert!(host.contains("let error_constructor_metas = ErrorMessageConstructorKind::ALL"));
        assert!(
            host.contains("ErrorMessageConstructorKind::ALL.map(|_| self.reserve_temp_local())")
        );
        assert!(!host.contains("error_constructor_locals[0]"));
        let created_constructors = host
            .split_once("for index in 0..error_constructor_metas.len() {")
            .expect("created-realm Error-family constructor loop should exist")
            .1
            .split_once("let error_is_error_payload_local")
            .expect("created-realm Error constructor loop should be bounded")
            .0;
        assert_eq!(
            created_constructors
                .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
                .count(),
            1,
            "every created-realm Error-family constructor must carry its active function identity"
        );
        let normalized_created_constructors = created_constructors
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(normalized_created_constructors.contains(
            "constructor_local, HEAP_FUNCTION_ENV_HANDLE_OFFSET, constructor_local, function,"
        ));
        assert!(created_constructors.contains("emit_function_value_payload_in_realm("));
    }

    #[test]
    fn string_empty_split_structurally_walks_utf16_code_units() {
        let source = include_str!("builtins/string.rs");
        let helper = source
            .split_once("mod empty_string_split_units {")
            .expect("empty-split local domain should exist")
            .1
            .split_once("mod string_code_unit_access {")
            .expect("empty-split local domain should end before code-unit access")
            .0;
        let split = source
            .split_once("pub(crate) fn emit_string_split_from_string_locals(")
            .expect("ordinary String split emitter should exist")
            .1
            .split_once("pub(crate) fn emit_string_split_regexp_source_from_string_locals(")
            .expect("ordinary String split emitter should have a bounded body")
            .0;

        assert_eq!(
            split.matches("empty_string_split_units::emit(").count(),
            1,
            "the empty-separator branch must delegate to the private UTF-16 unit coordinator once"
        );
        for local in ["UnitIndexLocal", "UnitLengthLocal", "OneUnitLocal"] {
            assert_eq!(
                helper.matches(&format!("struct {local}")).count(),
                1,
                "the {local} domain must have one opaque definition"
            );
        }
        assert!(
            helper.contains("index: UnitIndexLocal,"),
            "the one-unit materializer must require a UTF-16 unit index"
        );
        assert!(
            helper.contains("one: OneUnitLocal,"),
            "the one-unit materializer must require the one-code-unit width"
        );
        assert_eq!(
            helper
                .matches("emit_utf16_code_unit_range_payload_from_locals(")
                .count(),
            1,
            "the typed one-unit boundary must use the authoritative UTF-16 range operation"
        );
        assert!(
            !helper.contains("emit_decode_utf8_scalar_at_index("),
            "the coordinator must not advance one split element per direct scalar decode; the authoritative UTF-16 helpers may decode internally"
        );
        assert!(
            !helper.contains("emit_string_slice_payload_from_locals("),
            "the coordinator must not materialize one split element as a raw byte slice"
        );
    }

    #[test]
    fn string_char_access_structurally_uses_typed_utf16_units() {
        let string = include_str!("builtins/string.rs");
        let standard = include_str!("builtins/standard.rs");
        let array = include_str!("builtins/array.rs");
        let helper = string
            .split_once("mod string_code_unit_access {")
            .expect("String code-unit access local domain should exist")
            .1
            .split_once("pub(crate) enum UriCodecKind")
            .expect("String code-unit access domain should end before URI codecs")
            .0;
        let char_at_wrapper = string
            .split_once("pub(crate) fn emit_string_char_at_from_locals(")
            .expect("shared charAt locals entry point should exist")
            .1
            .split_once("pub(crate) fn emit_string_at_from_locals(")
            .expect("charAt locals entry point should have a bounded body")
            .0;
        let at_wrapper = string
            .split_once("pub(crate) fn emit_string_at_from_locals(")
            .expect("shared at locals entry point should exist")
            .1
            .split_once("pub(crate) fn emit_string_at_method_call(")
            .expect("at locals entry point should have a bounded body")
            .0;
        let direct_char_at = string
            .split_once("pub(crate) fn emit_string_char_at_method_call(")
            .expect("optimized direct charAt emitter should exist")
            .1
            .split_once("pub(crate) fn emit_string_match_method_call(")
            .expect("optimized direct charAt emitter should have a bounded body")
            .0;
        let direct_builtin = array
            .split_once("pub(crate) fn emit_array_direct_builtin_method_call(")
            .expect("shared direct builtin caller should exist")
            .1
            .split_once("pub(crate) fn emit_array_push_method_call(")
            .expect("shared direct builtin caller should have a bounded body")
            .0;
        let standard_char_at = standard
            .split_once("StandardBuiltinId::StringPrototypeCharAt => {")
            .expect("standard charAt arm should exist")
            .1
            .split_once("StandardBuiltinId::StringPrototypeAt => {")
            .expect("standard charAt arm should have a bounded body")
            .0;
        let standard_at = standard
            .split_once("StandardBuiltinId::StringPrototypeAt => {")
            .expect("standard at arm should exist")
            .1
            .split_once("StandardBuiltinId::StringPrototypeCharCodeAt => {")
            .expect("standard at arm should have a bounded body")
            .0;

        for local in ["UnitIndexLocal", "UnitLengthLocal", "OneUnitLocal"] {
            assert_eq!(
                helper.matches(&format!("struct {local}")).count(),
                1,
                "the char-access domain must own one opaque {local}"
            );
            assert!(
                helper.contains(&format!("struct {local}(u32);")),
                "{local} must keep its raw local handle private"
            );
        }
        assert_eq!(helper.matches("#[must_use]").count(), 3);
        assert!(!helper.contains("derive(Clone, Copy)"));
        assert!(helper.contains("index: &UnitIndexLocal,"));
        assert!(helper.contains("one: &OneUnitLocal,"));
        assert_eq!(
            helper
                .matches("emit_utf16_code_unit_range_payload_from_locals(")
                .count(),
            1,
            "the typed one-unit materializer must have one authoritative UTF-16 range call"
        );
        for forbidden in [
            "emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(",
            "emit_string_slice_payload_from_locals(",
            "emit_decode_utf8_scalar_at_index(",
        ] {
            assert!(
                !helper.contains(forbidden),
                "the char-access coordinator must not contain alternate materialization `{forbidden}`"
            );
        }
        assert_eq!(helper.matches("pub(super) fn emit_char_at(").count(), 1);
        assert_eq!(helper.matches("pub(super) fn emit_at(").count(), 1);
        assert!(helper.contains("Method::CharAt => {"));
        assert!(helper.contains("Method::At => {"));
        assert!(!helper.contains("_ =>"));
        assert!(helper.contains("Instruction::I64TruncSatF64S"));

        assert_eq!(
            char_at_wrapper
                .matches("string_code_unit_access::emit_char_at(")
                .count(),
            1
        );
        assert_eq!(
            at_wrapper
                .matches("string_code_unit_access::emit_at(")
                .count(),
            1
        );
        assert_eq!(
            direct_char_at
                .matches("self.emit_array_direct_builtin_method_call(")
                .count(),
            1
        );
        assert!(direct_char_at.contains("StandardBuiltinId::StringPrototypeCharAt,"));
        let receiver_evaluation = direct_builtin
            .find("self.compile_expr_to_locals(")
            .expect("direct builtin caller must evaluate its receiver");
        let argument_evaluation = direct_builtin
            .find("self.emit_call_args_vector(args, function)")
            .expect("direct builtin caller must evaluate its complete argument list");
        let builtin_entry = direct_builtin
            .find("self.emit_direct_js_call_with_argv(")
            .expect("direct builtin caller must enter the standard builtin after evaluation");
        assert!(
            receiver_evaluation < argument_evaluation && argument_evaluation < builtin_entry,
            "receiver and complete argument evaluation must precede receiver/index coercion"
        );

        assert_eq!(
            standard_char_at
                .matches("self.emit_string_char_at_from_locals(")
                .count(),
            1
        );
        assert_eq!(
            standard_at
                .matches("self.emit_string_at_from_locals(")
                .count(),
            1
        );
        for body in [direct_char_at, standard_char_at, standard_at] {
            for forbidden in [
                "emit_value_to_string_payload(",
                "emit_value_to_number_payload(",
                "emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(",
                "emit_utf16_code_unit_range_payload_from_locals(",
                "emit_string_slice_payload_from_locals(",
            ] {
                assert!(
                    !body.contains(forbidden),
                    "char-access call sites must delegate coercion and materialization, not call `{forbidden}`"
                );
            }
        }
    }

    #[test]
    fn error_prototype_to_string_has_typed_ordered_observable_phases() {
        let source = include_str!("builtins/errors.rs");
        let operations = include_str!("operations.rs");
        let body = source
            .split_once("fn emit_error_prototype_to_string(")
            .expect("Error.prototype.toString emitter should exist")
            .1
            .split_once("fn emit_error_to_string_prepare_name(")
            .expect("Error.prototype.toString body should end at its name phase")
            .0;
        let prepare_name = source
            .split_once("fn emit_error_to_string_prepare_name(")
            .expect("Error.prototype.toString name phase should exist")
            .1
            .split_once("fn emit_error_to_string_message_and_result(")
            .expect("the name phase should end at the message phase")
            .0;
        let message_and_result = source
            .split_once("fn emit_error_to_string_message_and_result(")
            .expect("Error.prototype.toString message phase should exist")
            .1
            .split_once("fn emit_error_to_string_value_to_string_local(")
            .expect("the message phase should have a bounded body")
            .0;
        let value_to_string = source
            .split_once("fn emit_error_to_string_value_to_string_local(")
            .expect("Error.prototype.toString conversion boundary should exist")
            .1
            .split_once("pub(crate) fn emit_install_error_cause_from_arg(")
            .expect("the conversion boundary should have a bounded body")
            .0;

        assert_eq!(
            body.matches("emit_is_heap_object_like_tag_i32(receiver_tag_local, function)")
                .count(),
            1,
            "receiver admission must use the shared object-representation authority once"
        );
        for representation in ["Object", "Function", "Array", "Arguments"] {
            assert!(
                !body.contains(&format!("ValueKind::{representation}")),
                "the builtin body must not maintain a second {representation} admission list"
            );
        }
        let prepare_call = body
            .find("emit_error_to_string_prepare_name(")
            .expect("the builtin must prepare name");
        let message_call = body
            .find("emit_error_to_string_message_and_result(")
            .expect("the builtin must consume name in its message phase");
        assert!(
            prepare_call < message_call,
            "message lookup must be emitted only after name preparation"
        );

        assert!(
            source.contains(
                "#[must_use = \"the prepared Error name must be consumed before reading message\"]\nstruct PreparedErrorNameLocal"
            ),
            "the cross-phase name state must be private and must-use"
        );
        assert!(
            prepare_name.contains("Result<PreparedErrorNameLocal, EmitError>"),
            "the name phase must return typed prepared state"
        );
        assert_eq!(prepare_name.matches("self.emit_object_read(").count(), 1);
        assert!(
            prepare_name
                .find("self.emit_object_read(")
                .expect("name phase must Get name")
                < prepare_name
                    .find("self.emit_error_to_string_value_to_string_local(")
                    .expect("name phase must ToString name"),
            "name Get must precede name ToString"
        );
        assert_eq!(
            prepare_name
                .matches("self.emit_error_to_string_value_to_string_local(")
                .count(),
            1,
            "name conversion must cross the routed ToString boundary once"
        );
        assert!(
            message_and_result.contains("prepared_name: PreparedErrorNameLocal"),
            "the message phase must require prepared name state"
        );
        assert_eq!(
            message_and_result
                .matches("prepared_name.into_local()")
                .count(),
            1
        );
        assert_eq!(
            message_and_result.matches("self.emit_object_read(").count(),
            1
        );
        assert_eq!(
            message_and_result
                .matches("self.emit_error_to_string_value_to_string_local(")
                .count(),
            1,
            "message conversion must cross the routed ToString boundary once"
        );
        assert_eq!(
            value_to_string
                .matches("emit_tagged_to_primitive_locals_in_current_function_realm(")
                .count(),
            1,
            "ToPrimitive must use the fixed current-function-realm wrapper"
        );
        assert_eq!(
            value_to_string
                .matches("emit_current_function_realm_primitive_to_string_local(")
                .count(),
            1,
            "primitive ToString must consume the matching current-realm token"
        );
        assert!(
            !value_to_string.contains("self.emit_tagged_to_primitive_locals("),
            "the builtin must not select the existing main-Realm ToPrimitive wrapper"
        );
        assert!(
            !value_to_string.contains("self.emit_primitive_to_string_payload("),
            "the builtin must not select the existing main-Realm primitive ToString wrapper"
        );

        let current_primitive = operations
            .split_once("pub(crate) fn emit_tagged_to_primitive_locals_in_current_function_realm(")
            .expect("the fixed current-realm ToPrimitive wrapper should exist")
            .1
            .split_once("pub(crate) fn emit_current_function_realm_primitive_to_string_local(")
            .expect("the current-realm ToPrimitive wrapper should have a bounded body")
            .0;
        let current_string = operations
            .split_once("pub(crate) fn emit_current_function_realm_primitive_to_string_local(")
            .expect("the fixed current-realm primitive ToString wrapper should exist")
            .1
            .split_once("fn emit_tagged_to_primitive_locals_pending(")
            .expect("the current-realm primitive ToString wrapper should have a bounded body")
            .0;
        assert!(current_primitive
            .contains("let error_realm = ConversionErrorRealm::CurrentFunctionRealm;"));
        assert!(current_primitive.contains("ConversionErrorRealmSource::Fixed(error_realm)"));
        assert!(current_primitive.contains("error_realm,"));
        assert!(current_string.contains("error_realm,"));
        assert!(current_string.contains("ConversionErrorRealmSource::Fixed(error_realm)"));
        assert!(current_string.contains("emit_primitive_to_string_payload_with_error_realm("));

        let helper_call = operations
            .split_once("fn emit_value_to_primitive_via_helper_if_outlined(")
            .expect("the outlined ToPrimitive boundary should exist")
            .1
            .split_once("pub(crate) fn emit_tagged_to_primitive_locals(")
            .expect("the outlined ToPrimitive boundary should have a bounded body")
            .0;
        assert!(helper_call.contains("self.emit_conversion_error_realm_argument(error_realm"));
        assert!(helper_call.contains("for _ in 0..3"));
        assert!(helper_call.contains("LocalGet(self.current_env_local)"));
        assert!(
            operations.contains("ConversionErrorRealmSource::RuntimeHelperArgument"),
            "the outlined helper body must decode the forwarded closed realm word"
        );
        assert!(
            operations.contains("ConversionErrorRealm::MainRealm.abi_word()")
                && operations.contains("ConversionErrorRealm::CurrentFunctionRealm.abi_word()"),
            "the helper decoder must cover both conversion-error realm words"
        );
    }

    #[test]
    fn operations_emits_to_boolean_spec_operation() {
        let source = parse("Boolean(globalThis.flag);", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=2"));
        let artifact = emit(&program).expect("spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn plain_async_loop_awaits_emit_a_valid_module() {
        // Each of these lowers to a `StatementIr::GeneratorLoop` that the plain
        // async body compiles against `HEAP_ASYNC_RESUME_STATE_OFFSET`, running
        // one iteration per invocation of the body.
        for source in [
            "(async function(){ let t = 0; for (let i = 0; i < 3; i++) { t += await Promise.resolve(i); } print(t); })();",
            "(async function(){ let t = 0; for (let i = 0; i < 3; i++) { const v = await Promise.resolve(i); t += v; } print(t); })();",
            "(async function(){ let n = 0; while (n < 3) { n++; await Promise.resolve(n); } print(n); })();",
            "(async function(){ const out = []; for (const x of [1,2,3]) { out.push(await Promise.resolve(x)); } print(out); })();",
        ] {
            let artifact = emit_script(source)
                .unwrap_or_else(|err| panic!("{source} should emit: {err:?}"));
            expect_valid_module(&artifact, 0);
        }
    }

    #[test]
    fn full_bootstrap_emits_without_proto_source_reference() {
        let artifact = emit_script("this;").expect("full bootstrap script should emit");

        expect_valid_module(&artifact, 0);
    }

    /// Reads the Wasm `name` section back out of an emitted module.
    ///
    /// Wasmtime builds its per-function symbol as
    /// `wasm[0]::function[N]::<clean_symbol(name)>` from exactly this section,
    /// so a module that emits it turns an anonymous native-compilation failure
    /// into a named one.
    fn function_names(artifact: &WasmArtifact) -> BTreeMap<u32, String> {
        let mut names = BTreeMap::new();
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            let Payload::CustomSection(section) = payload.expect("module should parse") else {
                continue;
            };
            let wasmparser::KnownCustom::Name(subsections) = section.as_known() else {
                continue;
            };
            for subsection in subsections {
                if let wasmparser::Name::Function(map) =
                    subsection.expect("name subsection should parse")
                {
                    for naming in map {
                        let naming = naming.expect("function naming should parse");
                        names.insert(naming.index, naming.name.to_string());
                    }
                }
            }
        }
        names
    }

    /// Number of *function* imports, which is the index the first code-section
    /// body occupies. Read back from the encoded module rather than from the
    /// emitter's own variable, so it is an independent witness of the base the
    /// name section indices must start from.
    fn imported_function_count(artifact: &WasmArtifact) -> u32 {
        let mut count = 0u32;
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            let Payload::ImportSection(reader) = payload.expect("module should parse") else {
                continue;
            };
            for import in reader.into_imports() {
                let import = import.expect("import should parse");
                if matches!(
                    import.ty,
                    wasmparser::TypeRef::Func(_) | wasmparser::TypeRef::FuncExact(_)
                ) {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn emitted_modules_name_every_function() {
        let artifact = emit_script("function outer() { return 1; } outer();")
            .expect("named function script should emit");
        expect_valid_module(&artifact, 1);

        let names = function_names(&artifact);
        // The index *base* is the load-bearing half. `names.len() == code_entries`
        // and every prefix check below stay green if `ModuleCode::new` were given
        // `0` instead of the imported function count — every name would then be
        // off by that count and wasmtime's `wasm[0]::function[N]` label, the
        // entire point of the section, would point at the wrong function. `main`
        // is the first body pushed, so it must sit at exactly the first
        // non-imported index.
        let first_body_index = imported_function_count(&artifact);
        assert!(first_body_index > 0, "the module must import functions");
        assert_eq!(
            names.keys().next(),
            Some(&first_body_index),
            "name section must start at the first non-imported function index: {names:?}"
        );
        assert_eq!(
            names.get(&first_body_index).map(String::as_str),
            Some("lila::main"),
            "main is pushed first, so it must own the first non-imported index: {names:?}"
        );
        assert!(
            names.values().any(|name| name.starts_with("js::outer")),
            "user functions must be named: {names:?}"
        );
        assert!(
            names.values().any(|name| name == "helper::heap_alloc"),
            "runtime helpers must be named: {names:?}"
        );
        assert!(
            names.values().any(|name| name.starts_with("builtin::")),
            "compiled builtins must be named: {names:?}"
        );

        // Every code-section entry is named, because every entry goes through
        // `ModuleCode::push(EmittedFunction)` and an `EmittedFunction` cannot be
        // built without an identity.
        let mut code_entries = 0usize;
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            if let Payload::CodeSectionEntry(_) = payload.expect("module should parse") {
                code_entries += 1;
            }
        }
        assert_eq!(names.len(), code_entries);
    }

    #[test]
    fn debug_dump_attributes_the_largest_emitted_function() {
        let artifact = emit_script("this;").expect("full bootstrap script should emit");
        let largest = artifact
            .debug_dump
            .lines()
            .find(|line| line.starts_with("largest emitted function: "))
            .expect("debug_dump should attribute the largest emitted body");
        // The `none` fallback line carries no keys, so the presence of the
        // measured `key=value` shape is what proves a body was attributed. These
        // are exactly the keys `tests/emit_golden.rs::largest_function` parses.
        for key in ["index=", "bytes=", "locals=", "kind=", "name="] {
            assert!(largest.contains(key), "{largest}");
        }
        let most_locals = artifact
            .debug_dump
            .lines()
            .find(|line| line.starts_with("most locals in an emitted function: "))
            .expect("debug_dump should report the most-locals body separately");
        for key in ["index=", "bytes=", "locals=", "kind=", "name="] {
            assert!(most_locals.contains(key), "{most_locals}");
        }
        assert!(
            artifact
                .debug_dump
                .lines()
                .any(|line| line.starts_with("emitted code bytes: ")),
            "{}",
            artifact.debug_dump
        );
    }

    /// The typed per-function report has exactly one row per code-section
    /// entry in the encoded module.
    ///
    /// **What in here can actually fail, and what cannot.** The `largest` vs
    /// `debug_dump` comparison below is near-tautological by construction:
    /// `emit()` builds `function_sizes` and renders the `largest emitted
    /// function:` line from the *same* slice in adjacent statements, so the two
    /// cannot disagree without an edit that deliberately splits them. It is kept
    /// as a guard against exactly that re-split — two independently-maintained
    /// size reports are how `runtime helper functions: 27` came to disagree with
    /// a counted 32 + 1 — but it is not evidence.
    ///
    /// The falsifiable assertion is `function_sizes.len() == code_entries`,
    /// counted by an independent `wasmparser` walk of the encoded bytes. That is
    /// what the name promises and it is the only part that can catch a real
    /// defect.
    ///
    /// This test also does **not** cover every emit path: it calls the
    /// in-process `emit_script` helper only. `Engine::emit_wasm_on_current_thread`
    /// and `run_with_wasm_aot_inner` — the two paths where the dump was actually
    /// being dropped — are covered in `lila-engine`, not here, because this
    /// crate cannot reach them.
    #[test]
    fn typed_report_row_count_matches_the_code_section() {
        let artifact = emit_script("this;").expect("full bootstrap script should emit");
        assert!(
            !artifact.function_sizes.is_empty(),
            "every emitted module has at least `lila::main`"
        );

        // Independent witness: the number of typed rows must equal the number
        // of code-section entries actually in the encoded module.
        let mut code_entries = 0usize;
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            if let Payload::CodeSectionEntry(_) = payload.expect("module should parse") {
                code_entries += 1;
            }
        }
        assert_eq!(artifact.function_sizes.len(), code_entries);

        let largest = artifact
            .function_sizes
            .iter()
            .max_by_key(|summary| summary.body_bytes)
            .expect("a non-empty report has a largest entry");
        let line = artifact
            .debug_dump
            .lines()
            .find(|line| line.starts_with("largest emitted function: "))
            .expect("debug_dump should attribute the largest emitted body");
        assert!(
            line.ends_with(&format!(" name={}", largest.name)),
            "typed report says {} but debug_dump says {line}",
            largest.name
        );
        assert!(
            line.contains(&format!(" bytes={} ", largest.body_bytes.bytes())),
            "typed report says {} bytes but debug_dump says {line}",
            largest.body_bytes.bytes()
        );
        assert!(
            line.contains(&format!(" index={} ", largest.wasm_index)),
            "typed report says index {} but debug_dump says {line}",
            largest.wasm_index
        );

        // The category comes from `FunctionIdentity::category`, an exhaustive
        // match, so this also pins that a bootstrap module's biggest body is
        // still classified rather than falling into some unnamed bucket.
        assert!(
            [
                "main",
                "script",
                "builtin",
                "builtin-stub",
                "host-builtin",
                "runtime-helper"
            ]
            .contains(&largest.category),
            "unclassified category {}",
            largest.category
        );
    }

    /// A user function that does one dynamic-key property read must not carry a
    /// six-figure body.
    ///
    /// The probe is the exact text measured on this tree on 2026-08-09 with
    /// `LILA_WASM_DUMP=... lila build wasm` plus a code-section/`name`-section
    /// read-back. Ablation of that same probe, one line at a time:
    ///
    /// | probe body | `js::probe#f0` |
    /// |---|---|
    /// | `return 0;` | 2,215 |
    /// | `var A = k.split('-'); return A.length;` | 14,754 |
    /// | ... plus `x = A[0];` (static index) | 14,911 |
    /// | ... plus `x = A[i];` (**this probe**) | **87,101** |
    /// | ... plus a second `y = A[j];` | 159,811 |
    ///
    /// So one dynamic key costs `(159,811 - 14,754) / 2 = 72,528` bytes and the
    /// floor for this probe once ToPropertyKey/ToPrimitive are outlined is the
    /// 14,754-byte row plus a call. The budget is set at 30,000: comfortably
    /// above that floor, less than half of the pre-split 87,101, so it is RED
    /// before the split and GREEN after without being a tripwire on ordinary
    /// drift.
    ///
    /// **The absolute budget alone is not enough**, and that is why the static
    /// control below exists. The post-split floor is ~14,900 (the static-index
    /// row) plus a call, so *anything* from ~15,000 to 30,000 satisfies the
    /// budget — including a half-landed split in which the ToPropertyKey seam
    /// fires and the ToPrimitive one does not, or the reverse. The budget
    /// asserts a constant; it cannot express the relationship it means.
    ///
    /// So the test emits a second probe that is the same text with a **static**
    /// index (`x = A[0];`) and asserts that the dynamic body costs the static
    /// body plus at most [`DYNAMIC_KEY_MARGIN_BYTES`]. That is the real claim:
    /// *a dynamic key costs about what a static key costs, plus a call.* The
    /// margin is 10,000 rather than the few hundred bytes a seam plus a call
    /// should really cost, because the post-split delta has not been measured —
    /// but 10,000 is one seventh of the 72,528 bytes a single inline copy of
    /// either composite occupies, so no partially-fired seam can hide inside it.
    /// Tighten it towards ~1,000 once the delta is counted.
    ///
    /// It asserts on a **named** function. Asserting on the largest body in the
    /// module would be vacuous: in any bootstrap-heavy module that is a builtin
    /// (`builtin::Object.defineProperty`, 375,534 bytes in this very probe),
    /// which stays green while the user function regresses by an order of
    /// magnitude.
    ///
    /// Note the module contains *two* bodies for this function —
    /// `js::probe#f0` and `js::probe#f0$exact_helper_context$0` — so the check
    /// is over every body whose name starts with the function's name, and it
    /// requires at least one to exist rather than passing on an empty set.
    #[test]
    fn emitted_function_bodies_stay_under_budget() {
        const PROBE: &str = "function probe(k, i, j) {\n  \
             var A = k.split('-');\n  \
             var x = '';\n  \
             x = A[i];\n  \
             return x;\n\
             }\n\
             print(probe('a-b-c', 0, 1));\n";
        /// The same text with a static index. Every other line is identical, so
        /// the difference between the two largest `js::probe#` bodies is the
        /// cost of the dynamic key and nothing else.
        const STATIC_CONTROL: &str = "function probe(k, i, j) {\n  \
             var A = k.split('-');\n  \
             var x = '';\n  \
             x = A[0];\n  \
             return x;\n\
             }\n\
             print(probe('a-b-c', 0, 1));\n";
        const BUDGET_BYTES: u32 = 30_000;
        /// See the doc comment: one inline copy of either composite is 72,528
        /// bytes, so a margin this size cannot conceal a half-fired seam.
        const DYNAMIC_KEY_MARGIN_BYTES: u32 = 10_000;

        /// Largest emitted body whose name starts with `js::probe#`. There are
        /// two (`js::probe#f0` and `js::probe#f0$exact_helper_context$0`), and
        /// an empty set must fail rather than pass vacuously.
        fn largest_probe_body(artifact: &WasmArtifact) -> (String, u32) {
            let probe_bodies = artifact
                .function_sizes
                .iter()
                .filter(|summary| summary.name.starts_with("js::probe#"))
                .collect::<Vec<_>>();
            assert!(
                !probe_bodies.is_empty(),
                "the probe function must be emitted, or this budget is vacuous: {:?}",
                artifact
                    .function_sizes
                    .iter()
                    .map(|summary| summary.name.as_str())
                    .collect::<Vec<_>>()
            );
            let largest = probe_bodies
                .iter()
                .max_by_key(|summary| summary.body_bytes.bytes())
                .expect("non-empty");
            (largest.name.clone(), largest.body_bytes.bytes())
        }

        let artifact = emit_script(PROBE).expect("probe script should emit");
        let probe_bodies = artifact
            .function_sizes
            .iter()
            .filter(|summary| summary.name.starts_with("js::probe#"))
            .collect::<Vec<_>>();
        assert!(
            !probe_bodies.is_empty(),
            "the probe function must be emitted, or this budget is vacuous: {:?}",
            artifact
                .function_sizes
                .iter()
                .map(|summary| summary.name.as_str())
                .collect::<Vec<_>>()
        );
        for body in probe_bodies {
            assert!(
                body.body_bytes.bytes() <= BUDGET_BYTES,
                "{} is {} bytes against a budget of {BUDGET_BYTES}; \
                 one dynamic-key property read should not cost a six-figure body",
                body.name,
                body.body_bytes.bytes()
            );
        }

        // The relational half. `BUDGET_BYTES` above is a constant and cannot
        // distinguish "both composites outlined" from "one of the two".
        let (dynamic_name, dynamic_bytes) = largest_probe_body(&artifact);
        let control = emit_script(STATIC_CONTROL).expect("static control script should emit");
        let (static_name, static_bytes) = largest_probe_body(&control);
        assert!(
            dynamic_bytes >= static_bytes,
            "a dynamic key cannot be cheaper than a static one: \
             {dynamic_name} is {dynamic_bytes} bytes, {static_name} is {static_bytes}"
        );
        let delta = dynamic_bytes - static_bytes;
        assert!(
            delta <= DYNAMIC_KEY_MARGIN_BYTES,
            "a dynamic key costs {delta} bytes over the static control \
             ({dynamic_name} {dynamic_bytes} vs {static_name} {static_bytes}), \
             against a margin of {DYNAMIC_KEY_MARGIN_BYTES}. One inline copy of the \
             ToPrimitive/ToPropertyKey composite is 72,528 bytes, so a delta this \
             large means at least one of the two seams did not fire — read the two \
             numbers rather than only raising the margin"
        );
    }

    /// The `LILA_EMIT_SIZE_REPORT_PATH` sink writes one line per emitted
    /// function, from the same traversal as the typed report, with the largest
    /// body first.
    ///
    /// This exists because the sink was the one mechanism the size-report work
    /// added specifically so that "I set the variable and saw nothing" could not
    /// be read as "there are no large functions" — and it was itself protected
    /// only by review. The env read is deliberately *not* exercised here (it
    /// would be visible to every other test in the process); this drives
    /// `write_size_report_file`, which is everything the env wrapper does after
    /// reading the variable.
    #[test]
    fn the_size_report_file_is_the_same_traversal_as_the_typed_report() {
        let artifact = emit_script("this;").expect("full bootstrap script should emit");
        let path =
            std::env::temp_dir().join(format!("lila-emit-size-report-{}.txt", std::process::id()));
        crate::emitted_function::write_size_report_file(&path, &artifact.function_sizes);

        let written = std::fs::read_to_string(&path).expect("the sink must write the report file");
        let _ = std::fs::remove_file(&path);

        let lines = written.lines().collect::<Vec<_>>();
        assert_eq!(
            lines.len(),
            artifact.function_sizes.len(),
            "the report file must hold one row per typed summary"
        );

        let largest_bytes = artifact
            .function_sizes
            .iter()
            .map(|summary| summary.body_bytes.bytes())
            .max()
            .expect("a non-empty report has a largest entry");
        assert!(
            lines[0].contains(&format!(" bytes={largest_bytes} ")),
            "the first report row must be a largest body ({largest_bytes} bytes), got {:?}",
            lines[0]
        );
        // `report_lines` breaks size ties by ascending wasm index while
        // `EmittedFunctionSummary::largest` keeps the last maximum, so assert on
        // the row's own identity rather than assuming the two pick the same tie.
        let reported_name = lines[0]
            .rsplit_once(" name=")
            .map(|(_, name)| name)
            .expect("every report row ends with name=");
        assert!(
            artifact.function_sizes.iter().any(|summary| {
                summary.name == reported_name && summary.body_bytes.bytes() == largest_bytes
            }),
            "the first report row names {reported_name}, which is not a largest typed summary"
        );
    }

    #[test]
    fn runtime_helper_count_is_derived_not_asserted() {
        // The point of this test is that the reported figure is *derived from
        // the registry*, not hand-written: the literal `27` it replaces had
        // drifted from the truth by five. So the expectation is spelled from
        // `RuntimeHelperId::ALL` and a new helper moves both sides together.
        //
        // Exactly one helper is conditional (`JSON.stringify`'s value helper).
        // That does NOT make the emitted count vary by script: `emit` gates it
        // on `compiled_standard_builtins.contains(JsonStringify)`, and the
        // default bootstrap installs the full global object, so
        // `JSON.stringify` has a compiled body for every script — including one
        // that never mentions `JSON`. Asserting a lower count for `this;` would
        // be asserting a demand-driven bootstrap this backend does not have
        // yet, which is why that assertion failed the first time it was run.
        let conditional = RuntimeHelperId::ALL
            .iter()
            .filter(|helper| helper.is_conditional())
            .count();
        assert_eq!(
            conditional, 1,
            "only JSON.stringify's value helper is conditional; \
             a second conditional helper needs this test to distinguish them"
        );

        let expected = format!("runtime helper functions: {}", RuntimeHelperId::ALL.len());
        for source in ["this;", "JSON.stringify({});"] {
            let artifact = emit_script(source).expect("script should emit");
            assert!(
                artifact.debug_dump.lines().any(|line| line == expected),
                "expected `{expected}` for `{source}`\n{}",
                artifact.debug_dump
            );
        }
    }

    #[test]
    fn call_spread_iterator_paths_emit_valid_module() {
        let artifact = emit_script(
            r#"
function iterable(values) {
  return {
    [Symbol.iterator]() {
      let index = 0;
      return {
        next() {
          if (index === values.length) return { done: true };
          return { done: false, value: values[index++] };
        }
      };
    }
  };
}
function collect() { return arguments.length; }
class Base { constructor(first, second) { this.total = first + second; } }
class Derived extends Base {
  constructor(values) { super(1, ...values); }
}
let source = iterable([2, 3]);
let indirect = collect;
collect(0, ...source, 4);
indirect(...source);
new Base(...source);
new Derived(source);
"#,
        )
        .expect("call spread iterator paths should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn object_literal_copy_data_properties_module_validates() {
        let artifact = emit_script(
            r#"
let calls = [];
let symbol = Symbol("copied");
let source = { get visible() { calls.push("get"); return 2; } };
source[symbol] = 3;
Object.defineProperty(source, "hidden", { value: 4, enumerable: false });
let proxy = new Proxy(source, {
  ownKeys(target) {
    calls.push("keys");
    return ["visible", "hidden", symbol];
  },
  getOwnPropertyDescriptor(target, key) {
    calls.push(key === symbol ? "descriptor:symbol" : "descriptor:" + key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  get(target, key) {
    calls.push(key === symbol ? "get:symbol" : "get:" + key);
    return Reflect.get(target, key);
  }
});
let result = { before: 1, ...null, ...undefined, ...proxy, visible: 5, ..."xy" };
result.visible + result[symbol] + result[0] + result[1] + calls.length;
"#,
        )
        .expect("object literal CopyDataProperties paths should emit");

        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn generator_parameter_modules_emit_valid_intrinsic_global_references() {
        let artifact = emit_script("function* f({ value }) {} f({ value: 1 });")
            .expect("generator parameter script should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_parameter_initialization_module_validates() {
        let artifact = emit_script(
            "let g = async function* (value = (g.prototype = null)) {}; let oldPrototype = g.prototype; let iterator = g(); Object.getPrototypeOf(iterator) !== oldPrototype;",
        )
        .expect("async-generator parameter initialization should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_invalid_receiver_rejection_module_validates() {
        let artifact = emit_script(
            "async function* stream() {} function* syncStream() {} let methods = [stream.prototype.next, stream.prototype.return, stream.prototype.throw]; let receivers = [1, {}, function () {}, syncStream()]; for (let methodIndex = 0; methodIndex < methods.length; methodIndex += 1) { for (let receiverIndex = 0; receiverIndex < receivers.length; receiverIndex += 1) { methods[methodIndex].call(receivers[receiverIndex]).then(undefined, function () {}); } }",
        )
        .expect("invalid async-generator receivers should emit rejected promises");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_iterator_async_dispose_module_validates() {
        let artifact = emit_script(
            "async function* stream() {} let prototype = Object.getPrototypeOf(Object.getPrototypeOf(stream.prototype)); let fulfilled = Object.create(prototype); fulfilled.return = function (value) { return Promise.resolve(value); }; let rejected = Object.create(prototype); rejected.return = function () { return Promise.reject(1); }; let absent = Object.create(prototype); prototype[Symbol.asyncDispose].call(fulfilled); prototype[Symbol.asyncDispose].call(rejected).catch(function () {}); prototype[Symbol.asyncDispose].call(absent);",
        )
        .expect("AsyncIterator asyncDispose paths should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_no_suspension_conditional_module_validates() {
        let artifact = emit_script(
            "async function* choose(flag) { let value = 0; if (flag) { value = 1; if (false) value = 2; } else { value = 3; } return value; } choose(true).next(); choose(false).next();",
        )
        .expect("async-generator ordinary conditionals should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_direct_yield_conditionals_validate() {
        for source in [
            "async function* choose(flag) { if (flag) yield 1; return 2; } choose(true).next(); choose(false).next();",
            "async function* choose(flag) { if (flag) yield 1; else yield 2; return 3; } choose(true).next(); choose(false).next();",
            "async function* choose(flag) { var value = flag ? 1 : 3; if (flag) { value += 1; yield value; value += 1; } else { value += 2; yield value; value += 2; } return value; } choose(true).next(); choose(false).next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator direct-yield branch should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_await_conditional_suspension_remains_explicit() {
        let error = emit_script(
            "async function* choose(flag) { if (flag) await 1; return 2; } choose(true).next();",
        )
        .expect_err("async-generator conditional Await should remain explicit");

        assert!(error
            .to_string()
            .contains("does not yet support branches containing suspension"));
    }

    #[test]
    fn completed_async_generator_return_reaction_module_validates() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_call(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Function),
                ExprIr::FunctionValue(
                    StandardBuiltinId::AsyncGeneratorPrototypeReturn.function_id(),
                ),
            ),
            TypedExpr::from_info(ValueInfo::new(ValueKind::Undefined), ExprIr::Undefined),
            vec![TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(1.0f64.to_bits()),
            )],
        ));
        script.body.result_kind = ValueKind::Dynamic;

        let artifact =
            emit(&program).expect("completed async-generator return reaction should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_body_await_dispatcher_module_validates() {
        let artifact = emit_script(
            "let started = false; async function* stream(value) { started = true; await value; return value; } const iterator = stream(1); iterator.next();",
        )
        .expect("async-generator body Await dispatcher should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_await_terminal_modules_validate() {
        for source in [
            "async function* stream(promise) { await promise; return 1; } stream(Promise.resolve()).next();",
            "async function* stream(value) { return value; } stream(1).next();",
            "async function* stream(value) { return await value; } stream(1).next();",
            "async function* stream(promise) { await promise; } stream(Promise.reject(1)).next();",
        ] {
            let artifact = emit_script(source).expect("async-generator body Await should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_yield_lifecycle_modules_validate() {
        for source in [
            "async function* stream() { yield; } const iterator = stream(); iterator.next(); iterator.next();",
            "async function* stream() { yield yield 1; } const iterator = stream(); iterator.next(); iterator.next(); iterator.next();",
            "async function* stream() { yield 1; throw 2; } const iterator = stream(); iterator.next(); iterator.return(2); iterator.next();",
            "async function* stream() { yield 1; throw 2; } const iterator = stream(); iterator.next(); iterator.return(Promise.resolve(2)); iterator.next();",
            "async function* stream() { yield 1; throw 2; } const reason = {}; const iterator = stream(); iterator.next(); iterator.throw(reason); iterator.next();",
            "async function* stream() { yield 1; throw 2; } const reason = Promise.resolve(2); const iterator = stream(); iterator.next(); iterator.throw(reason); iterator.next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator Yield lifecycle should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_yield_thenable_modules_validate() {
        for source in [
            "let thenable = { then: function (resolve, reject) { resolve(1); reject(2); } }; async function* stream() { yield thenable; } stream().next();",
            "let thenable = { then: function (resolve, reject) { reject(1); resolve(2); } }; async function* stream() { yield thenable; } stream().next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator thenable yield should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_yield_await_staging_module_validates() {
        let artifact = emit_script(
            "async function* stream(value) { yield await value; } const iterator = stream(1); iterator.next(); iterator.next();",
        )
        .expect("async-generator yield-await staging should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_delegation_module_validates() {
        let artifact = emit_script(
            "async function* inner() { yield 1; } async function* outer() { yield* inner(); } const iterator = outer(); iterator.next(); iterator.next();",
        )
        .expect("async-generator delegation should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_delegation_completion_module_validates() {
        for source in [
            "let source = { [Symbol.asyncIterator]: function () { return this; }, next: function () { return Promise.resolve({ value: 1, done: true }); } }; async function* outer() { var completion = yield* source; return completion; } outer().next();",
            "let source = { [Symbol.iterator]: function () { return this; }, next: function () { return { value: 1, done: false }; }, return: function (value) { return { value: value, done: true }; } }; let iterator = (async function*() { yield* source; }()); iterator.next(); iterator.return(2);",
            "let source = { [Symbol.iterator]: function () { return this; }, next: function () { return { value: 1, done: false }; }, throw: function (value) { return { value: value, done: true }; } }; let iterator = (async function*() { var completion = yield* source; return completion; }()); iterator.next(); iterator.throw(2);",
        ] {
            let artifact =
                emit_script(source).expect("async-generator delegated completion should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_rejected_yield_routing_modules_validate() {
        for source in [
            "async function* stream(reason) { yield Promise.reject(reason); yield 'unreachable'; } stream({}).next();",
            "async function* source(reason) { yield Promise.reject(reason); } async function* stream(reason) { for await (let value of source(reason)) { yield value; } } stream({}).next();",
            "async function* stream(reason) { for await (let value of [Promise.reject(reason)]) { yield value; } } stream({}).next();",
        ] {
            let artifact = emit_script(source)
                .expect("async-generator rejected yield routing should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_yield_spread_modules_validate() {
        for source in [
            "let source = { [Symbol.iterator]: function () { return this; }, next: function () { return { done: true }; } }; [0, ...source, 1];",
            "async function* stream() { yield [...yield]; } stream().next();",
            "async function* stream() { yield [...yield yield]; } stream().next();",
            "async function* stream() { yield { ...yield, y: 1, ...yield yield }; } stream().next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator yield spread should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_yield_spread_conditional_remains_explicit() {
        let error = emit_script(
            "async function* stream(flag) { yield [...(flag ? yield [] : [])]; } stream(true).next();",
        )
        .expect_err("conditional yield inside spread should remain explicit");

        assert!(error
            .to_string()
            .contains("generator expression suspension"));
    }

    #[test]
    fn promise_all_iterable_module_validates() {
        let artifact = emit_script(
            r#"let iterable = {};
iterable[Symbol.iterator] = function () {
  let index = 0;
  return { next: function () {
    if (index === 2) return { done: true };
    return { done: false, value: index++ };
  } };
};
Promise.all(iterable).then(function (values) { return values.length; });"#,
        )
        .expect("Promise.all iterable should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn promise_race_iterable_module_validates() {
        let artifact = emit_script(
            r#"let iterable = {};
iterable[Symbol.iterator] = function () {
  let index = 0;
  return { next: function () {
    if (index === 2) return { done: true };
    return { done: false, value: index++ };
  } };
};
Promise.race(iterable).then(function (value) { return value; });"#,
        )
        .expect("Promise.race iterable should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn promise_finally_module_validates() {
        let artifact = emit_script(
            r#"Promise.resolve(1)
  .finally(function () { return Promise.resolve(2); })
  .then(function (value) { return value; });
Promise.reject(3)
  .finally(function () {})
  .catch(function (reason) { return reason; });"#,
        )
        .expect("Promise.prototype.finally should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn iterator_zip_keyed_module_validates() {
        let artifact = emit_script(
            r#"
const key = Symbol("key");
const inputs = { first: [1], second: [2, 3], [key]: [4] };
const iterator = Iterator.zipKeyed(inputs, {
  mode: "longest",
  padding: { first: 5, [key]: 6 },
});
const first = iterator.next().value;
const second = iterator.next().value;
Object.getPrototypeOf(first) === null
  && first.first === 1
  && first.second === 2
  && first[key] === 4
  && second.first === 5
  && second.second === 3
  && second[key] === 6;
"#,
        )
        .expect("Iterator.zipKeyed should emit");

        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn object_from_entries_module_validates() {
        let artifact = emit_script(
            r#"
const result = Object.fromEntries([["first", 1], ["second", 2]]);
result.first === 1 && result.second === 2;
"#,
        )
        .expect("Object.fromEntries should emit");

        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn async_generator_mixed_await_and_yield_module_validates() {
        let artifact = emit_script(
            "async function* stream(value) { await value; yield value; } stream(1).next();",
        )
        .expect("mixed async-generator suspension should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_resumable_loop_modules_validate() {
        for source in [
            "async function* stream() { for (let i = 0; i < 0; i++) { yield i; } yield 9; } stream().next();",
            "async function* stream() { for (let i = 0; i < 1; i++) { yield i; } yield 9; } stream().next();",
            "async function* stream() { for (let i = 0; i < 3; i++) { yield Promise.resolve(i * 2); } yield 9; } stream().next();",
            "async function* stream() { for (let i = 0; i < 2; i++) { let observed; try { observed = value; } catch (error) { observed = 'tdz'; } yield observed; let value = i; } } stream().next();",
            "let observed; async function* stream() { for (let i = 0; i < 1; i++) { let value = 7; yield value; observed = value; } } stream().next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator resumable loop should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_resumable_await_loop_module_validates() {
        let artifact = emit_script(
            "async function tick(value) { return value; }
             async function* stream() {
                 for (let i = 0; i < 2; i++) {
                     await tick(i);
                 }
                 return 0;
             }
             stream().next();",
        )
        .expect("async-generator resumable Await loop should emit");

        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn async_generator_terminal_completion_and_completed_queue_modules_validate() {
        for source in [
            "async function* stream() {} const iterator = stream(); iterator.next(); iterator.next(); iterator.throw(1); iterator.return(2);",
            "const stream = async function* named() {}; stream().next(); stream().throw(1); stream().return(2);",
            "async function* stream() { return 1; } stream().next();",
            "async function* stream() { throw 1; } stream().next();",
        ] {
            let artifact =
                emit_script(source).expect("async-generator terminal body completion should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_body_dispatcher_rejects_for_await_iteration() {
        // A `yield` in the body is supported: its resume states are allocated
        // inside the loop's own state span, so the loop re-enters on them. A
        // nested `for await` is not, because its four states nest inside that
        // same span and the inner loop would be entered by the outer loop's
        // per-iteration gate rather than by its own.
        let error = emit_script(
            "async function* stream(source) { for await (const outer of source) { for await (const inner of outer) { print(inner); } } }",
        )
        .expect_err("a nested for-await body should remain refused");

        assert!(
            error
                .to_string()
                .contains("does not yet support for-await iteration"),
            "{error}"
        );
    }

    #[test]
    fn async_generator_for_await_with_a_suspending_body_modules_validate() {
        for source in [
            // The headline shape: a non-transparent yield, so the delegation
            // shortcut does not apply and the generic emitter runs.
            "async function* stream(source) { for await (const value of source) { print(value); yield value * 2; } }
             stream([1, 2]).next();",
            // A statement after the suspension, which only runs on the
            // invocation that resumes at the body's own state.
            "async function* stream(source) { for await (const value of source) { yield value; print(value); } }
             stream([1, 2]).next();",
            // Two suspensions in one body: three invocations per iteration.
            "async function* stream(source) { for await (const value of source) { yield value; yield value + 1; } }
             stream([1, 2]).next();",
            // The loop sits between other suspensions, so its span starts above
            // zero and the enclosing dispatcher has to route into it.
            "async function* stream(source) { yield 0; for await (const value of source) { print(value); yield value; } yield 1; }
             stream([1, 2]).next();",
            // `var` takes the storage-without-environment path for the binding.
            "async function* stream(source) { for await (var value of source) { yield value; print(value); } }
             stream([1, 2]).next();",
        ] {
            let artifact = emit_script(source)
                .expect("for-await with a suspending body should emit in an async generator");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn async_generator_for_await_with_a_suspension_free_body_modules_validate() {
        for source in [
            "async function* stream(source) { for await (const value of source) { print(value); } }
             stream([1, 2]).next();",
            "async function* stream(source) { for await (const value of source) { if (value) continue; break; } yield 1; }
             stream([1, 2]).next();",
            "async function* stream(source) { yield 0; for await (const value of source) { print(value); } yield 1; }
             stream([1, 2]).next();",
        ] {
            let artifact = emit_script(source)
                .expect("for-await with a suspension-free body should emit in an async generator");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn map_cross_realm_new_target_modules_validate() {
        for source in [
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
Reflect.construct(Map, [], C);"#,
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
var bound = C.bind(null);
Reflect.construct(Map, [], bound);"#,
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
var revocable;
revocable = Proxy.revocable(C, {
  get: function(target, key) {
    if (key === "prototype") revocable.revoke();
    return null;
  }
});
try { Reflect.construct(Map, [], revocable.proxy); } catch (error) {}"#,
        ] {
            let artifact = emit_script(source).expect("Map newTarget script should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn set_cross_realm_new_target_modules_validate() {
        for source in [
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
Reflect.construct(Set, [], C);"#,
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
var bound = C.bind(null);
Reflect.construct(Set, [], bound);"#,
            r#"var other = __lilaCreateRealm().global;
var C = other.Object;
C.prototype = null;
var revocable;
revocable = Proxy.revocable(C, {
  get: function(target, key) {
    if (key === "prototype") revocable.revoke();
    return null;
  }
});
try { Reflect.construct(Set, [], revocable.proxy); } catch (error) {}"#,
        ] {
            let artifact = emit_script(source).expect("Set newTarget script should emit");
            expect_valid_module(&artifact, 1);
        }
    }

    #[test]
    fn operations_emits_to_numeric_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_numeric(TypedExpr::from_info(
                ValueInfo::new(ValueKind::BigInt),
                ExprIr::BigInt(BigIntLiteralIr::from_i64(1)),
            )));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToNumeric spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn arbitrary_precision_bigint_literal_initializes_heap_record_and_limbs() {
        let source = parse("184467440737095516161234567890n;", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("arbitrary precision BigInt should emit");

        expect_valid_module(&artifact, 0);
        for (value, offset) in [(1_234_567_890, 0), (10_000_000_000, 8)] {
            assert!(
                contains_i64_const_store_at_offset(&artifact.bytes, value, offset),
                "BigInt literal should initialize magnitude limb {offset}"
            );
        }
        assert!(
            contains_i64_const_store_at_offset(&artifact.bytes, 2, HEAP_BIGINT_LIMBS_LEN_OFFSET,),
            "BigInt record should retain both magnitude limbs"
        );
        assert!(
            contains_i64_const_store_at_offset(&artifact.bytes, 2, HEAP_BIGINT_LIMBS_CAP_OFFSET,),
            "BigInt record capacity should cover both magnitude limbs"
        );
    }

    #[test]
    fn operations_emits_is_callable_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_is_callable(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Function),
                ExprIr::FunctionValue(StandardBuiltinId::MathMax.function_id()),
            )));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("IsCallable spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn planning_roots_number_static_method_reached_through_getv_call() {
        let source = parse(
            r#"
            let actual = Number.isNaN(NaN);
            if (actual !== true) throw actual;
            if (Number.isNaN("NaN") !== false) throw "string must not coerce";
            if (Number.isFinite(Infinity) !== false) throw "infinity must stay non-finite";
            262;
            "#,
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let script = program.script.as_ref().expect("script ir should exist");

        assert!(script_references_standard_builtin(
            script,
            StandardBuiltinId::NumberIsNaN
        ));
        assert!(!should_stub_standard_builtin(
            script,
            StandardBuiltinId::NumberIsNaN
        ));
        assert!(script_references_standard_builtin(
            script,
            StandardBuiltinId::NumberIsFinite
        ));
        assert!(!should_stub_standard_builtin(
            script,
            StandardBuiltinId::NumberIsFinite
        ));
    }

    #[test]
    fn operations_emits_is_constructor_spec_operation() {
        let source = parse(
            "let value = function C() {}; __lilaIsConstructor(value);",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower_with_host_surface_policy(&source, HostSurfacePolicy::Test262);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("IsConstructor spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_is_property_key_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_is_property_key(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("key".to_string()),
            )));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("IsPropertyKey spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_number_spec_operation() {
        let source = parse("let value = \"42\"; Number(value);", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToNumber spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_primitive_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_to_primitive(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::ObjectLiteral(vec![]),
            ),
            ToPrimitiveHint::String,
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToPrimitive spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_ordinary_object_to_primitive_default_concat_validates() {
        // "a" + {}: OrdinaryToPrimitive on a plain object must fall back to the
        // inherited Object.prototype.toString default ("[object Object]") instead of
        // throwing a TypeError.
        let artifact = emit_script(r#""a" + {};"#).expect("string concat with object should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn operations_ordinary_object_to_primitive_default_loose_equality_validates() {
        // {} == "[object Object]": abstract equality coerces the object through the
        // same OrdinaryToPrimitive default path.
        let artifact = emit_script(
            r#"let o = {};
o == "[object Object]";"#,
        )
        .expect("loose equality with object should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn operations_ordinary_object_to_primitive_respects_own_hooks() {
        // Own valueOf / toString / @@toPrimitive still take precedence over the
        // inherited default fallback.
        let artifact = emit_script(
            r#"let a = { toString() { return "x"; } };
let b = { valueOf() { return 1; } };
let c = { [Symbol.toPrimitive]() { return "s"; } };
("" + a) + (b + 0) + ("" + c);"#,
        )
        .expect("objects with own coercion hooks should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn operations_in_operator_type_error_reaches_active_catch_handler() {
        // `"x" in 1` throws a TypeError that must be caught by the enclosing
        // try/catch rather than escaping the function via an over-shooting branch.
        let artifact = emit_script(
            r#"try {
  "x" in 1;
} catch (e) {
  e instanceof TypeError;
}"#,
        )
        .expect("`in` on a non-object should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn operations_instanceof_dynamic_rhs_guard_validates() {
        // A right-hand side that is not a single statically-known constructor reaches
        // the runtime OrdinaryHasInstance guard in emit_instanceof_i32, which throws a
        // TypeError about the `instanceof` operand when the value is not callable/an
        // object at runtime. The union constructor keeps the RHS off the static-prototype
        // fast path so the guard is actually emitted.
        let artifact = emit_script(
            r#"function pick(flag) {
  let Ctor = flag ? Array : Object;
  try {
    return ({}) instanceof Ctor;
  } catch (e) {
    return e instanceof TypeError;
  }
}
pick(true);"#,
        )
        .expect("instanceof with a dynamic rhs should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn operations_emits_to_bigint_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_to_bigint(
            TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
        ));
        script.body.result_kind = ValueKind::BigInt;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToBigInt spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_string_spec_operation() {
        let source = parse("let value = 42; String(value);", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToString spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_object_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_object(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("boxed".to_string()),
            )));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToObject spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_property_key_spec_operation_for_string_result() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_property_key(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(7.0f64.to_bits()),
            )));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToPropertyKey spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_property_key_spec_operation_for_symbol_result() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_property_key(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Symbol),
                ExprIr::Symbol { description: None },
            )));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToPropertyKey symbol spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_integer_or_infinity_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(
            TypedExpr::spec_to_integer_or_infinity(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("-3.7".to_string()),
            )),
        );
        script.body.result_kind = ValueKind::Number;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToIntegerOrInfinity spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_length_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_length(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("3.7".to_string()),
            )));
        script.body.result_kind = ValueKind::Number;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToLength spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_to_index_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_to_index(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("3".to_string()),
            )));
        script.body.result_kind = ValueKind::Number;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("ToIndex spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_strict_equality_spec_operation() {
        let source = parse("let value = 1; value === 1;", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("StrictEqualityComparison spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_now_builtins_emit() {
        let source = parse(
            "Temporal.Now.timeZoneId(); Temporal.Now.instant(); Temporal.Now.zonedDateTimeISO();",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.Now members should emit");

        assert!(!artifact.bytes.is_empty());
        assert!(
            artifact
                .debug_dump
                .contains("import func: lila_host.wall_clock_millis"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn temporal_duration_family_emits() {
        // Every member installs together, so naming one accessor has to bring
        // the whole prototype with it.
        let source = parse(
            "const d = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);\n\
             d.years; d.sign; d.blank; d.toString(); d.toJSON(); d.negated(); d.abs();\n\
             d.with({ hours: 1 }); d.add({ hours: 1 }); d.subtract({ hours: 1 });\n\
             d.round('seconds'); d.total('seconds');\n\
             Temporal.Duration.from('P1Y'); Temporal.Duration.compare(d, d);",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.Duration should emit");

        assert!(!artifact.bytes.is_empty());
        assert!(
            artifact.debug_dump.contains("Temporal.Duration.prototype"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn temporal_now_namespace_exists_from_a_bare_temporal_reference() {
        // `Temporal.Now` has to be observable even when no member is named, so
        // the namespace must not be gated on a member reference.
        let source =
            parse("typeof Temporal.Now;", ParseOptions::script()).expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.Now namespace should emit");

        assert!(!artifact.bytes.is_empty());
        // The clock readers are not named, so the host import must stay out.
        assert!(
            !artifact
                .debug_dump
                .contains("import func: lila_host.wall_clock_millis"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn temporal_instant_equals_builtin_emits() {
        let source = parse(
            "new Temporal.Instant(1n).equals(new Temporal.Instant(1n));",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.Instant.prototype.equals should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_zoned_date_time_from_builtin_emits() {
        let source = parse(
            "Temporal.ZonedDateTime.from(\"1970-01-01T00:00Z[UTC]\");",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.ZonedDateTime.from should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_zoned_date_time_from_property_bags_emit() {
        let source = parse(
            r#"
            Temporal.ZonedDateTime.from({
                year: 1976,
                monthCode: "M11",
                day: 18,
                timeZone: "+01:00"
            }, { overflow: "constrain" });
            var arrayBag = [];
            arrayBag.year = 1970;
            arrayBag.month = 1;
            arrayBag.day = 1;
            arrayBag.timeZone = "UTC";
            Temporal.ZonedDateTime.from(arrayBag);
            function functionBag() {}
            functionBag.year = 1970;
            functionBag.month = 1;
            functionBag.day = 1;
            functionBag.timeZone = "UTC";
            Temporal.ZonedDateTime.from(functionBag, function () {});
            "#,
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact =
            emit(&program).expect("Temporal.ZonedDateTime.from property bags should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_zoned_date_time_offset_accessors_emit() {
        let source = parse(
            "const value = new Temporal.ZonedDateTime(0n, \"+01:30\"); \
             value.offset; value.offsetNanoseconds;",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact = emit(&program).expect("Temporal.ZonedDateTime offset accessors should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_zoned_date_time_civil_accessors_and_equals_emit() {
        let source = parse(
            r#"
            const value = new Temporal.ZonedDateTime(-1n, "+01:30");
            value.year;
            value.month;
            value.monthCode;
            value.day;
            value.hour;
            value.minute;
            value.second;
            value.millisecond;
            value.microsecond;
            value.nanosecond;
            value.equals({ year: 1970, month: 1, day: 1, timeZone: "+01:30" });
            "#,
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact =
            emit(&program).expect("Temporal.ZonedDateTime civil accessors and equals should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn temporal_zoned_date_time_with_time_zone_emits() {
        let source = parse(
            r#"
            const value = new Temporal.ZonedDateTime(0n, "UTC");
            value.withTimeZone("+0130");
            value.withTimeZone("2021-08-19T17:30Z");
            value.withTimeZone(new Temporal.ZonedDateTime(1n, "-08"));
            "#,
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
        let artifact =
            emit(&program).expect("Temporal.ZonedDateTime.prototype.withTimeZone should emit");

        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_loose_equality_spec_operation() {
        let source = parse("let value = 1; value == \"1\";", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("IsLooselyEqual spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_same_value_spec_operation() {
        let source =
            parse("Object.is(NaN, NaN);", ParseOptions::script()).expect("script should parse");
        let program = lower(&source);
        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("SameValue spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_same_value_zero_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_same_value_zero(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(0.0f64.to_bits()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number((-0.0f64).to_bits()),
            ),
        ));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("SameValueZero spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_get_v_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_get_v(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("GetV spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_get_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_get(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("Get spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_has_property_spec_operation() {
        let source =
            parse("\"flag\" in globalThis;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_has_property(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
        ));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("HasProperty spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_create_data_property_or_throw_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_create_data_property_or_throw(
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Object),
                    ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
                ),
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::String),
                    ExprIr::String("flag".to_string()),
                ),
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Number),
                    ExprIr::Number(1.0f64.to_bits()),
                ),
            ));
        script.body.result_kind = ValueKind::Undefined;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("property_writes=1"));
        let artifact =
            emit(&program).expect("CreateDataPropertyOrThrow spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_set_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_set(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(1.0f64.to_bits()),
            ),
        ));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("property_writes=1"));
        let artifact = emit(&program).expect("Set spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_delete_property_or_throw_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] =
            StatementIr::Expression(TypedExpr::spec_delete_property_or_throw(
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Object),
                    ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
                ),
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::String),
                    ExprIr::String("flag".to_string()),
                ),
            ));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("deletes=1"));
        let artifact = emit(&program).expect("DeletePropertyOrThrow spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_has_own_property_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_has_own_property(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
        ));
        script.body.result_kind = ValueKind::Boolean;

        assert!(program.ir_summary().contains("spec_operations=1"));
        let artifact = emit(&program).expect("HasOwnProperty spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_get_method_spec_operation() {
        let source =
            parse("globalThis.flag;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_get_method(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("flag".to_string()),
            ),
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("property_reads=1"));
        let artifact = emit(&program).expect("GetMethod spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_call_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_call(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Function),
                ExprIr::FunctionValue(StandardBuiltinId::MathMax.function_id()),
            ),
            TypedExpr::from_info(ValueInfo::new(ValueKind::Undefined), ExprIr::Undefined),
            vec![TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(1.0f64.to_bits()),
            )],
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("calls=1"));
        let artifact = emit(&program).expect("Call spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn operations_emits_construct_spec_operation() {
        let source = parse("0;", ParseOptions::script()).expect("script should parse");
        let mut program = lower(&source);
        let script = program.script.as_mut().expect("script ir should exist");
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_construct(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Function),
                ExprIr::FunctionValue(StandardBuiltinId::ArrayConstructor.function_id()),
            ),
            vec![TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::Number(1.0f64.to_bits()),
            )],
        ));
        script.body.result_kind = ValueKind::Dynamic;

        assert!(program.ir_summary().contains("spec_operations=1"));
        assert!(program.ir_summary().contains("constructs=1"));
        let artifact = emit(&program).expect("Construct spec operation should emit");
        assert!(!artifact.bytes.is_empty());
    }

    fn data_segment_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut collected = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.expect("wasm parse should succeed") {
                Payload::DataSection(reader) => {
                    for segment in reader {
                        let segment = segment.expect("data segment should decode");
                        match segment.kind {
                            wasmparser::DataKind::Active { .. } | wasmparser::DataKind::Passive => {
                                collected.extend_from_slice(segment.data);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        collected
    }

    #[test]
    fn string_pool_encodes_literal_lone_surrogates_as_wtf8_bytes() {
        let encoded = format!("{JS_STRING_SURROGATE_SENTINEL}D800");
        assert_eq!(
            StringPool::runtime_bytes_for_string(&encoded),
            vec![0xED, 0xA0, 0x80]
        );
    }

    #[test]
    fn string_pool_escapes_literal_surrogate_sentinel() {
        let encoded = format!("{JS_STRING_SURROGATE_SENTINEL}{JS_STRING_SURROGATE_SENTINEL}");
        assert_eq!(
            StringPool::runtime_bytes_for_string(&encoded),
            JS_STRING_SURROGATE_SENTINEL.to_string().as_bytes().to_vec()
        );
    }

    fn contains_i64_const(bytes: &[u8], needle: i64) -> bool {
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::CodeSectionEntry(body) = payload.expect("wasm parse should succeed") {
                let mut reader = body
                    .get_operators_reader()
                    .expect("operators should decode");
                while !reader.eof() {
                    if let Operator::I64Const { value } =
                        reader.read().expect("operator should decode")
                    {
                        if value == needle {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn contains_i64_const_store_at_offset(bytes: &[u8], value: i64, offset: u64) -> bool {
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::CodeSectionEntry(body) = payload.expect("wasm parse should succeed") {
                let mut reader = body
                    .get_operators_reader()
                    .expect("operators should decode");
                let mut previous_i64_const = None;
                while !reader.eof() {
                    match reader.read().expect("operator should decode") {
                        Operator::I64Const { value } => previous_i64_const = Some(value),
                        Operator::I64Store { memarg }
                            if previous_i64_const == Some(value) && memarg.offset == offset =>
                        {
                            return true;
                        }
                        _ => previous_i64_const = None,
                    }
                }
            }
        }
        false
    }

    fn global_init_i64s(bytes: &[u8]) -> Vec<i64> {
        let mut values = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::GlobalSection(reader) = payload.expect("wasm parse should succeed") {
                for global in reader {
                    let global = global.expect("global should decode");
                    if let wasmparser::ValType::I64 = global.ty.content_type {
                        let mut init = global.init_expr.get_operators_reader();
                        match init.read().expect("global init op should decode") {
                            Operator::I64Const { value } => values.push(value),
                            op => panic!("unexpected i64 global init op: {op:?}"),
                        }
                    }
                }
            }
        }
        values
    }

    fn memory_initial_pages(bytes: &[u8]) -> Vec<u64> {
        let mut pages = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::MemorySection(reader) = payload.expect("wasm parse should succeed") {
                for memory in reader {
                    pages.push(memory.expect("memory should decode").initial);
                }
            }
        }
        pages
    }

    fn code_body_context(bytes: &[u8], offset: usize) -> String {
        let mut defined_index = 0usize;
        for payload in Parser::new(0).parse_all(bytes) {
            let payload = payload.expect("wasm parse should succeed");
            if let Payload::CodeSectionEntry(body) = payload {
                let range = body.range();
                if range.start <= offset && offset < range.end {
                    let mut nearby = Vec::new();
                    let mut reader = body
                        .get_operators_reader()
                        .expect("operators should decode");
                    let mut depth = 1usize;
                    while !reader.eof() {
                        match reader.read_with_offset() {
                            Ok((op, op_offset)) => {
                                if op_offset.saturating_add(512) >= offset
                                    && op_offset <= offset + 64
                                {
                                    nearby.push(format!("d{depth} {op_offset:#x}: {op:?}"));
                                }
                                match op {
                                    Operator::Block { .. }
                                    | Operator::Loop { .. }
                                    | Operator::If { .. } => depth += 1,
                                    Operator::End => depth = depth.saturating_sub(1),
                                    _ => {}
                                }
                            }
                            Err(err) => {
                                nearby.push(format!("operator decode error: {err}"));
                                break;
                            }
                        }
                    }
                    return format!(
                        "function body #{defined_index} byte range {:#x}..{:#x}; nearby ops: {}",
                        range.start,
                        range.end,
                        nearby.join("; ")
                    );
                }
                defined_index += 1;
            }
        }
        "no containing function body found".to_string()
    }

    fn validation_error_offset(message: &str) -> Option<usize> {
        let marker = "offset 0x";
        let start = message.find(marker)? + marker.len();
        let hex = message[start..]
            .chars()
            .take_while(|ch| ch.is_ascii_hexdigit())
            .collect::<String>();
        usize::from_str_radix(&hex, 16).ok()
    }

    /// Validates with `wasmparser` directly (rather than an engine's own
    /// validator) so the accepted proposal set can be pinned to match the
    /// production wasmtime configuration (`lila-engine`'s
    /// `run_with_wasm_aot_inner`: threads, function-references, gc, and
    /// exceptions all enabled) instead of an embedding engine's own default
    /// feature set, which may lag behind the production target.
    fn expect_valid_module(artifact: &WasmArtifact, _script_function_count: usize) {
        let features = WasmFeatures::default()
            | WasmFeatures::THREADS
            | WasmFeatures::FUNCTION_REFERENCES
            | WasmFeatures::GC
            | WasmFeatures::EXCEPTIONS;
        Validator::new_with_features(features)
            .validate_all(&artifact.bytes[..])
            .unwrap_or_else(|err| {
                let message = err.to_string();
                let context = validation_error_offset(&message)
                    .map(|offset| code_body_context(&artifact.bytes, offset))
                    .unwrap_or_else(|| "no validation offset found".to_string());
                panic!("module should validate: {message}; {context}");
            });
    }

    fn declared_function_local_counts(bytes: &[u8]) -> Vec<u32> {
        let mut counts = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            let Payload::CodeSectionEntry(body) = payload.expect("wasm parse should succeed")
            else {
                continue;
            };
            let locals = body
                .get_locals_reader()
                .expect("function locals should decode");
            let mut count = 0_u32;
            for local in locals {
                let (consecutive_count, _) = local.expect("function local should decode");
                count = count
                    .checked_add(consecutive_count)
                    .expect("function local count should fit in u32");
            }
            counts.push(count);
        }
        counts
    }

    #[test]
    fn emitted_module_validates() {
        let artifact = emit_script("let x = 40; const y = 2; x + y;").expect("emit should work");
        expect_valid_module(&artifact, 0);
        assert!(artifact.debug_dump.contains("export func: main"));
        assert!(artifact.debug_dump.contains("export global: result_tag"));
    }

    #[test]
    fn runtime_gc_root_follows_the_actual_fixed_and_template_globals() {
        let cases = [
            (
                "fixed globals only",
                "1;",
                GLOBAL_INDEX_REGISTRY.len() as u32,
            ),
            (
                "one template-object global",
                r#"function tag(parts) { return parts[0]; } tag`one`;"#,
                GLOBAL_INDEX_REGISTRY.len() as u32 + 1,
            ),
            (
                "two template-object globals",
                r#"function tag(parts) { return parts[0]; } tag`one`; tag`two`;"#,
                GLOBAL_INDEX_REGISTRY.len() as u32 + 2,
            ),
        ];

        for (label, source, expected_root_index) in cases {
            let artifact = emit_script(source)
                .unwrap_or_else(|error| panic!("{label} source should emit: {error}"));
            expect_valid_module(&artifact, 0);

            let mut global_count = 0_u32;
            let mut reference_globals = Vec::new();
            for payload in Parser::new(0).parse_all(&artifact.bytes) {
                let Payload::GlobalSection(reader) = payload.expect("module should parse") else {
                    continue;
                };
                for (index, global) in reader.into_iter().enumerate() {
                    let global = global.expect("global should decode");
                    global_count += 1;
                    if matches!(global.ty.content_type, wasmparser::ValType::Ref(_)) {
                        reference_globals.push(index as u32);
                    }
                }
            }

            assert_eq!(
                reference_globals,
                [expected_root_index],
                "{label} must have one typed root at the actual next global index"
            );
            assert_eq!(
                global_count,
                expected_root_index + 1,
                "{label} must seal the global section immediately after its root"
            );
        }
    }

    #[test]
    fn runtime_gc_anchor_is_rooted_across_main_and_cleared_on_exit() {
        let artifact = emit_script(
            "function allocate() { return { value: 1 }; } allocate(); Promise.resolve(0);",
        )
        .expect("root-lifecycle fixture should emit");
        expect_valid_module(&artifact, 1);

        let mut module_types = Vec::new();
        let mut root = None;
        let mut global_count = 0_u32;
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            match payload.expect("module should parse") {
                Payload::TypeSection(reader) => {
                    for group in reader {
                        module_types.extend(
                            group
                                .expect("runtime type group should decode")
                                .into_types(),
                        );
                    }
                }
                Payload::GlobalSection(reader) => {
                    for (index, global) in reader.into_iter().enumerate() {
                        let global = global.expect("global should decode");
                        global_count += 1;
                        let wasmparser::ValType::Ref(reference_type) = global.ty.content_type
                        else {
                            continue;
                        };
                        assert!(root.is_none(), "the capability root is the sole GC global");
                        assert!(
                            global.ty.mutable,
                            "the root must support establish and clear"
                        );
                        assert!(!global.ty.shared, "the per-instance root is not shared");
                        assert!(
                            reference_type.is_nullable(),
                            "the cleared root must be null"
                        );
                        let wasmparser::HeapType::Concrete(anchor_type) =
                            reference_type.heap_type()
                        else {
                            panic!("the root must retain the concrete anchor type");
                        };
                        let anchor_type = anchor_type
                            .as_module_index()
                            .expect("the emitted anchor type uses a module index");
                        let mut init = global.init_expr.get_operators_reader();
                        assert!(matches!(
                            init.read().expect("root initializer should decode"),
                            Operator::RefNull {
                                hty: wasmparser::HeapType::Concrete(initializer_type)
                            } if initializer_type.as_module_index() == Some(anchor_type)
                        ));
                        assert!(matches!(
                            init.read().expect("root initializer should end"),
                            Operator::End
                        ));
                        root = Some((index as u32, anchor_type));
                    }
                }
                _ => {}
            }
        }
        let (root_global, anchor_type) = root.expect("module must declare the typed GC root");
        assert_eq!(
            root_global + 1,
            global_count,
            "the root must be appended after every established global index"
        );

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RootEvent {
            HolderFieldGet,
            AnchorFieldGet,
            RootGet,
            RootSet,
            RootNull,
            RefAsNonNull,
            Call,
            Return,
        }

        let mut events = Vec::new();
        let mut holder_type = None;
        let mut constructed_types = Vec::new();
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            let Payload::CodeSectionEntry(body) = payload.expect("module should parse") else {
                continue;
            };
            for operator in body
                .get_operators_reader()
                .expect("main operators should decode")
            {
                let event = match operator.expect("main operator should decode") {
                    Operator::StructGet {
                        struct_type_index,
                        field_index,
                    } if struct_type_index == anchor_type => {
                        assert_eq!(field_index, 0, "the anchor ABI field is ordinal zero");
                        Some(RootEvent::AnchorFieldGet)
                    }
                    Operator::StructGet {
                        struct_type_index,
                        field_index,
                    } => {
                        assert_eq!(field_index, 0, "the holder strong edge is ordinal zero");
                        assert!(
                            holder_type
                                .replace(struct_type_index)
                                .is_none_or(|existing| existing == struct_type_index),
                            "main must not access two candidate holder layouts"
                        );
                        Some(RootEvent::HolderFieldGet)
                    }
                    Operator::StructNew { struct_type_index } => {
                        constructed_types.push(struct_type_index);
                        None
                    }
                    Operator::GlobalGet { global_index } if global_index == root_global => {
                        Some(RootEvent::RootGet)
                    }
                    Operator::GlobalSet { global_index } if global_index == root_global => {
                        Some(RootEvent::RootSet)
                    }
                    Operator::RefNull {
                        hty: wasmparser::HeapType::Concrete(reference_type),
                    } if reference_type.as_module_index() == Some(anchor_type) => {
                        Some(RootEvent::RootNull)
                    }
                    Operator::RefAsNonNull => Some(RootEvent::RefAsNonNull),
                    Operator::Call { .. } | Operator::CallIndirect { .. } => Some(RootEvent::Call),
                    Operator::Return
                    | Operator::ReturnCall { .. }
                    | Operator::ReturnCallIndirect { .. } => Some(RootEvent::Return),
                    _ => None,
                };
                if let Some(event) = event {
                    events.push(event);
                }
            }
            break;
        }

        let holder_type = holder_type.expect("main must traverse the typed holder field");
        assert_eq!(
            holder_type,
            anchor_type + 1,
            "the holder must be registered immediately after its anchor dependency"
        );
        assert_eq!(
            constructed_types,
            [anchor_type, holder_type],
            "main must construct the registered anchor and then its holder"
        );
        let anchor = module_types
            .get(anchor_type as usize)
            .expect("root must name a declared anchor type")
            .unwrap_struct();
        assert_eq!(anchor.fields.len(), 1, "the anchor has one ABI field");
        assert!(
            !anchor.fields[0].mutable,
            "the anchor ABI field is immutable"
        );
        assert_eq!(
            anchor.fields[0].element_type,
            wasmparser::StorageType::Val(wasmparser::ValType::I32),
            "the anchor ABI field is an i32"
        );
        let holder = module_types
            .get(holder_type as usize)
            .expect("holder access must name a declared type")
            .unwrap_struct();
        assert_eq!(holder.fields.len(), 1, "the holder has one strong edge");
        assert!(!holder.fields[0].mutable, "the holder edge is immutable");
        let wasmparser::StorageType::Val(wasmparser::ValType::Ref(holder_reference)) =
            holder.fields[0].element_type
        else {
            panic!("the holder field must be a typed reference");
        };
        assert!(
            !holder_reference.is_nullable(),
            "the holder's anchor edge must be non-null"
        );
        let wasmparser::HeapType::Concrete(holder_target) = holder_reference.heap_type() else {
            panic!("the holder field must name the concrete anchor type");
        };
        assert_eq!(
            holder_target.as_module_index(),
            Some(anchor_type),
            "the holder field and root must name the same anchor layout"
        );

        let initial_set = events
            .iter()
            .position(|event| *event == RootEvent::RootSet)
            .expect("main must establish its root");
        assert!(
            events[..initial_set].contains(&RootEvent::HolderFieldGet),
            "the holder's typed strong edge must feed the root"
        );
        let final_set = events
            .iter()
            .rposition(|event| *event == RootEvent::RootSet)
            .expect("main must clear its root");
        let root_null = events[..final_set]
            .iter()
            .rposition(|event| *event == RootEvent::RootNull)
            .expect("root cleanup must store a typed null");
        let anchor_get = events[..root_null]
            .iter()
            .rposition(|event| *event == RootEvent::AnchorFieldGet)
            .expect("root cleanup must verify the anchor ABI");
        let non_null = events[..anchor_get]
            .iter()
            .rposition(|event| *event == RootEvent::RefAsNonNull)
            .expect("root cleanup must reject an absent root");
        let root_get = events[..non_null]
            .iter()
            .rposition(|event| *event == RootEvent::RootGet)
            .expect("root cleanup must load the typed global");
        let call = events[..root_get]
            .iter()
            .rposition(|event| *event == RootEvent::Call)
            .expect("the root must survive at least one main call");
        assert!(
            initial_set < call
                && call < root_get
                && root_get < non_null
                && non_null < anchor_get
                && anchor_get < root_null
                && root_null < final_set,
            "root lifecycle events are out of order: {events:?}"
        );
        for return_index in events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| (*event == RootEvent::Return).then_some(index))
        {
            let clear_set = events[..return_index]
                .iter()
                .rposition(|event| *event == RootEvent::RootSet)
                .expect("every main return follows a root store");
            assert_ne!(
                clear_set, initial_set,
                "a main return bypassed root verification and cleanup: {events:?}"
            );
            assert!(
                clear_set >= 4
                    && events[clear_set - 4..=clear_set]
                        == [
                            RootEvent::RootGet,
                            RootEvent::RefAsNonNull,
                            RootEvent::AnchorFieldGet,
                            RootEvent::RootNull,
                            RootEvent::RootSet,
                        ],
                "a main return did not verify and clear the typed root: {events:?}"
            );
        }
    }

    #[test]
    fn dynamic_number_exponentiation_module_validates_with_runtime_pow_import() {
        let artifact = emit_script(
            "let base = 9; let exponent = 0.5; base ** exponent + Math.pow(base, exponent);",
        )
        .expect("dynamic Number exponentiation should emit");

        expect_valid_module(&artifact, 0);
        assert!(
            artifact
                .debug_dump
                .contains("import func: lila_host.number_pow"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn date_current_time_consumers_import_wall_clock_milliseconds() {
        for (source, consumer) in [
            ("Date.now();", "Date.now"),
            ("Date();", "Date function call"),
            ("new Date();", "zero-argument Date construction"),
        ] {
            let artifact = emit_script(source)
                .unwrap_or_else(|error| panic!("{consumer} script should emit: {error}"));

            expect_valid_module(&artifact, 0);
            assert!(
                artifact
                    .debug_dump
                    .contains("import func: lila_host.wall_clock_millis"),
                "{consumer} omitted its clock import:\n{}",
                artifact.debug_dump
            );
        }

        let artifact = emit_script("262;").expect("constant script should emit");
        assert!(
            !artifact
                .debug_dump
                .contains("import func: lila_host.wall_clock_millis"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn date_clock_import_access_is_centralized_in_date_builtins() {
        let date_source = include_str!("builtins/date.rs");
        let standard_source = include_str!("builtins/standard.rs");
        let date_now_dispatch = standard_source
            .split_once("StandardBuiltinId::DateNow => {")
            .expect("Date.now dispatch arm should exist")
            .1
            .split_once("StandardBuiltinId::DateParse => {")
            .expect("Date.now dispatch arm should be bounded")
            .0;
        let date_constructor_dispatch = standard_source
            .split_once("StandardBuiltinId::DateConstructor => {")
            .expect("Date constructor dispatch arm should exist")
            .1
            .split_once("StandardBuiltinId::DatePrototypeGetTime")
            .expect("Date constructor dispatch arm should be bounded")
            .0;

        assert_eq!(
            date_source
                .matches(".wall_clock_millis_import_function_index()")
                .count(),
            1,
            "Date must have one clock-import access point"
        );
        for dispatch in [date_now_dispatch, date_constructor_dispatch] {
            assert!(
                !dispatch.contains(".wall_clock_millis_import_function_index()"),
                "Date dispatch must route clock reads through the closed source in date.rs"
            );
        }
    }

    #[test]
    fn math_random_alone_imports_the_typed_host_random_capability() {
        let artifact = emit_script("Math.random();").expect("Math.random script should emit");

        expect_valid_module(&artifact, 0);
        assert!(
            artifact
                .debug_dump
                .contains("import func: lila_host.random_f64"),
            "{}",
            artifact.debug_dump
        );

        let artifact = emit_script("262;").expect("constant script should emit");
        assert!(
            !artifact
                .debug_dump
                .contains("import func: lila_host.random_f64"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn canonical_locale_list_alone_imports_the_typed_intl_host_call() {
        let artifact = emit_script("Intl.getCanonicalLocales(['iw-IL']);")
            .expect("canonical locale list should emit");

        expect_valid_module(&artifact, 0);
        assert!(
            artifact
                .debug_dump
                .contains("import func: lila_host.intl_call"),
            "{}",
            artifact.debug_dump
        );
        let expected_identity = lila_intl::embedded_locale_data_identity()
            .expect("embedded Intl identity should be valid")
            .artifact_identity();
        let identity_sections = Parser::new(0)
            .parse_all(&artifact.bytes)
            .filter_map(|payload| {
                let Payload::CustomSection(section) = payload.expect("module should parse") else {
                    return None;
                };
                (section.name() == lila_intl::INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION)
                    .then(|| section.data().to_vec())
            })
            .collect::<Vec<_>>();
        assert_eq!(identity_sections.len(), 1);
        assert_eq!(
            identity_sections[0].as_slice(),
            expected_identity.as_bytes()
        );

        let artifact = emit_script("262;").expect("constant script should emit");
        assert!(
            !artifact
                .debug_dump
                .contains("import func: lila_host.intl_call"),
            "{}",
            artifact.debug_dump
        );
        assert!(Parser::new(0).parse_all(&artifact.bytes).all(|payload| {
            !matches!(
                payload.expect("module should parse"),
                Payload::CustomSection(section)
                    if section.name() == lila_intl::INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION
            )
        }));
    }

    #[test]
    fn emitted_functions_declare_only_referenced_temporary_locals() {
        let artifact = emit_script(
            r#"
function ordinary(value) { return value + 1; }
let buffer = new ArrayBuffer(8);
let view = new Uint8Array(buffer);
let parsed = JSON.parse("1.1e-1");
let serialized = JSON.stringify({ parsed });
ordinary(view.byteLength) === 9 && /1/.test(serialized);
"#,
        )
        .expect("ordinary, builtin, JSON, decimal, and RegExp paths should emit");
        expect_valid_module(&artifact, 1);

        let local_counts = declared_function_local_counts(&artifact.bytes);
        let declared_main_local_count = local_counts
            .first()
            .copied()
            .expect("emitted module should contain a main function");
        let reported_main_local_count = artifact
            .debug_dump
            .lines()
            .find_map(|line| line.strip_prefix("locals: "))
            .expect("debug dump should report main locals")
            .parse::<u32>()
            .expect("reported main local count should be numeric");
        assert_eq!(reported_main_local_count, declared_main_local_count);
        let max_local_count = local_counts
            .iter()
            .copied()
            .max()
            .expect("emitted module should contain functions");
        assert!(
            max_local_count < 2048,
            "emitted function retained temporary-local planning capacity: {max_local_count}"
        );
    }

    #[test]
    fn named_class_accessor_capture_module_validates() {
        let artifact = emit_script(
            "var seen;
             class C {
                 get value() { return C; }
                 set value(next) { seen = C; }
             }
             const original = C;
             const instance = new C();
             instance.value;
             instance.value = null;
             seen === original;",
        )
        .expect("named class accessors should emit");
        expect_valid_module(&artifact, 3);
    }

    #[test]
    fn ordered_class_elements_module_validates() {
        let artifact = emit_script(
            "let order = '';
             function key(name) { order += name; return name; }
             class C {
                 [key('a')]() {}
                 static first = (order += '1', C.later());
                 static { order += '2'; }
                 [key('b')]() {}
                 static #private = (order += '3', 3);
                 static later() { return 1; }
                 before = #instance in this;
                 #instance = 1;
             }
             new C();
             order === 'ab123';",
        )
        .expect("ordered class elements should emit");
        expect_valid_module(&artifact, 8);
    }

    #[test]
    fn class_instance_element_boundaries_module_validates() {
        let artifact = emit_script(
            "let order = '';
             class Base { constructor() { order += 's'; } }
             class Derived extends Base {
                 field = (order += 'f', 1);
                 constructor(value = (order += 'p', 1)) {
                     order += value;
                     (() => super())();
                     order += 'a';
                 }
             }
             new Derived();
             order === 'p1sfa';",
        )
        .expect("constructor-bound instance elements should emit");
        expect_valid_module(&artifact, 4);
    }

    #[test]
    fn strict_class_callable_arguments_module_validates() {
        let artifact = emit_script(
            "class C {
                 constructor(value) { value = 2; this.first = arguments[0]; }
                 method(value) { value = 2; return arguments[0]; }
                 get value() { return arguments.length; }
                 set value(next) { next = 2; this.second = arguments[0]; }
                 static method(value) { value = 2; return arguments[0]; }
             }
             const instance = new C(1);
             instance.method(1);
             instance.value = 1;
             C.method(1);",
        )
        .expect("strict class callable arguments should emit");
        expect_valid_module(&artifact, 6);
    }

    #[test]
    fn computed_class_field_keys_module_validates() {
        let artifact = emit_script(
            "let calls = 0;
             function key(name) { calls += 1; return name; }
             class C {
                 [key('instance')] = 1;
                 static [key('shared')] = 2;
             }
             const first = new C();
             const second = new C();
             calls === 2 && first.instance === 1 && second.instance === 1 && C.shared === 2;",
        )
        .expect("computed class field keys should emit");
        expect_valid_module(&artifact, 4);
    }

    #[test]
    fn private_expression_operands_are_collected_before_emission() {
        let artifact = emit_script(
            "class C {
                 #value = 0;
                 read() { return (void 'private-read-target-literal', this).#value; }
                 write() {
                     (void 'private-write-target-literal', this).#value =
                         'private-write-value-literal';
                 }
                 has() { return #value in (void 'private-in-rhs-literal', this); }
             }
             const instance = new C();
             instance.read();
             instance.write();
             instance.has();",
        )
        .expect("private expression operands should be collected");
        expect_valid_module(&artifact, 4);
    }

    #[test]
    fn private_in_rhs_boundary_module_validates() {
        let artifact = emit_script(
            "class C {
                 #field;
                 nonObject() {
                     try { #field in {} << 0; } catch (error) {
                         return error.name === 'TypeError';
                     }
                 }
                 unresolvable() {
                     try { #field in missingName; } catch (error) {
                         return error.name === 'ReferenceError';
                     }
                 }
             }
             const instance = new C();
             instance.nonObject() && instance.unresolvable();",
        )
        .expect("private-in RHS boundaries should emit");
        expect_valid_module(&artifact, 3);
    }

    #[test]
    fn private_assignment_reference_module_validates() {
        let artifact = emit_script(
            "class C {
                 #field;
                 assign(iterable, object) {
                     for (this.#field of iterable) {}
                     for (this.#field in object) {}
                     [this.#field, ...this.#field] = iterable;
                     ({ value: this.#field } = { value: 1 });
                     return this.#field;
                 }
             }
             new C().assign([1, 2], { first: 1, second: 2 });",
        )
        .expect("private assignment references should emit");
        expect_valid_module(&artifact, 4);
    }

    #[test]
    fn private_callable_source_names_module_validates() {
        let artifact = emit_script(
            "class C {
                 #instanceMethod() {}
                 static #staticMethod() {}
                 instanceName() { return this.#instanceMethod.name; }
                 static staticName() { return this.#staticMethod.name; }
                 publicMethod() {}
             }
             const instance = new C();
             instance.instanceName() === '#instanceMethod'
                 && C.staticName() === '#staticMethod'
                 && C.publicMethod.name === 'C.publicMethod';",
        )
        .expect("private callable source names should emit");
        expect_valid_module(&artifact, 5);
    }

    #[test]
    fn optional_private_access_module_validates() {
        let artifact = emit_script(
            "class C {
                 #field = 1;
                 get #value() { return this.#field; }
                 #method() { return this; }
                 read(o) { return o?.c.#field; }
                 readGetter(o) { return o?.#value; }
                 call(o) { return o?.#method(); }
             }
             const instance = new C();
             instance.read({ c: instance }) === 1
                 && instance.read(null) === undefined
                 && instance.readGetter(instance) === 1
                 && instance.call(instance) === instance;",
        )
        .expect("optional private access should emit");
        expect_valid_module(&artifact, 7);
    }

    #[test]
    fn json_parse_number_validation_module_validates() {
        let artifact = emit_script(r#"JSON.parse("00");"#).expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn string_pad_end_utf16_prefix_module_validates() {
        let artifact =
            emit_script(r#""abc".padEnd(6, "\uD83D\uDCA9");"#).expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn string_pad_start_utf16_prefix_module_validates() {
        let artifact =
            emit_script(r#""abc".padStart(6, "\uD83D\uDCA9");"#).expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_get_own_property_descriptor_module_validates() {
        let artifact = emit_script(
            r#"var target = {};
Object.defineProperty(target, "attr", { value: 1, configurable: true });
var proxy = new Proxy(target, {});
Object.getOwnPropertyDescriptor(proxy, "attr");"#,
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_set_define_property_fallback_module_validates() {
        let artifact = emit_script(
            r#"
var desc;
var p = new Proxy({}, {
  defineProperty: function(target, key, candidate) {
    desc = candidate;
    return true;
  }
});
p.a = 0;
desc;
"#,
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn outlined_ordinary_set_receiver_paths_validate() {
        let artifact = emit_script(
            r#"
var setterReceiver;
var prototype = {};
Object.defineProperty(prototype, "value", {
  set(next) { setterReceiver = this; }
});
var receiver = Object.create(prototype);
receiver.value = 1;

var symbol = Symbol("value");
Reflect.set(receiver, symbol, 2);

function updateMapped(argument) {
  Object.defineProperty(arguments, "0", { writable: true });
  arguments[0] = 3;
  return argument;
}
updateMapped(2);

var array = [];
Reflect.set(array, "length", 1);

var target = {};
Object.defineProperty(target, "fixed", {
  configurable: false,
  writable: false,
  value: 4
});
var proxy = new Proxy(target, { set() { return true; } });
try { proxy.fixed = 5; } catch (error) {}
setterReceiver === receiver;
"#,
        )
        .expect("ordinary receiver set paths should emit");
        expect_valid_module(&artifact, 3);
    }

    #[test]
    fn string_script_emits_memory_and_data() {
        let artifact = emit_script("const s = \"hi\"; s;").expect("emit should work");
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
    }

    #[test]
    fn preseeded_string_bytes_and_literal_payloads_are_stable() {
        let artifact = emit_script("\",\";").expect("emit should work");
        let data = data_segment_bytes(&artifact.bytes);
        let mut expected_prefix = vec![b' '; 11];
        expected_prefix.extend_from_slice(b"\n: ,undefinednulltruefalse");
        assert!(
            data.starts_with(&expected_prefix),
            "unexpected data prefix: {:?}",
            &data[..data.len().min(32)]
        );
        assert!(
            contains_i64_const(
                &artifact.bytes,
                ((((STATIC_DATA_OFFSET as u64) + 14) << 32) | 1) as i64,
            ),
            "comma literal payload should be emitted as packed offset/len"
        );
        let globals = global_init_i64s(&artifact.bytes);
        assert!(
            globals.contains(&(align_heap_start(data.len()) as i64)),
            "heap ptr global should start after static data"
        );
    }

    #[test]
    fn regexp_program_data_is_aligned_deduplicated_and_before_the_heap() {
        let artifact = emit_script("\",\"; /[a-c]/; /[a-c]/g;").expect("emit should work");
        let data = data_segment_bytes(&artifact.bytes);
        let encoded = lila_ir::RegExpProgram::compile("[a-c]", "")
            .expect("class program should compile")
            .encode();
        let offsets = data
            .windows(encoded.len())
            .enumerate()
            .filter_map(|(offset, candidate)| (candidate == encoded).then_some(offset))
            .collect::<Vec<_>>();

        assert_eq!(offsets.len(), 1, "identical programs should share one blob");
        let program_ptr = STATIC_DATA_OFFSET as usize + offsets[0];
        assert_eq!(program_ptr % 8, 0, "program blob must be i64-aligned");
        assert!(
            contains_i64_const(&artifact.bytes, program_ptr as i64),
            "literal allocation should embed the collected program pointer"
        );
        assert!(
            contains_i64_const(
                &artifact.bytes,
                (encoded.len() / lila_ir::REGEXP_INSTRUCTION_WIDTH) as i64,
            ),
            "literal allocation should embed the instruction count"
        );
        assert!(
            contains_i64_const(
                &artifact.bytes,
                ((((STATIC_DATA_OFFSET as u64) + 14) << 32) | 1) as i64,
            ),
            "appending program data must not move existing string payloads"
        );
        assert!(
            global_init_i64s(&artifact.bytes).contains(&(align_heap_start(data.len()) as i64)),
            "heap must start after the appended program data"
        );
    }

    #[test]
    fn regexp_static_program_refs_preserve_capture_count_metadata() {
        let capture_program =
            lila_ir::RegExpProgram::compile(r"(\d+)", "").expect("capture program should compile");
        let no_capture_program = lila_ir::RegExpProgram::compile(r"\d+", "")
            .expect("non-capture program should compile");
        let mut pool = StringPool::default();

        let capture_ref = pool.collect_regexp_program_for_test(&capture_program);
        let no_capture_ref = pool.collect_regexp_program_for_test(&no_capture_program);

        assert_eq!(capture_ref.capture_count, 1);
        assert_eq!(no_capture_ref.capture_count, 0);
        assert_eq!(
            capture_ref.split_count as usize,
            capture_program
                .instructions
                .iter()
                .filter(|instruction| instruction.opcode == lila_ir::REGEXP_OPCODE_SPLIT)
                .count()
        );
        assert_eq!(
            no_capture_ref.split_count as usize,
            no_capture_program
                .instructions
                .iter()
                .filter(|instruction| instruction.opcode == lila_ir::REGEXP_OPCODE_SPLIT)
                .count()
        );
        assert_eq!(capture_ref.repeatable_split_count, 1);
        assert_eq!(no_capture_ref.repeatable_split_count, 1);
    }

    #[test]
    fn regexp_static_program_refs_distinguish_repeatable_splits() {
        let cases = [
            ("a?a?", 2, 0),
            ("a?b*", 2, 1),
            ("(a|b)*", 2, 2),
            (r"(?<=\w+)f", 2, 1),
        ];
        let mut pool = StringPool::default();
        for (pattern, split_count, repeatable_split_count) in cases {
            let program =
                lila_ir::RegExpProgram::compile(pattern, "").expect("program should compile");
            let reference = pool.collect_regexp_program_for_test(&program);
            assert_eq!(reference.split_count, split_count, "{pattern}");
            assert_eq!(
                reference.repeatable_split_count, repeatable_split_count,
                "{pattern}"
            );
        }
    }

    #[test]
    fn regexp_static_program_dedup_key_includes_capture_count() {
        let no_capture_program =
            lila_ir::RegExpProgram::compile("a", "").expect("program should compile");
        let mut capture_program = no_capture_program.clone();
        capture_program.capture_count = 1;
        assert_eq!(no_capture_program.encode(), capture_program.encode());
        assert_ne!(
            RegExpProgramStaticKey::from_program(&no_capture_program),
            RegExpProgramStaticKey::from_program(&capture_program),
            "capture metadata must be part of static-program identity"
        );

        let mut pool = StringPool::default();
        let no_capture_ref = pool.collect_regexp_program_for_test(&no_capture_program);
        let capture_ref = pool.collect_regexp_program_for_test(&capture_program);
        assert_ne!(no_capture_ref.ptr, capture_ref.ptr);
        assert_eq!(pool.bytes.len(), no_capture_program.encode().len() * 2);
    }

    #[test]
    fn regexp_literal_initializes_capture_count_slot() {
        let artifact = emit_script(r"/(\d+)/;").expect("emit should work");
        assert!(
            contains_i64_const_store_at_offset(
                &artifact.bytes,
                1,
                HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
            ),
            "literal allocation should initialize the immutable capture count"
        );

        let no_capture_artifact = emit_script(r"/\d+/;").expect("emit should work");
        assert!(
            contains_i64_const_store_at_offset(
                &no_capture_artifact.bytes,
                0,
                HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
            ),
            "no-capture literal allocation should initialize capture count to zero"
        );
    }

    #[test]
    fn regexp_literal_initializes_split_count_slot() {
        let artifact = emit_script(r"/(a|b)/;").expect("emit should work");
        assert!(
            contains_i64_const_store_at_offset(
                &artifact.bytes,
                1,
                HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
            ),
            "literal allocation should initialize immutable split metadata"
        );

        let no_choice_artifact = emit_script(r"/(a)/;").expect("emit should work");
        assert!(
            contains_i64_const_store_at_offset(
                &no_choice_artifact.bytes,
                0,
                HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
            ),
            "choice-free literal allocation should initialize split metadata to zero"
        );
    }

    #[test]
    fn regexp_literal_initializes_repeatable_split_count_slot() {
        let artifact = emit_script(r"/a?b*/;").expect("emit should work");
        assert!(contains_i64_const_store_at_offset(
            &artifact.bytes,
            1,
            HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
        ));

        let no_repeat_artifact = emit_script(r"/a?a?/;").expect("emit should work");
        assert!(contains_i64_const_store_at_offset(
            &no_repeat_artifact.bytes,
            0,
            HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
        ));
    }

    #[test]
    fn constructed_constant_regexp_collects_deduplicated_program_and_initializes_slots() {
        let artifact =
            emit_script(r#"/(a|b)*/; new RegExp("(a|b)*", "");"#).expect("emit should work");
        let program =
            lila_ir::RegExpProgram::compile("(a|b)*", "").expect("program should compile");
        let encoded = program.encode();
        let data = data_segment_bytes(&artifact.bytes);
        assert_eq!(
            data.windows(encoded.len())
                .filter(|candidate| *candidate == encoded)
                .count(),
            1,
            "identical constructed programs should share one blob"
        );
        let program_ptr = STATIC_DATA_OFFSET as i64
            + data
                .windows(encoded.len())
                .position(|candidate| candidate == encoded)
                .expect("program data should be present") as i64;
        let reference = {
            let mut pool = StringPool::default();
            pool.collect_regexp_program_for_test(&program)
        };
        for (value, offset) in [
            (program_ptr, HEAP_REGEXP_PROGRAM_PTR_OFFSET),
            (
                reference.instruction_count as i64,
                HEAP_REGEXP_PROGRAM_INSTRUCTION_COUNT_OFFSET,
            ),
            (
                reference.capture_count as i64,
                HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
            ),
            (
                reference.split_count as i64,
                HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
            ),
            (
                reference.repeatable_split_count as i64,
                HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
            ),
        ] {
            assert!(
                contains_i64_const_store_at_offset(&artifact.bytes, value, offset),
                "constructed regexp should initialize matcher slot {offset}"
            );
        }
    }

    #[test]
    fn large_static_string_data_increases_initial_memory_pages() {
        let source = format!("\"{}\";", "x".repeat(WASM_PAGE_SIZE as usize));
        let artifact = emit_script(&source).expect("emit should work");
        let pages = memory_initial_pages(&artifact.bytes);
        assert!(pages[0] >= 2);
    }

    #[test]
    fn supports_assignment_branching_and_loops() {
        let artifact = emit_script(
            "let i = 0; let sum = 0; for (; i < 5; i = i + 1) { if (i === 2) { continue; } if (i === 4) { break; } sum = sum + i; } sum;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_updates_and_compound_assignment() {
        let artifact = emit_script("let sum = 0; for (let i = 0; i < 4; i++) { sum += i; } sum;")
            .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_switch_labels_and_debugger() {
        let artifact = emit_script(
            "let x = 0; outer: while (x < 3) { x += 1; switch (x) { case 1: continue outer; case 2: debugger; break outer; default: break; } } x;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_direct_function_calls_and_recursion() {
        let artifact = emit_script(
            "function up(n) { if (n === 0) { return 0; } return up(n - 1) + 1; } up(3);",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
        assert!(artifact.debug_dump.contains("internal functions: "));
    }

    #[test]
    fn outlined_function_call_preserves_receiver_and_arguments_module_validates() {
        let artifact = emit_script(
            "function combine(left, right) { return this.base + left + right; } let receiver = { base: 4, combine: combine }; receiver.combine(2, 3);",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn outlined_function_call_throw_routing_module_validates() {
        let artifact = emit_script(
            "function fail(value) { throw value; } let caught = 0; try { fail(7); } catch (error) { caught = error; } caught;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn outlined_proxy_fallback_call_module_validates() {
        let artifact = emit_script(
            "function combine(left, right) { return this.base + left + right; } let callable = new Proxy(combine, {}); let receiver = { base: 4, callable: callable }; receiver.callable(2, 3);",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn outlined_dynamic_property_read_normalizes_computed_key_once_module_validates() {
        let artifact = emit_script(
            "let calls = 0; let key = { [Symbol.toPrimitive]() { calls += 1; return \"value\"; } }; let object = { value: 3 }; let absent = null; let skipped = absent?.[key]; let read = object?.[key]; calls === 1 && skipped === undefined && read === 3;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn outlined_dynamic_property_read_preserves_runtime_exotics_module_validates() {
        let artifact = emit_script(
            "function readArguments() { return arguments?.length === 3 && arguments?.[1] === 2; } let key = Symbol(\"key\"); let object = { [key]: 5 }; let values = [1, 2]; \"ab\"?.[1] === \"b\" && values?.length === 2 && object?.[key] === 5 && readArguments(1, 2, 3);",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn outlined_dynamic_property_read_preserves_proxy_receiver_module_validates() {
        let artifact = emit_script(
            "let seen = false; let proxy; let target = { get value() { return this === proxy ? 7 : 0; } }; proxy = new Proxy(target, { get(target, key, receiver) { seen = key === \"value\" && receiver === proxy; return Reflect.get(target, key, receiver); } }); proxy?.value === 7 && seen;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn ordinary_computed_dynamic_property_reads_have_bounded_incremental_body_growth() {
        let single_read = emit_script(
            r#"
function choose(flag) { return flag ? { value: 1 } : null; }
let object = choose(true);
let key = "value";
object[key];
"#,
        )
        .expect("single ordinary computed property read should emit");
        let repeated_reads = emit_script(
            r#"
function choose(flag) { return flag ? { value: 1 } : null; }
let object = choose(true);
let key = "value";
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
object[key];
"#,
        )
        .expect("repeated ordinary computed property reads should emit");
        expect_valid_module(&single_read, 1);
        expect_valid_module(&repeated_reads, 1);

        let main_body_bytes = |artifact: &WasmArtifact| {
            Parser::new(0)
                .parse_all(&artifact.bytes)
                .find_map(
                    |payload| match payload.expect("wasm parse should succeed") {
                        Payload::CodeSectionEntry(body) => Some(body.range().len()),
                        _ => None,
                    },
                )
                .expect("emitted module should contain a main function")
        };
        let single_read_body_bytes = main_body_bytes(&single_read);
        let repeated_read_body_bytes = main_body_bytes(&repeated_reads);
        let incremental_body_bytes = repeated_read_body_bytes
            .checked_sub(single_read_body_bytes)
            .expect("repeated reads should not shrink the main function");
        assert!(
            incremental_body_bytes < 64 * 1024,
            "eleven additional outlined reads added {incremental_body_bytes} bytes \
             ({single_read_body_bytes} -> {repeated_read_body_bytes})"
        );
    }

    #[test]
    fn statically_nullish_computed_property_read_emits_after_throw_path() {
        let artifact = emit_script(
            r#"
let calls = 0;
function key() { calls += 1; return "value"; }
try { null[key()]; } catch (error) {}
calls;
"#,
        )
        .expect("statically nullish computed property read should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn object_and_array_scripts_emit_memory_with_agent_capability_import() {
        let artifact =
            emit_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;").expect("emit should work");
        expect_valid_module(&artifact, 0);
        assert!(artifact
            .debug_dump
            .contains("import func: lila_host.agent_can_suspend"));
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
    }

    #[test]
    fn test262_agent_builtins_import_the_agent_call_and_split_memories() {
        let artifact =
            emit_script("__lilaAgentSleep(1);").expect("Test262 agent host call should emit");
        expect_valid_module(&artifact, 0);

        assert!(artifact
            .debug_dump
            .contains("import func: lila_host.agent_call"));
        assert!(artifact
            .debug_dump
            .contains("import memory: lila_host.private_memory"));
        assert!(artifact
            .debug_dump
            .contains("import memory: lila_host.shared_memory"));
    }

    #[test]
    fn product_lowering_cannot_reauthorize_a_test262_name_in_aot() {
        let source = parse("__lilaAgentSleep;", ParseOptions::script())
            .expect("product script should parse");
        let artifact =
            emit(&lower(&source)).expect("an unresolved global identifier is handled at runtime");

        assert!(!artifact
            .debug_dump
            .contains("import func: lila_host.agent_call"));
    }

    #[test]
    fn supports_sparse_array_assignment_module_validates() {
        let artifact = emit_script("let a = [1]; a[2] = 4; a[2];").expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_object_property_assignment_module_validates() {
        let artifact = emit_script("let o = {}; o.x = 1; o.x;").expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_object_return_from_function() {
        let artifact =
            emit_script("function box(x) { let o = { x: x }; return o; } let o = box(2); o.x;")
                .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn supports_chained_heap_access_and_array_length() {
        let artifact = emit_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        )
        .expect("emit should work");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn supports_heap_growth_beyond_initial_capacity() {
        let source = format!(
            "let o = {{}}; {} o.k64;",
            (0..65)
                .map(|index| format!("o[\"k{index}\"] = {index};"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let artifact = emit_script(&source).expect("emit should work");
        expect_valid_module(&artifact, 0);
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
    }

    #[test]
    fn supports_dynamic_primitive_string_concat() {
        let artifact = emit_script("\"a\" + \"b\";").expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_primitive_coercion_core() {
        let artifact = emit_script("1 == \"1\"; \"2\" - 1; \"10\" > \"2\"; void 1; (1, 2);")
            .expect("emit should work");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn supports_heap_coercion_core() {
        let artifact = emit_script(
            "\"a\" + {}; let o = { valueOf() { return 2; } }; o + 1; [1, 2] + 3; ({}) == 1; [2] < 3; function f() { return arguments + \"\"; } f(1, 2);",
        )
        .expect("heap coercion should emit");
        expect_valid_module(&artifact, 2);
    }

    #[test]
    fn supports_dynamic_value_plus_proven_string() {
        let artifact = emit_script(
            "function choose(flag) { if (flag) return 1; return {}; } function format(message) { return message + \" suffix\"; } format(choose(true));",
        )
        .expect("dynamic plus string should emit");
        expect_valid_module(&artifact, 2);
    }

    #[test]
    fn supports_host_gc_builtin_as_explicit_unsupported_throw() {
        let artifact = emit_script("if (typeof gc === \"function\") { gc(); }")
            .expect("gc host builtin should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn supports_typeof_heap_values() {
        let object_artifact =
            emit_script("let obj = {}; typeof obj;").expect("object typeof script should emit");
        expect_valid_module(&object_artifact, 0);

        let function_artifact = emit_script("let f = function() {}; typeof f;")
            .expect("function typeof script should emit");
        expect_valid_module(&function_artifact, 1);
    }

    #[test]
    fn supports_global_var_object_write() {
        let artifact = emit_script("var x = 1; x;").expect("var write script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_eval_target_module_validates() {
        let artifact = emit_script("var proxy = new Proxy(eval, {}); proxy();")
            .expect("proxy script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_non_callable_target_call_module_validates() {
        let artifact = emit_script(
            r#"var p = new Proxy({}, {});
try {
  p();
} catch (error) {
  error instanceof TypeError;
}"#,
        )
        .expect("proxy non-callable script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_revocation_function_metadata_module_validates() {
        let artifact = emit_script(
            r#"var revocationFunction = Proxy.revocable({}, {}).revoke;
Object.getOwnPropertyDescriptor(revocationFunction, "length");
Object.getOwnPropertyDescriptor(revocationFunction, "name");
Object.getOwnPropertyNames(revocationFunction);"#,
        )
        .expect("proxy revocation metadata script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn typedarray_own_property_keys_module_validates() {
        let artifact = emit_script(
            r#"var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var view = new Uint8Array(buffer, 1);
var symbol = Symbol("key");
view.visible = 1;
Object.defineProperty(view, "hidden", { value: 2, enumerable: false });
view[symbol] = 3;
Reflect.ownKeys(view);
Object.getOwnPropertyNames(view);
Reflect.ownKeys(view.subarray(1));
buffer.resize(8);
Reflect.ownKeys(view);
buffer.resize(0);
Reflect.ownKeys(view);"#,
        )
        .expect("typed array own property keys script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn object_seal_module_validates() {
        let artifact = emit_script(
            r#"var target = { value: 1 };
Object.seal(target);
Object.getOwnPropertyDescriptor(target, "value");
var array = [1];
Object.seal(array);
var proxy = new Proxy({ x: 1 }, {
  preventExtensions: function(target) {
    return Reflect.preventExtensions(target);
  },
  ownKeys: function(target) {
    return Reflect.ownKeys(target);
  },
  defineProperty: function(target, key, descriptor) {
    return Reflect.defineProperty(target, key, descriptor);
  }
});
Object.seal(proxy);"#,
        )
        .expect("Object.seal script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn typedarray_get_own_property_descriptor_module_validates() {
        let artifact = emit_script(
            r#"var numeric = new Uint8Array([42]);
Object.getOwnPropertyDescriptor(numeric, "0");
Object.getOwnPropertyDescriptor(numeric, "-0");
Object.getOwnPropertyDescriptor(numeric, "1.0");
Object.getOwnPropertyDescriptor(numeric, "1.1");
Object.getOwnPropertyDescriptor(numeric, "Infinity");
var bigint = new BigInt64Array([42n]);
Object.getOwnPropertyDescriptor(bigint, 0);
var detached = new Uint8Array([1]);
__lilaDetachArrayBuffer(detached.buffer);
Object.getOwnPropertyDescriptor(detached, 0);
var other = __lilaCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__lilaDetachArrayBuffer(otherDetached.buffer);
Object.getOwnPropertyDescriptor(otherDetached, 0);"#,
        )
        .expect("typed array get own property script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn typedarray_has_property_module_validates() {
        let artifact = emit_script(
            r#"var view = new Uint8Array([42, 43]);
Reflect.has(view, 0);
Reflect.has(view, "-0");
Reflect.has(view, "1.0");
Reflect.has(view, "Infinity");
var bigint = new BigInt64Array([42n]);
Reflect.has(bigint, 0);
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var tracking = new Uint8Array(buffer, 1);
Reflect.has(tracking, 2);
buffer.resize(1);
Reflect.has(tracking, 0);
var detached = new Uint8Array([1]);
__lilaDetachArrayBuffer(detached.buffer);
Reflect.has(detached, 0);"#,
        )
        .expect("typed array has property script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn typedarray_delete_module_validates() {
        let artifact = emit_script(
            r#"var numeric = new Uint8Array([42]);
delete numeric[0];
delete numeric["-0"];
delete numeric["1.1"];
Reflect.deleteProperty(numeric, "Infinity");
var bigint = new BigInt64Array([42n]);
delete bigint[0];
var shared = new Uint8Array(new SharedArrayBuffer(1));
delete shared[0];
var detached = new Uint8Array([1]);
__lilaDetachArrayBuffer(detached.buffer);
delete detached[0];
function strictDelete(view) {
  "use strict";
  delete view[0];
}
try { strictDelete(numeric); } catch (error) {}
var proxy = new Proxy(numeric, { deleteProperty: function() { return true; } });
delete proxy[0];"#,
        )
        .expect("typed array delete script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn atomics_modules_import_private_and_capped_shared_memories() {
        for source in [
            "var view = new Int32Array(new SharedArrayBuffer(8)); view[0] = 1; Atomics.add(view, 0, 2); Atomics.compareExchange(view, 0, 3, 4); view[0];",
            "var view = new Int32Array(new SharedArrayBuffer(8)); var add = Atomics['add']; add(view, 0, 2);",
        ] {
            let artifact = emit_script(source).expect("Atomics script should emit");
            expect_valid_module(&artifact, 0);

            let mut memory_imports = Vec::new();
            let mut atomic_memory_indexes = Vec::new();
            for payload in Parser::new(0).parse_all(&artifact.bytes) {
                match payload.expect("wasm parse should succeed") {
                Payload::ImportSection(reader) => {
                    for imports in reader {
                        match imports.expect("import should decode") {
                            wasmparser::Imports::Single(_, import) => {
                                if let wasmparser::TypeRef::Memory(memory) = import.ty {
                                    memory_imports.push((
                                        import.name.to_string(),
                                        memory.shared,
                                        memory.maximum,
                                    ));
                                }
                            }
                            wasmparser::Imports::Compact1 { items, .. } => {
                                for import in items {
                                    let import = import.expect("compact import should decode");
                                    if let wasmparser::TypeRef::Memory(memory) = import.ty {
                                        memory_imports.push((
                                            import.name.to_string(),
                                            memory.shared,
                                            memory.maximum,
                                        ));
                                    }
                                }
                            }
                            wasmparser::Imports::Compact2 { ty, names, .. } => {
                                if let wasmparser::TypeRef::Memory(memory) = ty {
                                    for name in names {
                                        memory_imports.push((
                                            name.expect("compact import name should decode")
                                                .to_string(),
                                            memory.shared,
                                            memory.maximum,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Payload::CodeSectionEntry(body) => {
                    let mut reader = body
                        .get_operators_reader()
                        .expect("operators should decode");
                    while !reader.eof() {
                        match reader.read().expect("operator should decode") {
                            Operator::I32AtomicRmwAdd { memarg }
                            | Operator::I32AtomicRmwCmpxchg { memarg }
                            | Operator::I32AtomicLoad { memarg }
                            | Operator::I32AtomicStore { memarg } => {
                                atomic_memory_indexes.push(memarg.memory);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            }

            assert_eq!(
                memory_imports,
                vec![
                    ("private_memory".to_string(), false, None),
                    ("shared_memory".to_string(), true, Some(16_384)),
                ],
                "source: {source}"
            );
            assert!(!atomic_memory_indexes.is_empty(), "source: {source}");
            assert!(
                atomic_memory_indexes.iter().all(|index| *index == 1),
                "source: {source}"
            );
        }
    }

    #[test]
    fn atomics_wait_async_modules_import_monotonic_timeout_host_functions() {
        let artifact = emit_script(
            "var view = new Int32Array(new SharedArrayBuffer(4)); Atomics.waitAsync(view, 0, 0, 1);",
        )
        .expect("Atomics.waitAsync script should emit");
        expect_valid_module(&artifact, 0);

        assert!(artifact
            .debug_dump
            .contains("import func: lila_host.monotonic_clock_nanos"));
        assert!(artifact
            .debug_dump
            .contains("import func: lila_host.sleep_nanos"));
    }

    #[test]
    fn typedarray_define_own_property_module_validates() {
        let artifact = emit_script(
            r#"var numeric = new Uint8Array([1]);
Object.defineProperty(numeric, 0, { value: 2 });
Reflect.defineProperty(numeric, "-0", { value: 3 });
Reflect.defineProperty(numeric, "1.1", { value: 4 });
Reflect.defineProperty(numeric, "Infinity", { value: 5 });
var bigint = new BigInt64Array([1n]);
Object.defineProperty(bigint, 0, { value: 2n, configurable: true });
var ordinary = Symbol("ordinary");
Reflect.defineProperty(numeric, "1.0", { value: 6 });
Reflect.defineProperty(numeric, ordinary, { value: 7 });
var detached = new Uint8Array([1]);
__lilaDetachArrayBuffer(detached.buffer);
Reflect.defineProperty(detached, 0, { value: 2 });
var proxy = new Proxy(numeric, { defineProperty: function() { return true; } });
Object.defineProperty(proxy, 0, { value: 8 });"#,
        )
        .expect("typed array define own property script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn typedarray_set_module_validates() {
        let artifact = emit_script(
            r#"var target = new Uint8Array([1]);
var directValue = { valueOf: function() { return 2; } };
target["-0"] = directValue;
target["1.1"] = directValue;
target["-1"] = directValue;
Reflect.set(target, 0, directValue);
Reflect.set(target, "1.1", directValue);
var receiver = {};
Reflect.set(target, 0, directValue, receiver);
Reflect.set(target, "1.1", directValue, receiver);
var typedReceiver = new Uint8Array([0]);
Reflect.set(target, 0, 257, typedReceiver);
var inheritedReceiver = Object.create(target);
inheritedReceiver[0] = directValue;
inheritedReceiver["1.1"] = directValue;"#,
        )
        .expect("typed array set script should emit");
        expect_valid_module(&artifact, 0);
    }

    #[test]
    fn proxy_revoked_cross_realm_call_module_validates() {
        let artifact = emit_script(
            r#"var other = __lilaCreateRealm();
var OProxy = other.global.Proxy;
var proxyObj = OProxy.revocable(function() {}, {});
var proxy = proxyObj.proxy;
proxyObj.revoke();

var caught = false;
try {
  proxy();
} catch (error) {
  caught = true;
  if (Object.getPrototypeOf(error) !== other.global.TypeError.prototype) {
    throw "revoked proxy wrong realm";
  }
}

if (!caught) throw "revoked proxy missing TypeError";"#,
        )
        .expect("cross-realm revoked proxy script should emit");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn supports_object_returning_to_primitive_hook_fallback() {
        let artifact = emit_script("let o = { valueOf() { return {}; } }; o + 1;")
            .expect("object-returning valueOf should fall back to toString");
        expect_valid_module(&artifact, 1);
    }

    #[test]
    fn supports_coercive_compound_assignment() {
        let artifact = emit_script("let s = \"a\"; s += \"b\";").expect("emit should work");
        expect_valid_module(&artifact, 0);
    }
}
