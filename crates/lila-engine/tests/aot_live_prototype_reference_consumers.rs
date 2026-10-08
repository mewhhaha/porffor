use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("live prototype controls failed: {error}\n{source}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(observed.output_events.is_empty(), "{source}");
    }
}

#[test]
fn optional_named_and_symbol_gets_invalidate_before_call_arguments() {
    assert_modes(
        r#"
        let state;
        let trace = '';
        let receiver;
        Object.defineProperty(RegExp.prototype, 'test', {
            configurable: true,
            get() {
                if (this !== receiver) throw 'named-get-receiver';
                trace += 'get;';
                state.value = 'named';
                return function (argument) {
                    if (this !== receiver) throw 'named-call-receiver';
                    trace += 'call;';
                    return argument;
                };
            }
        });
        receiver = /x/;
        state = { value: 1 };
        const named = receiver?.test?.((trace += 'arg;', typeof state.value));
        if (named !== 'string' || trace !== 'get;arg;call;') throw 'named-order';

        Object.defineProperty(String.prototype, 'toUpperCase', {
            configurable: true,
            get() {
                state.value = 'primitive';
                return function (argument) { return argument; };
            }
        });
        state = { value: 1 };
        if ('x'?.toUpperCase?.(typeof state.value) !== 'string') throw 'primitive-order';
        if ('xy'?.slice?.(1) !== 'y') throw 'native-candidate';
        if ('😀x'?.[Symbol.iterator]?.().next().value !== '😀') throw 'string-iterator';

        Object.defineProperty(Array.prototype, Symbol.iterator, {
            configurable: true,
            get() {
                if (this !== receiver) throw 'symbol-get-receiver';
                state.value = 'symbol';
                return function (argument) {
                    if (this !== receiver) throw 'symbol-call-receiver';
                    return argument;
                };
            }
        });
        receiver = [];
        state = { value: 1 };
        if (receiver?.[Symbol.iterator]?.(typeof state.value) !== 'string') throw 'symbol-order';
        true;
        "#,
    );
}

#[test]
fn compound_get_observes_replaced_and_previously_absent_prototype_descriptors() {
    assert_modes(
        r#"
        let state;
        let stored;
        Object.defineProperty(RegExp.prototype, 'test', {
            configurable: true,
            get() { state.value = 'named'; return false; },
            set(value) { stored = value; }
        });
        const named = /x/;
        state = { value: 1 };
        if ((named.test ||= typeof state.value) !== 'string' || stored !== 'string') {
            throw 'named-compound';
        }

        Object.defineProperty(Object.prototype, 'inheritedSlot', {
            configurable: true,
            get() { state.value = 'absent'; return false; },
            set(value) { stored = value; }
        });
        const ordinary = {};
        state = { value: 1 };
        if ((ordinary.inheritedSlot ||= typeof state.value) !== 'string' || stored !== 'string') {
            throw 'absent-compound';
        }

        Object.defineProperty(RegExp.prototype, Symbol.match, {
            configurable: true,
            get() { state.value = 'symbol'; return false; },
            set(value) { stored = value; }
        });
        const symbolic = /x/;
        state = { value: 1 };
        if ((symbolic[Symbol.match] ||= typeof state.value) !== 'string' || stored !== 'string') {
            throw 'symbol-compound';
        }
        const own = { inheritedSlot: 1 };
        own.inheritedSlot += 2;
        if (own.inheritedSlot !== 3) throw 'own-data';
        true;
        "#,
    );
}

#[test]
fn super_get_retains_receiver_and_observes_mutated_intrinsic_before_arguments() {
    assert_modes(
        r#"
        let state;
        let receiver;
        class Derived extends RegExp {
            named() { return super.test(typeof state.value); }
            symbolic() { return super[Symbol.match](typeof state.value); }
        }
        const descriptor = {
            configurable: true,
            get() {
                if (this !== receiver) throw 'super-get-receiver';
                state.value = 'super';
                return function (argument) {
                    if (this !== receiver) throw 'super-call-receiver';
                    return argument;
                };
            }
        };
        Object.defineProperty(RegExp.prototype, 'test', descriptor);
        Object.defineProperty(RegExp.prototype, Symbol.match, descriptor);
        receiver = new Derived('x');
        state = { value: 1 };
        if (receiver.named() !== 'string') throw 'super-named-order';
        state = { value: 1 };
        if (receiver.symbolic() !== 'string') throw 'super-symbol-order';
        true;
        "#,
    );
}

#[test]
fn iterator_from_missing_shape_entries_do_not_prove_protocol_gets_effect_free() {
    assert_modes(
        r#"
        const from = Iterator.from;
        let state;
        let trace = '';
        Object.defineProperty(Object.prototype, Symbol.iterator, {
            configurable: true,
            get() { trace += 'iterator;'; state.value = 'iterator'; return undefined; }
        });
        Object.defineProperty(Object.prototype, 'next', {
            configurable: true,
            get() {
                trace += 'next;';
                state.value = 'next';
                return function () { return { done: true }; };
            }
        });
        const input = {};
        state = { value: 1 };
        const wrapped = from(input);
        if (typeof state.value !== 'string' || state.value !== 'next') throw 'protocol-effects';
        if (trace !== 'iterator;next;') throw 'protocol-order';
        if (wrapped.next().done !== true) throw 'wrapped-native';
        true;
        "#,
    );
}

#[test]
fn iterator_from_observes_prototype_traps_and_can_return_an_alternate_callable() {
    assert_modes(
        r#"
        const from = Iterator.from;
        let state;
        let traps = 0;
        const prototype = new Proxy({}, {
            getPrototypeOf() { traps++; state.value = 'prototype'; return null; }
        });
        const input = {
            __proto__: prototype,
            [Symbol.iterator]: undefined,
            next() { return { done: true }; }
        };
        state = { value: 1 };
        from(input);
        if (traps !== 1 || typeof state.value !== 'string') throw 'prototype-effects';

        function callable() { return 71; }
        callable.next = function () { return { done: true }; };
        Object.setPrototypeOf(callable, Iterator.prototype);
        const iterable = { [Symbol.iterator]() { return callable; } };
        const result = from(iterable);
        if (result !== callable || typeof result !== 'function' || result() !== 71) {
            throw 'alternate-callable';
        }
        const array = [];
        array.next = callable.next;
        Object.setPrototypeOf(array, Iterator.prototype);
        const arrayResult = from({ [Symbol.iterator]() { return array; } });
        if (arrayResult !== array || !Array.isArray(arrayResult)) throw 'alternate-array';
        true;
        "#,
    );
}
