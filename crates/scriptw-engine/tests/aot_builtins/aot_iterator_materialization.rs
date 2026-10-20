use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("iterator materialization compiles and executes through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{outcome:?}"
        );
    }
}

#[test]
fn array_from_observes_iterability_and_consumes_the_actual_iterator_once() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let iter = (function () {
            let n = g();
            return { [Symbol.iterator]: null, next: () => n.next() };
        })();
        if (Array.from(iter).length !== 0) throw 'array-like fallback';
        if (Array.from(Iterator.from(iter)).join(',') !== '0,1,2') throw 'first consumption';
        if (Iterator.from(iter).toArray().length !== 0) throw 'replayed consumed iterator';
        const existing = g();
        if (Iterator.from(existing) !== existing) throw 'existing iterator identity';
        true;
    "#,
    );
}

#[test]
fn iterator_materialization_keeps_own_methods_next_effects_and_abrupt_values() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let calls = 0;
        let iter = (function () {
            let n = g();
            return {
                [Symbol.iterator]: null,
                next: () => { calls++; return n.next(); },
                toArray() { return 'own method'; }
            };
        })();
        if (iter.toArray() !== 'own method' || calls !== 0) throw 'own method';
        if (Iterator.prototype.toArray.call(iter).join(',') !== '0,1,2' || calls !== 4) throw 'next effects';
        const marker = {};
        iter = (function () {
            let n = g();
            return {
                [Symbol.iterator]: null,
                next: () => { throw marker; return n.next(); }
            };
        })();
        let caught;
        try { Array.from(Iterator.from(iter)); } catch (error) { caught = error; }
        if (caught !== marker) throw 'next abrupt';
        true;
    "#,
    );
}

#[test]
fn materialization_uses_replaced_or_shadowed_constructors_and_fresh_generator_calls() {
    assert_modes(
        r#"
        function* g() { yield 0; yield 1; yield 2; }
        let iter = (function () {
            let n = g();
            return { [Symbol.iterator]: null, next: () => n.next() };
        })();
        {
            const Array = { from(value) { return value; } };
            if (Array.from(iter) !== iter) throw 'shadowed Array';
        }
        Array.from = function (value) { return value; };
        if (Array.from(iter) !== iter) throw 'replaced Array.from';
        Iterator.from = function (value) { return { toArray() { return value; } }; };
        if (Iterator.from(iter).toArray() !== iter) throw 'replaced Iterator.from';

        let entered = 0;
        var create = function* () { entered++; yield entered; };
        const first = create(), second = create();
        if (typeof create !== 'function' || first === second || entered !== 0) throw 'generator creation';
        if (first.next().value !== 1 || second.next().value !== 2 || entered !== 2) throw 'generator execution';
        true;
    "#,
    );
}

#[test]
fn reflect_argument_lists_resolve_block_generator_bindings_in_each_loop_iteration() {
    assert_modes(
        r#"
        function count() { return {numArgsReceived: arguments.length}; }
        for (var method of [Reflect.apply, (target, receiver, list) => Reflect.construct(target, list)]) {
            eval('0');
            if (method(count, undefined, {0: 1}).numArgsReceived !== 0) throw 'array-like count';
            function* values() { yield 1; yield 2; }
            if (method(count, undefined, values()).numArgsReceived !== 0) throw 'iterator must not be consumed';
            if (values().next().value !== 1) throw 'generator body';
        }
        true;
    "#,
    );
}

#[test]
fn block_hoistable_declarations_initialize_before_statements_and_escape_per_iteration() {
    assert_modes(
        r#"
        var saved = [];
        for (let index = 0; index < 2; index++) {
            eval('0');
            saved.push([ordinary, values, task, stream]);
            function ordinary() { return index; }
            function* values() { yield index; }
            async function task() { return index; }
            async function* stream() { yield index; }
        }
        if (saved[0][0]() !== 0 || saved[1][0]() !== 1) throw 'ordinary per-iteration capture';
        if (saved[0][1]().next().value !== 0 || saved[1][1]().next().value !== 1) throw 'generator per-iteration capture';
        for (var row of saved) {
            if (row[2].name !== 'task' || !(row[2]() instanceof Promise)) throw 'async function';
            if (row[3].name !== 'stream' || row[3]()[Symbol.asyncIterator]() === undefined) throw 'async generator';
        }
        if (saved[0][1] === saved[1][1] || saved[0][2] === saved[1][2] || saved[0][3] === saved[1][3]) throw 'reused block function identity';
        if (typeof values !== 'undefined' || typeof task !== 'undefined' || typeof stream !== 'undefined') throw 'leaked non-ordinary declaration';
        true;
    "#,
    );
}

#[test]
fn case_block_hoistable_declarations_exist_when_their_source_case_is_unreached() {
    assert_modes(
        r#"
        eval('0');
        var result;
        switch (0) {
            case 0:
                if (typeof ordinary !== 'function' || typeof values !== 'function' || typeof task !== 'function' || typeof stream !== 'function') throw 'missing CaseBlock binding';
                result = ordinary() + values().next().value;
                if (!(task() instanceof Promise) || stream()[Symbol.asyncIterator]() === undefined) throw 'wrong async function kind';
                break;
            case 1:
                function ordinary() { return 20; }
                function* values() { yield 22; }
                async function task() { return 1; }
                async function* stream() { yield 2; }
        }
        if (result !== 42) throw 'unreached source case was not instantiated';
        if (typeof values !== 'undefined' || typeof task !== 'undefined' || typeof stream !== 'undefined') throw 'CaseBlock leak';
        true;
    "#,
    );
}

#[test]
fn escaped_sibling_functions_observe_live_reassigned_generator_bindings() {
    assert_modes(
        r#"
        var saved;
        {
            eval('0');
            function* values() { yield values; }
            async function task() { return values; }
            async function* stream() { yield values; }
            function read() { return [values, task, stream]; }
            var old = values;
            var replacement = function () { return 17; };
            values = replacement;
            saved = [old, read];
        }
        if (saved[0]().next().value !== replacement) throw 'generator self binding snapshot';
        var current = saved[1]();
        if (current[0] !== replacement || current[1].name !== 'task' || current[2].name !== 'stream') throw 'sibling capture mismatch';
        true;
    "#,
    );
}
