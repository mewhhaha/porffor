use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_enumeration(source: &str) {
    assert_completion(&format!("{source}\ntrue;"), "boolean(true)");
}

fn assert_completion(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("for-in failed: {error}\n{source}"));
    assert!(outcome.note.contains(expected), "{}", outcome.note);
}

#[test]
fn own_and_inherited_keys_share_order_for_objects_arrays_and_strings() {
    assert_enumeration(
        r#"
function keys(value) {
  var result = '';
  for (var key in value) result += key + ',';
  return result;
}
var prototype = { inherited: 1 };
var object = { z: 1, 2: 1, 1: 1, a: 1 };
Object.setPrototypeOf(object, prototype);
if (keys(object) !== '1,2,z,a,inherited,') throw 'object order';
var array = [3, 4];
array.extra = 1;
Object.setPrototypeOf(array, prototype);
if (keys(array) !== '0,1,extra,inherited,') throw 'array order';
String.prototype.inherited = 1;
var boxed = new String('ab');
boxed.extra = 1;
if (keys(boxed) !== '0,1,extra,inherited,') throw 'boxed string order';
if (keys('ab') !== '0,1,inherited,') throw 'primitive string order';
var direct = '';
for (var key in 'ab') direct += key + ',';
if (direct !== '0,1,inherited,') throw 'static string order';
function argumentsKeys() { return keys(arguments); }
if (argumentsKeys(1, 2) !== '0,1,') throw 'arguments order';
"#,
    );
}

#[test]
fn primitive_heads_box_and_nullish_heads_only_evaluate_the_expression() {
    assert_enumeration(
        r#"
Object.prototype.inherited = 1;
var numeric = '', boolean = '', bigint = '', symbol = '';
for (var key in 7) numeric += key + ',';
for (var key in false) boolean += key + ',';
for (var key in 7n) bigint += key + ',';
for (var key in Symbol()) symbol += key + ',';
if (numeric !== 'inherited,' || boolean !== numeric
    || bigint !== numeric || symbol !== numeric) throw 'primitive prototype';
var heads = 0, bodies = 0;
function nullish(value) { heads++; return value; }
for (var key in nullish(null)) bodies++;
for (var key in nullish(undefined)) bodies++;
for (var key in (heads++, null)) bodies++;
if (heads !== 3 || bodies !== 0) throw 'nullish head effects';
"#,
    );
}

#[test]
fn proxy_enumeration_reads_each_level_lazily_and_uses_intrinsic_methods() {
    assert_enumeration(
        r#"
var trace = '';
var symbol = Symbol('ignored');
var prototype = new Proxy(Object.create(null), {
  ownKeys() { trace += 'pkeys;'; return ['a', 'hidden', 'p']; },
  getOwnPropertyDescriptor(target, key) {
    trace += 'pd:' + key + ';';
    return { value: 1, enumerable: true, configurable: true };
  },
  getPrototypeOf() { trace += 'pproto;'; return null; }
});
var proxy = new Proxy(Object.create(null), {
  ownKeys() { trace += 'keys;'; return [symbol, 'a', 'hidden']; },
  getOwnPropertyDescriptor(target, key) {
    trace += 'd:' + key + ';';
    return { value: 1, enumerable: key !== 'hidden', configurable: true };
  },
  getPrototypeOf() { trace += 'proto;'; return prototype; }
});
Reflect.ownKeys = Reflect.getOwnPropertyDescriptor = Reflect.getPrototypeOf = function () {
  throw 'mutable Reflect method';
};
Object.keys = Object.getOwnPropertyNames = function () { throw 'mutable Object method'; };
for (var key in proxy) trace += 'body:' + key + ';';
if (trace !== 'keys;d:a;body:a;d:hidden;proto;pkeys;pd:p;body:p;pproto;') throw trace;
trace = '';
for (var key in proxy) { trace += 'body:' + key + ';'; break; }
if (trace !== 'keys;d:a;body:a;') throw 'eager enumeration: ' + trace;
"#,
    );
}

#[test]
fn deletion_shadowing_and_prototype_changes_use_current_descriptors() {
    assert_enumeration(
        r#"
var prototype = { shadowed: 1, deleted: 1, inherited: 1 };
var object = Object.create(prototype);
object.first = 1;
Object.defineProperty(object, 'shadowed', { value: 2, enumerable: false });
object.deleted = 2;
Object.defineProperty(object, 'accessor', {
  enumerable: true, configurable: true,
  get() { throw 'enumeration must not read values'; }
});
var keys = '';
for (var key in object) {
  keys += key + ',';
  if (key === 'first') {
    delete object.deleted;
    object.addedTooLate = 1;
    prototype.addedBeforeTraversal = 1;
  }
}
if (keys !== 'first,accessor,deleted,inherited,addedBeforeTraversal,') throw keys;
"#,
    );
}

#[test]
fn abrupt_internal_methods_and_body_completions_stop_enumeration() {
    assert_enumeration(
        r#"
var marker = {};
function expectThrow(target) {
  var caught;
  try { for (var key in target) {} } catch (error) { caught = error; }
  if (caught !== marker) throw 'wrong abrupt completion';
}
expectThrow(new Proxy({}, { ownKeys() { throw marker; } }));
expectThrow(new Proxy({}, {
  ownKeys() { return ['a']; }, getOwnPropertyDescriptor() { throw marker; }
}));
expectThrow(new Proxy({}, {
  ownKeys() { return []; }, getPrototypeOf() { throw marker; }
}));
var trace = '';
var proxy = new Proxy({ a: 1, b: 1 }, {
  getOwnPropertyDescriptor(target, key) {
    trace += key;
    return Object.getOwnPropertyDescriptor(target, key);
  },
  getPrototypeOf() { throw 'body completion must stop traversal'; }
});
function first() { for (var key in proxy) return key; }
if (first() !== 'a' || trace !== 'a') throw 'return completion';
trace = '';
var caught;
try { for (var key in proxy) throw marker; } catch (error) { caught = error; }
if (caught !== marker || trace !== 'a') throw 'body throw completion';
var revoked = Proxy.revocable({}, {});
revoked.revoke();
try { for (var key in revoked.proxy) {} } catch (error) { caught = error; }
if (!(caught instanceof TypeError)) throw 'revoked proxy';
var parseInt;
var bodyRan = false;
Object.defineProperty(this, 'parseInt', {
  configurable: true,
  get() { return ''; },
  set(value) { throw marker; }
});
caught = undefined;
try { for (parseInt in { a: 1 }) bodyRan = true; }
catch (error) { caught = error; }
if (caught !== marker || bodyRan) throw 'key publication completion';
"#,
    );
}

#[test]
fn iteration_bindings_and_labelled_finally_completions_keep_their_environment() {
    assert_enumeration(
        r#"
var captures = [];
for (let key in { a: 1, b: 1, c: 1 }) captures.push(function () { return key; });
if (captures[0]() !== 'a' || captures[1]() !== 'b' || captures[2]() !== 'c')
  throw 'per-iteration let binding';
var constants = [];
for (const key in { a: 1, b: 1 }) constants.push(() => key);
if (constants[0]() !== 'a' || constants[1]() !== 'b') throw 'const binding';
var shared = [];
for (var key in { a: 1, b: 1 }) shared.push(function () { return key; });
if (shared[0]() !== 'b' || shared[1]() !== 'b') throw 'var binding';
var trace = '';
outer: for (let outer in { a: 1, b: 1, c: 1 }) {
  for (let inner in { x: 1, y: 1 }) {
    try {
      trace += outer + inner;
      if (outer === 'a') continue outer;
      break outer;
    } finally { trace += '!'; }
  }
}
if (trace !== 'ax!bx!') throw trace;
var caught;
var target = { a: 1 };
try { for (let target in target) {} } catch (error) { caught = error; }
if (!(caught instanceof ReferenceError)) throw 'head TDZ';
"#,
    );
}

#[test]
fn loop_value_survives_descriptor_filtering_prototype_calls_and_publication() {
    assert_completion(
        r#"
var prototype = Object.create(null);
Object.defineProperty(prototype, 'hidden', { value: 1 });
var target = new Proxy({ a: 1, b: 1 }, {
  getPrototypeOf() { return prototype; }
});
for (var key in target) 37;
"#,
        "number(37)",
    );
}

#[test]
fn assignment_heads_preserve_the_empty_body_completion() {
    assert_completion(
        "var target = {}; for (target.key in { a: 1 }) {}",
        "undefined",
    );
    assert_completion("var first; for ([first] in { ab: 1 }) {}", "undefined");
}

#[test]
fn builtin_and_assertion_shaped_loops_execute_their_observable_bodies() {
    assert_enumeration(
        r#"
TypeError.extra = 1;
var keys = '';
for (var key in TypeError) keys += key + ',';
if (keys !== 'extra,') throw 'constructor own properties';
Number.extra = 1;
var calls = 0;
var assert = { notSameValue(actual, forbidden) { calls++; } };
for (var key in Number) assert.notSameValue(key, 'MAX_VALUE');
if (calls !== 1) throw 'assertion body skipped';
Object.defineProperty(this, 'parseInt', { enumerable: true });
var absent = true;
for (var key in this) { if (key === 'parseInt') absent = false; }
if (absent) throw 'global property enumerability';
"#,
    );
}
