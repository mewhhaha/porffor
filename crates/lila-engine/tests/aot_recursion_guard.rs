use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_recursion_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
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
        .unwrap_or_else(|error| panic!("recursion must remain catchable: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn stack_exhaustion_runs_finally_and_releases_budget_after_unwind() {
    assert_recursion_script(
        r#"
        let entered = 0;
        let unwound = 0;
        function recurse() {
            entered++;
            try { return recurse(); }
            finally { unwound++; }
        }
        let errors = [];
        for (let attempt = 0; attempt < 2; attempt++) {
            try { recurse(); } catch (error) { errors.push(error); }
            if (entered === 0 || entered !== unwound) throw new Error('finally did not unwind');
            entered = 0;
            unwound = 0;
        }
        errors.length === 2 && errors[0] !== errors[1] &&
          errors.every(error => error instanceof RangeError && error.name === 'RangeError');
    "#,
    );
}

#[test]
fn ordinary_eval_binding_and_generator_recursion_are_catchable() {
    assert_recursion_script(
        r#"
        function eval() { return eval(); }
        let ordinary;
        try { eval(); } catch (error) { ordinary = error; }
        function* recursiveGenerator() {
            for (const value of recursiveGenerator()) yield value;
        }
        let generator;
        try { recursiveGenerator().next(); } catch (error) { generator = error; }
        ordinary instanceof RangeError && generator instanceof RangeError;
    "#,
    );
}

#[test]
fn strict_tail_recursion_keeps_reusing_native_frames_through_guard_wrappers() {
    assert_recursion_script(
        r#"
        function countdown(n) {
            "use strict";
            return n === 0 ? true : countdown(n - 1);
        }
        countdown(400000);
    "#,
    );
}

#[test]
fn recursion_errors_use_the_executing_function_realm() {
    assert_recursion_script(
        r#"
        const other = __lilaCreateRealm();
        // This literal is prepared by the compiler; no runtime source parser
        // is bundled into the artifact.
        const recurse = other.evalScript('(function recurse() { return recurse(); })');
        let caught;
        try { recurse(); } catch (error) { caught = error; }
        caught instanceof other.global.RangeError && !(caught instanceof RangeError);
    "#,
    );
}

#[test]
fn guarded_host_calls_accept_their_declared_environment_layouts() {
    assert_recursion_script(
        r#"
        const foreign = __lilaCreateRealm().global;
        const parse = parseInt;
        const parseForeign = foreign.parseFloat;
        const output = print;
        print('direct');
        output('indirect');
        parseInt('12', 10) === 12 && parse('13', 10) === 13 &&
          parseFloat('14.5') === 14.5 && Number.parseInt('15') === 15 &&
          parseForeign('16.5') === 16.5 && foreign.Number.parseInt('17') === 17;
    "#,
    );
}
