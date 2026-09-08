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
         try {{ holder.invoke({source:?}); }} catch (error) {{ print('caught'); }} \
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
    for (intrinsic, source, kind) in [
        ("Function", "return 1", DynamicFunctionKind::Ordinary),
        (
            "Object.getPrototypeOf(function*(){}).constructor",
            "yield 1",
            DynamicFunctionKind::Generator,
        ),
        (
            "Object.getPrototypeOf(async function(){}).constructor",
            "return 1",
            DynamicFunctionKind::Async,
        ),
        (
            "Object.getPrototypeOf(async function*(){}).constructor",
            "yield 1",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        assert_runtime_capability_rejection(
            intrinsic,
            source,
            DynamicSourceRuntimeOperation::Function(kind),
        );
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
                  try { holder.invoke('1'); } catch (error) {}";
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
            Some(30_000),
            true,
            String::new(),
        )
        .expect_err("root and worker failures must remain observable")
}

const EVAL_WORKER: &str = "var holder = { invoke: eval }; \
    var hook = new Proxy(function() {}, {}); hook(); holder.invoke('1');";

#[test]
fn multiple_workers_retain_distinct_runtime_capability_reasons() {
    let function_worker = "var holder = { invoke: Function }; \
        var hook = new Proxy(function() {}, {}); hook(); holder.invoke('return 1');";
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
        Some(WasmExecutionFailureKind::ConcurrentFailure)
    );
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
    assert!(error.message().contains("worker marker"), "{error}");
}

#[test]
fn agent_start_retains_compile_diagnostics_before_any_worker_execution() {
    for (worker, has_parse_diagnostic, has_ir_diagnostic) in
        [("function {", true, false), ("eval('1');", false, true)]
    {
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
