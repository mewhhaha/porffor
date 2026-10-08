use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("iterator materialization compiles and executes through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{outcome:?}"
        );
    }
}

#[test]
fn array_from_observes_iterability_and_consumes_the_actual_iterator_once() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let iter = (function () {
            let n = g();
            return { [Symbol.iterator]: null, next: () => n.next() };
        })();
        if (Array.from(iter).length !== 0) throw 'array-like fallback';
        if (Array.from(Iterator.from(iter)).join(',') !== '0,1,2') throw 'first consumption';
        if (Iterator.from(iter).toArray().length !== 0) throw 'replayed consumed iterator';
        const existing = g();
        if (Iterator.from(existing) !== existing) throw 'existing iterator identity';
        true;
    "#,
    );
}

#[test]
fn iterator_materialization_keeps_own_methods_next_effects_and_abrupt_values() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let calls = 0;
        let iter = (function () {
            let n = g();
            return {
                [Symbol.iterator]: null,
                next: () => { calls++; return n.next(); },
                toArray() { return 'own method'; }
            };
        })();
        if (iter.toArray() !== 'own method' || calls !== 0) throw 'own method';
        if (Iterator.prototype.toArray.call(iter).join(',') !== '0,1,2' || calls !== 4) throw 'next effects';
        const marker = {};
        iter = (function () {
            let n = g();
            return {
                [Symbol.iterator]: null,
                next: () => { throw marker; return n.next(); }
            };
        })();
        let caught;
        try { Array.from(Iterator.from(iter)); } catch (error) { caught = error; }
        if (caught !== marker) throw 'next abrupt';
        true;
    "#,
    );
}

#[test]
fn materialization_uses_replaced_or_shadowed_constructors_and_fresh_generator_calls() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let iter = (function () {
            let n = g();
            return { [Symbol.iterator]: null, next: () => n.next() };
        })();
        {
            const Array = { from(value) { return value; } };
            if (Array.from(iter) !== iter) throw 'shadowed Array';
        }
        Array.from = function (value) { return value; };
        if (Array.from(iter) !== iter) throw 'replaced Array.from';
        Iterator.from = function (value) { return { toArray() { return value; } }; };
        if (Iterator.from(iter).toArray() !== iter) throw 'replaced Iterator.from';

        let entered = 0;
        var create = function* () { entered++; yield entered; };
        const first = create(), second = create();
        if (typeof create !== 'function' || first === second || entered !== 0) throw 'generator creation';
        if (first.next().value !== 1 || second.next().value !== 2 || entered !== 2) throw 'generator execution';
        true;
    "#,
    );
}
