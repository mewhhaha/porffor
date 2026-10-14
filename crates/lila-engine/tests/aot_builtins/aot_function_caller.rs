use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn legacy_caller_is_null_only_on_ordinary_sloppy_functions() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
function inner() { return inner.caller; }
function outer() { return inner(); }
function strictOuter() { "use strict"; var result = inner(); return result; }
var descriptor = Object.getOwnPropertyDescriptor(inner, 'caller');
print(descriptor.value === null && !descriptor.writable && !descriptor.enumerable && !descriptor.configurable);
print(inner.caller === null && outer() === outer && strictOuter() === null);
print(outer.call(null) === outer && outer.apply(null) === outer && outer.bind(null)() === outer);
print(Function().caller === null);
function strictFunction() { "use strict"; }
var object = { method() {}, get accessor() {} };
var restricted = [
  strictFunction, () => {}, function* () {}, async function () {}, async () => {}, async function* () {},
  object.method, Object.getOwnPropertyDescriptor(object, 'accessor').get, class C {}, inner.bind(null),
  Function, Array, print
];
var absent = 0;
var throws = 0;
for (var index = 0; index < restricted.length; index++) {
  if (!Object.prototype.hasOwnProperty.call(restricted[index], 'caller')) absent++;
  try { restricted[index].caller; } catch (error) { if (error instanceof TypeError) throws++; }
}
print(absent === restricted.length && throws === restricted.length);
var poison = Object.getOwnPropertyDescriptor(Function.prototype, 'caller');
print(typeof poison.get === 'function' && poison.get === poison.set
  && !poison.enumerable && poison.configurable);
try { Function.prototype.caller; } catch (error) { print(error instanceof TypeError); }
try { Object.defineProperty(inner, 'caller', { value: strictFunction }); }
catch (error) { print(error instanceof TypeError && inner.caller === null); }
void 0;
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("caller property policy executes through Wasm AOT");
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 8]
    );
}

fn assert_reflection_script(source: &str, label: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{label}: {error}"));
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{label}: {:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine(
            "legacy-reflection-ok".to_string()
        )],
        "{label}"
    );
}

const REFLECTION_HARNESS: &str =
    include_str!("../fixtures/legacy_callable_reflection/canonical_harness.js");

#[test]
fn original_legacy_reflection_staging_bodies_keep_all_eleven_executions() {
    let cases = [
        (
            "arguments-property-access-in-function",
            include_str!(
                "../fixtures/legacy_callable_reflection/arguments-property-access-in-function.js"
            ),
        ),
        (
            "function-caller-skips-eval-frames",
            include_str!(
                "../fixtures/legacy_callable_reflection/function-caller-skips-eval-frames.js"
            ),
        ),
        (
            "function-properties",
            include_str!("../fixtures/legacy_callable_reflection/function-properties.js"),
        ),
        (
            "regress-577648-1",
            include_str!("../fixtures/legacy_callable_reflection/regress-577648-1.js"),
        ),
        (
            "regress-577648-2",
            include_str!("../fixtures/legacy_callable_reflection/regress-577648-2.js"),
        ),
        (
            "regress-584355",
            include_str!("../fixtures/legacy_callable_reflection/regress-584355.js"),
        ),
        (
            "regress-586482-1",
            include_str!("../fixtures/legacy_callable_reflection/regress-586482-1.js"),
        ),
        (
            "regress-586482-2",
            include_str!("../fixtures/legacy_callable_reflection/regress-586482-2.js"),
        ),
        (
            "regress-586482-3",
            include_str!("../fixtures/legacy_callable_reflection/regress-586482-3.js"),
        ),
        (
            "regress-586482-4",
            include_str!("../fixtures/legacy_callable_reflection/regress-586482-4.js"),
        ),
    ];
    for (name, body) in cases {
        let source = format!("{REFLECTION_HARNESS}\n{body}\nprint('legacy-reflection-ok');\n");
        assert_reflection_script(&source, name);
        if name == "regress-584355" {
            let strict = format!("'use strict';\n{source}");
            assert_reflection_script(&strict, "strict regress-584355");
        }
    }
}

#[test]
fn recursive_arguments_and_abrupt_exits_restore_each_actual_activation() {
    let body = r#"
var saved = [], marker = {}, trace = '';
function recursive(depth, token) {
  var active = recursive.arguments;
  saved.push(active);
  assert.sameValue(active, arguments);
  assert.sameValue(active[0], depth);
  assert.sameValue(active[1], token);
  assert.sameValue(active.callee, recursive);
  assert.sameValue(recursive.caller, depth === 2 ? outer : recursive);
  // Legal SameValue redefinitions and freezing retain the actual descriptors;
  // restoration must still remove the ended invocation's values.
  Object.defineProperty(recursive, 'caller', {value: recursive.caller});
  Object.defineProperty(recursive, 'arguments', {value: recursive.arguments});
  Object.freeze(recursive);
  if (depth) {
    var nested = recursive(depth - 1, token + 1);
    assert.sameValue(nested, token + 1);
    assert.sameValue(recursive.arguments, active);
    assert.sameValue(recursive.arguments[1], token);
  }
  assert.sameValue(recursive.arguments, active);
  return token;
}
function failure() {
  assert.sameValue(failure.caller, outer);
  assert.sameValue(failure.arguments, arguments);
  throw marker;
}
function probe() { return probe.caller; }
function outer() {
  var active = outer.arguments;
  assert.sameValue(recursive(2, 30), 30);
  assert.sameValue(recursive.arguments, null);
  assert.sameValue(recursive.caller, null);
  try { failure(9); } catch (error) {
    assert.sameValue(error, marker);
    assert.sameValue(failure.arguments, null);
    assert.sameValue(failure.caller, null);
    assert.sameValue(probe(), outer);
    trace += 'c';
  } finally {
    assert.sameValue(outer.arguments, active);
    assert.sameValue(probe(), outer);
    trace += 'f';
  }
  return active;
}
var first = outer(17);
assert.sameValue(first[0], 17);
assert.sameValue(first.callee, outer);
assert.sameValue(trace, 'cf');
assert.sameValue(saved.length, 3);
assert.sameValue(saved[0] === saved[1], false);
assert.sameValue(saved[1] === saved[2], false);
assert.sameValue(outer.arguments, null);
assert.sameValue(outer.caller, null);
assert.sameValue(probe(), null);
print('legacy-reflection-ok');
"#;
    assert_reflection_script(
        &format!("{REFLECTION_HARNESS}\n{body}"),
        "recursive and abrupt activation restoration",
    );
}

#[test]
fn private_arguments_survive_source_binding_replacements_and_mapped_writes() {
    let body = r#"
function overwritten(value) {
  var actual = overwritten.arguments;
  assert.sameValue(actual, arguments);
  value = 12;
  assert.sameValue(actual[0], 12);
  actual[0] = 13;
  assert.sameValue(value, 13);
  arguments = 'replacement';
  assert.sameValue(overwritten.arguments, actual);
  assert.sameValue(overwritten.arguments[0], 13);
  return actual;
}
function named(arguments, tail) {
  var actual = named.arguments;
  assert.sameValue(arguments, 'source-parameter');
  assert.sameValue(actual[0], 'source-parameter');
  assert.sameValue(actual[1], tail);
  assert.sameValue(actual.length, 2);
  arguments = 'changed';
  assert.sameValue(named.arguments, actual);
  assert.sameValue(actual[0], 'changed');
  return actual;
}
function nonSimple(value = 4) {
  assert.sameValue(nonSimple.arguments, arguments);
  assert.sameValue(arguments[0], 8);
  value = 99;
  assert.sameValue(nonSimple.arguments[0], 8);
  assert.throws(TypeError, function() { nonSimple.arguments.callee; });
  return nonSimple.arguments;
}
var actual = overwritten(1);
assert.sameValue(actual[0], 13);
assert.sameValue(overwritten.arguments, null);
assert.sameValue(named('source-parameter', 6)[0], 'changed');
assert.sameValue(named.arguments, null);
assert.sameValue(nonSimple(8)[0], 8);
assert.sameValue(nonSimple.arguments, null);
var descriptor = Object.getOwnPropertyDescriptor(overwritten, 'arguments');
assert.sameValue(descriptor.value, null);
assert.sameValue(descriptor.writable, false);
assert.sameValue(descriptor.enumerable, false);
assert.sameValue(descriptor.configurable, false);
print('legacy-reflection-ok');
"#;
    assert_reflection_script(
        &format!("{REFLECTION_HARNESS}\n{body}"),
        "private arguments identity and mapping",
    );
}

#[test]
fn strict_nonordinary_and_native_callers_are_opaque_and_forwarders_are_transparent() {
    let body = r#"
function inspect() { return inspect.caller; }
function forwarding() {
  assert.sameValue(inspect.call(null), forwarding);
  assert.sameValue(inspect.apply(null, []), forwarding);
  assert.sameValue(Reflect.apply(inspect, null, []), forwarding);
  assert.sameValue(inspect.bind(null)(), forwarding);
  assert.sameValue(eval('inspect();'), forwarding);
  assert.sameValue(eval("eval('inspect();');"), forwarding);
  var fn = inspect.call;
  assert.sameValue(fn.call(inspect, null), forwarding);
  var old = inspect.call;
  inspect.call = function replacement() { return inspect(); };
  assert.sameValue(inspect.call(), inspect.call);
  inspect.call = old;
  assert.sameValue([1].map(inspect)[0], null);
  function* generator() { yield inspect(); }
  assert.sameValue(generator().next().value, null);
  var arrow = () => inspect();
  assert.sameValue(arrow(), null);
  var object = { method() { return inspect(); } };
  assert.sameValue(object.method(), null);
  return inspect();
}
assert.sameValue(forwarding(), forwarding);
function strictCaller() {
  'use strict';
  var result = inspect();
  assert.sameValue(result, null);
  assert.throws(TypeError, function() { strictCaller.caller; });
  assert.throws(TypeError, function() { strictCaller.arguments; });
  return result;
}
function top() {
  assert.sameValue(strictCaller(), null);
  assert.sameValue(inspect(), top);
}
top();
assert.sameValue(inspect.caller, null);
assert.sameValue(inspect.arguments, null);
print('legacy-reflection-ok');
"#;
    assert_reflection_script(
        &format!("{REFLECTION_HARNESS}\n{body}"),
        "opaque and transparent actual callers",
    );
}

#[test]
fn activation_restoration_covers_parameter_throws_construction_and_suspension() {
    let body = r#"
var marker = {}, pending;
function probe() { return probe.caller; }
function defaultFailure() {
  assert.sameValue(defaultFailure.caller, defaulted);
  assert.sameValue(defaulted.arguments.length, 0);
  throw marker;
}
function defaulted(value = defaultFailure()) { return value; }
function Constructed(value) {
  assert.sameValue(Constructed.caller, outer);
  assert.sameValue(Constructed.arguments, arguments);
  this.value = value;
}
function nativeCallback() {
  assert.sameValue(nativeCallback.caller, null);
  throw marker;
}
function* suspended() {
  assert.sameValue(probe(), null);
  yield 3;
  assert.sameValue(probe(), null);
  return 4;
}
async function asynchronous() {
  assert.sameValue(probe(), null);
  await Promise.resolve(5);
  assert.sameValue(probe(), null);
  return 5;
}
function outer() {
  try { defaulted(); } catch (error) { assert.sameValue(error, marker); }
  assert.sameValue(defaulted.caller, null);
  assert.sameValue(defaulted.arguments, null);
  assert.sameValue(probe(), outer);
  assert.sameValue(new Constructed(7).value, 7);
  assert.sameValue(Constructed.caller, null);
  assert.sameValue(Constructed.arguments, null);
  try { [1].map(nativeCallback); } catch (error) { assert.sameValue(error, marker); }
  assert.sameValue(nativeCallback.caller, null);
  assert.sameValue(nativeCallback.arguments, null);
  assert.sameValue(probe(), outer);
  var generator = suspended();
  assert.sameValue(probe(), outer);
  assert.sameValue(generator.next().value, 3);
  assert.sameValue(probe(), outer);
  assert.sameValue(generator.next().value, 4);
  assert.sameValue(probe(), outer);
  pending = asynchronous();
  assert.sameValue(probe(), outer);
}
outer();
assert.sameValue(probe(), null);
pending.then(function finished(value) {
  assert.sameValue(value, 5);
  assert.sameValue(probe(), finished);
  print('legacy-reflection-ok');
});
"#;
    assert_reflection_script(
        &format!("{REFLECTION_HARNESS}\n{body}"),
        "parameter, constructor, native throw and resumable activation exits",
    );
}

#[test]
fn original_legacy_arguments_property_bodies_keep_both_execution_ids() {
    for (label, body) in [
        (
            "10.6-13-a-2",
            include_str!("../fixtures/legacy_callable_reflection/10.6-13-a-2.js"),
        ),
        (
            "10.6-13-a-3",
            include_str!("../fixtures/legacy_callable_reflection/10.6-13-a-3.js"),
        ),
    ] {
        let source = format!(
            "{REFLECTION_HARNESS}\n{body}\nassert(test1.caller === null && test2.caller === null);\nprint('legacy-reflection-ok');\n"
        );
        assert_reflection_script(&source, label);
    }
}
