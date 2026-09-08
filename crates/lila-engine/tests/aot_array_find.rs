use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("find regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn borrowed_typed_array_observes_own_and_inherited_length() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  for (var inherited = 0; inherited < 2; inherited++) {
    var source = new Uint8Array([2, 4, 6]), trace = '';
    var holder = inherited ? Object.create(Object.getPrototypeOf(source)) : source;
    Object.defineProperty(holder, 'length', { get() { trace += 'length;'; return { valueOf() { trace += 'number;'; return 1; } }; } });
    if (inherited) Object.setPrototypeOf(source, holder);
    var result = methods[m].call(source, function(value, index, receiver) { trace += index + ':' + value + ';'; ok = ok && receiver === source; return true; });
    ok = ok && trace === 'length;number;0:2;' && result === (m % 2 ? 0 : 2);
  }
}
ok;
"#,
    );
}

#[test]
fn borrowed_typed_array_length_errors_precede_predicate_validation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var marker = {};
for (var m = 0; m < methods.length; m++) {
  var source = new Uint8Array([1]), seen = 0;
  Object.defineProperty(source, 'length', { get() { seen++; throw marker; } });
  try { methods[m].call(source, null); ok = false; } catch (e) { ok = ok && e === marker; }
  ok = ok && seen === 1;
}
ok;
"#,
    );
}

#[test]
fn arguments_length_getter_and_coercion_are_observable() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = (function() { return arguments; })(2, 4, 6), trace = '';
  Object.defineProperty(source, 'length', { get() { trace += 'length;'; return { [Symbol.toPrimitive](hint) { trace += hint + ';'; return 1; } }; } });
  var result = methods[m].call(source, function(value, index, receiver) { trace += index + ':' + value + ';'; ok = ok && receiver === source; return true; });
  ok = ok && trace === 'length;number;0:2;' && result === (m % 2 ? 0 : 2);
}
ok;
"#,
    );
}

#[test]
fn arguments_length_extension_visits_missing_indices() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = (function() { return arguments; })(7), trace = '';
  source.length = 3;
  var result = methods[m].call(source, function(value, index) { trace += index + ':' + value + ';'; return false; });
  ok = ok && trace === (m < 2 ? '0:7;1:undefined;2:undefined;' : '2:undefined;1:undefined;0:7;');
  ok = ok && result === (m % 2 ? -1 : undefined);
}
ok;
"#,
    );
}

#[test]
fn arguments_deleted_index_uses_live_inherited_getter() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = (function() { return arguments; })(1, 2, 3), reads = 0;
  delete source[1];
  var proto = Object.create(Object.getPrototypeOf(source));
  Object.defineProperty(proto, '1', { get() { reads++; ok = ok && this === source; return 9; } });
  Object.setPrototypeOf(source, proto);
  var result = methods[m].call(source, function(value) { return value === 9; });
  ok = ok && reads === 1 && result === (m % 2 ? 1 : 9);
}
ok;
"#,
    );
}

#[test]
fn global_object_is_a_valid_generic_receiver() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
Object.defineProperty(globalThis, 'length', { value: 2, configurable: true, writable: true });
globalThis[0] = 3; globalThis[1] = 5;
for (var m = 0; m < methods.length; m++) {
  var result = methods[m].call(globalThis, function(value, index, receiver) { ok = ok && receiver === globalThis; return true; });
  ok = ok && result === (m % 2 ? (m < 2 ? 0 : 1) : (m < 2 ? 3 : 5));
}
delete globalThis.length; delete globalThis[0]; delete globalThis[1];
ok;
"#,
    );
}

#[test]
fn strings_are_boxed_once_and_passed_as_the_callback_receiver() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var boxed, trace = '';
  var result = methods[m].call('ab', function(value, index, receiver) {
    if (boxed === undefined) boxed = receiver;
    ok = ok && typeof receiver === 'object' && receiver === boxed && receiver.valueOf() === 'ab';
    trace += index + ':' + value + ';'; return false;
  });
  ok = ok && trace === (m < 2 ? '0:a;1:b;' : '1:b;0:a;') && result === (m % 2 ? -1 : undefined);
  boxed = undefined;
}
ok;
"#,
    );
}

#[test]
fn boxed_primitives_observe_inherited_length_and_indices() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
Object.defineProperty(Number.prototype, 'length', { value: 1, configurable: true });
Object.defineProperty(Number.prototype, '0', { get() { return this.valueOf() + 1; }, configurable: true });
for (var m = 0; m < methods.length; m++) {
  var result = methods[m].call(6, function(value, index, receiver) { ok = ok && typeof receiver === 'object' && receiver.valueOf() === 6 && index === 0; return true; });
  ok = ok && result === (m % 2 ? 0 : 7);
}
delete Number.prototype.length; delete Number.prototype[0];
ok;
"#,
    );
}

#[test]
fn proxy_receivers_get_every_index_without_has_property() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var trace = '', source = new Proxy({ length: 3, 0: 4, 2: 8 }, {
    get(t, key, receiver) { trace += 'get:' + key + ';'; return Reflect.get(t, key, receiver); },
    has() { throw 'find must not call HasProperty'; }
  });
  source = new Proxy(source, {});
  var result = methods[m].call(source, function(value, index, receiver) { ok = ok && receiver === source; trace += 'call:' + index + ':' + value + ';'; return false; });
  var expected = m < 2 ? 'get:length;get:0;call:0:4;get:1;call:1:undefined;get:2;call:2:8;' : 'get:length;get:2;call:2:8;get:1;call:1:undefined;get:0;call:0:4;';
  ok = ok && trace === expected && result === (m % 2 ? -1 : undefined);
}
ok;
"#,
    );
}

#[test]
fn nested_callable_proxy_preserves_this_and_three_arguments() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = [3, 5], context = {}, trace = '', seen = 0;
  var target = function(value, index, receiver) { ok = ok && this === context && arguments.length === 3 && receiver === source; seen++; return false; };
  var inner = new Proxy(target, { apply(fn, that, args) { trace += 'inner' + args[1] + ';'; return Reflect.apply(fn, that, args); } });
  var outer = new Proxy(inner, { apply(fn, that, args) { trace += 'outer' + args[1] + ';'; return Reflect.apply(fn, that, args); } });
  var result = methods[m].call(source, outer, context);
  ok = ok && seen === 2 && result === (m % 2 ? -1 : undefined);
  ok = ok && trace === (m < 2 ? 'outer0;inner0;outer1;inner1;' : 'outer1;inner1;outer0;inner0;');
}
ok;
"#,
    );
}

#[test]
fn all_eight_methods_keep_predicates_alive_and_return_no_match_sentinels() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var typedMethods = [Uint8Array.prototype.find, Uint8Array.prototype.findIndex, Uint8Array.prototype.findLast, Uint8Array.prototype.findLastIndex];
var falsey = [undefined, null, false, 0, -0, NaN, '', 0n];
for (var typed = 0; typed < 2; typed++) {
  for (var m = 0; m < methods.length; m++) {
    var source = typed ? new Uint8Array(8) : [1, 2, 3, 4, 5, 6, 7, 8], seen = 0;
    var method = typed ? typedMethods[m] : methods[m];
    var result = method.call(source, function() { return falsey[seen++]; });
    ok = ok && seen === 8 && result === (m % 2 ? -1 : undefined);
  }
}
ok;
"#,
    );
}

#[test]
fn revoked_callable_proxy_is_validated_but_not_called_on_empty_inputs() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var revoked = Proxy.revocable(function() {}, {}); revoked.revoke();
  var result = methods[m].call({ length: 0 }, revoked.proxy);
  ok = ok && result === (m % 2 ? -1 : undefined);
  try { methods[m].call({ length: 1 }, revoked.proxy); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  try { methods[m].call({ length: 0 }, new Proxy({}, {})); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn predicate_revocation_stops_at_the_next_call() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var calls = 0, reads = 0;
  var source = new Proxy({ length: 3 }, { get(t, key) { if (key === 'length') return 3; reads++; return 7; } });
  var revocable = Proxy.revocable(function() { calls++; revocable.revoke(); return false; }, {});
  try { methods[m].call(source, revocable.proxy); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  ok = ok && calls === 1 && reads === 2;
}
ok;
"#,
    );
}

#[test]
fn length_is_captured_once_before_callback_mutation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var reads = 0, length = 3, trace = '';
  var source = { get length() { reads++; return length; }, 0: 1, 1: 2, 2: 3 };
  methods[m].call(source, function(value, index) { length = 0; source[3] = 99; trace += index + ':' + value + ';'; return false; });
  ok = ok && reads === 1 && trace === (m < 2 ? '0:1;1:2;2:3;' : '2:3;1:2;0:1;');
}
ok;
"#,
    );
}

#[test]
fn deleted_and_inherited_properties_remain_live_in_both_directions() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var proto = {}, source = Object.create(proto), trace = '', seen = 0;
  source.length = 3; source[0] = 1; source[1] = 2; source[2] = 3;
  methods[m].call(source, function(value, index) {
    trace += index + ':' + value + ';';
    if (seen++ === 0) { delete source[1]; proto[1] = 9; delete source[m < 2 ? 2 : 0]; }
    return false;
  });
  ok = ok && trace === (m < 2 ? '0:1;1:9;2:undefined;' : '2:3;1:9;0:undefined;');
}
ok;
"#,
    );
}

#[test]
fn matched_value_is_the_value_read_before_callback_mutation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var value = {}, source = [value], calls = 0;
  var result = methods[m].call(source, function(element, index, receiver) { calls++; receiver[0] = 99; return {}; });
  ok = ok && calls === 1 && source[0] === 99 && result === (m % 2 ? 0 : value);
}
ok;
"#,
    );
}

#[test]
fn matching_short_circuits_before_later_getters() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var trace = '', source = new Proxy({ length: 3 }, { get(t, key) {
    trace += key + ';';
    if (key === 'length') return 3;
    if (key === (m < 2 ? '0' : '2')) return 8;
    throw 'unexpected later Get';
  } });
  var result = methods[m].call(source, function() { return true; });
  ok = ok && trace === (m < 2 ? 'length;0;' : 'length;2;') && result === (m % 2 ? (m < 2 ? 0 : 2) : 8);
}
ok;
"#,
    );
}

#[test]
fn length_coercion_and_index_get_abrupt_completions_preserve_identity() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var marker = {};
for (var m = 0; m < methods.length; m++) {
  var trace = '';
  try { methods[m].call({ get length() { trace += 'length;'; throw marker; } }, null); ok = false; } catch (e) { ok = ok && e === marker; }
  try { methods[m].call({ length: { valueOf() { trace += 'number;'; throw marker; } } }, null); ok = false; } catch (e) { ok = ok && e === marker; }
  try { methods[m].call({ length: 1, get 0() { trace += 'index;'; throw marker; } }, function() { trace += 'callback;'; }); ok = false; } catch (e) { ok = ok && e === marker; }
  ok = ok && trace === 'length;number;index;';
}
ok;
"#,
    );
}

#[test]
fn callback_throw_stops_before_later_index_reads() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var marker = {};
for (var m = 0; m < methods.length; m++) {
  var reads = 0, calls = 0, source = new Proxy({ length: 3 }, { get(t, key) { if (key === 'length') return 3; reads++; return 1; } });
  try { methods[m].call(source, function() { calls++; throw marker; }); ok = false; } catch (e) { ok = ok && e === marker; }
  ok = ok && reads === 1 && calls === 1;
}
ok;
"#,
    );
}

#[test]
fn to_length_floors_clamps_and_rejects_non_numeric_primitives() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var lengths = [-1, NaN, undefined, '0', 1.9, '0x2'], counts = [0, 0, 0, 0, 1, 2];
for (var m = 0; m < methods.length; m++) {
  for (var n = 0; n < lengths.length; n++) {
    var seen = 0, result = methods[m].call({ length: lengths[n] }, function() { seen++; return false; });
    ok = ok && seen === counts[n] && result === (m % 2 ? -1 : undefined);
  }
  try { methods[m].call({ length: 1n }, function() {}); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  try { methods[m].call({ length: Symbol() }, function() {}); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn huge_lengths_do_not_trap_or_truncate_reverse_indices() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var trace = '', source = new Proxy({ length: Infinity }, { get(t, key) { if (key === 'length') return Infinity; trace += key; return 7; } });
  var result = methods[m].call(source, function(value, index) { ok = ok && index === (m < 2 ? 0 : 9007199254740990); return true; });
  ok = ok && trace === (m < 2 ? '0' : '9007199254740990') && result === (m % 2 ? (m < 2 ? 0 : 9007199254740990) : 7);
}
ok;
"#,
    );
}

#[test]
fn borrowed_typed_array_length_can_exceed_private_extent() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = new Uint8Array([2]), trace = '';
  Object.defineProperty(source, 'length', { value: 3 });
  methods[m].call(source, function(value, index) { trace += index + ':' + value + ';'; return false; });
  ok = ok && trace === (m < 2 ? '0:2;1:undefined;2:undefined;' : '2:undefined;1:undefined;0:2;');
}
ok;
"#,
    );
}

#[test]
fn borrowed_resizable_typed_arrays_keep_bound_but_read_live_elements() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var buffer = new ArrayBuffer(3, { maxByteLength: 6 }), source = new Uint8Array(buffer), trace = '', seen = 0;
  source[0] = 1; source[1] = 2; source[2] = 3;
  methods[m].call(source, function(value, index) { trace += index + ':' + value + ';'; if (seen++ === 0) buffer.resize(0); return false; });
  ok = ok && trace === (m < 2 ? '0:1;1:undefined;2:undefined;' : '2:3;1:undefined;0:undefined;');
}
ok;
"#,
    );
}

#[test]
fn strict_typed_array_validation_remains_separate_from_generic_length() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.find, Array.prototype.findIndex, Array.prototype.findLast, Array.prototype.findLastIndex];
var ok = true;
var typedMethods = [Uint8Array.prototype.find, Uint8Array.prototype.findIndex, Uint8Array.prototype.findLast, Uint8Array.prototype.findLastIndex];
for (var m = 0; m < methods.length; m++) {
  var source = new Uint8Array([3, 5]), calls = 0;
  Object.defineProperty(source, 'length', { get() { throw 'strict TypedArray method must not Get length'; } });
  var result = typedMethods[m].call(source, function() { calls++; return false; });
  ok = ok && calls === 2 && result === (m % 2 ? -1 : undefined);
  try { typedMethods[m].call({ length: 0 }, function() {}); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  var buffer = new ArrayBuffer(2, { maxByteLength: 4 }), fixed = new Uint8Array(buffer, 0, 2); buffer.resize(0);
  try { typedMethods[m].call(fixed, function() {}); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  Object.defineProperty(fixed, 'length', { value: 1 }); calls = 0;
  result = methods[m].call(fixed, function(value) { calls++; ok = ok && value === undefined; return false; });
  ok = ok && calls === 1 && result === (m % 2 ? -1 : undefined);
}
ok;
"#,
    );
}
