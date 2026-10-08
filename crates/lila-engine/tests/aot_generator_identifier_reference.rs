use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_identifier_reference(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
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
        .expect("persistent Identifier Reference executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(expected.into())]
    );
}

#[test]
fn suspended_global_and_declarative_references_preserve_order_strictness_and_logical_selection() {
    for (directive, strict) in [("", false), ("'use strict';\n", true)] {
        observe_identifier_reference(
            &format!(
                "{directive}var strictReferenceFixture = {strict};\n{}",
                include_str!("fixtures/generator_staged_operands/identifier_references.js"),
            ),
            "generator-identifier-reference:ok",
        );
    }
}

#[test]
fn suspended_with_references_keep_the_selected_object_across_unscopables_and_property_changes() {
    observe_identifier_reference(
        r#"
        function check(value, message) { if (!value) throw new Error(message); }
        var trace = [], outer = { value: 100 }, masks = { value: false };
        var target = { value: { valueOf() { trace.push('coerce'); return 3; } } };
        target[Symbol.unscopables] = masks;
        var selected = new Proxy(target, {
            has(object, key) { if (key === 'value') trace.push('has'); return key in object; },
            get(object, key) {
                if (key === Symbol.unscopables) trace.push('unscopables');
                if (key === 'value') trace.push('get');
                return object[key];
            },
            set(object, key, value) {
                if (key === 'value') trace.push('set:' + value);
                object[key] = value; return true;
            }
        });
        function* compound() { with (outer) { with (selected) { return value += yield 'rhs'; } } }
        var iterator = compound();
        check(iterator.next().value === 'rhs', 'with compound suspends');
        check(trace.join(',') === 'has,unscopables,has,get', 'ordered HasBinding then GetValue');
        masks.value = true; delete target.value; gc();
        check(iterator.next(4).value === 7 && target.value === 7 && outer.value === 100,
              'original selected object receives PutValue after deletion and hidden-binding change');
        check(trace.join(',') === 'has,unscopables,has,get,coerce,has,set:7',
              'PutValue rechecks original property without HasBinding or unscopables');
        trace = []; target.value = 0; masks.value = false;
        function* skipped() { with (selected) { return value &&= yield 'unreachable'; } }
        var done = skipped().next();
        check(done.done && done.value === 0 && trace.join(',') === 'has,unscopables,has,get',
              'with logical skip retires Reference without SetMutableBinding');
        target.value = true;
        function* logical() { with (selected) { return value &&= sum(yield 'one', yield 'two'); } }
        function sum(a, b) { return a + b; }
        var logicalIterator = logical(); trace = [];
        check(logicalIterator.next().value === 'one', 'with selected first RHS yield');
        masks.value = true; target.value = false; gc();
        check(logicalIterator.next(2).value === 'two', 'selection retained through second RHS yield');
        check(logicalIterator.next(5).value === 7 && target.value === 7,
              'logical PutValue uses original selected object');
        function* hiddenTdz() { with ({ value: 4 }) { value += yield 'shadow'; } let value; }
        var shadow = hiddenTdz();
        check(shadow.next().value === 'shadow' && shadow.next(2).done,
              'successful with selection hides an uninitialized declarative fallback');
        function* missingTdz() { with ({}) { value += yield 'unreachable'; } let value; }
        var caught = null;
        try { missingTdz().next(); } catch (error) { caught = error; }
        check(caught instanceof ReferenceError, 'missed with selection checks original fallback TDZ before yield');
        print('generator-with-reference:ok');
    "#,
        "generator-with-reference:ok",
    );
}
