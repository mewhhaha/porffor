use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run(source: &str) -> Vec<HostOutputEvent> {
    lila_engine::configure_compilation_jobs(1).unwrap();
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
