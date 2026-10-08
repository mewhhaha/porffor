use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run_lifecycle(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let observed = Engine::new(RealmBuilder::new().build())
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
        .expect("mixed With lifecycle uses the actual Wasm compiler and request queue");
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

#[test]
fn mixed_with_boxes_the_completed_head_once_and_retains_the_original_closure_record() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed With head'); }
        let reader;
        async function* stream() {
            var x = 1;
            with (await (yield 'head')) {
                reader = function () { return x; };
                await 0;
                gc();
                yield x;
            }
            yield x;
            return reader;
        }
        async function drive() {
            const view = {x: 7};
            const iterator = stream();
            const head = await iterator.next();
            check(head.value === 'head' && !head.done);
            const body = await iterator.next(view);
            check(body.value === 7 && !body.done && reader() === 7);
            view.x = 9;
            gc();
            check(reader() === 9);
            const outside = await iterator.next();
            check(outside.value === 1 && !outside.done && reader() === 9);
            const last = await iterator.next();
            check(last.value === reader && last.done && last.value() === 9);
            gc();
            check(reader() === 9);
            const invalid = stream();
            await invalid.next();
            let rejected = false;
            try { await invalid.next(null); }
            catch (error) { rejected = error instanceof TypeError; }
            check(rejected && reader() === 9);
            print('mixed-with-head:ok');
        }
        drive();
    "#,
        &["mixed-with-head:ok"],
    );
}

#[test]
fn mixed_with_put_keeps_selected_object_record_after_unscopables_and_queue_mutation() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed With Reference'); }
        let backing = 2;
        let gets = 0;
        let selectedHas = 0;
        let writes = 0;
        const blocked = {x: false};
        const target = {x: 2};
        target[Symbol.unscopables] = blocked;
        const view = new Proxy(target, {
            has: function (object, key) {
                if (key === 'x') selectedHas += 1;
                return Reflect.has(object, key);
            },
            get: function (object, key, receiver) {
                if (key === 'x') { gets += 1; return backing; }
                return Reflect.get(object, key, receiver);
            },
            set: function (object, key, value, receiver) {
                if (key === 'x') { writes += 1; backing = value; return true; }
                return Reflect.set(object, key, value, receiver);
            }
        });
        let outer;
        async function* stream() {
            var x = 900;
            var assigned;
            outer = function () { return x; };
            with (view) {
                assigned = (x += await (yield 'rhs'));
                gc();
                yield assigned;
            }
            return x;
        }
        async function drive() {
            const iterator = stream();
            const first = await iterator.next();
            check(first.value === 'rhs' && gets === 1 && selectedHas === 2);
            let release;
            const waiting = new Promise(function (resolve) { release = resolve; });
            const pending = iterator.next(waiting);
            const queued = iterator.next();
            backing = 40;
            blocked.x = true;
            gc();
            check(outer() === 900);
            release(4);
            const written = await pending;
            check(written.value === 6 && !written.done);
            check(backing === 6 && gets === 1 && selectedHas === 3 && writes === 1);
            const last = await queued;
            check(last.value === 900 && last.done && outer() === 900);
            print('mixed-with-reference:ok');
        }
        drive();
    "#,
        &["mixed-with-reference:ok"],
    );
}

#[test]
fn mixed_with_injected_and_rejected_whole_completions_leave_before_outer_finally_lookup() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed With cleanup'); }
        const reason = {value: 23};
        async function* stream(view) {
            var x = 91;
            try {
                with (view) {
                    try { x += await (yield 'rhs'); }
                    finally {
                        await 0;
                        gc();
                        check(x === 2);
                        yield 'inner';
                    }
                }
            } finally {
                await 0;
                gc();
                check(x === 91);
                yield 'outer';
            }
        }
        async function drive(mode) {
            const view = {x: 2};
            const iterator = stream(view);
            check((await iterator.next()).value === 'rhs');
            let injected;
            if (mode === 'return') injected = iterator.return(reason);
            else if (mode === 'throw') injected = iterator.throw(reason);
            else injected = iterator.next(Promise.reject(reason));
            const queued = iterator.next();
            const inner = await injected;
            check(inner.value === 'inner' && !inner.done && view.x === 2);
            gc();
            const outer = await queued;
            check(outer.value === 'outer' && !outer.done && view.x === 2);
            if (mode === 'return') {
                const final = await iterator.next();
                check(final.value === reason && final.done && view.x === 2);
            } else {
                let caught;
                try { await iterator.next(); } catch (error) { caught = error; }
                check(caught === reason && view.x === 2);
            }
            check((await iterator.next()).done);
            print('mixed-with-cleanup:' + mode);
        }
        async function all() { await drive('return'); await drive('throw'); await drive('reject'); }
        all();
    "#,
        &[
            "mixed-with-cleanup:return",
            "mixed-with-cleanup:throw",
            "mixed-with-cleanup:reject",
        ],
    );
}
