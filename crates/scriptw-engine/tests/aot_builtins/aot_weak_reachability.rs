use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run(source: &str) -> Vec<HostOutputEvent> {
    scriptw_engine::configure_compilation_jobs(1).unwrap();
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .unwrap();
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    outcome.output_events
}

#[test]
fn weak_storage_preserves_undefined_presence_symbols_and_callback_mutation() {
    run(r#"
        const key = {}, symbol = Symbol('key'); const map = new WeakMap([[key, undefined], [symbol, 3]]);
        if (!map.has(key) || map.get(key) !== undefined || map.get(symbol) !== 3) throw 'presence';
        let calls = 0;
        if (map.getOrInsertComputed(key, () => { calls++; }) !== undefined || calls) throw 'present callback';
        try { map.getOrInsertComputed(key, null); throw 'callable validation'; } catch (e) { if (!(e instanceof TypeError)) throw e; }
        const other = {};
        if (map.getOrInsertComputed(other, function(k) { if (k !== other) throw 'callback arg'; map.set(k, 7); return 9; }) !== 9 || map.get(other) !== 9) throw 'computed overwrite';
        const set = new WeakSet([symbol, Symbol.iterator, key]);
        if (!set.has(symbol) || !set.delete(key) || set.has(key)) throw 'weak set';
        for (const invalid of [null, 1, 'x', Symbol.for('registered')]) {
            if (map.has(invalid) || map.delete(invalid) || map.get(invalid) !== undefined || set.has(invalid)) throw 'invalid read';
        }
    "#);
}

#[test]
fn weak_ref_keeps_targets_through_construction_and_successful_deref_jobs() {
    let events = run(r#"
        let weak;
        (function() { weak = new WeakRef({marker: 19}); })();
        gc(); if (weak.deref().marker !== 19) throw 'constructor keep';
        Promise.resolve().then(function() {
            gc(); if (weak.deref() !== undefined) throw 'job clear';
            (function() { const target = {marker: 42}; weak = new WeakRef(target); })();
            gc(); if (weak.deref().marker !== 42) throw 'job constructor keep';
            gc(); if (weak.deref().marker !== 42) throw 'deref keep';
        }).then(function() { gc(); if (weak.deref() !== undefined) throw 'next job clear'; print('kept roots:ok'); });
    "#);
    assert_eq!(
        events,
        vec![HostOutputEvent::PrintLine("kept roots:ok".into())]
    );
}

#[test]
fn ephemeron_chains_survive_and_key_value_cycles_are_reclaimed() {
    let events = run(r#"
        const map = new WeakMap(); let live, last, cycle;
        (function() {
            const a = {}, b = {}, c = {marker: 31}; live = a; last = new WeakRef(c);
            map.set(b, c); map.set(a, b);
            const key = {}; map.set(key, {key}); cycle = new WeakRef(key);
            if (map.get(key).key !== key) throw 'cycle storage';
        })();
        Promise.resolve().then(function() {
            gc(); if (last.deref().marker !== 31 || cycle.deref() !== undefined) throw 'fixed point or cycle'; live = undefined;
        }).then(function() { gc(); if (last.deref() !== undefined) throw 'dead ephemeron chain'; print('ephemerons:ok'); });
    "#);
    assert_eq!(
        events,
        vec![HostOutputEvent::PrintLine("ephemerons:ok".into())]
    );
}

#[test]
fn an_unreachable_collection_does_not_retain_values_for_live_keys() {
    let events = run(r#"
        const key = {}; let value;
        (function() { const map = new WeakMap(); const target = {}; map.set(key, target); value = new WeakRef(target); })();
        Promise.resolve().then(function() { gc(); if (value.deref() !== undefined) throw 'dead collection owner'; print('dead owner:ok'); });
    "#);
    assert_eq!(
        events,
        vec![HostOutputEvent::PrintLine("dead owner:ok".into())]
    );
}

#[test]
fn cleanup_is_a_job_and_can_unregister_remaining_cleared_cells() {
    let events = run(r#"
        const token = Symbol('unregister'); let calls = 0;
        const registry = new FinalizationRegistry(function(holding) { 'use strict';
            if (this !== undefined || holding !== 2) throw 'cleanup arguments'; calls++;
            if (!registry.unregister(token)) throw 'unregister cleared cell'; print('cleanup:2');
        });
        (function() { registry.register({}, 1, token); registry.register({}, 2); })();
        gc(); if (calls) throw 'cleanup ran inside job';
        Promise.resolve().then(function() { if (calls) throw 'cleanup jumped queue'; print('promise'); });
    "#);
    assert_eq!(
        events,
        vec![
            HostOutputEvent::PrintLine("promise".into()),
            HostOutputEvent::PrintLine("cleanup:2".into())
        ]
    );
}

#[test]
fn cleanup_throw_is_reported_and_remaining_cells_wait_for_another_job() {
    let events = run(r#"
        const registry = new FinalizationRegistry(function(holding) {
            if (holding === 2) { Promise.resolve().then(() => print('between')); throw new Error('cleanup marker'); }
            print('cleanup:1');
        });
        (function() { registry.register({}, 1); registry.register({}, 2); })(); gc();
    "#);
    assert_eq!(events.len(), 3, "{events:?}");
    assert!(
        matches!(&events[0], HostOutputEvent::ReportedException(message) if message.contains("cleanup marker")),
        "{events:?}"
    );
    assert_eq!(
        events[1..],
        [
            HostOutputEvent::PrintLine("between".into()),
            HostOutputEvent::PrintLine("cleanup:1".into())
        ]
    );
}

#[test]
fn unregister_after_target_collection_prevents_cleanup() {
    let events = run(r#"
        const token = {}; const registry = new FinalizationRegistry(() => { throw 'unexpected cleanup'; });
        let weak;
        (function() { const target = {}; registry.register(target, 42, token); weak = new WeakRef(target); })();
        Promise.resolve().then(function() {
            gc(); if (weak.deref() !== undefined) throw 'target not collected';
            if (!registry.unregister(token) || registry.unregister(token)) throw 'unregister';
            print('unregister:ok');
        });
    "#);
    assert_eq!(
        events,
        vec![HostOutputEvent::PrintLine("unregister:ok".into())]
    );
}

#[test]
fn pending_reaction_closures_keep_cycles_until_settlement_and_release_them_afterward() {
    for directive in ["", "\"use strict\";\n"] {
        let events = run(&format!(
            r#"{directive}
                let weak, settle;
                (function() {{
                    const target = {{marker: 41, text: 'x\ud800\udfff'}};
                    target.self = target;
                    weak = new WeakRef(target);
                    const pending = new Promise(function(resolve) {{ settle = resolve; }});
                    pending.then(function(value) {{
                        gc();
                        if (value !== 19 || target.marker !== 41 || target.self !== target ||
                            target.text.length !== 3 || target.text.charCodeAt(1) !== 0xd800 ||
                            target.text.charCodeAt(2) !== 0xdfff) throw 'reaction roots';
                        print('reaction:ok');
                    }});
                }})();
                Promise.resolve().then(function() {{
                    gc();
                    const target = weak.deref();
                    if (target === undefined || target.marker !== 41 || target.self !== target)
                        throw 'pending reaction lost its cycle';
                    settle(19);
                    settle = undefined;
                }}).then(function() {{
                    gc();
                    if (weak.deref() !== undefined) throw 'settled reaction retained its cycle';
                    print('released:ok');
                }});
            "#
        ));
        assert_eq!(
            events,
            vec![
                HostOutputEvent::PrintLine("reaction:ok".into()),
                HostOutputEvent::PrintLine("released:ok".into()),
            ]
        );
    }
}

#[test]
fn emitted_cyclic_churn_reuses_the_product_gc_heap() {
    scriptw_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            r#"{directive}
                const live = {{marker: 41}};
                live.self = live;
                const samples = [];
                function churn(round) {{
                    for (let index = 0; index < 512; index++) {{
                        const left = {{round, index}};
                        const right = {{left, marker: 73}};
                        left.right = right;
                        if (index === 0) samples.push(new WeakRef(left));
                        if (right.left !== left || left.right.marker !== 73)
                            throw 'cycle allocation';
                    }}
                }}
                for (let round = 0; round < 128; round++) {{
                    churn(round);
                    gc();
                    if (live.marker !== 41 || live.self !== live)
                        throw 'retained cycle';
                }}
                Promise.resolve().then(function() {{
                    gc();
                    for (const sample of samples) {{
                        if (sample.deref() !== undefined) throw 'released cycle retained';
                    }}
                    if (live.self !== live) throw 'live cycle lost';
                    print('churn:128:512:ok');
                }});
                void 0;
            "#
        );
        let engine = Engine::new(RealmBuilder::new().build());
        let compilation = engine
            .profile_script_compilation(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
            )
            .unwrap();
        let profile = engine
            .profile_wasm_execution(&compilation.artifact, Some(30_000), false)
            .unwrap();
        assert_eq!(profile.outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(profile.outcome.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            profile.outcome.completion
        );
        assert_eq!(
            profile.outcome.output_events,
            vec![HostOutputEvent::PrintLine("churn:128:512:ok".into())]
        );
        // These are reserved-capacity snapshots, not allocation/live-byte or
        // pause counters. The original product configuration is used intact.
        assert!(profile.heap.capacity_before_execution_bytes <= 16 * 1024 * 1024);
        assert!(
            profile.heap.capacity_after_execution_bytes <= 16 * 1024 * 1024,
            "cyclic churn grew the product GC heap: {:?}",
            profile.heap
        );
    }
}

#[test]
fn escaped_closure_roots_foreign_cycles_until_release_in_a_later_job() {
    for directive in ["", "\"use strict\";\n"] {
        let events = run(&format!(
            r#"{directive}
                let keep, weak;
                (function() {{
                    const foreign = __scriptwCreateRealm().global;
                    const key = new foreign.Object();
                    const local = {{marker: 73, foreign: key}};
                    key.marker = 41;
                    key.local = local;
                    key.self = key;
                    weak = new WeakRef(key);
                    keep = function() {{
                        gc();
                        if (key.marker !== 41 || key.self !== key ||
                            key.local !== local || local.foreign !== key ||
                            local.marker !== 73 ||
                            Object.getPrototypeOf(key) !== foreign.Object.prototype ||
                            Object.getPrototypeOf(local) !== Object.prototype)
                            throw 'cross-realm cycle roots';
                        print('foreign:rooted');
                    }};
                }})();
                Promise.resolve().then(function() {{
                    gc();
                    if (weak.deref() === undefined) throw 'closure lost foreign cycle';
                    keep();
                    keep = undefined;
                }}).then(function() {{
                    gc();
                    if (weak.deref() !== undefined) throw 'released foreign cycle retained';
                    print('foreign:released');
                }});
            "#
        ));
        assert_eq!(
            events,
            vec![
                HostOutputEvent::PrintLine("foreign:rooted".into()),
                HostOutputEvent::PrintLine("foreign:released".into()),
            ]
        );
    }
}

#[test]
fn weak_collection_indexes_preserve_key_identity_after_movement_and_dead_key_churn() {
    let source = r#"
        const map = new WeakMap(), set = new WeakSet();
        const object = {}, symbol = Symbol('local'), fn = function() {}, proxy = new Proxy({}, {});
        const equalShape = {}, keys = [object, symbol, fn, proxy, equalShape];
        const values = keys.map(function(key, index) { return {key, index}; });
        for (let i = 0; i < keys.length; i++) { map.set(keys[i], values[i]); set.add(keys[i]); }
        for (let round = 0; round < 32; round++) {
            (function() {
                for (let i = 0; i < 256; i++) { const dead = {}; map.set(dead, {round, i}); set.add(dead); }
            })();
            gc();
            for (let i = 0; i < keys.length; i++) {
                if (!map.has(keys[i]) || map.get(keys[i]) !== values[i] || !set.has(keys[i])) throw 'moved key identity';
            }
            const old = values[round % keys.length], key = keys[round % keys.length];
            if (!map.delete(key) || map.delete(key) || map.has(key) || map.get(key) !== undefined) throw 'map deletion';
            if (!set.delete(key) || set.delete(key) || set.has(key)) throw 'set deletion';
            map.set(key, old); set.add(key);
        }
        const watches = (function() {
            const a = {}, b = {}, c = {};
            map.set(a, b); map.set(b, a); set.add(c);
            return [new WeakRef(a), new WeakRef(b), new WeakRef(c)];
        })();
        Promise.resolve().then(function() {
            gc();
            for (const weak of watches) if (weak.deref() !== undefined) throw 'index retained dead key';
            for (let i = 0; i < keys.length; i++) if (map.get(keys[i]) !== values[i] || !set.has(keys[i])) throw 'job key identity';
            print('weak-index:ok');
        });
    "#;
    for strict in [false, true] {
        let source = if strict {
            format!("'use strict';{source}")
        } else {
            source.into()
        };
        assert_eq!(
            run(&source),
            vec![HostOutputEvent::PrintLine("weak-index:ok".into())]
        );
    }
}
