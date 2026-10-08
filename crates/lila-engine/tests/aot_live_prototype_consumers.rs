use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("prototype consumers compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{observed:?}"
        );
    }
}

#[test]
fn inherited_descriptor_getters_keep_effects_and_own_field_shadowing() {
    assert_modes(
        r#"
        let state = { value: 1 };
        let reads = 0;
        Object.defineProperty(Object.prototype, 'enumerable', {
            __proto__: null,
            configurable: true,
            get() { reads++; state.value = 'inherited'; return true; }
        });
        state = { value: 1 };
        const target = {};
        Object.defineProperty(target, 'first', { value: 7 });
        if (state.value + '!' !== 'inherited!' || reads !== 1) throw 'Object descriptor effects';
        state = { value: 1 };
        Reflect.defineProperty(target, 'second', { value: 11 });
        if (state.value + '!' !== 'inherited!' || reads !== 2) throw 'Reflect descriptor effects';
        state = { value: 1 };
        Object.defineProperty(target, 'own', { value: 13, enumerable: false });
        if (state.value !== 1 || reads !== 2) throw 'own field must shadow inherited getter';
        if (Object.keys(target).join(',') !== 'first,second') throw 'inherited enumerable values';
        delete Object.prototype.enumerable;
        const marker = {};
        Object.defineProperty(Object.prototype, 'configurable', {
            __proto__: null, configurable: true, get() { throw marker; }
        });
        let caught;
        try { Object.defineProperty(target, 'abrupt', { value: 17 }); }
        catch (error) { caught = error; }
        delete Object.prototype.configurable;
        if (caught !== marker || Object.hasOwn(target, 'abrupt')) throw 'inherited throw';
        262;
        "#,
    );
}

#[test]
fn bound_constructor_and_class_heritage_observe_live_inherited_prototype() {
    assert_modes(
        r#"
        function Base() { this.own = 7; }
        const Bound = Base.bind(null);
        const inherited = { method() { return 23; } };
        let state = { value: 1 };
        let reads = 0;
        Object.defineProperty(Function.prototype, 'prototype', {
            configurable: true,
            get() { reads++; state.value = 'live'; return inherited; }
        });
        state = { value: 1 };
        class Derived extends Bound {
            invoke() { return super.method(); }
        }
        if (state.value + '!' !== 'live!' || reads !== 1) throw 'heritage getter effects';
        const derived = new Derived();
        if (derived.own !== 7 || derived.invoke() !== 23) throw 'derived prototype method';
        if (Object.getPrototypeOf(Derived.prototype) !== inherited) throw 'heritage prototype';
        state = { value: 1 };
        const reflected = Reflect.construct(Base, [], Bound);
        if (state.value + '!' !== 'live!' || reads !== 2) throw 'newTarget getter effects';
        if (Object.getPrototypeOf(reflected) !== inherited || reflected.method() !== 23) {
            throw 'newTarget prototype';
        }
        const ordinary = new Bound();
        if (ordinary.own !== 7 || Object.getPrototypeOf(ordinary) !== Base.prototype) {
            throw 'bound constructor target prototype';
        }
        delete Function.prototype.prototype;
        262;
        "#,
    );
}

#[test]
fn unrelated_global_definition_keeps_actual_realm_wrapper_execution() {
    assert_modes(
        r#"
        function makeRealm() { return __lilaCreateRealm(); }
        Object.defineProperty(globalThis, 'unrelated', { value: 0 });
        if (makeRealm().evalScript('17;') !== 17) throw 'runtime realm wrapper';
        262;
        "#,
    );
}
