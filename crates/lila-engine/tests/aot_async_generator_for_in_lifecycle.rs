use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run_lifecycle(source: &str, expected: &[&str], strict_modes: &[bool]) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for &strict in strict_modes {
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
            .expect(
                "mixed ForIn lifecycle compiles ordinary JavaScript to the actual native Wasm path",
            );
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
fn mixed_for_in_queued_head_and_nested_resumes_keep_one_cursor_and_fresh_per_key_cells() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed ForIn cursor'); }
        const whole = {marker: 41};
        const proto = {p: 1, hidden: 2};
        const input = Object.create(proto); input.a = 1; input.b = 2;
        Object.defineProperty(input, 'hidden', {value: 3, enumerable: false});
        Object.defineProperty(input, 'return', {get: function () { throw 'for-in-must-not-close'; }});
        Object.defineProperty(input, Symbol.iterator, {get: function () { throw 'for-in-must-not-acquire-iterator'; }});
        const readers = [];
        let heads = 0;
        function head() { heads++; return 'head'; }
        async function* stream() {
            for (let key in await (yield head())) {
                { let inner = key;
                  readers.push(function () { return [key, inner]; });
                  await 0; gc(); yield readers[readers.length - 1]();
                  await 0; gc(); yield key;
                }
            }
            return whole;
        }
        async function* headTdz() {
            let key = 'outer';
            try { for (let key in (yield key)) { throw 'head-tdz-missing'; } }
            catch (error) { check(error instanceof ReferenceError); await 0; yield key; }
        }
        async function drive() {
            const iterator = stream();
            check((await iterator.next()).value === 'head' && heads === 1);
            let release;
            const waiting = new Promise(function (resolve) { release = resolve; });
            const pending = iterator.next(waiting), queued = iterator.next();
            gc(); release(input);
            let result = await pending; check(result.value[0] === 'a' && result.value[1] === 'a');
            check((await queued).value === 'a' && heads === 1);
            delete input.b; gc();
            result = await iterator.next(); check(result.value[0] === 'p' && result.value[1] === 'p');
            check((await iterator.next()).value === 'p');
            result = await iterator.next(); check(result.done && result.value === whole && heads === 1);
            gc(); check(readers.length === 2 && readers[0]()[0] === 'a' && readers[0]()[1] === 'a'
                && readers[1]()[0] === 'p' && readers[1]()[1] === 'p');
            const tdz = headTdz(); check((await tdz.next()).value === 'outer');
            check((await tdz.next()).done);
            print('mixed-for-in-cursor:ok');
        }
        drive();
    "#,
        &["mixed-for-in-cursor:ok"],
        &[false, true],
    );
}

#[test]
fn mixed_for_in_selects_each_key_reference_once_and_retains_body_rhs_reference_across_mutation() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed ForIn Reference'); }
        var slot = 'outside';
        let gets = 0, sets = 0;
        const blocked = {slot: false}, target = {slot: 0};
        target[Symbol.unscopables] = blocked;
        const view = new Proxy(target, {
            get: function (object, key, receiver) { if (key === 'slot') gets++; return Reflect.get(object, key, receiver); },
            set: function (object, key, value, receiver) { if (key === 'slot') sets++; return Reflect.set(object, key, value, receiver); }
        });
        async function* stream() {
            let iteration = 0;
            with (view) {
                for (slot in await (yield 'head')) {
                    if (iteration === 0) {
                        const assigned = (slot += await (yield 'rhs'));
                        yield assigned;
                    } else yield slot;
                    iteration++;
                }
            }
            return slot;
        }
        async function drive() {
            const iterator = stream(); check((await iterator.next()).value === 'head');
            check((await iterator.next({a:1,b:2})).value === 'rhs'
                && target.slot === 'a' && gets === 1 && sets === 1);
            target.slot = 'mutated'; blocked.slot = true; gc();
            check((await iterator.next(7)).value === 'a7'
                && target.slot === 'a7' && gets === 1 && sets === 2 && slot === 'outside');
            check((await iterator.next()).value === 'b' && slot === 'b' && target.slot === 'a7' && sets === 2);
            const done = await iterator.next(); check(done.done && done.value === 'b');
            print('mixed-for-in-reference:ok');
        }
        drive();
    "#,
        &["mixed-for-in-reference:ok"],
        &[false],
    );
}

#[test]
fn mixed_for_in_queued_abrupts_retire_cursor_after_inner_finalizers_without_iterator_close() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed ForIn completion'); }
        const reason = {marker: 43};
        const input = {a:1,b:2};
        Object.defineProperty(input, 'return', {get: function () { throw 'for-in-must-not-close'; }});
        let read, replace;
        async function* stream() {
            var value = 1;
            read = function () { return value; }; replace = function (next) { value = next; };
            for (let key in await (yield 'head')) {
                try { value += await (yield 'rhs'); yield value; break; }
                finally { await 0; gc(); yield 'finally'; }
            }
            return reason;
        }
        async function ready() {
            const iterator = stream(); check((await iterator.next()).value === 'head');
            check((await iterator.next(input)).value === 'rhs'); return iterator;
        }
        async function normal() {
            const iterator = await ready(); replace(100); gc();
            check((await iterator.next(4)).value === 5 && read() === 5);
            check((await iterator.next()).value === 'finally');
            const done = await iterator.next(); check(done.done && done.value === reason);
            print('mixed-for-in-normal:ok');
        }
        async function abrupt(mode) {
            const iterator = await ready();
            let injected;
            if (mode === 'return') injected = iterator.return(reason);
            else if (mode === 'throw') injected = iterator.throw(reason);
            else injected = iterator.next(Promise.reject(reason));
            const queued = iterator.next().then(function (result) { return {normal: result}; },
                function (error) { return {thrown: error}; });
            const finalizer = await injected;
            check(!finalizer.done && finalizer.value === 'finally' && read() === 1); gc();
            const done = await queued;
            if (mode === 'return') check(done.normal.done && done.normal.value === reason);
            else check(done.thrown === reason);
            check(read() === 1 && (await iterator.next()).done);
            print('mixed-for-in-' + mode + ':ok');
        }
        async function all() { await normal(); await abrupt('return'); await abrupt('throw'); await abrupt('reject'); }
        all();
    "#,
        &[
            "mixed-for-in-normal:ok",
            "mixed-for-in-return:ok",
            "mixed-for-in-throw:ok",
            "mixed-for-in-reject:ok",
        ],
        &[false, true],
    );
}
