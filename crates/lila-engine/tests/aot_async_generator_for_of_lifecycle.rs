use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_iterator_lifecycle(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = if strict {
            format!("\"use strict\";\n{source}")
        } else {
            source.to_owned()
        };
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
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
            .expect("complete mixed iterator phases use the JavaScript to Wasm compiler");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).to_owned()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn mixed_iterator_queued_return_closes_nested_initializer_before_original_outer_record() {
    observe_iterator_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed iterator close'); }
        const whole = {marker: 95}, closeError = {marker: 96}; whole.self = whole;
        whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };
        async function run(protocol, fail) {
            const events = []; let nextReads = 0, nextCalls = 0;
            const inner = {[Symbol.iterator]: function () { return {
                next: function () { return {done: false, value: undefined}; },
                return: function () { events.push('inner'); return {}; }
            }; }};
            const iterator = {
                get next() { nextReads++; return function () {
                    check(this === iterator); nextCalls++;
                    return {done: false, value: inner};
                }; },
                get return() { events.push('get-outer'); return function () {
                    check(this === iterator); events.push('outer');
                    if (fail) throw closeError;
                    if (protocol === 'sync') return {get then() { throw 'sync-close-awaited'; }};
                    const closed = Promise.resolve().then(function () { events.push('close-awaited'); gc(); return {}; });
                    return protocol === 'fallback' ? {done: true, value: closed} : closed;
                }; }
            };
            const input = {[Symbol.iterator]: function () { return iterator; }};
            if (protocol === 'async') input[Symbol.asyncIterator] = function () { return iterator; };
            async function* syncValues() {
                try { for (const [value = await (yield 'default')] of input) { yield value; } }
                finally { await 0; gc(); yield 'finally'; }
            }
            async function* awaitedValues() {
                try { for await (const [value = await (yield 'default')] of input) { yield value; } }
                finally { await 0; gc(); yield 'finally'; }
            }
            const stream = protocol === 'sync' ? syncValues() : awaitedValues();
            check((await stream.next()).value === 'default');
            check(nextReads === 1 && nextCalls === 1 && events.length === 0);
            Object.defineProperty(iterator, 'next', {value: function () { throw 'next-reacquired'; }});
            gc();
            const returning = stream.return(whole);
            const queued = stream.next().then(function (result) { return {result: result}; },
                function (error) { return {error: error}; });
            const finalizer = await returning;
            check(!finalizer.done && finalizer.value === 'finally');
            check(events.join(',') === (fail || protocol === 'sync'
                ? 'inner,get-outer,outer' : 'inner,get-outer,outer,close-awaited'));
            const done = await queued;
            if (fail) check(done.error === closeError);
            else check(done.result.done && done.result.value === whole && done.result.value.self === whole);
            check((await stream.next()).done && nextReads === 1 && nextCalls === 1);
            print('mixed-iterator-' + protocol + (fail ? '-close-error' : '') + ':ok');
        }
        async function all() {
            await run('sync', false); await run('async', false);
            await run('fallback', false); await run('async', true);
        }
        all();
    "#,
        &[
            "mixed-iterator-sync:ok",
            "mixed-iterator-async:ok",
            "mixed-iterator-fallback:ok",
            "mixed-iterator-async-close-error:ok",
        ],
    );
}

#[test]
fn mixed_iterator_next_errors_do_not_close_but_initializer_rejection_preserves_whole_throw() {
    observe_iterator_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed iterator protocol'); }
        const whole = {marker: 97}, closeError = {marker: 98};
        async function run(protocol, mode) {
            let closes = 0, calls = 0;
            const iterator = {
                next: function () {
                    calls++;
                    if (mode === 'next') throw whole;
                    if (mode === 'reject') return Promise.reject(whole);
                    return {get done() { if (mode === 'done') throw whole; return false; },
                        get value() { if (mode === 'value') throw whole; return [undefined]; }};
                },
                return: function () { closes++; throw closeError; }
            };
            const input = {[Symbol.iterator]: function () { return iterator; }};
            if (protocol === 'async') input[Symbol.asyncIterator] = function () { return iterator; };
            async function* syncValues() {
                try { for (const [value = await (yield 'default')] of input) { yield value; } }
                finally { await 0; gc(); yield 'finally'; }
            }
            async function* awaitedValues() {
                try { for await (const [value = await (yield 'default')] of input) { yield value; } }
                finally { await 0; gc(); yield 'finally'; }
            }
            const stream = protocol === 'sync' ? syncValues() : awaitedValues();
            let first = await stream.next();
            if (mode === 'initializer') {
                check(first.value === 'default');
                first = await stream.next(Promise.reject(whole));
            }
            check(first.value === 'finally' && !first.done);
            let error; try { await stream.next(); } catch (caught) { error = caught; }
            check(error === whole && calls === 1 && closes === (mode === 'initializer' ? 1 : 0));
            check((await stream.next()).done);
            print('mixed-iterator-' + protocol + '-' + mode + ':ok');
        }
        async function all() {
            await run('sync', 'next'); await run('sync', 'done'); await run('sync', 'value');
            await run('sync', 'initializer'); await run('async', 'reject');
            await run('async', 'done'); await run('async', 'value'); await run('async', 'initializer');
        }
        all();
    "#,
        &[
            "mixed-iterator-sync-next:ok",
            "mixed-iterator-sync-done:ok",
            "mixed-iterator-sync-value:ok",
            "mixed-iterator-sync-initializer:ok",
            "mixed-iterator-async-reject:ok",
            "mixed-iterator-async-done:ok",
            "mixed-iterator-async-value:ok",
            "mixed-iterator-async-initializer:ok",
        ],
    );
}
