use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_array_arguments_coercion(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("Array/Arguments coercion failed: {error}\n{source}"));
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".to_string())],
        "{source}"
    );
}

#[test]
fn array_and_arguments_numeric_hooks_preserve_bigint_results() {
    assert_array_arguments_coercion(
        r#"
var array = [1];
array.valueOf = function () { return 7n; };
if ((array - 1n) + 1n !== 7n) throw 'array arithmetic result';
if ((array & 3n) + 1n !== 4n || (~array) + 1n !== -7n) throw 'array bitwise result';
function fromArguments() {
  arguments.valueOf = function () { return 7n; };
  if ((arguments - 1n) + 1n !== 7n) throw 'arguments arithmetic result';
  if ((arguments ** 2n) + 1n !== 50n) throw 'arguments exponent result';
}
fromArguments(1);
function savedArray() {
  var value = [1], original = value;
  value -= (original.valueOf = function () { return 7n; }, value = null, 1n);
  return value + 1n;
}
if (savedArray() !== 7n) throw 'saved array receiver';
print('ok');
"#,
    );
}

#[test]
fn exotic_hooks_receive_live_receivers_and_requested_hints() {
    assert_array_arguments_coercion(
        r#"
function argumentsObject() { return arguments; }
function strictArgumentsObject() { 'use strict'; return arguments; }
function check(receiver) {
  var trace = '';
  receiver[Symbol.toPrimitive] = function (hint) {
    if (this !== receiver || arguments.length !== 1) throw 'exotic receiver';
    trace += hint + ';';
    return hint === 'string' ? 'key' : 7n;
  };
  if (receiver + 1n !== 8n || receiver - 1n !== 6n) throw 'exotic numeric result';
  if (String(receiver) !== 'key') throw 'exotic string result';
  var keyed = {};
  keyed[receiver] = 42;
  if (keyed.key !== 42) throw 'exotic property key';
  if (trace !== 'default;number;string;string;') throw trace;
}
check([1]);
check(argumentsObject(1));
check(strictArgumentsObject(1));
print('ok');
"#,
    );
}

#[test]
fn ordinary_hooks_observe_hint_order_and_getter_side_effects() {
    assert_array_arguments_coercion(
        r#"
function argumentsObject() { return arguments; }
function strictArgumentsObject() { 'use strict'; return arguments; }
function check(receiver) {
  var trace = '';
  Object.defineProperty(receiver, 'valueOf', {
    configurable: true,
    get: function () {
      trace += 'v';
      return function () {
        if (this !== receiver || arguments.length !== 0) throw 'valueOf receiver';
        trace += 'V';
        return receiver;
      };
    }
  });
  Object.defineProperty(receiver, 'toString', {
    configurable: true,
    get: function () {
      trace += 's';
      return function () {
        if (this !== receiver || arguments.length !== 0) throw 'toString receiver';
        trace += 'S';
        return 7n;
      };
    }
  });
  if (receiver - 1n !== 6n || trace !== 'vVsS') throw 'number hook order';
  trace = '';
  if (String(receiver) !== '7' || trace !== 'sS') throw 'string hook order';
  trace = '';
  Object.defineProperty(receiver, 'valueOf', { value: function () { return 9n; } });
  if (receiver - 1n !== 8n || trace !== '') throw 'replaced hook';
}
check([1]);
check(argumentsObject(1));
check(strictArgumentsObject(1));
print('ok');
"#,
    );
}

#[test]
fn abrupt_and_nonprimitive_hooks_reach_the_active_catch() {
    assert_array_arguments_coercion(
        r#"
function argumentsObject() { return arguments; }
function strictArgumentsObject() { 'use strict'; return arguments; }
function check(receiver) {
  var sentinel = {}, seen;
  Object.defineProperty(receiver, Symbol.toPrimitive, {
    configurable: true,
    get: function () { throw sentinel; }
  });
  try { receiver - 1n; } catch (error) { seen = error; }
  if (seen !== sentinel) throw 'abrupt getter';
  Object.defineProperty(receiver, Symbol.toPrimitive, {
    configurable: true,
    value: function () { throw sentinel; }
  });
  seen = undefined;
  try { String(receiver); } catch (error) { seen = error; }
  if (seen !== sentinel) throw 'abrupt call';
  Object.defineProperty(receiver, Symbol.toPrimitive, {
    configurable: true,
    value: function () { return {}; }
  });
  seen = undefined;
  try { receiver + 1; } catch (error) { seen = error; }
  if (!(seen instanceof TypeError)) throw 'nonprimitive exotic result';
  Object.defineProperty(receiver, Symbol.toPrimitive, { value: 1 });
  seen = undefined;
  try { receiver - 1; } catch (error) { seen = error; }
  if (!(seen instanceof TypeError)) throw 'noncallable exotic hook';
  delete receiver[Symbol.toPrimitive];
  receiver.valueOf = function () { return receiver; };
  receiver.toString = function () { return receiver; };
  seen = undefined;
  try { receiver - 1; } catch (error) { seen = error; }
  if (!(seen instanceof TypeError)) throw 'ordinary hook exhaustion';
}
check([1]);
check(argumentsObject(1));
check(strictArgumentsObject(1));
print('ok');
"#,
    );
}

#[test]
fn array_string_conversion_uses_live_join_and_indexed_reads() {
    assert_array_arguments_coercion(
        r#"
var array = [1, 2];
var trace = '';
Object.defineProperty(array, '1', {
  get: function () { trace += 'index;'; return [3, 4]; }
});
if (String(array) !== '1,3,4' || trace !== 'index;') throw 'canonical indexed join';
array.join = function () {
  if (this !== array || arguments.length !== 0) throw 'join receiver';
  return 7n;
};
if (array - 1n !== 6n || array + 1n !== 8n || String(array) !== '7') throw 'live join result';
array.join = null;
if (String(array) !== '[object Array]') throw 'noncallable join fallback';
var callable = function named() {};
callable[Symbol.toPrimitive] = function (hint) {
  if (hint !== 'string') throw hint;
  return 'function';
};
if (String([[1, 2], callable, null, undefined]) !== '1,2,function,,') throw 'nested element conversion';
print('ok');
"#,
    );
}

#[test]
fn inherited_hooks_and_missing_prototypes_remain_observable() {
    assert_array_arguments_coercion(
        r#"
function argumentsObject() { return arguments; }
var array = [1];
var prototype = Object.create(Array.prototype);
prototype.valueOf = function () { return 7n; };
Object.setPrototypeOf(array, prototype);
if (array - 1n !== 6n) throw 'inherited array hook';
var args = argumentsObject(1);
if (String(args) !== '[object Arguments]') throw 'arguments default';
args[Symbol.toStringTag] = 'Custom';
if (String(args) !== '[object Custom]') throw 'arguments string tag';
Object.setPrototypeOf(args, { valueOf: function () { return 9n; } });
if (args - 1n !== 8n) throw 'inherited arguments hook';
Object.setPrototypeOf(array, null);
Object.setPrototypeOf(args, null);
var arrayRejected = false, argumentsRejected = false;
try { String(array); } catch (error) { arrayRejected = error instanceof TypeError; }
try { String(args); } catch (error) { argumentsRejected = error instanceof TypeError; }
if (!arrayRejected || !argumentsRejected) throw 'missing prototype hooks';
print('ok');
"#,
    );
}
