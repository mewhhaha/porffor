use lila_engine::{
    CompileOptions, Engine, EngineError, ExecutionBackend, HostSurfacePolicy, RealmBuilder,
    RunOptions, WasmExecutionFailureKind,
};
use lila_ir::{DynamicFunctionKind, DynamicSourceRuntimeOperation};

fn assert_runtime_capability_rejection(
    intrinsic: &str,
    source: &str,
    expected: DynamicSourceRuntimeOperation,
) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let script = format!(
        "var holder = {{ invoke: {intrinsic} }}; \
         var hook = new Proxy(function() {{}}, {{}}); hook(); \
         try {{ holder.invoke({source:?} + '/*' + Math.random() + '*/'); }} catch (error) {{ print('caught'); }} \
         print('continued');"
    );
    let error = Engine::new(RealmBuilder::new().build())
        .run_script(
            &script,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("a dynamic source capability rejection cannot be caught by JavaScript");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![expected],
        "{error}"
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
}

#[test]
fn late_eval_intrinsic_rejections_keep_the_runtime_operation() {
    assert_runtime_capability_rejection("eval", "1", DynamicSourceRuntimeOperation::Eval);
    assert_runtime_capability_rejection(
        "__lilaRealmEvalScript",
        "1",
        DynamicSourceRuntimeOperation::RealmEvalScript,
    );
}

#[test]
fn late_dynamic_constructors_keep_each_runtime_operation() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for (intrinsic, body, kind) in [
        ("Function", "return value;", DynamicFunctionKind::Ordinary),
        (
            "Object.getPrototypeOf(function*(){}).constructor",
            "yield value;",
            DynamicFunctionKind::Generator,
        ),
        (
            "Object.getPrototypeOf(async function(){}).constructor",
            "return value;",
            DynamicFunctionKind::Async,
        ),
        (
            "Object.getPrototypeOf(async function*(){}).constructor",
            "yield value;",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        let invoke = match kind {
            DynamicFunctionKind::Ordinary => "print(created(41)); print('continued');",
            DynamicFunctionKind::Generator => {
                "var result = created(41).next(); if (result.done) throw 'generator completed early'; print(result.value); print('continued');"
            }
            DynamicFunctionKind::Async => {
                "created(41).then(function(value) { print(value); print('continued'); });"
            }
            DynamicFunctionKind::AsyncGenerator => {
                "created(41).next().then(function(result) { if (result.done) throw 'async generator completed early'; print(result.value); print('continued'); });"
            }
        };
        for prefix in ["", "'use strict';"] {
            // The comment is constructed from runtime code units. Its exact
            // converted tuple must not be supplied by finite source discovery.
            let script = format!(
                r#"
{prefix}
var trace = '';
var holder = {{ invoke: {intrinsic} }};
var hook = new Proxy(function() {{}}, {{}}); hook();
var parameter = {{ get toString() {{ trace += 'parameter-get;'; return function() {{
  trace += 'parameter-convert;'; return String.fromCharCode(118,97,108,117,101);
}}; }} }};
var body = {{ get toString() {{ trace += 'body-get;'; return function() {{
  trace += 'body-convert;'; return {body:?} + String.fromCharCode(47,42,114,117,110,116,105,109,101,42,47);
}}; }} }};
var created = holder.invoke(parameter, body);
if (created.length !== 1 || Object.getPrototypeOf(created) !== holder.invoke.prototype)
  throw 'constructor protocol metadata';
print(trace);
{invoke}
"#
            );
            let engine = Engine::new(RealmBuilder::new().build());
            let options = CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            };
            let unit = engine
                .compile_script(&script, options.clone())
                .expect("outer source lowers");
            let ir = unit.ir.script.as_ref().expect("Script IR");
            assert!(ir.prepared_dynamic_functions.len() <= 256);
            let actual_body = format!("{body}/*runtime*/");
            assert!(
                !ir.prepared_dynamic_functions
                    .iter()
                    .any(|entry| entry.kind == kind
                        && entry
                            .arguments
                            .iter()
                            .map(String::as_str)
                            .eq(["value", actual_body.as_str()])),
                "each actual constructor kind must reach the native source miss"
            );
            let outcome = engine
                .observe_script(
                    &script,
                    options,
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        timeout_ms: Some(30_000),
                        ..RunOptions::default()
                    },
                )
                .expect("late Function-family sources execute through the native compiler");
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            assert!(
                matches!(
                    outcome.completion,
                    lila_engine::ObservedCompletion::Normal(_)
                ),
                "{:?}",
                outcome.completion
            );
            assert_eq!(
                outcome.output_events,
                [
                    "parameter-get;parameter-convert;body-get;body-convert;",
                    "41",
                    "continued"
                ]
                .map(|line| lila_engine::HostOutputEvent::PrintLine(line.to_owned()))
                .to_vec()
            );
        }
    }
}

#[test]
fn late_non_string_eval_and_overwritten_callables_keep_ordinary_semantics() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"
var holder = { invoke: eval }, marker = {}, boxed = new String('1');
var hook = new Proxy(function() {}, {}); hook();
var ok = holder.invoke() === undefined && holder.invoke(17) === 17 &&
  holder.invoke(marker) === marker && holder.invoke(boxed) === boxed;
holder.invoke = function(source) { return source + '!'; };
hook();
ok && holder.invoke('ordinary') === 'ordinary!';
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("ordinary callbacks and non-string eval values must execute normally");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn definitely_deleted_eval_throws_reference_error_before_evaluating_arguments() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"
var calls = 0;
var referenceErrorBeforeArguments = false;
delete eval;
try {
  eval((calls++, 'source'));
} catch (error) {
  referenceErrorBeforeArguments = error instanceof ReferenceError && calls === 0;
}
referenceErrorBeforeArguments;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("deleted eval is an ordinary unresolvable identifier");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn agent_runtime_capability_failure_keeps_its_typed_reason() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let worker = "var holder = { invoke: eval }; \
                  var hook = new Proxy(function() {}, {}); hook(); \
                  try { holder.invoke('1/*' + Math.random() + '*/'); } catch (error) {}";
    let error = Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            &format!("__lilaAgentStart({worker:?});"),
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            Some(30_000),
            true,
            String::new(),
        )
        .expect_err("the owner must retain a worker's dynamic source capability failure");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval],
        "{error}"
    );
    assert!(error.ir_diagnostic().is_none(), "{error}");
}

fn agent_script_error(source: &str) -> EngineError {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            // Cold worker compilation runs inside Agent.start's root execution deadline.
            Some(120_000),
            true,
            String::new(),
        )
        .expect_err("root and worker failures must remain observable")
}

const RUNTIME_DIRECT_EVAL: &str = "eval('1/*' + Math.random() + '*/');";
const COMPILE_UNSUPPORTED_REALM_SCRIPT: &str = "__lilaRealmEvalScript(globalThis.unknownSource);";
const RUNTIME_INDIRECT_EVAL: &str = "globalThis.unknownSource = '1/*' + Math.random() + '*/'; \
     eval.call(undefined, globalThis.unknownSource);";

const EVAL_WORKER: &str = "var holder = { invoke: eval }; \
    var hook = new Proxy(function() {}, {}); hook(); holder.invoke('1/*' + Math.random() + '*/');";

#[test]
fn direct_eval_worker_reports_runtime_source_rejection_after_compilation() {
    let error = agent_script_error(&format!("__lilaAgentStart({RUNTIME_DIRECT_EVAL:?});"));
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval],
        "{error}"
    );
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource)
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
}

#[test]
fn indirect_eval_worker_reports_runtime_source_rejection_after_compilation() {
    let error = agent_script_error(&format!("__lilaAgentStart({RUNTIME_INDIRECT_EVAL:?});"));
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

#[test]
fn multiple_workers_retain_distinct_runtime_capability_reasons() {
    let function_worker = "var holder = { invoke: Function }; \
        var hook = new Proxy(function() {}, {}); hook(); holder.invoke('return 1/*' + Math.random() + '*/');";
    let error = agent_script_error(&format!(
        "__lilaAgentStart({EVAL_WORKER:?}); __lilaAgentStart({function_worker:?});"
    ));
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![
            DynamicSourceRuntimeOperation::Eval,
            DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Ordinary),
        ],
        "{error}"
    );
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource)
    );
}

#[test]
fn worker_compile_and_runtime_capabilities_remain_unsupported_together() {
    let error = agent_script_error(&format!(
        "__lilaAgentStart({EVAL_WORKER:?}); __lilaAgentStart({COMPILE_UNSUPPORTED_REALM_SCRIPT:?});"
    ));
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource),
        "{error}"
    );
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval],
        "{error}"
    );
    assert_eq!(error.wasm_javascript_exception_constructor_name(), None);
    assert!(
        error
            .message()
            .contains("failed to compile or start Test262 agent"),
        "{error}"
    );
    let compile_gap = lila_ir::IrDiagnostic::unsupported_dynamic_source(
        lila_ir::DynamicSourceGap::runtime_source(lila_ir::DynamicSourceKind::RealmEvalScript),
    );
    assert!(error.message().contains(&compile_gap.message), "{error}");
    assert!(
        error
            .message()
            .contains(&DynamicSourceRuntimeOperation::Eval.to_string()),
        "{error}"
    );
}

#[test]
fn root_js_exception_and_worker_capability_keep_both_failures() {
    let error = agent_script_error(&format!(
        "__lilaAgentStart({EVAL_WORKER:?}); throw new TypeError('root marker');"
    ));
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::ConcurrentFailure)
    );
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
    assert!(error.message().contains("root marker"), "{error}");
    assert!(error.message().contains("dynamic-source"), "{error}");
}

#[test]
fn mixed_worker_exceptions_and_capabilities_are_not_root_js_exceptions() {
    let error = agent_script_error(&format!(
        "__lilaAgentStart(\"throw new TypeError('worker marker');\"); __lilaAgentStart({EVAL_WORKER:?});"
    ));
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::ConcurrentFailure),
        "{error}"
    );
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
    assert!(error.message().contains("worker marker"), "{error}");
}

#[test]
fn agent_start_retains_compile_diagnostics_before_any_worker_execution() {
    for (worker, has_parse_diagnostic, has_ir_diagnostic) in [
        ("function {", true, false),
        (COMPILE_UNSUPPORTED_REALM_SCRIPT, false, true),
    ] {
        let error = agent_script_error(&format!("__lilaAgentStart({worker:?});"));
        assert_eq!(
            error.parse_diagnostic().is_some(),
            has_parse_diagnostic,
            "{error}"
        );
        assert_eq!(
            error.ir_diagnostic().is_some(),
            has_ir_diagnostic,
            "{error}"
        );
        assert_eq!(error.wasm_execution_failure_kind(), None, "{error}");
        assert!(
            error.runtime_dynamic_source_operations().is_empty(),
            "{error}"
        );
    }
}
