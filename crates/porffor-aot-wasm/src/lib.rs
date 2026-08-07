use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use porffor_ir::{
    private_brand_key, private_data_key, ArithmeticBinaryOp, ArrayDestructuringElementIr,
    ArrayDestructuringPatternIr, BindingMode, BitwiseBinaryOp, BlockIr,
    CallableToStringRepresentation, ClassDefinitionIr, ClassElementDefinitionIr,
    ClassElementExecutionKind, ClassFieldKeyIr, ClassFunctionKind, ClassHeritageKind,
    ClassInstanceElementPlanIr, ClassMethodPlacementIr, ClassStaticElementIr,
    DeleteIdentifierKindIr, DestructuringPropertyKeyIr, DestructuringTargetIr, EqualityBinaryOp,
    ExprIr, ForInOfEnvironmentIr, ForInitIr, ForLexicalEnvironmentIr, FunctionExecutionKind,
    FunctionFlavor, FunctionId, FunctionIr, FunctionParamIr, GeneratorResumeModeIr,
    GeneratorTryPlanIr, HeapShape, HostBuiltinId, JsonStaticValueIr, KindSet, LexicalEnvironmentIr,
    LogicalBinaryOp, NumericUpdateOp, ObjectPropertyIr, ObjectShapeProperty, OwnedEnvBindingIr,
    PrivateNameId, PropertyKeyIr, RelationalBinaryOp, ScriptGlobalBindingIr,
    ScriptGlobalBindingKind, ScriptIr, SpecOperationIr, StandardBuiltinId, StatementIr,
    SwitchCaseIr, ToPrimitiveHint, TypedExpr, UnaryNumericOp, UpdateReturnMode, ValueInfo,
    ValueKind, VarDeclaratorIr, AGGREGATE_ERROR_NAME, ARRAY_BUFFER_NAME, ARRAY_NAME, ATOMICS_NAME,
    BIGINT64_ARRAY_NAME, BIGUINT64_ARRAY_NAME, BOOLEAN_NAME, DATA_VIEW_NAME, DATE_NAME,
    DATE_VALUE_SLOT, ERROR_NAME, EVAL_ERROR_NAME, FLOAT32_ARRAY_NAME, FLOAT64_ARRAY_NAME,
    FUNCTION_NAME, GLOBAL_THIS_NAME, HOST_PARSE_FLOAT_FUNCTION_ID, INT16_ARRAY_NAME,
    INT32_ARRAY_NAME, INT8_ARRAY_NAME, IS_CONSTRUCTOR_NAME, JSON_NAME,
    JS_STRING_SURROGATE_SENTINEL, LEXICAL_ARGUMENTS_NAME, LEXICAL_HOME_OBJECT_NAME,
    LEXICAL_NEW_TARGET_NAME, LEXICAL_THIS_NAME, MAP_NAME, MATH_NAME, NUMBER_NAME, OBJECT_NAME,
    PORFFOR_GENERATOR_THROW_SLOT, PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT,
    PORFFOR_STATIC_GENERATOR_VALUES_METHOD, PRINT_NAME, PROXY_NAME, RANGE_ERROR_NAME,
    REFERENCE_ERROR_NAME, REFLECT_NAME, REGEXP_NAME, SET_NAME, SHARED_ARRAY_BUFFER_NAME,
    STRING_NAME, SUPPRESSED_ERROR_NAME, SYMBOL_NAME, SYNTAX_ERROR_NAME, TEMPORAL_DURATION_NAME,
    TEMPORAL_NOW_NAME, TEMPORAL_PLAIN_DATE_NAME, TEMPORAL_PLAIN_DATE_TIME_NAME,
    TEMPORAL_PLAIN_MONTH_DAY_NAME, TEMPORAL_PLAIN_TIME_NAME, TEMPORAL_PLAIN_YEAR_MONTH_NAME,
    TYPE_ERROR_NAME, UINT16_ARRAY_NAME, UINT32_ARRAY_NAME, UINT8_ARRAY_NAME,
    UINT8_CLAMPED_ARRAY_NAME, URI_ERROR_NAME,
};
use wasm_encoder::{BlockType, Function, Ieee64, Instruction, MemArg, ValType};

mod abi;
mod bigint;
mod builtins;
mod control_flow;
mod data;
mod emit;
mod environments;
mod expressions;
mod functions;
mod generator_delegation;
mod heap;
mod intrinsics;
mod module;
mod modules;
mod objects;
mod operations;
mod planning;
mod runtime_abi;
use abi::*;
use bigint::BigIntHelperOp;
use builtins::*;
use data::*;
pub use emit::emit;
pub(crate) use emit::{
    BindingStorage, CompletionKind, ControlFrameKind, FunctionBuilder, IteratorCloseOnThrowLocals,
    LabelTargets, LoopTargets, OrdinarySetDataOnReceiverEmission, ReturnAbi,
};
use heap::*;
use intrinsics::*;
use module::*;
use modules::module_unit_guard_count;
use planning::*;
pub use runtime_abi::{decode_heap_bigint_decimal, WasmRuntimeDecodeError, WasmRuntimeValueTag};

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
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::{lower, BigIntLiteralIr};
    use wasmparser::{Operator, Parser, Payload, Validator, WasmFeatures};

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
            r#"var other = __porfCreateRealm().global;
var C = other.Object;
C.prototype = null;
Reflect.construct(Map, [], C);"#,
            r#"var other = __porfCreateRealm().global;
var C = other.Object;
C.prototype = null;
var bound = C.bind(null);
Reflect.construct(Map, [], bound);"#,
            r#"var other = __porfCreateRealm().global;
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
            r#"var other = __porfCreateRealm().global;
var C = other.Object;
C.prototype = null;
Reflect.construct(Set, [], C);"#,
            r#"var other = __porfCreateRealm().global;
var C = other.Object;
C.prototype = null;
var bound = C.bind(null);
Reflect.construct(Set, [], bound);"#,
            r#"var other = __porfCreateRealm().global;
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
                .contains("import func: porf_host.wall_clock_millis"),
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
                .contains("import func: porf_host.wall_clock_millis"),
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
    /// production wasmtime configuration (`porffor-engine`'s
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
    fn dynamic_number_exponentiation_module_validates_with_runtime_pow_import() {
        let artifact = emit_script(
            "let base = 9; let exponent = 0.5; base ** exponent + Math.pow(base, exponent);",
        )
        .expect("dynamic Number exponentiation should emit");

        expect_valid_module(&artifact, 0);
        assert!(
            artifact
                .debug_dump
                .contains("import func: porf_host.number_pow"),
            "{}",
            artifact.debug_dump
        );
    }

    #[test]
    fn date_now_module_imports_wall_clock_milliseconds() {
        let artifact = emit_script("Date.now();").expect("Date.now script should emit");

        expect_valid_module(&artifact, 0);
        assert!(
            artifact
                .debug_dump
                .contains("import func: porf_host.wall_clock_millis"),
            "{}",
            artifact.debug_dump
        );

        let artifact = emit_script("262;").expect("constant script should emit");
        assert!(
            !artifact
                .debug_dump
                .contains("import func: porf_host.wall_clock_millis"),
            "{}",
            artifact.debug_dump
        );
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
        let encoded = porffor_ir::RegExpProgram::compile("[a-c]", "")
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
                (encoded.len() / porffor_ir::REGEXP_INSTRUCTION_WIDTH) as i64,
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
        let capture_program = porffor_ir::RegExpProgram::compile(r"(\d+)", "")
            .expect("capture program should compile");
        let no_capture_program = porffor_ir::RegExpProgram::compile(r"\d+", "")
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
                .filter(|instruction| instruction.opcode == porffor_ir::REGEXP_OPCODE_SPLIT)
                .count()
        );
        assert_eq!(
            no_capture_ref.split_count as usize,
            no_capture_program
                .instructions
                .iter()
                .filter(|instruction| instruction.opcode == porffor_ir::REGEXP_OPCODE_SPLIT)
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
                porffor_ir::RegExpProgram::compile(pattern, "").expect("program should compile");
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
            porffor_ir::RegExpProgram::compile("a", "").expect("program should compile");
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
            porffor_ir::RegExpProgram::compile("(a|b)*", "").expect("program should compile");
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
            .contains("import func: porf_host.agent_can_suspend"));
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
    }

    #[test]
    fn test262_agent_builtins_import_the_agent_call_and_split_memories() {
        let artifact =
            emit_script("__porfAgentSleep(1);").expect("Test262 agent host call should emit");
        expect_valid_module(&artifact, 0);

        assert!(artifact
            .debug_dump
            .contains("import func: porf_host.agent_call"));
        assert!(artifact
            .debug_dump
            .contains("import memory: porf_host.private_memory"));
        assert!(artifact
            .debug_dump
            .contains("import memory: porf_host.shared_memory"));
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
__porfDetachArrayBuffer(detached.buffer);
Object.getOwnPropertyDescriptor(detached, 0);
var other = __porfCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__porfDetachArrayBuffer(otherDetached.buffer);
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
__porfDetachArrayBuffer(detached.buffer);
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
__porfDetachArrayBuffer(detached.buffer);
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
            .contains("import func: porf_host.monotonic_clock_nanos"));
        assert!(artifact
            .debug_dump
            .contains("import func: porf_host.sleep_nanos"));
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
__porfDetachArrayBuffer(detached.buffer);
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
