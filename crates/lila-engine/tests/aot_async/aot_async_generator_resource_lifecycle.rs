use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_resources(source: &str, expected: &[&str]) {
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
            .expect("mixed resource scope uses ordinary JavaScript to Wasm compilation");
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
fn mixed_resource_scope_keeps_empty_await_flags_and_true_sync_methods_on_one_capability() {
    observe_resources(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed resource flags'); }
        const whole = {marker: 101};
        async function run(shape) {
            const events = []; let reads = 0;
            const middle = {get [Symbol.dispose]() {
                reads++; events.push('get-middle');
                return function () {
                    check(this === middle); events.push('middle');
                    Promise.resolve().then(function () { events.push('middle-job'); });
                    return {get then() { throw 'true-sync-method-return-must-be-ignored'; }};
                };
            }};
            const top = {[Symbol.asyncDispose]: function () {
                events.push('async'); return Promise.resolve().then(function () { events.push('async-job'); });
            }};
            async function* adjacent() {
                { using resource = await (yield 'init');
                  await using a = null, b = undefined; yield 'body'; }
                events.push('after'); return whole;
            }
            async function* separated() {
                { await using a = null;
                  using resource = await (yield 'init');
                  await using b = undefined; yield 'body'; }
                events.push('after'); return whole;
            }
            async function* methodAwaited() {
                { await using a = null;
                  using resource = await (yield 'init');
                  await using b = top; yield 'body'; }
                events.push('after'); return whole;
            }
            const stream = shape === 'adjacent' ? adjacent()
                : shape === 'separated' ? separated() : methodAwaited();
            check((await stream.next()).value === 'init');
            gc(); check((await stream.next(middle)).value === 'body');
            check(reads === 1 && events.join(',') === 'get-middle');
            Object.defineProperty(middle, Symbol.dispose, {get: function () { throw 'method-reacquired'; }});
            gc(); const done = await stream.next(); check(done.done && done.value === whole);
            await 0;
            const expected = shape === 'adjacent' ? 'get-middle,middle,after,middle-job'
                : shape === 'separated' ? 'get-middle,middle,middle-job,after'
                : 'get-middle,async,async-job,middle,after,middle-job';
            check(events.join(',') === expected && reads === 1);
            print('mixed-resource-' + shape + ':ok');
        }
        async function all() { await run('adjacent'); await run('separated'); await run('method'); }
        all();
    "#,
        &[
            "mixed-resource-adjacent:ok",
            "mixed-resource-separated:ok",
            "mixed-resource-method:ok",
        ],
    );
}

#[test]
fn mixed_resource_partial_registration_disposes_only_prior_entries_before_queued_completion() {
    observe_resources(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed partial resources'); }
        const whole = {marker: 102}, disposalError = {marker: 103}; whole.self = whole;
        async function run(mode) {
            const events = []; let reads = 0, readSecond;
            const first = {get [Symbol.dispose]() {
                reads++; return function () { check(this === first); events.push('first'); throw disposalError; };
            }};
            const bad = {get [Symbol.asyncDispose]() { events.push('get-second'); throw whole; }};
            async function* values() {
                try {
                    { readSecond = function () { return second; };
                      using resource = await (yield 'first');
                      await using second = await (yield 'second');
                      yield 'unreachable'; }
                } finally { await 0; gc(); yield 'finally'; }
                return whole;
            }
            const stream = values(); check((await stream.next()).value === 'first');
            gc(); check((await stream.next(first)).value === 'second');
            Object.defineProperty(first, Symbol.dispose, {get: function () { throw 'method-reacquired'; }});
            const pending = mode === 'return' ? stream.return(whole) : stream.next(bad);
            const queued = stream.next().then(function (result) { return {result: result}; },
                function (error) { return {error: error}; });
            const finalizer = await pending; check(finalizer.value === 'finally' && !finalizer.done);
            check(events.join(',') === (mode === 'return' ? 'first' : 'get-second,first'));
            const done = await queued;
            if (mode === 'return') check(done.error === disposalError);
            else check(done.error instanceof SuppressedError && done.error.error === disposalError && done.error.suppressed === whole);
            let tdz; try { readSecond(); } catch (error) { tdz = error; }
            check(tdz instanceof ReferenceError && reads === 1 && (await stream.next()).done);
            gc(); print('mixed-resource-partial-' + mode + ':ok');
        }
        async function all() { await run('return'); await run('getter'); }
        all();
    "#,
        &[
            "mixed-resource-partial-return:ok",
            "mixed-resource-partial-getter:ok",
        ],
    );
}

#[test]
fn mixed_resource_loop_head_keeps_original_iteration_cell_until_disposal_then_advances_or_closes() {
    observe_resources(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed resource iteration'); }
        const whole = {marker: 104}; whole.self = whole;
        async function run() {
            const events = [], readers = []; let next = 0, gets = 0;
            function resource(index) {
                const value = {index: index, get [Symbol.asyncDispose]() {
                    gets++; return function () {
                        check(this === value); events.push('dispose' + index);
                        return Promise.resolve().then(function () {
                            gc(); check(readers[index - 1]() === value);
                            events.push('disposed' + index);
                        });
                    };
                }};
                return value;
            }
            const first = resource(1), second = resource(2);
            const source = {[Symbol.asyncIterator]: function () { return {
                next: function () { events.push('next' + (++next));
                    return Promise.resolve({done: false, value: next === 1 ? first : second}); },
                return: function () { events.push('close'); return Promise.resolve({}); }
            }; }};
            async function* values() {
                for await (await using selected of await (yield 'source')) {
                    readers.push(function () { return selected; });
                    yield selected;
                    if (selected === first) continue;
                    break;
                }
                return whole;
            }
            const stream = values(); check((await stream.next()).value === 'source');
            check((await stream.next(source)).value === first);
            Object.defineProperty(first, Symbol.asyncDispose, {get: function () { throw 'method-reacquired'; }});
            gc(); check((await stream.next()).value === second);
            check(events.join(',') === 'next1,dispose1,disposed1,next2');
            Object.defineProperty(second, Symbol.asyncDispose, {get: function () { throw 'method-reacquired'; }});
            const done = await stream.next();
            check(done.done && done.value === whole && gets === 2 && next === 2);
            check(events.join(',') === 'next1,dispose1,disposed1,next2,dispose2,disposed2,close');
            gc(); check(readers[0]() === first && readers[1]() === second);
            print('mixed-resource-iteration-order:ok');
        }
        run();
    "#,
        &["mixed-resource-iteration-order:ok"],
    );
}

#[test]
fn mixed_resource_classic_head_uses_one_capability_until_test_exit_or_queued_return() {
    observe_resources(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed classic resource'); }
        const whole = {marker: 105}; whole.self = whole;
        async function run(mode) {
            const events = [], readers = []; let gets = 0, release, entered;
            const paused = new Promise(function (resolve) { release = resolve; });
            const started = new Promise(function (resolve) { entered = resolve; });
            const value = {get [Symbol.asyncDispose]() {
                gets++; return function () {
                    check(this === value); events.push('dispose'); entered();
                    return paused.then(function () {
                        gc(); for (const read of readers) check(read() === value);
                        events.push('disposed');
                    });
                };
            }};
            async function* values() {
                try {
                    for (await using selected = await (yield 'init');
                         await (yield 'test'); await (yield 'update')) {
                        readers.push(function () { return selected; });
                        yield selected; continue;
                    }
                    events.push('after');
                } finally { await 0; gc(); yield 'finally'; }
                return whole;
            }
            const stream = values(); check((await stream.next()).value === 'init');
            check((await stream.next(value)).value === 'test');
            check((await stream.next(true)).value === value);
            Object.defineProperty(value, Symbol.asyncDispose, {get: function () { throw 'method-reacquired'; }});
            let pending, queued;
            if (mode === 'normal') {
                check((await stream.next()).value === 'update');
                check((await stream.next()).value === 'test');
                check((await stream.next(true)).value === value);
                check((await stream.next()).value === 'update');
                check((await stream.next()).value === 'test');
                pending = stream.next(false);
            } else pending = stream.return(whole);
            let queueFinished = false;
            queued = stream.next().then(function (result) { queueFinished = true; return result; });
            await started; gc();
            check(!queueFinished && events.join(',') === 'dispose' && gets === 1);
            check(readers.length === (mode === 'normal' ? 2 : 1));
            release(); const finalizer = await pending;
            check(!finalizer.done && finalizer.value === 'finally');
            const done = await queued; check(done.done && done.value === whole);
            check(events.join(',') === (mode === 'normal' ? 'dispose,disposed,after' : 'dispose,disposed'));
            gc(); for (const read of readers) check(read() === value);
            print('mixed-resource-classic-' + mode + ':ok');
        }
        async function all() { await run('normal'); await run('return'); } all();
    "#,
        &[
            "mixed-resource-classic-normal:ok",
            "mixed-resource-classic-return:ok",
        ],
    );
}

#[test]
fn mixed_resource_switch_blocks_close_on_fallthrough_and_preserve_captured_cells_through_cleanup() {
    observe_resources(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed case resources'); }
        const whole = {marker: 106}; whole.self = whole;
        async function run(mode) {
            const events = []; let firstGets = 0, secondGets = 0, readFirst, readSecond, release, entered;
            const paused = new Promise(function (resolve) { release = resolve; });
            const started = new Promise(function (resolve) { entered = resolve; });
            const firstValue = {get [Symbol.dispose]() {
                firstGets++; return function () {
                    check(this === firstValue); events.push('first');
                    return {get then() { throw 'sync-disposal-return-adopted'; }};
                };
            }};
            const secondValue = {get [Symbol.asyncDispose]() {
                secondGets++;
                if (mode === 'getter') throw whole;
                return function () {
                    check(this === secondValue); events.push('second'); entered();
                    return paused.then(function () {
                        gc(); check(readFirst() === firstValue && readSecond() === secondValue);
                        events.push('second-done');
                    });
                };
            }};
            async function* values() {
                switch (await (yield 'head')) {
                    case await (yield 'selector'): {
                        using first = firstValue;
                        readFirst = function () { return first; };
                        yield 'first';
                    }
                    case 2: {
                        await using second = secondValue;
                        readSecond = function () { return second; };
                        yield 'body'; break;
                    }
                    default: {
                        using unused = {get [Symbol.dispose]() { throw 'unselected-registration'; }};
                    }
                }
                events.push('after'); return whole;
            }
            const stream = values(); check((await stream.next()).value === 'head');
            check((await stream.next(1)).value === 'selector');
            check((await stream.next(1)).value === 'first');
            check(events.length === 0 && firstGets === 1 && secondGets === 0);
            Object.defineProperty(firstValue, Symbol.dispose, {get: function () { throw 'first-method-reacquired'; }});
            if (mode === 'getter') {
                let error; try { await stream.next(); } catch (caught) { error = caught; }
                check(error === whole && events.join(',') === 'first' && secondGets === 1);
                gc(); check(readFirst() === firstValue && (await stream.next()).done);
            } else {
                check((await stream.next()).value === 'body');
                check(events.join(',') === 'first' && readFirst() === firstValue && readSecond() === secondValue);
                Object.defineProperty(secondValue, Symbol.asyncDispose, {get: function () { throw 'second-method-reacquired'; }});
                const pending = mode === 'return' ? stream.return(whole)
                    : mode === 'throw' ? stream.throw(whole) : stream.next();
                const outcome = pending.then(function (result) { return {result: result}; },
                    function (error) { return {error: error}; });
                let queueFinished = false;
                const queued = stream.next().then(function (result) { queueFinished = true; return result; });
                await started; gc(); check(!queueFinished && events.join(',') === 'first,second');
                release(); const done = await outcome;
                if (mode === 'throw') check(done.error === whole);
                else check(done.result.done && done.result.value === whole);
                check((await queued).done && firstGets === 1 && secondGets === 1);
                check(events.join(',') === (mode === 'normal' ? 'first,second,second-done,after' : 'first,second,second-done'));
                gc(); check(readFirst() === firstValue && readSecond() === secondValue);
            }
            print('mixed-resource-case-' + mode + ':ok');
        }
        async function all() { await run('normal'); await run('return'); await run('throw'); await run('getter'); } all();
    "#,
        &[
            "mixed-resource-case-normal:ok",
            "mixed-resource-case-return:ok",
            "mixed-resource-case-throw:ok",
            "mixed-resource-case-getter:ok",
        ],
    );
}
