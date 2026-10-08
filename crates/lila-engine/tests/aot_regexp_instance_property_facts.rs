use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("RegExp property controls failed: {error}\n{source}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(observed.output_events.is_empty(), "{source}");
    }
}

#[test]
fn new_regexp_instances_observe_replaced_inherited_getters_and_keep_own_properties() {
    assert_modes(
        r#"
        let state;
        Object.defineProperty(RegExp.prototype, 'source', {
            configurable: true, get() { state.value = 'source'; return 41; }
        });
        Object.defineProperty(RegExp.prototype, 'global', {
            configurable: true, get() { state.value = 'global'; return 42; }
        });
        Object.defineProperty(RegExp.prototype, 'flags', {
            configurable: true, get() { state.value = 'flags'; return 43; }
        });

        const sourceReceiver = /x/g;
        state = { value: 1 };
        const source = sourceReceiver.source;
        if (source !== 41 || typeof state.value !== 'string') throw 'source';

        const globalReceiver = /x/g;
        state = { value: 1 };
        const global = globalReceiver.global;
        if (global !== 42 || typeof state.value !== 'string') throw 'global';

        const flagsReceiver = new RegExp('x', 'g');
        state = { value: 1 };
        const flags = flagsReceiver.flags;
        if (flags !== 43 || typeof state.value !== 'string') throw 'flags';
        if (!Object.prototype.hasOwnProperty.call(flagsReceiver, 'lastIndex')) throw 'own-index';
        if (Object.prototype.hasOwnProperty.call(flagsReceiver, 'flags')) throw 'inherited-flags';
        flagsReceiver.lastIndex = 'own';
        if (flagsReceiver.lastIndex !== 'own') throw 'mutable-own-index';
        Object.defineProperty(flagsReceiver, 'source', { value: 51 });
        flagsReceiver.exec = function () { return 52; };
        flagsReceiver[Symbol.match] = function () { return 53; };
        if (flagsReceiver.source !== 51 || flagsReceiver.exec() !== 52) throw 'own-name';
        if (flagsReceiver[Symbol.match]('x') !== 53) throw 'own-symbol';
        true;
        "#,
    );
}

#[test]
fn ordinary_regexp_call_can_return_array_and_callable_pattern_objects() {
    assert_modes(
        r#"
        function callable() { return 61; }
        callable[Symbol.match] = true;
        callable.constructor = RegExp;
        const sameCallable = RegExp(callable);
        if (sameCallable !== callable || typeof sameCallable !== 'function') throw 'callable-kind';
        if (sameCallable() !== 61) throw 'callable-result';

        const array = [71];
        array[Symbol.match] = true;
        array.constructor = RegExp;
        const sameArray = RegExp(array);
        if (sameArray !== array || !Array.isArray(sameArray)) throw 'array-kind';
        if (sameArray.length !== 1 || sameArray[0] !== 71) throw 'array-result';

        const existing = /x/;
        existing.lastIndex = 'retained';
        existing.exec = function () { return 81; };
        const sameRegExp = RegExp(existing);
        if (sameRegExp !== existing || sameRegExp.lastIndex !== 'retained') throw 'regexp-reuse';
        if (sameRegExp.exec() !== 81) throw 'retained-own-method';
        true;
        "#,
    );
}

#[test]
fn returned_callable_patterns_keep_argument_and_method_receiver_observation() {
    assert_modes(
        r#"
        function callable(value) { return this.prefix + value; }
        const first = { prefix: 1, invoke: callable };
        if (first.invoke(2) !== 3) throw 'initial numeric observation';
        callable[Symbol.match] = true;
        callable.constructor = RegExp;
        const second = { prefix: 'kept:', invoke: RegExp(callable) };
        if (second.invoke !== callable) throw 'callable identity';
        if (second.invoke('value') !== 'kept:value') throw 'later argument and receiver';
        true;
        "#,
    );
}

#[test]
fn returned_function_constructor_prepares_its_original_finite_source() {
    assert_modes(
        r#"
        const pattern = Function;
        pattern[Symbol.match] = true;
        pattern.constructor = RegExp;
        const constructor = RegExp(pattern);
        if (constructor !== pattern) throw 'constructor identity';
        const compiled = constructor('value', 'return value + 2;');
        if (compiled(40) !== 42) throw 'prepared function result';
        true;
        "#,
    );
}
