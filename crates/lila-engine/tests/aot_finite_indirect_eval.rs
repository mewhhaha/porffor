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
        "{:?}: {:?}\n{source}",
        outcome.completion,
        outcome.output_events
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
fn source_parameter_hints_preserve_defaults_and_ordinary_replaced_targets() {
    assert_trace(
        r#"
function create(ignored, target, source = '47;', result = target(source)) { return result; }
print(create(0, eval));
print(create(0, source => source));
function earlier(source, result = (0, eval)(source)) { return result; }
print(earlier('53;'));
"#,
        HostSurfacePolicy::Product,
        &["47", "47;", "53"],
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
let firstRealm = __lilaCreateRealm();
let secondRealm = __lilaCreateRealm();
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
fn saved_foreign_evaluators_keep_instance_getter_definitions_and_fresh_brands() {
    assert_trace(
        r#"
let firstRealm = __lilaCreateRealm();
let secondRealm = __lilaCreateRealm();
let firstEval = firstRealm.global.eval;
let secondEval = secondRealm.global.eval;
let text = `(class {
  get #value() { return 'test262'; }
  read(other) { return other.#value; }
  has(other) { return #value in other; }
})`;
let create = function(target) { return new (target(text)); };
let first = create(firstEval);
let second = create(secondEval);
let repeated = create(firstEval);
let peer = new first.constructor();
if (first.read(first) !== 'test262' || second.read(second) !== 'test262' ||
    repeated.read(repeated) !== 'test262' || first.read(peer) !== 'test262') {
  throw 'matching getter brand';
}
if (first.constructor === second.constructor || first.constructor === repeated.constructor ||
    !(first instanceof firstRealm.global.Object) || !(second instanceof secondRealm.global.Object)) {
  throw 'class evaluation identity';
}
if (!first.has(first) || !first.has(peer) || first.has(second) || first.has(repeated) ||
    !second.has(second) || second.has(first) || !repeated.has(repeated) || repeated.has(first)) {
  throw 'getter brand membership';
}
function rejects(operation, expected) {
  try { operation(); } catch (error) {
    if (error.constructor !== expected || error instanceof TypeError) throw 'wrong getter error realm';
    return;
  }
  throw 'getter brand check did not throw';
}
rejects(() => first.read(second), firstRealm.global.TypeError);
rejects(() => second.read(first), secondRealm.global.TypeError);
rejects(() => first.read(repeated), firstRealm.global.TypeError);
rejects(() => repeated.read(first), firstRealm.global.TypeError);
print('getter brands');
"#,
        HostSurfacePolicy::Test262,
        &["getter brands"],
    );
}

#[test]
fn foreign_instance_setters_and_methods_keep_their_defining_private_environment() {
    assert_trace(
        r#"
let firstRealm = __lilaCreateRealm();
let secondRealm = __lilaCreateRealm();
let firstEval = firstRealm.global.eval;
let secondEval = secondRealm.global.eval;
let text = `(class {
  #value = 0;
  writes = 0;
  set #write(value) { this.#value = value; this.writes++; }
  #method(increment) { this.#value += increment; return this.#value; }
  write(other, value) { other.#write = value; }
  call(other, increment) { return other.#method(increment); }
  read() { return this.#value; }
})`;
let create = function(target) { return new (target(text)); };
let first = create(firstEval);
let second = create(secondEval);
let repeated = create(firstEval);
let peer = new first.constructor();
first.write(first, 11);
second.write(second, 22);
repeated.write(repeated, 33);
first.write(peer, 44);
if (first.call(first, 1) !== 12 || second.call(second, 2) !== 24 ||
    repeated.call(repeated, 3) !== 36 || first.call(peer, 4) !== 48) {
  throw 'matching setter and method receiver';
}
function rejects(operation, expected) {
  try { operation(); } catch (error) {
    if (error.constructor !== expected || error instanceof TypeError) throw 'wrong instance error realm';
    return;
  }
  throw 'instance brand check did not throw';
}
rejects(() => first.write(second, 99), firstRealm.global.TypeError);
rejects(() => second.write(first, 99), secondRealm.global.TypeError);
rejects(() => first.write(repeated, 99), firstRealm.global.TypeError);
rejects(() => repeated.write(first, 99), firstRealm.global.TypeError);
rejects(() => first.call(second, 99), firstRealm.global.TypeError);
rejects(() => second.call(first, 99), secondRealm.global.TypeError);
rejects(() => first.call(repeated, 99), firstRealm.global.TypeError);
rejects(() => repeated.call(first, 99), firstRealm.global.TypeError);
if (first.read() !== 12 || second.read() !== 24 || repeated.read() !== 36 || peer.read() !== 48 ||
    first.writes !== 1 || second.writes !== 1 || repeated.writes !== 1 || peer.writes !== 1) {
  throw 'failed private writes changed state';
}
print('instance setter and method brands');
"#,
        HostSurfacePolicy::Test262,
        &["instance setter and method brands"],
    );
}

#[test]
fn foreign_static_fields_accessors_and_methods_keep_fresh_class_brands() {
    assert_trace(
        r#"
let firstRealm = __lilaCreateRealm();
let secondRealm = __lilaCreateRealm();
let firstEval = firstRealm.global.eval;
let secondEval = secondRealm.global.eval;
let text = `(class {
  static #field = 1;
  static #writes = 0;
  static get #accessor() { return this.#field; }
  static set #accessor(value) { this.#field = value; this.#writes++; }
  static #method(increment) { return this.#field + increment; }
  static field(other) { return other.#field; }
  static putField(other, value) { other.#field = value; }
  static get(other) { return other.#accessor; }
  static set(other, value) { other.#accessor = value; }
  static call(other, increment) { return other.#method(increment); }
  static writes() { return this.#writes; }
  static has(other) { return #field in other && #accessor in other && #method in other; }
})`;
let create = function(target) { return target(text); };
let first = create(firstEval);
let second = create(secondEval);
let repeated = create(firstEval);
if (first === second || first === repeated || first.field(first) !== 1 ||
    second.field(second) !== 1 || repeated.field(repeated) !== 1) throw 'static class evaluation identity';
first.putField(first, 2);
if (first.get(first) !== 2) throw 'matching static field write';
first.set(first, 11);
second.set(second, 22);
repeated.set(repeated, 33);
if (first.get(first) !== 11 || second.get(second) !== 22 || repeated.get(repeated) !== 33 ||
    first.call(first, 1) !== 12 || second.call(second, 2) !== 24 || repeated.call(repeated, 3) !== 36) {
  throw 'matching static accessor and method receiver';
}
if (!first.has(first) || first.has(second) || first.has(repeated) ||
    !second.has(second) || second.has(first) || !repeated.has(repeated) || repeated.has(first)) {
  throw 'static brand membership';
}
function rejects(operation, expected) {
  try { operation(); } catch (error) {
    if (error.constructor !== expected || error instanceof TypeError) throw 'wrong static error realm';
    return;
  }
  throw 'static brand check did not throw';
}
rejects(() => first.field(second), firstRealm.global.TypeError);
rejects(() => second.field(first), secondRealm.global.TypeError);
rejects(() => first.field(repeated), firstRealm.global.TypeError);
rejects(() => first.putField(second, 99), firstRealm.global.TypeError);
rejects(() => second.putField(first, 99), secondRealm.global.TypeError);
rejects(() => first.putField(repeated, 99), firstRealm.global.TypeError);
rejects(() => first.get(second), firstRealm.global.TypeError);
rejects(() => second.get(first), secondRealm.global.TypeError);
rejects(() => first.get(repeated), firstRealm.global.TypeError);
rejects(() => first.set(second, 99), firstRealm.global.TypeError);
rejects(() => second.set(first, 99), secondRealm.global.TypeError);
rejects(() => first.set(repeated, 99), firstRealm.global.TypeError);
rejects(() => first.call(second, 99), firstRealm.global.TypeError);
rejects(() => second.call(first, 99), secondRealm.global.TypeError);
rejects(() => first.call(repeated, 99), firstRealm.global.TypeError);
if (first.get(first) !== 11 || second.get(second) !== 22 || repeated.get(repeated) !== 33 ||
    first.writes() !== 1 || second.writes() !== 1 || repeated.writes() !== 1) {
  throw 'failed static writes changed state';
}
print('static brands');
"#,
        HostSurfacePolicy::Test262,
        &["static brands"],
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
let realm = __lilaCreateRealm();
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
let realm = __lilaCreateRealm();
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
