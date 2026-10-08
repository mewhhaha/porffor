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
            .expect("mixed Switch lifecycle compiles the actual JavaScript to Wasm");
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
fn mixed_switch_queued_selection_preserves_whole_discriminant_default_order_and_caseblock_cells() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed Switch selection'); }
        const needle = {marker: 11};
        needle[Symbol.toPrimitive] = function () { throw 'selector-must-use-strict-equality'; };
        const whole = {marker: 17};
        const events = [];
        let reader;
        async function* stream() {
            switch (await (yield 'discriminant')) {
                case await (yield 'first-selector'):
                    throw 'wrong-first-body';
                default:
                    events.push('default');
                    yield 'default-body';
                case await (yield 'second-selector'):
                    let local = needle;
                    reader = function () { return [local, caseFunction]; };
                    events.push('second');
                    await 0;
                    gc();
                    yield reader();
                case (events.push('third-selector'), await (yield 'third-selector')):
                    function caseFunction() { return local; }
                    events.push('fallthrough');
                    const kept = await (yield 'initializer');
                    check(kept === whole);
                    gc();
                    yield kept;
                    break;
            }
            yield 'after';
            return reader;
        }
        async function drive(defaulted) {
            events.length = 0;
            const iterator = stream();
            check((await iterator.next()).value === 'discriminant');
            gc();
            check((await iterator.next(needle)).value === 'first-selector');
            let release;
            const waiting = new Promise(function (resolve) { release = resolve; });
            const pending = iterator.next(waiting);
            const queued = iterator.next(defaulted ? 0 : needle);
            gc();
            release(0);
            check((await pending).value === 'second-selector');
            let result = await queued;
            if (defaulted) {
                check(result.value === 'third-selector');
                result = await iterator.next(0);
                check(result.value === 'default-body');
                result = await iterator.next();
            }
            const pair = result.value;
            check(!result.done && pair[0] === needle && pair[1]() === needle);
            gc();
            check((await iterator.next()).value === 'initializer');
            result = await iterator.next(whole);
            check(!result.done && result.value === whole);
            check((await iterator.next()).value === 'after');
            result = await iterator.next();
            check(result.done && result.value === reader && reader()[1] === pair[1]);
            check(events.join(',') === (defaulted ? 'third-selector,default,second,fallthrough' : 'second,fallthrough'));
            gc();
            check(reader()[0] === needle && reader()[1]() === needle);
            print(defaulted ? 'mixed-switch-default:ok' : 'mixed-switch-match:ok');
        }
        async function all() { await drive(false); await drive(true); }
        all();
    "#,
        &["mixed-switch-match:ok", "mixed-switch-default:ok"],
    );
}

#[test]
fn mixed_switch_generated_terminal_values_keep_their_operand_scope_and_original_tdz() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed Switch terminal scope'); }
        const whole = {marker: 23};
        const readers = [];
        async function* phased() {
            if (await (yield 'condition')) {
                for (let index = 0; await (yield 'test'); index = await (yield 'update')) {
                    switch (await (yield 'discriminant')) {
                        case await (yield 'selector'):
                            readers.push(function () { return index; });
                            yield index;
                            break;
                        default: throw 'unreachable-default';
                    }
                }
            }
            return whole;
        }
        async function* tdz() {
            let later = whole;
            try {
                switch (yield 'tdz-discriminant') {
                    case later: throw 'missing-caseblock-tdz';
                    default:
                        const later = whole;
                        yield later;
                }
            } catch (error) {
                check(error instanceof ReferenceError);
                await 0;
                gc();
                yield later;
            }
        }
        function* ordinary() {
            for (let index = 0; yield 'ordinary-test'; index = yield 'ordinary-update') {
                switch (yield 'ordinary-discriminant') {
                    case yield 'ordinary-selector': yield index; break;
                    default: throw 'unreachable-ordinary-default';
                }
            }
            return whole;
        }
        async function drive() {
            const iterator = phased();
            check((await iterator.next()).value === 'condition');
            check((await iterator.next(true)).value === 'test');
            gc();
            check((await iterator.next(true)).value === 'discriminant');
            check((await iterator.next(7)).value === 'selector');
            check((await iterator.next(7)).value === 0);
            check((await iterator.next()).value === 'update');
            gc();
            check((await iterator.next(1)).value === 'test');
            const done = await iterator.next(false);
            check(done.done && done.value === whole && readers[0]() === 0);
            const rejected = tdz();
            check((await rejected.next()).value === 'tdz-discriminant');
            const caught = await rejected.next(0);
            check(!caught.done && caught.value === whole);
            check((await rejected.next()).done);
            const original = ordinary();
            check(original.next().value === 'ordinary-test');
            check(original.next(true).value === 'ordinary-discriminant');
            check(original.next(7).value === 'ordinary-selector');
            check(original.next(7).value === 0);
            check(original.next().value === 'ordinary-update');
            check(original.next(1).value === 'ordinary-test');
            const originalDone = original.next(false);
            check(originalDone.done && originalDone.value === whole);
            print('mixed-switch-terminals:ok');
        }
        drive();
    "#,
        &["mixed-switch-terminals:ok"],
    );
}

#[test]
fn mixed_switch_queue_and_finalizers_preserve_original_reference_and_whole_abrupt_completion() {
    run_lifecycle(
        r#"
        function check(ok) { if (!ok) throw new Error('mixed Switch completion'); }
        const reason = {marker: 31};
        let read, replace;
        async function* stream() {
            var value = 1;
            read = function () { return value; };
            replace = function (next) { value = next; };
            outer: for (let index = 0; index < 2; index++) {
                selected: switch (await (yield 'discriminant')) {
                    case await (yield 'selector'):
                        try {
                            value += await (yield 'rhs');
                            yield value;
                            continue outer;
                        } finally {
                            await 0;
                            gc();
                            yield 'finally';
                        }
                    default: break selected;
                }
            }
            return reason;
        }
        async function ready() {
            const iterator = stream();
            check((await iterator.next()).value === 'discriminant');
            check((await iterator.next(1)).value === 'selector');
            check((await iterator.next(1)).value === 'rhs');
            return iterator;
        }
        async function normal() {
            const iterator = await ready();
            replace(100);
            gc();
            check((await iterator.next(4)).value === 5 && read() === 5);
            check((await iterator.next()).value === 'finally');
            check((await iterator.next()).value === 'discriminant');
            check((await iterator.next(1)).value === 'selector');
            const done = await iterator.next(0);
            check(done.done && done.value === reason && read() === 5);
            print('mixed-switch-normal:ok');
        }
        async function abrupt(mode) {
            const iterator = await ready();
            let injected;
            if (mode === 'return') injected = iterator.return(reason);
            else if (mode === 'throw') injected = iterator.throw(reason);
            else injected = iterator.next(Promise.reject(reason));
            const queued = iterator.next().then(
                function (result) { return {normal: result}; },
                function (error) { return {thrown: error}; }
            );
            const finalizer = await injected;
            check(!finalizer.done && finalizer.value === 'finally' && read() === 1);
            gc();
            const final = await queued;
            if (mode === 'return') check(final.normal.done && final.normal.value === reason);
            else check(final.thrown === reason);
            check(read() === 1 && (await iterator.next()).done);
            print('mixed-switch-' + mode + ':ok');
        }
        async function all() { await normal(); await abrupt('return'); await abrupt('throw'); await abrupt('reject'); }
        all();
    "#,
        &[
            "mixed-switch-normal:ok",
            "mixed-switch-return:ok",
            "mixed-switch-throw:ok",
            "mixed-switch-reject:ok",
        ],
    );
}
