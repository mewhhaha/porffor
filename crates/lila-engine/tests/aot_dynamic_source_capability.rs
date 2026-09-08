use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
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
        error.runtime_dynamic_source_operation(),
        Some(expected),
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
        error.runtime_dynamic_source_operation(),
        Some(DynamicSourceRuntimeOperation::Eval),
        "{error}"
    );
    assert!(error.ir_diagnostic().is_none(), "{error}");
}
