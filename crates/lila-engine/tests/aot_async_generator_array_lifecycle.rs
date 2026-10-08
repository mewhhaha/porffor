use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_lifecycle(source: &str, expected: &[&str]) {
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
            .expect("mixed pattern lifecycle uses the ordinary JavaScript to Wasm compiler");
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
fn mixed_array_nested_close_preserves_queued_whole_completions_before_outer_finalizers() {
    observe_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed Array close'); }
        const whole = {marker: 83}, closeError = {marker: 84};
        whole.self = whole;
        whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };
        function source(label, first, events, fails) {
            let reads = 0, calls = 0;
            const iterator = {
                get next() {
                    reads++;
                    return function () { check(this === iterator); calls++; return {done: false, value: first}; };
                },
                get return() {
                    events.push('get:' + label);
                    return function () {
                        check(this === iterator); events.push('call:' + label);
                        if (fails) throw closeError;
                        return {get then() { throw 'synchronous-close-must-not-await'; }};
                    };
                }
            };
            return {input: {[Symbol.iterator]: function () { return iterator; }},
                reads: function () { return reads; }, calls: function () { return calls; }};
        }
        async function run(mode, fails) {
            const events = [], inner = source('inner', undefined, events, fails);
            const outer = source('outer', inner.input, events, false);
            async function* values() {
                try {
                    const [[value = await (yield 'default')]] = outer.input;
                    yield value;
                } finally { await 0; gc(); yield 'finally'; }
                return whole;
            }
            const iterator = values();
            check((await iterator.next()).value === 'default');
            check(inner.reads() === 1 && outer.reads() === 1 && events.length === 0);
            gc();
            if (mode === 'normal') {
                check((await iterator.next(7)).value === 7);
                check(events.join(',') === 'get:inner,call:inner,get:outer,call:outer');
                check((await iterator.next()).value === 'finally');
                const done = await iterator.next(); check(done.done && done.value === whole);
            } else {
                const pending = mode === 'return' ? iterator.return(whole)
                    : mode === 'throw' ? iterator.throw(whole) : iterator.next(Promise.reject(whole));
                const queued = iterator.next().then(function (result) { return {result: result}; },
                    function (error) { return {error: error}; });
                const finalizer = await pending;
                check(!finalizer.done && finalizer.value === 'finally');
                check(events.join(',') === 'get:inner,call:inner,get:outer,call:outer');
                const done = await queued;
                if (mode === 'return' && !fails) check(done.result.done && done.result.value === whole);
                else check(done.error === (mode === 'return' ? closeError : whole));
                check((await iterator.next()).done);
            }
            check(inner.calls() === 1 && outer.calls() === 1);
            gc(); print('mixed-array-' + mode + (fails ? '-close-error' : '') + ':ok');
        }
        async function all() {
            await run('normal', false); await run('return', false);
            await run('return', true); await run('throw', true); await run('reject', true);
        }
        all();
    "#,
        &[
            "mixed-array-normal:ok",
            "mixed-array-return:ok",
            "mixed-array-return-close-error:ok",
            "mixed-array-throw-close-error:ok",
            "mixed-array-reject-close-error:ok",
        ],
    );
}

#[test]
fn mixed_array_retains_cached_next_and_distinguishes_step_errors_from_target_and_default_errors() {
    observe_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed Array protocol'); }
        const whole = {marker: 85};
        async function cached() {
            let reads = 0, calls = 0, closes = 0;
            const iterator = {get next() {
                reads++;
                return function () { check(this === iterator); calls++;
                    return {done: false, value: calls === 1 ? undefined : 12}; };
            }, return: function () { closes++; return {}; }};
            const input = {[Symbol.iterator]: function () { return iterator; }};
            async function* values() {
                const [first = await (yield 'default'), second] = input;
                yield [first, second]; return whole;
            }
            const stream = values(); check((await stream.next()).value === 'default');
            Object.defineProperty(iterator, 'next', {value: function () { throw 'next-was-reacquired'; }});
            gc(); const pair = await stream.next(5);
            check(pair.value[0] === 5 && pair.value[1] === 12 && reads === 1 && calls === 2 && closes === 1);
            const done = await stream.next(); check(done.done && done.value === whole);
            print('mixed-array-cached-next:ok');
        }
        async function failure(mode) {
            let closes = 0;
            const iterator = {
                get next() {
                    if (mode === 'acquire') throw whole;
                    return function () {
                        if (mode === 'next') throw whole;
                        return {get done() { if (mode === 'done') throw whole; return false; },
                            get value() { if (mode === 'value') throw whole; return mode === 'default' ? undefined : 3; }};
                    };
                },
                get return() { closes++; return function () { return {}; }; }
            };
            const input = {[Symbol.iterator]: function () { return iterator; }};
            const sink = {set value(next) { check(next === 3); throw whole; }};
            async function* values() {
                try { [sink.value = await (yield 'default')] = input; }
                finally { await 0; gc(); yield 'finally'; }
            }
            const stream = values(); let first = await stream.next();
            if (mode === 'default') {
                check(first.value === 'default' && closes === 0);
                first = await stream.next(Promise.reject(whole));
            }
            check(!first.done && first.value === 'finally');
            check(closes === (mode === 'default' || mode === 'put' ? 1 : 0));
            try { await stream.next(); throw 'missing-whole-error'; }
            catch (error) { check(error === whole); }
            check((await stream.next()).done); print('mixed-array-' + mode + ':ok');
        }
        async function all() {
            await cached(); await failure('acquire'); await failure('next');
            await failure('done'); await failure('value'); await failure('put'); await failure('default');
        }
        all();
    "#,
        &[
            "mixed-array-cached-next:ok",
            "mixed-array-acquire:ok",
            "mixed-array-next:ok",
            "mixed-array-done:ok",
            "mixed-array-value:ok",
            "mixed-array-put:ok",
            "mixed-array-default:ok",
        ],
    );
}
