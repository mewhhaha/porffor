use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run_lifecycle(source: &str, expected: &[&str]) {
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
            .expect("mixed loop lifecycle compiles and executes through Wasm AOT");
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
fn mixed_loop_queue_keeps_original_reference_and_old_value_through_yield_then_await() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed queue'); }
        let replace;
        let read;
        async function* stream() {
            let value = 1;
            replace = function (next) { value = next; };
            read = function () { return value; };
            for (let i = 0; i < 1; i += 1) {
                value += await (yield 'rhs');
                gc();
                yield value;
            }
            return value;
        }
        async function drive() {
            const iterator = stream();
            const first = await iterator.next();
            check(first.value === 'rhs' && !first.done);
            let release;
            const waiting = new Promise(function (resolve) { release = resolve; });
            const next = iterator.next(waiting);
            const done = iterator.next();
            replace(40);
            gc();
            check(read() === 40);
            release(4);
            const second = await next;
            check(second.value === 5 && !second.done && read() === 5);
            const last = await done;
            check(last.value === 5 && last.done);
            print('mixed-queue:ok');
        }
        drive();
    "#,
        &["mixed-queue:ok"],
    );
}

#[test]
fn mixed_loop_queued_return_abandons_reference_before_awaiting_and_yielding_finally() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed return'); }
        const token = { value: 71 };
        let read;
        async function* stream() {
            let value = 2;
            read = function () { return value; };
            for (let i = 0; i < 1; i += 1) {
                try { value += await (yield 'rhs'); }
                finally { await 0; gc(); yield 'finally'; }
            }
            return value;
        }
        async function drive() {
            const iterator = stream();
            const first = await iterator.next();
            check(first.value === 'rhs' && !first.done);
            const returned = iterator.return(token);
            const queued = iterator.next();
            const cleanup = await returned;
            check(cleanup.value === 'finally' && !cleanup.done && read() === 2);
            gc();
            const final = await queued;
            check(final.value === token && final.done && read() === 2);
            const done = await iterator.next();
            check(done.value === undefined && done.done);
            print('mixed-return:ok');
        }
        drive();
    "#,
        &["mixed-return:ok"],
    );
}

#[test]
fn mixed_loop_injected_throw_and_rejected_await_keep_whole_reason_and_fresh_reference() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed throw'); }
        const reason = { value: 19 };
        async function* stream(reject) {
            let value = 2;
            for (let i = 0; i < 1; i += 1) {
                try { value += await (yield 'rhs'); }
                catch (error) {
                    check(error === reason);
                    gc();
                    yield 'caught';
                    value += await (yield 'fresh');
                }
                finally { await 0; gc(); }
            }
            return value;
        }
        async function drive(reject) {
            const iterator = stream(reject);
            const first = await iterator.next();
            check(first.value === 'rhs' && !first.done);
            let caught;
            if (reject) { caught = await iterator.next(Promise.reject(reason)); }
            else { caught = await iterator.throw(reason); }
            check(caught.value === 'caught' && !caught.done);
            const fresh = await iterator.next();
            check(fresh.value === 'fresh' && !fresh.done);
            const done = await iterator.next(3);
            check(done.value === 5 && done.done);
            print(reject ? 'mixed-reject:ok' : 'mixed-throw:ok');
        }
        async function run() { await drive(false); await drive(true); }
        run();
    "#,
        &["mixed-throw:ok", "mixed-reject:ok"],
    );
}

#[test]
fn mixed_loop_restores_original_enclosing_and_catch_cells_after_linear_resume() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed scope'); }
        const reason = { value: 23 };
        const replacement = { value: 29 };
        let inspect;
        let replace;
        let inspectCatch;
        let replaceCatch;
        async function* stream() {
            {
                let value = 3;
                inspect = function () { return value; };
                replace = function (next) { value = next; };
                yield 'linear';
                check(value === 7);
                for (let i = 0; i < 1; i += 1) {
                    try { await Promise.reject(reason); }
                    catch (error) {
                        inspectCatch = function () { return error; };
                        replaceCatch = function (next) { error = next; };
                        await 0;
                        gc();
                        yield error;
                        check(error === replacement);
                        if (await true) { value += await (yield 'update'); }
                        check(error === replacement && value === 10);
                    }
                }
                return value;
            }
        }
        async function drive() {
            const iterator = stream();
            const linear = await iterator.next();
            check(linear.value === 'linear' && !linear.done);
            replace(7);
            gc();
            const caught = await iterator.next();
            check(caught.value === reason && !caught.done && inspect() === 7);
            replaceCatch(replacement);
            gc();
            const update = await iterator.next();
            check(update.value === 'update' && !update.done && inspectCatch() === replacement);
            const done = await iterator.next(3);
            check(done.value === 10 && done.done && inspect() === 10 && inspectCatch() === replacement);
            print('mixed-scope:ok');
        }
        drive();
    "#,
        &["mixed-scope:ok"],
    );
}

#[test]
fn mixed_foreign_for_await_restores_captured_head_and_deepest_child_before_second_yield() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed foreign scope'); }
        const whole = { value: 37 };
        let readOuter;
        let readItem;
        let readInner;
        let replaceInner;
        async function* stream(source) {
            {
                let outer = whole;
                readOuter = function () { return outer; };
                for (let i = 0; i < 1; i += 1) { await 0; yield 'loop'; }
                for await (const item of source) {
                    readItem = function () { return item; };
                    {
                        let inner = item;
                        readInner = function () { return inner; };
                        replaceInner = function (next) { inner = next; };
                        yield inner;
                        gc();
                        yield inner;
                        check(readItem() === item && readOuter() === whole);
                    }
                }
                return outer;
            }
        }
        async function drive() {
            const iterator = stream([1, 2]);
            const loop = await iterator.next();
            check(loop.value === 'loop' && !loop.done);
            const first = await iterator.next();
            check(first.value === 1 && !first.done && readItem() === 1);
            replaceInner(11); gc();
            const second = await iterator.next();
            check(second.value === 11 && !second.done && readInner() === 11 && readItem() === 1);
            const third = await iterator.next();
            check(third.value === 2 && !third.done && readItem() === 2);
            replaceInner(22); gc();
            const fourth = await iterator.next();
            check(fourth.value === 22 && !fourth.done && readInner() === 22 && readItem() === 2);
            const done = await iterator.next();
            check(done.value === whole && done.done && readOuter() === whole);
            print('mixed-foreign-scope:ok');
        }
        drive();
    "#,
        &["mixed-foreign-scope:ok"],
    );
}

#[test]
fn mixed_resource_suffix_preserves_captured_parent_through_implicit_disposal_await() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed resource scope'); }
        const whole = { value: 41 };
        let read;
        let replace;
        const disposed = [];
        function makeSync() {
            return { [Symbol.dispose]: function () { gc(); disposed.push(read()); } };
        }
        function makeAsync() {
            return { [Symbol.asyncDispose]: async function () { await 0; gc(); disposed.push(read()); } };
        }
        async function* syncStream() {
            {
                let value = 13;
                read = function () { return value; };
                replace = function (next) { value = next; };
                for (let i = 0; i < 1; i += 1) { await 0; yield 'loop'; }
                using resource = makeSync();
                check(value === 17);
                yield 'held';
            }
            return whole;
        }
        async function* asyncStream() {
            {
                let value = 13;
                read = function () { return value; };
                replace = function (next) { value = next; };
                for (let i = 0; i < 1; i += 1) { await 0; yield 'loop'; }
                await using resource = makeAsync();
                check(value === 17);
                yield 'held';
            }
            return whole;
        }
        async function drive(factory, label) {
            const iterator = factory();
            const loop = await iterator.next();
            check(loop.value === 'loop' && !loop.done);
            replace(17); gc();
            const held = await iterator.next();
            check(held.value === 'held' && !held.done && read() === 17);
            replace(19); gc();
            const done = await iterator.return(whole);
            check(done.value === whole && done.done && read() === 19);
            print(label);
        }
        async function run() {
            await drive(syncStream, 'mixed-sync-resource:ok');
            await drive(asyncStream, 'mixed-async-resource:ok');
            check(disposed.join(',') === '19,19');
        }
        run();
    "#,
        &["mixed-sync-resource:ok", "mixed-async-resource:ok"],
    );
}
