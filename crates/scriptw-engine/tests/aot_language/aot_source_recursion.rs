use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_source_recursion(source: &str, expected: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("source recursion must complete through Wasm with catchable JS exceptions");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
        "{source}"
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(expected.to_owned())],
        "{source}"
    );
}

#[test]
fn original_pinned_recursion_fixture_completes_both_recursive_phases() {
    const ORIGINAL: &str = include_str!("../fixtures/source_recursion/original_sm_recursion.js");
    assert_eq!(
        ORIGINAL,
        include_str!("../../../../test262/vendor/test262/test/staging/sm/extensions/recursion.js"),
        "retain the complete pinned fixture, including both original catches and gc"
    );
    let source = format!(
        "var $262 = {{ gc: gc }};\n{ORIGINAL}\nprint('original-source-recursion:ok');\ntrue;"
    );
    assert_source_recursion(&source, "original-source-recursion:ok");
}

#[test]
fn recursion_throw_unwinds_finalizers_and_preserves_captured_roots_for_recovery() {
    assert_source_recursion(
        r#"
var captures = [];
var entered = 0;
var finalized = 0;
function cycle() {
  var current = { ordinal: entered++ };
  if (captures.length < 4) captures.push(() => current);
  try { cycle(); }
  finally { finalized++; }
}
var firstCount;
for (var attempt = 0; attempt < 4; attempt++) {
  captures = [];
  entered = 0;
  finalized = 0;
  var error;
  try { cycle(); } catch (caught) { error = caught; }
  if (!error || Object.getPrototypeOf(error) !== RangeError.prototype)
    throw new Error('recursion must throw RangeError');
  if (entered === 0 || finalized !== entered || captures.length !== 4)
    throw new Error('source finalizer or entry ordering');
  if (attempt === 0) firstCount = entered;
  else if (entered !== firstCount) throw new Error('source depth leaked after throw');
  gc();
  for (var index = 0; index < captures.length; index++) {
    if (captures[index]().ordinal !== index) throw new Error('lost captured invocation');
  }
  if (cycle.caller !== null || cycle.arguments !== null)
    throw new Error('source reflection leaked after throw');
}
var recovered = 0;
function internallyCaught() {
  try { internallyCaught(); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== RangeError.prototype) throw error;
    recovered++;
  }
  return 17;
}
for (var attempt = 0; attempt < 4; attempt++) {
  if (internallyCaught() !== 17) throw new Error('normal return after inner catch');
}
if (recovered !== 4) throw new Error('source depth leaked after caught completion');
function finite(n) { if (n === 0) return 23; return finite(n - 1); }
if (finite(20) !== 23) throw new Error('finite call after exhaustion');
print('recursion-recovery:ok');
true;
"#,
        "recursion-recovery:ok",
    );
}

#[test]
fn parameters_constructors_and_foreign_realm_entries_use_the_checked_body_boundary() {
    assert_source_recursion(
        r#"
var bodies = 0;
function parameters(value = parameters()) { bodies++; }
var error;
try { parameters(); } catch (caught) { error = caught; }
if (!error || Object.getPrototypeOf(error) !== RangeError.prototype || bodies !== 0)
  throw new Error('parameter recursion entered a rejected body');
class RecursiveConstructor {
  constructor() { new RecursiveConstructor(); }
}
error = undefined;
try { new RecursiveConstructor(); } catch (caught) { error = caught; }
if (!error || Object.getPrototypeOf(error) !== RangeError.prototype)
  throw new Error('constructor recursion');
var other = __scriptwCreateRealm().global;
var foreign = other.eval('(function anotherCycle() { anotherCycle(); })');
for (var attempt = 0; attempt < 2; attempt++) {
  error = undefined;
  try { foreign(); } catch (caught) { error = caught; }
  if (!error || Object.getPrototypeOf(error) !== other.RangeError.prototype)
    throw new Error('recursion error defining realm');
  if (Object.getPrototypeOf({}) !== Object.prototype)
    throw new Error('caller realm after recursion throw');
}
print('recursion-entry-realms:ok');
true;
"#,
        "recursion-entry-realms:ok",
    );
}

#[test]
fn generator_resumption_retires_depth_on_yield_return_and_throw() {
    assert_source_recursion(
        r#"
var token = {};
var calls = 0;
function advanceRecursively() { calls++; advanceRecursively(); }
function* values(value) {
  var saved = value;
  try {
    yield () => saved;
    try { advanceRecursively(); }
    catch (error) {
      if (Object.getPrototypeOf(error) !== RangeError.prototype) throw error;
    }
    yield saved;
    return saved;
  } finally { saved = token; }
}
for (var index = 0; index < 256; index++) {
  var iterator = values(index);
  var first = iterator.next();
  if (first.done || first.value() !== index) throw new Error('initial yield');
  if (index % 3 === 0) {
    var closed = iterator.return(token);
    if (!closed.done || closed.value !== token) throw new Error('Return retirement');
  } else if (index % 3 === 1) {
    var caught;
    try { iterator.throw(token); } catch (error) { caught = error; }
    if (caught !== token) throw new Error('Throw retirement');
  } else {
    var second = iterator.next();
    var last = iterator.next();
    if (second.done || second.value !== index || !last.done || last.value !== index)
      throw new Error('resume completion');
  }
  gc();
  if (first.value() !== token) throw new Error('retained closure after cleanup');
}
if (calls === 0) throw new Error('missing generator recursion');
print('recursion-generator-retirement:ok');
true;
"#,
        "recursion-generator-retirement:ok",
    );
}

#[test]
fn async_and_async_generator_suspension_leave_no_active_depth() {
    assert_source_recursion(
        r#"
var token = {};
async function* values() { yield token; await 0; yield token; return token; }
function cycle() { cycle(); }
async function resumed() {
  for (var index = 0; index < 256; index++) {
    await 0;
    var error;
    try { cycle(); } catch (caught) { error = caught; }
    if (!error || Object.getPrototypeOf(error) !== RangeError.prototype)
      throw new Error('recursion after await');
    var iterator = values();
    var first = await iterator.next();
    var second = await iterator.next();
    var last = await iterator.next();
    if (first.done || first.value !== token || second.done || second.value !== token
        || !last.done || last.value !== token)
      throw new Error('async generator retirement');
  }
  print('recursion-async-retirement:ok');
}
resumed();
true;
"#,
        "recursion-async-retirement:ok",
    );
}

#[test]
fn proper_tail_calls_retire_source_depth_before_one_hundred_thousand_steps() {
    assert_source_recursion(
        r#"
var token = {};
function step(n) {
  'use strict';
  if (n === 0) return token;
  return step(n - 1);
}
var proxy = new Proxy(step, {});
function forwarded(n) {
  'use strict';
  if (n === 0) return token;
  return Reflect.apply(forwarded, undefined, [n - 1]);
}
if (proxy(100000) !== token || forwarded(100000) !== token)
  throw new Error('tail activation depth must retire');
function cycle() { cycle(); }
var error;
try { cycle(); } catch (caught) { error = caught; }
if (!error || Object.getPrototypeOf(error) !== RangeError.prototype || step(100000) !== token)
  throw new Error('tail call after recursion recovery');
print('recursion-tail-retirement:ok');
true;
"#,
        "recursion-tail-retirement:ok",
    );
}
