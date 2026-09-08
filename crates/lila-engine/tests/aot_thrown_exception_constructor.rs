use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, RealmBuilder, RunOptions,
    WasmExecutionFailureKind,
};

#[test]
fn root_exception_constructor_metadata_uses_the_final_value_without_calling_user_code() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for (source, expected) in [
        (
            "try { throw new RangeError('primary'); } finally { \
             try { throw new TypeError('caught'); } catch (error) {} }",
            Some("RangeError"),
        ),
        (
            "Promise.resolve().then(function() { \
             try { throw new TypeError('caught job'); } catch (error) {} }); \
             throw new RangeError('primary');",
            Some("RangeError"),
        ),
        (
            "var error = new TypeError('primary'); \
             Object.defineProperty(error, 'constructor', { get: function() { \
               print('constructor getter ran'); return TypeError; \
             }}); throw error;",
            None,
        ),
        (
            "function ErrorConstructor() {} \
             Object.defineProperty(ErrorConstructor, 'name', { get: function() { \
               print('name getter ran'); return 'TypeError'; \
             }}); throw { constructor: ErrorConstructor };",
            None,
        ),
        (
            "throw new Proxy(new TypeError('primary'), { get: function() { \
               print('proxy trap ran'); return TypeError; \
             }});",
            None,
        ),
        (
            "throw Object.create(new Proxy(new TypeError('primary'), { \
               get: function() { print('prototype proxy trap ran'); return TypeError; } \
             }));",
            None,
        ),
        (
            "var error = []; error.constructor = RangeError; \
             Object.setPrototypeOf(error, { constructor: TypeError }); throw error;",
            Some("RangeError"),
        ),
        (
            "function fail(value) { arguments.constructor = RangeError; \
             Object.setPrototypeOf(arguments, { constructor: TypeError }); \
             throw arguments; } fail(1);",
            Some("RangeError"),
        ),
        (
            "var constructor = [1, 2]; constructor.name = 'RangeError'; \
             Object.setPrototypeOf(constructor, { name: 'TypeError' }); \
             throw { constructor: constructor };",
            Some("RangeError"),
        ),
        (
            "function fail(value) { 'use strict'; arguments.name = 'RangeError'; \
             Object.setPrototypeOf(arguments, { name: 'TypeError' }); \
             throw { constructor: arguments }; } fail(1);",
            Some("RangeError"),
        ),
        (
            "var error = [1]; \
             Object.defineProperty(error, 'constructor', { get: function() { \
               print('array constructor getter ran'); return RangeError; \
             }}); Object.setPrototypeOf(error, { constructor: TypeError }); throw error;",
            None,
        ),
        (
            "function fail(value) { \
             Object.defineProperty(arguments, 'name', { get: function() { \
               print('arguments name getter ran'); return 'RangeError'; \
             }}); Object.setPrototypeOf(arguments, { name: 'TypeError' }); \
             throw { constructor: arguments }; } fail(1);",
            None,
        ),
    ] {
        let engine = Engine::new(RealmBuilder::new().build());
        let options = RunOptions {
            backend: ExecutionBackend::WasmAot,
            timeout_ms: Some(30_000),
            ..RunOptions::default()
        };
        let failure = engine
            .run_script(source, CompileOptions::default(), options.clone())
            .expect_err("the final root throw must reach the engine");
        assert_eq!(
            failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException),
            "{source}: {failure}"
        );
        assert_eq!(
            failure.wasm_javascript_exception_constructor_name(),
            expected,
            "{source}: {failure}"
        );

        let observation = engine
            .observe_script(source, CompileOptions::default(), options)
            .expect("the same emitted artifact must expose the throw structurally");
        assert!(matches!(
            observation.completion,
            ObservedCompletion::Throw(_)
        ));
        assert!(
            observation.output_events.is_empty(),
            "exception metadata must not invoke getters or traps: {source}: {:?}",
            observation.output_events
        );
    }
}
