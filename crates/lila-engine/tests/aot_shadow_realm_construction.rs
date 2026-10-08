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
            .unwrap_or_else(|error| {
                panic!("ShadowRealm constructor controls failed: {error}\n{source}")
            });
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
fn construction_requires_new_and_subclasses_keep_native_brand_and_isolated_globals() {
    assert_modes(
        r#"
        try { ShadowRealm(); throw 'call-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        class Child extends ShadowRealm {}
        const first = new Child();
        const second = new ShadowRealm();
        if (Object.getPrototypeOf(first) !== Child.prototype) throw 'subclass-prototype';
        if (!(first instanceof ShadowRealm)) throw 'subclass-instance';
        if (first.evaluate('globalThis.marker = 37; marker') !== 37) throw 'subclass-brand';
        if (second.evaluate('typeof marker') !== 'undefined') throw 'shared-inner-global';
        if (typeof marker !== 'undefined') throw 'caller-global-leak';
        const namespaces = `
          Reflect.get({ value: 17 }, 'value') === 17 && Math.sqrt(81) === 9 &&
          JSON.parse('{"value":19}').value === 19 &&
          Atomics.load(new Int32Array(new SharedArrayBuffer(4)), 0) === 0 &&
          new Temporal.PlainDate(2000, 1, 2).year === 2000 &&
          Intl.getCanonicalLocales('en-us')[0] === 'en-US' &&
          [Reflect, Math, JSON, Atomics, Temporal, Intl].every(
            namespace => Object.getPrototypeOf(namespace) === Object.prototype
          );
        `;
        if (__lilaRealmEvalScript(namespaces) !== true) throw 'entry-namespace-products';
        if (first.evaluate(namespaces) !== true || second.evaluate(namespaces) !== true)
          throw 'shadow-namespace-products';
        const originalMath = Math;
        globalThis.Math = { sqrt() { throw 'entry-Math-copied'; } };
        const hostRealm = __lilaCreateRealm();
        globalThis.Math = originalMath;
        if (hostRealm.evalScript(namespaces) !== true) throw 'host-namespace-products';
        for (const name of ['Reflect', 'Math', 'JSON', 'Atomics', 'Temporal', 'Intl']) {
          if (hostRealm.global[name] === globalThis[name]) throw 'shared-namespace-object';
        }
        true;
        "#,
    );
}

#[test]
fn new_target_prototype_is_acquired_once_and_original_abrupt_value_is_preserved() {
    assert_modes(
        r#"
        let reads = 0;
        const prototype = {};
        const target = new Proxy(function () { throw 'new-target-body'; }, {
            get(object, key, receiver) {
                if (key === 'prototype') { reads++; return prototype; }
                return Reflect.get(object, key, receiver);
            }
        });
        const realm = Reflect.construct(ShadowRealm, [], target);
        if (reads !== 1 || Object.getPrototypeOf(realm) !== prototype) throw 'prototype-get';
        if (ShadowRealm.prototype.evaluate.call(realm, '2 + 3') !== 5) throw 'custom-prototype-brand';
        const abrupt = Symbol('prototype');
        const throwing = new Proxy(function () {}, {
            get(object, key, receiver) {
                if (key === 'prototype') { reads++; throw abrupt; }
                return Reflect.get(object, key, receiver);
            }
        });
        try { Reflect.construct(ShadowRealm, [], throwing); throw 'prototype-throw-lost'; }
        catch (error) { if (error !== abrupt) throw error; }
        if (reads !== 2) throw 'repeated-prototype-get';
        true;
        "#,
    );
}

#[test]
fn foreign_new_target_fallback_uses_its_prototype_realm_and_allocates_an_independent_inner_realm() {
    assert_modes(
        r#"
        const foreign = __lilaCreateRealm();
        const target = foreign.evalScript('(function Target() {})');
        target.prototype = 1;
        foreign.global.marker = 99;
        const realm = Reflect.construct(ShadowRealm, [], target);
        if (Object.getPrototypeOf(realm) !== foreign.global.ShadowRealm.prototype) {
            throw 'fallback-realm';
        }
        if (realm.evaluate('typeof marker') !== 'undefined') throw 'fallback-used-as-inner-realm';
        try { realm.evaluate(1); throw 'non-string-accepted'; }
        catch (error) {
            if (Object.getPrototypeOf(error) !== foreign.global.TypeError.prototype) throw error;
        }
        try { realm.evaluate('let = ;'); throw 'syntax-accepted'; }
        catch (error) {
            if (Object.getPrototypeOf(error) !== foreign.global.SyntaxError.prototype) throw error;
        }
        true;
        "#,
    );
}

#[test]
fn native_brand_validation_rejects_prototype_impostors_and_proxies_without_traps() {
    assert_modes(
        r#"
        const evaluate = ShadowRealm.prototype.evaluate;
        const fake = Object.create(ShadowRealm.prototype);
        try { evaluate.call(fake, '1'); throw 'prototype-impostor-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        let traps = 0;
        const proxy = new Proxy(new ShadowRealm(), {
            get() { traps++; throw 'get-trap'; },
            getPrototypeOf() { traps++; throw 'prototype-trap'; }
        });
        try { evaluate.call(proxy, '1'); throw 'proxy-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        if (traps !== 0) throw 'brand-check-used-proxy-traps';
        true;
        "#,
    );
}
