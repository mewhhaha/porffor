use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use porffor_ir::{
    private_brand_key, private_data_key, ArithmeticBinaryOp, BindingMode, BitwiseBinaryOp, BlockIr,
    CallableToStringRepresentation, ClassDefinitionIr, ClassFunctionKind, ClassHeritageKind,
    ClassMethodPlacementIr, DeleteIdentifierKindIr, EqualityBinaryOp, ExprIr, ForInitIr,
    FunctionFlavor, FunctionId, FunctionIr, FunctionParamIr, HeapShape, HostBuiltinId,
    JsonStaticValueIr, KindSet, LogicalBinaryOp, NumericUpdateOp, ObjectPropertyIr,
    ObjectShapeProperty, OwnedEnvBindingIr, PrivateNameId, PropertyKeyIr, RelationalBinaryOp,
    ScriptGlobalBindingIr, ScriptGlobalBindingKind, ScriptIr, SpecOperationIr, StandardBuiltinId,
    StatementIr, SwitchCaseIr, ToPrimitiveHint, TypedExpr, UnaryNumericOp, UpdateReturnMode,
    ValueInfo, ValueKind, VarDeclaratorIr, AGGREGATE_ERROR_NAME, ARRAY_BUFFER_BYTE_LENGTH_SLOT,
    ARRAY_BUFFER_DATA_PTR_SLOT, ARRAY_BUFFER_IMMUTABLE_SLOT, ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT,
    ARRAY_BUFFER_NAME, ARRAY_BUFFER_RESIZABLE_SLOT, ARRAY_BUFFER_SHARED_SLOT, ARRAY_NAME,
    ATOMICS_NAME, BOOLEAN_NAME, DATA_VIEW_BYTE_LENGTH_SLOT, DATA_VIEW_BYTE_OFFSET_SLOT,
    DATA_VIEW_DATA_PTR_SLOT, DATA_VIEW_LENGTH_TRACKING_SLOT, DATA_VIEW_NAME, DATE_NAME,
    DATE_VALUE_SLOT, ERROR_NAME, EVAL_ERROR_NAME, FLOAT32_ARRAY_NAME, FLOAT64_ARRAY_NAME,
    FUNCTION_NAME, GLOBAL_THIS_NAME, HOST_PARSE_FLOAT_FUNCTION_ID, INT16_ARRAY_NAME,
    INT32_ARRAY_NAME, INT8_ARRAY_NAME, IS_CONSTRUCTOR_NAME, JSON_NAME,
    JS_STRING_SURROGATE_SENTINEL, LEXICAL_ARGUMENTS_NAME, LEXICAL_NEW_TARGET_NAME,
    LEXICAL_THIS_NAME, MATH_NAME, NUMBER_NAME, OBJECT_NAME, PORFFOR_GENERATOR_THROW_SLOT,
    PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT, PORFFOR_STATIC_GENERATOR_VALUES_METHOD, PRINT_NAME,
    PROXY_NAME, RANGE_ERROR_NAME, REFERENCE_ERROR_NAME, REFLECT_NAME, REGEXP_NAME,
    SHARED_ARRAY_BUFFER_NAME, STRING_NAME, SUPPRESSED_ERROR_NAME, SYNTAX_ERROR_NAME,
    TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT, TYPED_ARRAY_BYTE_LENGTH_SLOT, TYPED_ARRAY_BYTE_OFFSET_SLOT,
    TYPED_ARRAY_ELEMENT_KIND_SLOT, TYPED_ARRAY_LENGTH_TRACKING_SLOT,
    TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT, TYPE_ERROR_NAME, UINT16_ARRAY_NAME, UINT32_ARRAY_NAME,
    UINT8_ARRAY_NAME, UINT8_CLAMPED_ARRAY_NAME, URI_ERROR_NAME,
};
use wasm_encoder::{BlockType, Function, Ieee64, Instruction, MemArg, ValType};

mod abi;
mod builtins;
mod control_flow;
mod data;
mod emit;
mod environments;
mod expressions;
mod functions;
mod heap;
mod module;
mod objects;
mod operations;
mod planning;
use abi::*;
use builtins::*;
use data::*;
pub use emit::emit;
pub(crate) use emit::{
    BindingStorage, CompletionKind, ControlFrameKind, FunctionBuilder, IteratorCloseOnThrowLocals,
    LabelTargets, LoopTargets, ReturnAbi,
};
use heap::*;
use module::*;
use planning::*;

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

fn typed_expr_has_typed_array_shape(expr: &TypedExpr) -> bool {
    expr.heap_shape
        .as_deref()
        .and_then(|shape| {
            read_static_heap_shape_property(shape, TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT)
        })
        .is_some()
}

static EMPTY_BLOCK: LazyLock<BlockIr> = LazyLock::new(|| BlockIr {
    statements: Vec::new(),
    result_kind: ValueKind::Undefined,
});

#[cfg(test)]
mod tests {
    use super::*;
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::{lower, BigIntLiteralIr};
    use wasmi::{Engine as WasmiEngine, Module as WasmiModule};
    use wasmparser::{Operator, Parser, Payload};

    fn emit_script(source: &str) -> Result<WasmArtifact, EmitError> {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        emit(&lower(&source))
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
    fn rejects_arbitrary_precision_bigint_literal_until_heap_storage_lands() {
        let source = parse("184467440737095516161234567890n;", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        let err = emit(&program).expect_err("arbitrary precision BigInt should not truncate");
        let message = err.to_string();
        assert!(
            message.contains("BigInt literal requires heap-backed arbitrary precision storage"),
            "{}",
            message
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
            "let value = function C() {}; __porfIsConstructor(value);",
            ParseOptions::script(),
        )
        .expect("script should parse");
        let program = lower(&source);
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
        script.body.statements[0] = StatementIr::Expression(TypedExpr::spec_to_property_key(
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Symbol),
                ExprIr::Symbol { description: None },
            ),
        ));
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

    fn expect_valid_module(artifact: &WasmArtifact, _script_function_count: usize) {
        let engine = WasmiEngine::default();
        WasmiModule::new(&engine, &artifact.bytes[..]).unwrap_or_else(|err| {
            let message = err.to_string();
            let context = validation_error_offset(&message)
                .map(|offset| code_body_context(&artifact.bytes, offset))
                .unwrap_or_else(|| "no validation offset found".to_string());
            panic!("module should validate: {message}; {context}");
        });
    }

    #[test]
    fn emitted_module_validates() {
        let artifact = emit_script("let x = 40; const y = 2; x + y;").expect("emit should work");
        expect_valid_module(&artifact, 0);
        assert!(artifact.debug_dump.contains("export func: main"));
        assert!(artifact.debug_dump.contains("export global: result_tag"));
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
    fn object_and_array_scripts_emit_memory_without_imports() {
        let artifact =
            emit_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;").expect("emit should work");
        expect_valid_module(&artifact, 0);
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
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
    fn proxy_revoked_cross_realm_call_module_validates() {
        let artifact = emit_script(
            r#"var other = __porfCreateRealm();
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
