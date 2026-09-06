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
        .expect("Array callback regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn length_get_and_coercion_precede_callback_validation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true, marker = {};
for (var m = 0; m < methods.length; m++) {
  var calls = '';
  var source = { get length() { calls += 'get;'; return { [Symbol.toPrimitive](hint) { calls += hint + ';'; return 0; } }; } };
  try { methods[m].call(source); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  try { methods[m].call(source, {}); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  try { methods[m].call({ get length() { throw marker; } }); ok = false; } catch (e) { ok = ok && e === marker; }
  try { methods[m].call({ length: { valueOf() { throw marker; } } }, null); ok = false; } catch (e) { ok = ok && e === marker; }
  ok = ok && calls === 'get;number;get;number;';
}
ok;
"#,
    );
}

#[test]
fn callable_and_nested_proxy_callbacks_preserve_this_and_arguments() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = [3, 5], context = {}, calls = '', seen = 0;
  var target = function(value, index, receiver) { ok = ok && this === context && arguments.length === 3 && receiver === source; seen++; return m === 3 ? false : value; };
  var inner = new Proxy(target, { apply(fn, that, args) { calls += 'inner' + args[1] + ';'; return Reflect.apply(fn, that, args); } });
  var outer = new Proxy(inner, { apply(fn, that, args) { calls += 'outer' + args[1] + ';'; return Reflect.apply(fn, that, args); } });
  var result = methods[m].call(source, outer, context);
  ok = ok && seen === 2 && calls === 'outer0;inner0;outer1;inner1;';
  if (m < 2) ok = ok && result.length === 2 && result[0] === 3 && result[1] === 5;
  else ok = ok && result === (m === 2);
}
ok;
"#,
    );
}

#[test]
fn captured_length_and_live_sparse_properties_survive_callback_mutation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = [1, 2, 3], calls = '';
  var result = methods[m].call(source, function(value, index, receiver) {
    calls += index + ':' + value + ';';
    if (index === 0) { delete receiver[1]; receiver[2] = 7; receiver[3] = 99; }
    return m === 3 ? false : value;
  });
  ok = ok && calls === '0:1;2:7;';
  if (m === 0) ok = ok && result.length === 3 && result[0] === 1 && !(1 in result) && result[2] === 7;
  if (m === 1) ok = ok && result.length === 2 && result[0] === 1 && result[1] === 7;
  if (m > 1) ok = ok && result === (m === 2);
}
ok;
"#,
    );
}

#[test]
fn proxy_receivers_observe_has_before_get_and_skip_holes() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var calls = '', original = { length: 3, 0: 4, 2: 8 };
  var inner = new Proxy(original, {
    get(t, key) { calls += 'get:' + key + ';'; return Reflect.get(t, key); },
    has(t, key) { calls += 'has:' + key + ';'; return Reflect.has(t, key); }
  });
  var source = new Proxy(inner, {});
  methods[m].call(source, function(value, index, receiver) {
    ok = ok && receiver === source;
    calls += 'call:' + index + ':' + value + ';';
    return m !== 3;
  });
  ok = ok && calls === 'get:length;has:0;get:0;call:0:4;has:1;has:2;get:2;call:2:8;';
}
ok;
"#,
    );
}

#[test]
fn borrowed_typed_arrays_observe_own_and_inherited_length() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  for (var inherited = 0; inherited < 2; inherited++) {
    var source = new Uint8Array([2, 4, 6]), gets = 0, seen = '';
    var holder = inherited ? Object.create(Object.getPrototypeOf(source)) : source;
    Object.defineProperty(holder, 'length', { get() { gets++; return { valueOf() { return 1; } }; } });
    if (inherited) Object.setPrototypeOf(source, holder);
    var result = methods[m].call(source, function(value, index, receiver) { seen += index + ':' + value + ';'; ok = ok && receiver === source; return m === 3 ? false : value; });
    ok = ok && gets === 1 && seen === '0:2;';
    if (m < 2) ok = ok && result.length === 1 && result[0] === 2;
    else ok = ok && result === (m === 2);
  }
}
ok;
"#,
    );
}

#[test]
fn borrowed_resizable_typed_arrays_use_live_has_with_a_captured_bound() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some];
var ok = true;
for (var m = 0; m < methods.length; m++) {
  var buffer = new ArrayBuffer(3, { maxByteLength: 6 }), source = new Uint8Array(buffer), calls = '';
  source[0] = 1; source[1] = 2; source[2] = 3;
  var result = methods[m].call(source, function(value, index) { calls += index + ':' + value + ';'; buffer.resize(1); return m !== 3; });
  ok = ok && calls === '0:1;';
  if (m === 0) ok = ok && result.length === 3 && result[0] === true && !(1 in result) && !(2 in result);
  if (m === 1) ok = ok && result.length === 1 && result[0] === 1;
  if (m > 1) ok = ok && result === (m === 2);
  buffer = new ArrayBuffer(1, { maxByteLength: 6 }); source = new Uint8Array(buffer); source[0] = 9; calls = '';
  methods[m].call(source, function(value, index) { calls += index + ';'; buffer.resize(3); source[1] = 7; return m !== 3; });
  ok = ok && calls === '0;';
}
ok;
"#,
    );
}

#[test]
fn proxy_species_construction_uses_original_bound_and_result_identity() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter], ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = [4, 6], resultTarget = {}, trace = '', species;
  species = new Proxy(function() {}, { construct(target, args, newTarget) {
    ok = ok && args.length === 1 && args[0] === (m === 0 ? 2 : 0) && newTarget === species;
    trace += 'construct;'; source[2] = 99; return resultTarget;
  } });
  Object.defineProperty(source, 'constructor', { get() { trace += 'constructor;'; return { get [Symbol.species]() { trace += 'species;'; return species; } }; } });
  var result = methods[m].call(source, function(value, index) { trace += 'call' + index + ';'; return m === 0 ? value * 2 : true; });
  ok = ok && result === resultTarget && !('length' in result) && result[0] === (m === 0 ? 8 : 4) && result[1] === (m === 0 ? 12 : 6) && !(2 in result);
  ok = ok && trace === 'constructor;species;construct;call0;call1;';
}
ok;
"#,
    );
}

#[test]
fn custom_species_defines_data_properties_without_setters_or_length_writes() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter], ok = true;
for (var m = 0; m < methods.length; m++) {
  var writes = 0, source = [3], proto = { set 0(value) { writes++; }, set length(value) { writes++; } };
  var target = Object.create(proto);
  source.constructor = { [Symbol.species]: function() { return target; } };
  var result = methods[m].call(source, function(value) { return value + 1; });
  var d = Object.getOwnPropertyDescriptor(result, '0');
  ok = ok && result === target && writes === 0 && d.value === (m === 0 ? 4 : 3) && d.writable && d.enumerable && d.configurable && !Object.hasOwn(result, 'length');
}
ok;
"#,
    );
}

#[test]
fn proxy_species_result_observes_only_define_property() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter], ok = true;
for (var m = 0; m < methods.length; m++) {
  var trace = '', target = new Proxy({}, {
    set() { throw 'unexpected Set'; },
    defineProperty(object, key, descriptor) {
      trace += key + ':' + descriptor.value + ';';
      ok = ok && descriptor.writable && descriptor.enumerable && descriptor.configurable;
      return Reflect.defineProperty(object, key, descriptor);
    }
  });
  var source = [4, 6]; source.constructor = { [Symbol.species]: function() { return target; } };
  var result = methods[m].call(source, function(value) { return value + 1; });
  ok = ok && result === target && trace === (m === 0 ? '0:5;1:7;' : '0:4;1:6;');
}
ok;
"#,
    );
}

#[test]
fn failed_result_definition_stops_before_later_callbacks() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter], ok = true;
for (var m = 0; m < methods.length; m++) {
  for (var proxy = 0; proxy < 2; proxy++) {
    var calls = 0, definitions = 0;
    var target = proxy ? new Proxy({}, { defineProperty() { definitions++; return false; } }) : Object.preventExtensions({});
    var source = [2, 3]; source.constructor = { [Symbol.species]: function() { return target; } };
    try { methods[m].call(source, function(value) { calls++; return true; }); ok = false; }
    catch(e) { ok = ok && e instanceof TypeError; }
    ok = ok && calls === 1 && definitions === proxy;
  }
}
ok;
"#,
    );
}

#[test]
fn filter_keeps_pre_callback_values_and_packs_selected_indices() {
    assert_wasm_true(
        r#"
var source = { length: 4, 0: 2, 2: 6, 3: 8 }, calls = '';
var result = Array.prototype.filter.call(source, function(value, index, receiver) {
  calls += index + ':' + value + ';'; receiver[index] = 99;
  return index !== 2;
});
result.length === 2 && result[0] === 2 && result[1] === 8 && calls === '0:2;2:6;3:8;';
"#,
    );
}

#[test]
fn quantifiers_short_circuit_without_species_or_value_coercion() {
    assert_wasm_true(
        r#"
var marker = {}, coercions = 0, seen = '', source = [7, 8];
Object.defineProperty(source, 'constructor', { get() { throw marker; } });
Object.defineProperty(source, '1', { get() { throw marker; } });
var truthy = { valueOf() { coercions++; throw marker; }, toString() { coercions++; throw marker; } };
var some = source.some(function(v, i) { seen += 's' + i + ';'; return truthy; });
var every = source.every(function(v, i) { seen += 'e' + i + ';'; return ''; });
some === true && every === false && coercions === 0 && seen === 's0;e0;' && [].every(function() { throw marker; }) === true && [].some(function() { throw marker; }) === false;
"#,
    );
}

#[test]
fn callback_validation_precedes_species_effects() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter], ok = true, marker = {};
for (var m = 0; m < methods.length; m++) {
  var source = [1], count = 0;
  Object.defineProperty(source, 'constructor', { get() { count++; throw marker; } });
  try { methods[m].call(source, {}); ok = false; } catch(e) { ok = ok && e instanceof TypeError; }
  ok = ok && count === 0;
  try { methods[m].call(source, function() { ok = false; }); ok = false; } catch(e) { ok = ok && e === marker; }
  ok = ok && count === 1;
  var bad = [1]; bad.constructor = { [Symbol.species]: {} };
  try { methods[m].call(bad, function() { ok = false; }); ok = false; } catch(e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn abrupt_has_get_and_call_preserve_identity_and_order() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some], ok = true, marker = {};
for (var m = 0; m < methods.length; m++) {
  for (var phase = 0; phase < 3; phase++) {
    var trace = '';
    var source = new Proxy({ length: 2, 0: 4, 1: 5 }, {
      has(t, k) { trace += 'has;'; if (phase === 0) throw marker; return true; },
      get(t, k) { if (k === 'length') return 2; trace += 'get;'; if (phase === 1) throw marker; return t[k]; }
    });
    try { methods[m].call(source, function() { trace += 'call;'; throw marker; }); ok = false; }
    catch(e) { ok = ok && e === marker; }
    ok = ok && trace === (phase === 0 ? 'has;' : phase === 1 ? 'has;get;' : 'has;get;call;');
  }
}
ok;
"#,
    );
}

#[test]
fn huge_lengths_are_clamped_without_wasm_conversion_traps() {
    assert_wasm_true(
        r#"
var ok = true, marker = {}, lengths = [1e300, Infinity, 9007199254740992];
var methods = [Array.prototype.filter, Array.prototype.every, Array.prototype.some];
for (var i = 0; i < lengths.length; i++) {
  var calls = '', source = new Proxy({ length: lengths[i] }, { has() { calls += 'has;'; throw marker; } });
  try { Array.prototype.map.call(source, function() { ok = false; }); ok = false; } catch(e) { ok = ok && e instanceof RangeError; }
  ok = ok && calls === '';
  for (var m = 0; m < methods.length; m++) {
    calls = '';
    try { methods[m].call(source, function() { ok = false; }); ok = false; } catch(e) { ok = ok && e === marker; }
    ok = ok && calls === 'has;';
  }
}
ok;
"#,
    );
}

#[test]
fn revoked_callable_proxy_validation_is_separate_from_invocation() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some], ok = true;
for (var m = 0; m < methods.length; m++) {
  var revocable = Proxy.revocable(function() {}, {}); revocable.revoke();
  var empty = methods[m].call([], revocable.proxy);
  if (m < 2) ok = ok && empty.length === 0; else ok = ok && empty === (m === 2);
  try { methods[m].call([1], revocable.proxy); ok = false; } catch(e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn primitive_receivers_are_boxed_and_nullish_receivers_throw() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some], ok = true;
for (var m = 0; m < methods.length; m++) {
  var receiver, calls = '';
  methods[m].call('ab', function(value, index, object) { if (index === 0) receiver = object; ok = ok && typeof object === 'object' && object === receiver; calls += index + value; return m !== 3; });
  ok = ok && calls === '0a1b';
  try { methods[m].call(null, function() {}); ok = false; } catch(e) { ok = ok && e instanceof TypeError; }
  try { methods[m].call(undefined, function() {}); ok = false; } catch(e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn inherited_index_added_during_callback_is_observed() {
    assert_wasm_true(
        r#"
var methods = [Array.prototype.map, Array.prototype.filter, Array.prototype.every, Array.prototype.some], ok = true;
for (var m = 0; m < methods.length; m++) {
  var source = [3, , 7], trace = '';
  try {
    var result = methods[m].call(source, function(value, index) { trace += index + ':' + value + ';'; if (index === 0) Array.prototype[1] = 5; return m === 3 ? false : value; });
    ok = ok && trace === '0:3;1:5;2:7;';
    if (m < 2) ok = ok && result.length === 3 && result[1] === 5;
  } finally { delete Array.prototype[1]; }
}
ok;
"#,
    );
}

#[test]
fn minimal_map_roots_result_definition_dependencies() {
    assert_wasm_true(
        r#"
var source = [2, 4];
var result = source.map(function(value) { return value * 3; });
result.length === 2 && result[0] === 6 && result[1] === 12;
"#,
    );
}

#[test]
fn minimal_filter_roots_result_definition_dependencies() {
    assert_wasm_true(
        r#"
var source = [2, 4];
var result = source.filter(function(value) { return value === 4; });
result.length === 1 && result[0] === 4;
"#,
    );
}
