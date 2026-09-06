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
        .expect("Array toLocaleString length regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn borrowed_typed_array_uses_own_length_but_direct_method_does_not() {
    assert_wasm_true(
        r#"
var source = new Uint8Array([7, 9]);
Object.defineProperty(source, 'length', { value: 1 });
Array.prototype.toLocaleString.call(source) === '7' && source.toLocaleString() === '7,9';
"#,
    );
}

#[test]
fn bigint_typed_array_uses_own_length_without_changing_element_kind() {
    assert_wasm_true(
        r#"
var source = new BigInt64Array([7n, 9n]);
Object.defineProperty(source, 'length', { value: 1 });
Array.prototype.toLocaleString.call(source) === '7' && source.toLocaleString() === '7,9';
"#,
    );
}

#[test]
fn inherited_typed_array_length_getter_keeps_receiver_identity() {
    assert_wasm_true(
        r#"
var source = new Uint8Array([7, 9]), count = 0, correctThis = false;
var prototype = Object.create(Uint8Array.prototype);
Object.defineProperty(prototype, 'length', { get() { count++; correctThis = this === source; return 1; } });
Object.setPrototypeOf(source, prototype);
var result = Array.prototype.toLocaleString.call(source);
result === '7' && count === 1 && correctThis && source.toLocaleString() === '7,9' && count === 1;
"#,
    );
}

#[test]
fn arguments_length_is_observable_instead_of_its_storage_extent() {
    assert_wasm_true(
        r#"
function check(a, b) {
  Object.defineProperty(arguments, 'length', { value: 1, configurable: true });
  var first = Array.prototype.toLocaleString.call(arguments);
  Object.defineProperty(arguments, 'length', { value: 3 });
  return first === '4' && Array.prototype.toLocaleString.call(arguments) === '4,5,';
}
check(4, 5);
"#,
    );
}

#[test]
fn arguments_length_accessor_is_called_once_with_the_original_receiver() {
    assert_wasm_true(
        r#"
function check(a, b) {
  var source = arguments, calls = 0, correctThis = false;
  Object.defineProperty(source, 'length', { get() { calls++; correctThis = this === source; return 1; } });
  return Array.prototype.toLocaleString.call(source) === '4' && calls === 1 && correctThis;
}
check(4, 5);
"#,
    );
}

#[test]
fn deleted_arguments_length_uses_the_prototype_chain() {
    assert_wasm_true(
        r#"
function check(a, b) {
  var source = arguments;
  delete source.length;
  if (Array.prototype.toLocaleString.call(source) !== '') return false;
  Object.setPrototypeOf(source, { length: 1 });
  return Array.prototype.toLocaleString.call(source) === '4';
}
check(4, 5);
"#,
    );
}

#[test]
fn length_get_and_numeric_coercion_precede_the_first_index_read() {
    assert_wasm_true(
        r#"
var calls = '';
var source = {
  get length() { calls += 'length;'; return { [Symbol.toPrimitive](hint) { calls += hint + ';'; return 1.9; } }; },
  get 0() { calls += 'index;'; return { toLocaleString() { calls += 'call;'; return 'ok'; } }; },
  get 1() { throw 'outside captured length'; }
};
Array.prototype.toLocaleString.call(source) === 'ok' && calls === 'length;number;index;call;';
"#,
    );
}

#[test]
fn typed_array_length_coercion_mutates_live_elements_before_iteration() {
    assert_wasm_true(
        r#"
var source = new Uint8Array([7, 9]), calls = '';
Object.defineProperty(source, 'length', {
  get() { calls += 'length;'; return { valueOf() { calls += 'valueOf;'; source[0] = 3; return 1.9; } }; }
});
Array.prototype.toLocaleString.call(source) === '3' && calls === 'length;valueOf;';
"#,
    );
}

#[test]
fn abrupt_length_get_and_coercion_do_not_read_elements() {
    assert_wasm_true(
        r#"
var marker = {}, reads = 0, ok = true;
var source = { get length() { throw marker; }, get 0() { reads++; return 7; } };
try { Array.prototype.toLocaleString.call(source); ok = false; } catch (e) { ok = ok && e === marker; }
Object.defineProperty(source, 'length', { value: { valueOf() { throw marker; } } });
try { Array.prototype.toLocaleString.call(source); ok = false; } catch (e) { ok = ok && e === marker; }
var typed = new Uint8Array([7]);
Object.defineProperty(typed, 'length', { get() { throw marker; } });
try { Array.prototype.toLocaleString.call(typed); ok = false; } catch (e) { ok = ok && e === marker; }
ok && reads === 0 && typed.toLocaleString() === '7';
"#,
    );
}

#[test]
fn generic_length_applies_to_length_to_zero_fractional_and_string_values() {
    assert_wasm_true(
        r#"
var lengths = [undefined, null, false, NaN, -Infinity, -1, -0.5, 0, 0.9, 1.9, '0x1', true];
var ok = true;
for (var i = 0; i < lengths.length; i++) {
  var source = new Uint8Array([7, 9]);
  Object.defineProperty(source, 'length', { value: lengths[i] });
  ok = ok && Array.prototype.toLocaleString.call(source) === (i < 9 ? '' : '7');
}
ok;
"#,
    );
}

#[test]
fn symbol_and_bigint_lengths_throw_before_any_index_access() {
    assert_wasm_true(
        r#"
var lengths = [Symbol('length'), 1n], reads = 0, ok = true;
for (var i = 0; i < lengths.length; i++) {
  var source = { length: lengths[i], get 0() { reads++; return 7; } };
  try { Array.prototype.toLocaleString.call(source); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
  var typed = new Uint8Array([7]);
  Object.defineProperty(typed, 'length', { value: lengths[i] });
  try { Array.prototype.toLocaleString.call(typed); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
}
ok && reads === 0;
"#,
    );
}

#[test]
fn large_lengths_reach_the_first_get_instead_of_truncating_to_signed_i32() {
    assert_wasm_true(
        r#"
var lengths = [65536, 2147483648, 4294967295];
var marker = {}, reads = 0, ok = true;
for (var i = 0; i < lengths.length; i++) {
  var source = { length: lengths[i], get 0() { reads++; throw marker; } };
  try { Array.prototype.toLocaleString.call(source); ok = false; } catch (e) { ok = ok && e === marker; }
}
ok && reads === lengths.length;
"#,
    );
}

#[test]
fn proxy_receiver_gets_length_once_then_live_indices_without_has() {
    assert_wasm_true(
        r#"
var calls = '', source;
var target = { length: 3, 0: 7, 2: 9 };
source = new Proxy(target, {
  get(t, key, receiver) {
    calls += key + ';';
    if (receiver !== source) throw 'wrong receiver';
    if (key === '0') { t.length = 1; t[2] = 5; t[3] = 99; }
    return Reflect.get(t, key, receiver);
  },
  has() { throw 'toLocaleString must not use HasProperty'; }
});
Array.prototype.toLocaleString.call(source) === '7,,5' && calls === 'length;0;1;2;';
"#,
    );
}

#[test]
fn captured_length_survives_element_calls_while_later_values_remain_live() {
    assert_wasm_true(
        r#"
var calls = 0, source = {
  get length() { calls++; return 3; },
  0: { toLocaleString() { Object.defineProperty(source, 'length', { value: 1 }); delete source[1]; source[2] = 5; source[3] = 99; return 'first'; } },
  1: 7,
  2: 9
};
Array.prototype.toLocaleString.call(source) === 'first,,5' && calls === 1;
"#,
    );
}

#[test]
fn resize_during_length_get_retains_returned_bound_and_live_typed_reads() {
    assert_wasm_true(
        r#"
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var source = new Uint8Array(buffer);
source[0] = 7; source[1] = 9; source[2] = 3; source[3] = 5;
var calls = 0;
Object.defineProperty(source, 'length', { get() { calls++; buffer.resize(1); return 3; } });
Array.prototype.toLocaleString.call(source) === '7,,' && calls === 1;
"#,
    );
}

#[test]
fn detach_during_length_coercion_keeps_the_generic_bound_without_validation() {
    assert_wasm_true(
        r#"
var buffer = new ArrayBuffer(2), source = new Uint8Array(buffer), calls = 0;
Object.defineProperty(source, 'length', { value: { valueOf() { calls++; buffer.transfer(); return 2; } } });
var result = Array.prototype.toLocaleString.call(source), directThrows = false;
try { source.toLocaleString(); } catch (e) { directThrows = e instanceof TypeError; }
result === ',' && calls === 1 && directThrows;
"#,
    );
}

#[test]
fn out_of_bounds_typed_array_observes_a_shadowed_generic_length() {
    assert_wasm_true(
        r#"
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var source = new Uint8Array(buffer, 0, 4), directThrows = false;
buffer.resize(1);
Object.defineProperty(source, 'length', { value: 2 });
var result = Array.prototype.toLocaleString.call(source);
try { source.toLocaleString(); } catch (e) { directThrows = e instanceof TypeError; }
result === ',' && directThrows;
"#,
    );
}

#[test]
fn unshadowed_typed_array_accessors_keep_odd_byte_and_detached_behavior() {
    assert_wasm_true(
        r#"
var buffer = new ArrayBuffer(6, { maxByteLength: 8 }), source = new Uint16Array(buffer);
source[0] = 7; source[1] = 9; source[2] = 3;
buffer.resize(5);
var first = Array.prototype.toLocaleString.call(source);
buffer.transfer();
first === '7,9' && Array.prototype.toLocaleString.call(source) === '';
"#,
    );
}

#[test]
fn nullish_primitive_and_function_receivers_use_generic_object_semantics() {
    assert_wasm_true(
        r#"
var ok = true;
try { Array.prototype.toLocaleString.call(null); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
try { Array.prototype.toLocaleString.call(undefined); ok = false; } catch (e) { ok = ok && e instanceof TypeError; }
function source(a, b) {}
source[0] = 7; source[1] = 9;
Object.defineProperty(source, 'length', { value: 1 });
ok && Array.prototype.toLocaleString.call('ab') === 'a,b' &&
  Array.prototype.toLocaleString.call(7) === '' && Array.prototype.toLocaleString.call(false) === '' &&
  Array.prototype.toLocaleString.call(source) === '7';
"#,
    );
}
