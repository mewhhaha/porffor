use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
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
        .expect("flatMap regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn length_get_and_coercion_precede_missing_or_non_callable_mapper() {
    assert_wasm_true(
        r#"
var calls = "", marker = {}, ok = true;
var receiver = { get length() {
  calls += "get;";
  return { [Symbol.toPrimitive](hint) { calls += hint + ";"; return 0; } };
}};
try { Array.prototype.flatMap.call(receiver); ok = false; }
catch (e) { ok = ok && e instanceof TypeError; }
try { Array.prototype.flatMap.call(receiver, {}); ok = false; }
catch (e) { ok = ok && e instanceof TypeError; }
var poisoned = { get length() { throw marker; } };
try { Array.prototype.flatMap.call(poisoned); ok = false; }
catch (e) { ok = ok && e === marker; }
var coercion = { length: { valueOf() { throw marker; } } };
try { Array.prototype.flatMap.call(coercion, null); ok = false; }
catch (e) { ok = ok && e === marker; }
ok && calls === "get;number;get;number;";
"#,
    );
}

#[test]
fn length_is_snapshotted_before_species_side_effects() {
    assert_wasm_true(
        r#"
var source = [7], calls = "";
Object.defineProperty(source, "constructor", { get() {
  calls += "constructor;";
  source[1] = 9;
  return { get [Symbol.species]() { calls += "species;"; return Array; } };
}});
var result = source.flatMap(function(value, index, receiver) {
  calls += "map" + index + ";";
  return [value, receiver === source];
});
result.length === 2 && result[0] === 7 && result[1] === true &&
calls === "constructor;species;map0;";
"#,
    );
}

#[test]
fn huge_lengths_reach_property_operations_without_wasm_traps() {
    assert_wasm_true(
        r#"
var lengths = [1e300, Infinity, 9007199254740992], marker = {}, ok = true;
for (var i = 0; i < lengths.length; i++) {
  var calls = "";
  var source = new Proxy({ length: lengths[i] }, {
    get(t, k) { calls += "get:" + k + ";"; return t[k]; },
    has(t, k) { calls += "has:" + k + ";"; throw marker; }
  });
  try { Array.prototype.flatMap.call(source, function() { ok = false; }); ok = false; }
  catch (e) { ok = ok && e === marker; }
  ok = ok && calls === "get:length;has:0;";
}
var emptyLengths = [-Infinity, -3, NaN, -0, 0.5];
for (var j = 0; j < emptyLengths.length; j++) {
  ok = ok && Array.prototype.flatMap.call({ length: emptyLengths[j] }, function() {
    ok = false;
  }).length === 0;
}
ok;
"#,
    );
}

#[test]
fn typed_array_own_and_inherited_length_are_observable() {
    assert_wasm_true(
        r#"
var own = new Uint8Array([2, 4, 6]), log = "";
Object.defineProperty(own, "length", { get() { log += "length;"; return 1.9; } });
var a = Array.prototype.flatMap.call(own, function(v, k, receiver) {
  log += "map" + k + ";";
  return [v, receiver === own];
});
var inherited = new Uint8Array([8, 10]);
var proto = Object.create(Uint8Array.prototype);
Object.defineProperty(proto, "length", { get() { log += "inherited;"; return 1; } });
Object.setPrototypeOf(inherited, proto);
var b = Array.prototype.flatMap.call(inherited, function(v) { return [v]; });
a.length === 2 && a[0] === 2 && a[1] === true && b.length === 1 && b[0] === 8 &&
log === "length;map0;inherited;";
"#,
    );
}

#[test]
fn typed_array_length_getters_and_callbacks_preserve_live_buffer_checks() {
    assert_wasm_true(
        r#"
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var view = new Uint8Array(buffer); view[0] = 3; view[1] = 5;
Object.defineProperty(view, "length", { get() { buffer.resize(2); return 4; } });
var count = 0;
var result = Array.prototype.flatMap.call(view, function(value) {
  count++; buffer.resize(0); return [value];
});
var detached = new Uint8Array([1, 2]);
detached.buffer.transfer();
Object.defineProperty(detached, "length", { value: 2 });
var empty = Array.prototype.flatMap.call(detached, function() { count += 100; });
result.length === 1 && result[0] === 3 && count === 1 && empty.length === 0;
"#,
    );
}

#[test]
fn callable_proxies_receive_this_and_all_mapper_arguments() {
    assert_wasm_true(
        r#"
var source = [4, 5], context = {}, calls = "";
var mapper = new Proxy(new Proxy(function(v) { return [v * 2]; }, {}), {
  apply(target, receiver, args) {
    calls += args[1];
    if (receiver !== context || args.length !== 3 || args[2] !== source) throw 1;
    return Reflect.apply(target, receiver, args);
  }
});
var result = source.flatMap(mapper, context);
var revoked = Proxy.revocable(function() {}, {}); revoked.revoke();
var empty = [].flatMap(revoked.proxy);
var threw = false;
try { [1].flatMap(revoked.proxy); } catch (e) { threw = e instanceof TypeError; }
result.length === 2 && result[0] === 8 && result[1] === 10 && calls === "01" &&
empty.length === 0 && threw;
"#,
    );
}

#[test]
fn nested_proxy_receivers_keep_length_constructor_has_get_order() {
    assert_wasm_true(
        r#"
var log = "", source = new Proxy(new Proxy([3, 6], {}), {
  get(t, k, r) { log += "get:" + k + ";"; return Reflect.get(t, k, r); },
  has(t, k) { log += "has:" + k + ";"; return Reflect.has(t, k); }
});
var result = Array.prototype.flatMap.call(source, function(v, i, r) {
  if (r !== source) throw 1;
  log += "map:" + i + ";"; return [v];
});
result.length === 2 && result[0] === 3 && result[1] === 6 &&
log === "get:length;get:constructor;has:0;get:0;map:0;has:1;get:1;map:1;";
"#,
    );
}

#[test]
fn mapped_proxy_arrays_preserve_traps_holes_and_one_level_depth() {
    assert_wasm_true(
        r#"
var log = "", deep = [9];
var mapped = new Proxy(new Proxy([2, , deep], {}), {
  get(t, k, r) { log += "get:" + k + ";"; return Reflect.get(t, k, r); },
  has(t, k) { log += "has:" + k + ";"; return Reflect.has(t, k); }
});
var count = 0;
var result = [0].flatMap(function() { count++; return mapped; });
result.length === 2 && result[0] === 2 && result[1] === deep && count === 1 &&
log === "get:length;has:0;get:0;has:1;has:2;get:2;";
"#,
    );
}

#[test]
fn revoked_mapper_results_and_abrupt_nested_properties_propagate() {
    assert_wasm_true(
        r#"
var revoked = Proxy.revocable([], {}); revoked.revoke();
var ok = false, marker = {}, calls = "";
try { [1].flatMap(function() { return revoked.proxy; }); }
catch (e) { ok = e instanceof TypeError; }
var mapped = new Proxy([2], {
  get(t, k, r) { calls += "get:" + k + ";"; return Reflect.get(t, k, r); },
  has(t, k) { calls += "has:" + k + ";"; throw marker; }
});
try { [1].flatMap(function() { return mapped; }); ok = false; }
catch (e) { ok = ok && e === marker; }
ok && calls === "get:length;has:0;";
"#,
    );
}

#[test]
fn species_accepts_proxy_constructors_and_array_constructor_objects() {
    assert_wasm_true(
        r#"
var source = [1, 2], output = {}, log = "";
var species = new Proxy(function() {}, { construct(t, args, newTarget) {
  if (args.length !== 1 || args[0] !== 0 || newTarget !== species) throw 1;
  log += "construct;"; return output;
}});
var constructor = [];
constructor[Symbol.species] = species;
source.constructor = constructor;
var result = source.flatMap(function(v) { log += "map;"; return [v + 2]; });
result === output && output[0] === 3 && output[1] === 4 &&
!Object.prototype.hasOwnProperty.call(output, "length") && log === "construct;map;map;";
"#,
    );
}

#[test]
fn target_writes_define_own_data_properties_without_invoking_setters() {
    assert_wasm_true(
        r#"
var calls = 0, proto = { set 0(value) { calls++; } }, target = Object.create(proto);
var source = [7];
source.constructor = { [Symbol.species]: function() { return target; } };
var result = source.flatMap(function(v) { return [v]; });
var d = Object.getOwnPropertyDescriptor(target, "0");
result === target && calls === 0 && d.value === 7 && d.writable && d.enumerable &&
d.configurable && !Object.prototype.hasOwnProperty.call(target, "length");
"#,
    );
}

#[test]
fn sparse_and_mutating_sources_use_live_has_property_after_the_snapshot() {
    assert_wasm_true(
        r#"
var proto = Object.create(Array.prototype); proto[1] = 7;
var source = [2, , 4]; Object.setPrototypeOf(source, proto);
var log = "";
var result = source.flatMap(function(v, k) {
  log += k;
  if (k === 0) { delete source[2]; source[3] = 9; }
  return [v];
});
result.length === 2 && result[0] === 2 && result[1] === 7 && log === "01";
"#,
    );
}

#[test]
fn non_arrays_are_not_flattened_even_when_spreadable() {
    assert_wasm_true(
        r#"
var object = { 0: 9, length: 1, [Symbol.isConcatSpreadable]: true };
var typed = new Uint8Array([4]);
var values = [object, typed, "ab", null, undefined];
var result = values.flatMap(function(v) { return v; });
var array = [8]; array[Symbol.isConcatSpreadable] = false;
var flattened = [array].flatMap(function(v) { return v; });
result.length === 5 && result[0] === object && result[1] === typed && result[2] === "ab" &&
result[3] === null && result[4] === undefined && flattened[0] === 8;
"#,
    );
}

#[test]
fn boxing_and_fractional_lengths_use_generic_array_like_semantics() {
    assert_wasm_true(
        r#"
var receiver;
var chars = Array.prototype.flatMap.call("ab", function(v, k, r) {
  receiver = r; return [v, k];
});
var fraction = Array.prototype.flatMap.call({ 0: 3, 1: 5, length: "1.9" }, function(v) { return [v]; });
var ok = chars.length === 4 && chars[0] === "a" && chars[1] === 0 && chars[2] === "b" &&
chars[3] === 1 && typeof receiver === "object" && receiver.valueOf() === "ab" &&
fraction.length === 1 && fraction[0] === 3;
var primitives = [true, 4, Symbol("x"), 2n];
for (var i = 0; i < primitives.length; i++) {
  ok = ok && Array.prototype.flatMap.call(primitives[i], function() { throw 1; }).length === 0;
}
for (var j = 0; j < 2; j++) {
  try { Array.prototype.flatMap.call(j === 0 ? null : undefined, function() {}); ok = false; }
  catch (e) { ok = ok && e instanceof TypeError; }
}
ok;
"#,
    );
}

#[test]
fn callback_and_species_errors_stop_observable_work() {
    assert_wasm_true(
        r#"
var source = [1, 2], marker = {}, log = "", target = {};
Object.defineProperty(source, "constructor", { configurable: true, get() {
  log += "constructor;"; throw marker;
}});
var ok = true;
try { source.flatMap(null); ok = false; }
catch (e) { ok = ok && e instanceof TypeError; }
ok = ok && log === "";
try { source.flatMap(function() { log += "map;"; }); ok = false; }
catch (e) { ok = ok && e === marker; }
ok = ok && log === "constructor;";
Object.defineProperty(source, "constructor", { value: { [Symbol.species]: function() { return target; } } });
try { source.flatMap(function(v) { if (v === 2) throw marker; return [v]; }); ok = false; }
catch (e) { ok = ok && e === marker; }
ok && target[0] === 1 && !Object.prototype.hasOwnProperty.call(target, "1");
"#,
    );
}

#[test]
fn source_has_and_get_errors_prevent_later_callbacks() {
    assert_wasm_true(
        r#"
var marker = {}, log = "", ok = true;
var source = new Proxy({ length: 2, 0: 3 }, {
  get(t, k) { log += "get:" + k + ";"; return t[k]; },
  has(t, k) { log += "has:" + k + ";"; throw marker; }
});
try { Array.prototype.flatMap.call(source, function() { log += "map;"; }); ok = false; }
catch (e) { ok = ok && e === marker; }
ok = ok && log === "get:length;has:0;";
source = { length: 1, get 0() { throw marker; } };
try { Array.prototype.flatMap.call(source, function() { ok = false; }); ok = false; }
catch (e) { ok = ok && e === marker; }
ok;
"#,
    );
}

#[test]
fn minimal_program_roots_target_property_definition_without_object_references() {
    assert_wasm_true(
        r#"
var result = [2, 3].flatMap(function(value) { return [value, value * 2]; });
var empty = [].flatMap(function() { throw 1; });
result.length === 4 && result[0] === 2 && result[1] === 4 &&
result[2] === 3 && result[3] === 6 && empty.length === 0;
"#,
    );
}
