use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
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
