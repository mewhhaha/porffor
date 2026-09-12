use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};
use lila_ir::DynamicSourceRuntimeOperation;

fn assert_trace(source: &str, policy: HostSurfacePolicy, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("finite indirect eval compiles and executes through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn forwarded_private_instance_classes_have_fresh_brands() {
    assert_trace(
        r#"
let text = `(class { get #value() { return 17; } read(other) { return other.#value; } })`;
let create = function(target) { return new (target(text)); };
let first = create(eval);
let second = create(eval);
print(first.read(first));
print(second.read(second));
print(first.constructor !== second.constructor);
try { first.read(second); } catch (error) { print(error instanceof TypeError); }
try { second.read(first); } catch (error) { print(error instanceof TypeError); }
"#,
        HostSurfacePolicy::Product,
        &["17", "17", "true", "true", "true"],
    );
}

#[test]
fn forwarded_private_static_classes_have_fresh_brands() {
    assert_trace(
        r#"
let text = `(class { static #value = 19; static read() { return this.#value; } })`;
let create = function(target) { return target(text); };
let first = create(eval);
let second = create(eval);
print(first.read());
print(second.read());
print(first !== second);
try { first.read.call(second); } catch (error) { print(error instanceof TypeError); }
try { second.read.call(first); } catch (error) { print(error instanceof TypeError); }
"#,
        HostSurfacePolicy::Product,
        &["19", "19", "true", "true", "true"],
    );
}

#[test]
fn foreign_eval_preserves_realm_and_private_environment_identity() {
    assert_trace(
        r#"
let firstRealm = $262.createRealm();
let secondRealm = $262.createRealm();
let text = `(class { #value = 21; read(other) { return other.#value; } })`;
let create = function(target) { return new (target(text)); };
let first = create(firstRealm.global.eval);
let second = create(secondRealm.global.eval);
print(first.read(first));
print(second.read(second));
print(first instanceof firstRealm.global.Object);
print(second instanceof secondRealm.global.Object);
try { first.read(second); } catch (error) {
  print(error instanceof firstRealm.global.TypeError);
  print(!(error instanceof TypeError));
}
try { second.read(first); } catch (error) { print(error instanceof secondRealm.global.TypeError); }
"#,
        HostSurfacePolicy::Test262,
        &["21", "21", "true", "true", "true", "true", "true"],
    );
}

#[test]
fn candidate_hints_preserve_live_sources_and_replaced_callable_identity() {
    assert_trace(
        r#"
let text = '31;';
function invoke(target) { return target(text); }
print(invoke(eval));
text = '32;';
print(invoke(eval));
let called = 0;
print(invoke(function(source) { called++; return source; }));
print(called);
let saved = eval;
let selected = saved;
print(selected(text, (selected = function() { return 99; })));
print(selected(text));
"#,
        HostSurfacePolicy::Product,
        &["31", "32", "32;", "1", "32", "99"],
    );
}

#[test]
fn callable_get_and_argument_abrupt_completion_keep_their_original_order() {
    assert_trace(
        r#"
let text = '41;';
let original = eval;
let trace = '';
let holder = { get eval() { trace += 'get;'; return original; } };
print(holder.eval((trace += 'arg;', text), (trace += 'extra;')));
print(trace);
let marker = {};
function fail() { trace += 'throw;'; throw marker; }
try { holder.eval(fail()); } catch (error) { print(error === marker); }
print(trace);
let calls = 0;
let object = { toString() { calls++; return '99;'; } };
print(holder.eval(object) === object);
print(calls);
"#,
        HostSurfacePolicy::Product,
        &[
            "41",
            "get;arg;extra;",
            "true",
            "get;arg;extra;get;throw;",
            "true",
            "0",
        ],
    );
}

#[test]
fn call_apply_empty_spread_and_realm_script_share_finite_text_candidates() {
    assert_trace(
        r#"
let text = '43;';
print(eval.call(undefined, text));
print(eval.apply(undefined, [text]));
print(Reflect.apply(eval, undefined, [text]));
print((0, eval)(...[], text, 'not source'));
let realm = $262.createRealm();
print(realm.evalScript(text));
"#,
        HostSurfacePolicy::Test262,
        &["43", "43", "43", "43", "43"],
    );
}

#[test]
fn syntax_errors_are_deferred_to_the_selected_eval_realm() {
    assert_trace(
        r#"
let realm = $262.createRealm();
let text = 'let = ;';
function invoke(target) { return target(text); }
print(invoke(function(source) { return source; }));
try { invoke(eval); } catch (error) { print(error instanceof SyntaxError); }
try { invoke(realm.global.eval); } catch (error) {
  print(error instanceof realm.global.SyntaxError);
  print(!(error instanceof SyntaxError));
}
print('after');
"#,
        HostSurfacePolicy::Test262,
        &["let = ;", "true", "true", "true", "after"],
    );
}

#[test]
fn a_source_outside_the_prepared_candidates_retains_its_runtime_capability_failure() {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let error = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"
let text = '23;';
(0, eval)(text);
text += '/*' + Math.random() + '*/';
(0, eval)(text);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect_err("a prepared candidate cannot replace a different runtime source");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval],
        "{error}"
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
}
