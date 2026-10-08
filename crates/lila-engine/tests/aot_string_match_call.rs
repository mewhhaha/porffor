use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
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
                ..RunOptions::default()
            },
        )
        .expect("String match calls must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn unchanged_strict_named_lookbehind_fixture_executes_with_pinned_assertions() {
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    const COMPARE_ARRAY: &str =
        include_str!("../../../test262/vendor/test262/harness/compareArray.js");
    const LOOKBEHIND: &str = include_str!(
        "../../../test262/vendor/test262/test/built-ins/RegExp/named-groups/lookbehind.js"
    );
    assert_wasm_true(&format!(
        "'use strict';\n{STA}\n{ASSERT}\n{COMPARE_ARRAY}\n{LOOKBEHIND}\ntrue;"
    ));
}

#[test]
fn inherited_match_getter_runs_before_all_arguments_and_preserves_receiver() {
    assert_wasm_true(
        r#"
var trace = '', getterThis, callThis, count, first, second;
var prototype = {};
Object.defineProperty(prototype, 'match', { get: function() {
  getterThis = this; trace += 'get;';
  return function(a, b) {
    callThis = this; count = arguments.length; first = a; second = b;
    trace += 'call;'; return 19;
  };
}});
var receiver = Object.create(prototype);
receiver.toString = function() { throw 'receiver must not be coerced'; };
function argument(value) {
  trace += value + ';';
  Object.defineProperty(receiver, 'match', {
    configurable: true, value: function() { throw 'method reread'; }
  });
  return value;
}
var result = receiver.match(argument('first'), argument('second'));
result === 19 && getterThis === receiver && callThis === receiver && count === 2 &&
  first === 'first' && second === 'second' && trace === 'get;first;second;call;';
"#,
    );
}

#[test]
fn replaced_string_match_accessor_observes_the_primitive_receiver() {
    assert_wasm_true(
        r#"
var saved = Object.getOwnPropertyDescriptor(String.prototype, 'match');
var getterThis, callThis, count, trace = '', result;
Object.defineProperty(String.prototype, 'match', {
  configurable: true,
  get: function() {
    'use strict'; getterThis = this; trace += 'get;';
    return function(a, b) {
      'use strict'; callThis = this; count = arguments.length;
      trace += 'call;'; return a + b;
    };
  }
});
try { result = 'text'.match((trace += 'first;', 2), (trace += 'second;', 5)); }
finally { Object.defineProperty(String.prototype, 'match', saved); }
result === 7 && getterThis === 'text' && callThis === 'text' && count === 2 &&
  trace === 'get;first;second;call;' && 'ab'.match(/b/)[0] === 'b';
"#,
    );
}

#[test]
fn intrinsic_match_reads_the_symbol_hook_after_arguments_without_coercing_receiver() {
    assert_wasm_true(
        r#"
var trace = '', hookThis, hookArgument, hookCount, result = {};
var receiver = {
  match: String.prototype.match,
  toString: function() { throw 'unexpected receiver conversion'; }
};
var pattern = {};
Object.defineProperty(pattern, Symbol.match, { get: function() {
  trace += 'hook-get;';
  if (this !== pattern) throw 'hook getter receiver';
  return function(value) {
    hookThis = this; hookArgument = value; hookCount = arguments.length;
    trace += 'hook-call;'; return result;
  };
}});
var actual = receiver.match((trace += 'first;', pattern), (trace += 'ignored;', 1));
actual === result && hookThis === pattern && hookArgument === receiver && hookCount === 1 &&
  trace === 'first;ignored;hook-get;hook-call;';
"#,
    );
}

#[test]
fn missing_symbol_hook_coerces_receiver_then_pattern_after_all_arguments() {
    assert_wasm_true(
        r#"
var trace = '';
var receiver = {
  match: String.prototype.match,
  toString: function() { trace += 'receiver;'; return 'abc'; }
};
var pattern = { toString: function() { trace += 'pattern;'; return 'b'; } };
Object.defineProperty(pattern, Symbol.match, { get: function() { trace += 'hook;'; return null; } });
var result = receiver.match((trace += 'first;', pattern), (trace += 'ignored;', 0));
result[0] === 'b' && result.index === 1 && result.input === 'abc' &&
  trace === 'first;ignored;hook;receiver;pattern;';
"#,
    );
}

#[test]
fn match_call_abrupt_completions_reach_catch_and_finally_in_evaluation_order() {
    assert_wasm_true(
        r#"
var marker = {}, trace = '', caught = 0;
function fail() { throw marker; }
var getter = {};
Object.defineProperty(getter, 'match', { get: fail });
try { getter.match(trace += 'unreached;'); }
catch (error) { if (error === marker) caught++; }
finally { trace += 'getter;'; }
var receiver = { match: function() { trace += 'unreached;'; } };
try { receiver.match(fail(), trace += 'unreached;'); }
catch (error) { if (error === marker) caught++; }
finally { trace += 'argument;'; }
try { ({ match: 0 }).match(trace += 'evaluated;'); }
catch (error) { if (error instanceof TypeError) caught++; }
finally { trace += 'not-callable;'; }
try { (void 0).match(trace += 'unreached;'); }
catch (error) { if (error instanceof TypeError) caught++; }
finally { trace += 'nullish;'; }
var hook = {};
Object.defineProperty(hook, Symbol.match, { get: fail });
try { 'abc'.match(hook); }
catch (error) { if (error === marker) caught++; }
finally { trace += 'hook;'; }
var coercible = { match: String.prototype.match, toString: fail };
try { coercible.match(undefined); }
catch (error) { if (error === marker) caught++; }
finally { trace += 'coercion;'; }
caught === 6 && trace === 'getter;argument;evaluated;not-callable;nullish;hook;coercion;';
"#,
    );
}

#[test]
fn proxy_match_get_and_apply_keep_receiver_arguments_and_thrown_identity() {
    assert_wasm_true(
        r#"
var marker = {}, trace = '', seenReceiver, seenThis, seenArgs;
var method = new Proxy(function() { throw 'target must not run'; }, {
  apply: function(target, receiver, args) {
    seenThis = receiver; seenArgs = args; trace += 'apply;'; return 23;
  }
});
var receiver = new Proxy({}, { get: function(target, key, actualReceiver) {
  if (key !== 'match') throw 'unexpected key';
  seenReceiver = actualReceiver; trace += 'get;'; return method;
}});
var result = receiver.match((trace += 'first;', 7), (trace += 'second;', 8));
var throwing = { match: new Proxy(function() {}, { apply: function() { throw marker; } }) };
var caught = false;
try { throwing.match(1); } catch (error) { caught = error === marker; }
result === 23 && seenReceiver === receiver && seenThis === receiver &&
  seenArgs.length === 2 && seenArgs[0] === 7 && seenArgs[1] === 8 && caught &&
  trace === 'get;first;second;apply;';
"#,
    );
}

#[test]
fn spread_arguments_finish_before_the_shared_match_operation() {
    assert_wasm_true(
        r#"
var trace = '', count = 0, received, receiver;
var iterable = {};
iterable[Symbol.iterator] = function() {
  trace += 'iterator;';
  return { next: function() {
    trace += 'next;'; count++;
    return count < 3 ? { value: count, done: false } : { done: true };
  }};
};
receiver = { match: function(a, b) {
  received = this; trace += 'call;'; return arguments.length === 2 && a === 1 && b === 2;
}};
var result = receiver.match(...iterable);
result && received === receiver && trace === 'iterator;next;next;next;call;';
"#,
    );
}

fn assert_symbol_modes(source: &str, expected: ObservedJsValue) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
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
            .expect("String symbol-method controls execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(expected.clone()),
            "{source}"
        );
        assert!(observed.output_events.is_empty(), "{source}");
    }
}

#[test]
fn all_six_symbol_hooks_preserve_proxy_arguments_bypasses_and_abrupt_identity() {
    assert_symbol_modes(
        include_str!("../../lila-cli/tests/fixtures/wasm_string_symbol_hooks.js"),
        ObservedJsValue::Number(ObservedNumber::from_f64(262.0)),
    );
}

#[test]
fn match_all_gets_the_original_once_then_invokes_the_created_receiver() {
    assert_symbol_modes(
        include_str!("../../lila-cli/tests/fixtures/wasm_string_match_all_flags_single_read.js"),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn borrowed_string_hooks_and_created_invokes_use_saved_defining_realm_intrinsics() {
    assert_symbol_modes(
        r#"
const other = __lilaCreateRealm().global;
const LocalObject = Object, ForeignObject = other.Object;
const ForeignError = other.Error;
const marker = new ForeignError('string-symbol-marker');
const localTypeErrorPrototype = TypeError.prototype;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const localRegExpPrototype = RegExp.prototype;
const foreignRegExpPrototype = other.RegExp.prototype;
const names = ['match', 'matchAll', 'replace', 'replaceAll', 'search', 'split'];
const symbols = [Symbol.match, Symbol.matchAll, Symbol.replace, Symbol.replace, Symbol.search, Symbol.split];
const localMethods = names.map(name => String.prototype[name]);
const foreignMethods = names.map(name => other.String.prototype[name]);
const createdIndexes = [0, 1, 4];
function expectTypeError(action, prototype) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'String native error Realm';
    return;
  }
  throw 'missing String TypeError';
}
function expectMarker(action) {
  let prior = 'before', finallyRuns = 0;
  try {
    try { prior = action(); } finally { finallyRuns++; }
    throw 'missing String original marker';
  } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'String original abrupt identity';
  }
  if (prior !== 'before' || finallyRuns !== 1) throw 'String abrupt prior assignment';
}
globalThis.TypeError = function() { throw 'mutable local TypeError'; };
other.TypeError = function() { throw 'mutable foreign TypeError'; };
globalThis.RegExp = function() { throw 'mutable local RegExp'; };
other.RegExp = function() { throw 'mutable foreign RegExp'; };
let directCases = 0, createdCases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const errorPrototype = direction === 0 ? foreignTypeErrorPrototype : localTypeErrorPrototype;
  const regexpPrototype = direction === 0 ? foreignRegExpPrototype : localRegExpPrototype;
  const ObjectConstructor = direction === 0 ? LocalObject : ForeignObject;
  for (let index = 0; index < methods.length; index++) {
    const method = methods[index], symbol = symbols[index];
    const receiver = new ObjectConstructor();
    receiver.toString = function() { throw 'noncallable hook must precede receiver conversion'; };
    const pattern = new ObjectConstructor();
    if (index !== 0) pattern[Symbol.match] = false;
    Object.defineProperty(pattern, symbol, { value: 0 });
    expectTypeError(() => method.call(receiver, pattern, {}), errorPrototype);
    let hookGets = 0;
    const nullishPattern = {};
    Object.defineProperty(nullishPattern, symbol, { get() { hookGets++; throw 'hook after null receiver'; } });
    expectTypeError(() => method.call(null, nullishPattern, {}), errorPrototype);
    if (hookGets !== 0) throw 'RequireObjectCoercible before original hook';
    directCases++;
  }
  for (let position = 0; position < createdIndexes.length; position++) {
    const index = createdIndexes[position], method = methods[index], symbol = symbols[index];
    const saved = Object.getOwnPropertyDescriptor(regexpPrototype, symbol);
    let gets = 0, calls = 0, conversions = 0, createdReceiver;
    const result = {}, target = function() { throw 'created Proxy target'; };
    const hook = new Proxy(target, { apply(actualTarget, receiver, args) {
      calls++;
      if (actualTarget !== target || receiver !== createdReceiver ||
          Object.getPrototypeOf(receiver) !== regexpPrototype || receiver.source !== 'b' ||
          receiver.flags !== (index === 1 ? 'g' : '') || args.length !== 1 || args[0] !== 'abc') {
        throw 'borrowed created RegExp Reference';
      }
      return result;
    }});
    Object.defineProperty(regexpPrototype, symbol, { configurable: true, get() {
      gets++;
      createdReceiver = this;
      return hook;
    }});
    const receiver = new ObjectConstructor();
    receiver.toString = function() { conversions++; return 'abc'; };
    try {
      if (method.call(receiver, 'b') !== result || gets !== 1 || calls !== 1 || conversions !== 1) {
        throw 'borrowed created Proxy observation';
      }
      for (let missing = 0; missing < 2; missing++) {
        Object.defineProperty(regexpPrototype, symbol, { configurable: true, value: missing === 0 ? null : undefined });
        expectTypeError(() => method.call('abc', 'b'), errorPrototype);
      }
      Object.defineProperty(regexpPrototype, symbol, { configurable: true, get() { throw marker; } });
      expectMarker(() => method.call('abc', 'b'));
      Object.defineProperty(regexpPrototype, symbol, { configurable: true,
        value: new Proxy(function() {}, { apply() { throw marker; } }) });
      expectMarker(() => method.call('abc', 'b'));
      createdCases++;
    } finally { Object.defineProperty(regexpPrototype, symbol, saved); }
  }
}
if (directCases !== 12 || createdCases !== 6) throw 'paired String Realm cohorts';
true;
"#,
        ObservedJsValue::Boolean(true),
    );
}
