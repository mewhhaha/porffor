use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions, WasmExecutionFailureKind,
};
use lila_ir::DynamicSourceRuntimeOperation;

fn assert_trace(source: &str, policy: HostSurfacePolicy, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("indirect eval uses its live argument guard");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {:?}\n{source}",
        outcome.completion,
        outcome.output_events
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| { HostOutputEvent::PrintLine((*line).to_string()) })
            .collect::<Vec<_>>(),
        "{source}"
    );
}

#[test]
fn mutable_global_objects_pass_through_without_primitive_source_arguments() {
    assert_trace(
        r#"
var x = {};
print((0, eval)(x) === x);
x = new Number(1);
print((0, eval)(x) === x);
x = new Boolean(true);
print((0, eval)(x) === x);
x = new String('1+1');
print((0, eval)(x) === x);
"#,
        HostSurfacePolicy::Product,
        &["true", "true", "true", "true"],
    );
}

#[test]
fn all_non_string_arguments_keep_identity_without_primitive_conversion() {
    assert_trace(
        r#"
var conversions = 0, marker = {};
var object = { [Symbol.toPrimitive]() { conversions++; throw marker; }, toString() { throw marker; } };
var values = [undefined, null, true, -0, 17, 1n, 1n << 80n, Symbol(), object, [], function() {}, new String('throw 1')];
function invoke(value) { return (0, eval)(value); }
for (const value of values) {
  if (!Object.is(invoke(value), value)) throw 'changed value';
  if (!Object.is(eval.call(undefined, value), value)) throw 'changed call value';
  if (!Object.is(eval.apply(undefined, [value]), value)) throw 'changed apply value';
  if (!Object.is(Reflect.apply(eval, undefined, [value]), value)) throw 'changed reflected value';
  if (!Object.is((0, eval)(...[value]), value)) throw 'changed spread value';
}
print(conversions);
print((0, eval)(...[]) === undefined);
"#,
        HostSurfacePolicy::Product,
        &["0", "true"],
    );
}

#[test]
fn live_callee_arguments_and_abrupt_completion_preserve_ordinary_call_order() {
    assert_trace(
        r#"
var original = eval, marker = {}, trace = [], object = {};
var holder = { get invoke() { trace.push('get'); return original; } };
function first() { trace.push('first'); return object; }
function extra() { trace.push('extra'); original = function(value) { trace.push('replacement'); return value; }; }
print(holder.invoke(first(), extra()) === object);
print(holder.invoke(object) === object);
function fail() { trace.push('throw'); throw marker; }
try { holder.invoke(first(), fail()); } catch (error) { print(error === marker); }
print(trace.join(','));
"#,
        HostSurfacePolicy::Product,
        &[
            "true",
            "true",
            "true",
            "get,first,extra,get,replacement,get,first,throw",
        ],
    );
}

#[test]
fn foreign_eval_and_substituted_callables_keep_the_actual_invocation() {
    assert_trace(
        r#"
var realm = __lilaCreateRealm(), foreignEval = realm.global.eval;
var value = new String('globalThis.changed = true');
print(foreignEval(value) === value);
print(realm.global.changed === undefined);
var target = foreignEval;
print(target(value, (target = function(source) { return source; })) === value);
print(target('ordinary source') === 'ordinary source');
"#,
        HostSurfacePolicy::Test262,
        &["true", "true", "true", "true"],
    );
}

#[test]
fn unprepared_primitive_strings_reject_the_runtime_capability_outside_javascript() {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let error = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"
globalThis.unknownSource = 'throw new Error("must not execute"); /*' + Math.random() + '*/';
try { eval.call(undefined, globalThis.unknownSource); }
catch (error) { throw new Error('must not catch capability rejection'); }
throw new Error('must not continue');
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect_err("unprepared source is a runtime AOT capability rejection");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval],
        "{error}"
    );
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource),
        "{error}"
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
    assert_eq!(
        error.wasm_javascript_exception_constructor_name(),
        None,
        "{error}"
    );
}
