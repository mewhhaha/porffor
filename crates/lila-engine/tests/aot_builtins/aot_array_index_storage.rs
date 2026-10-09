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
        .expect("Array index storage regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn rest_arrays_shrink_through_dense_holes_without_resurrecting_elements() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
var dense = collect(1, 2, 3, 4), holes = collect(1, 2, 3, 4);
delete holes[2];
dense.length = 0; dense.length = 4;
holes.length = 0; holes.length = 4;
var ok = dense.length === 4 && holes.length === 4;
for (var i = 0; i < 4; i++) ok = ok && !(i in dense) && !(i in holes);
ok && Object.keys(dense).length === 0 && Object.keys(holes).length === 0;
"#,
    );
}

#[test]
fn shrinking_length_preserves_nonconfigurable_stop_and_failure_modes() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
function blocked() {
  var result = collect(1, 2, 3, 4, 5);
  delete result[4];
  Object.defineProperty(result, '1', { configurable: false });
  return result;
}
var assigned = blocked(), defined = blocked(), strict = blocked();
var ok = !Reflect.set(assigned, 'length', 0);
ok = !Reflect.defineProperty(defined, 'length', { value: 0, writable: false }) && ok;
var caught = false;
try { (function() { 'use strict'; strict.length = 0; })(); }
catch (error) { caught = error instanceof TypeError; }
var arrays = [assigned, defined, strict];
for (var i = 0; i < arrays.length; i++) {
  var array = arrays[i];
  ok = ok && array.length === 2 && array[0] === 1 && array[1] === 2 &&
    !(2 in array) && !(3 in array) && !(4 in array);
}
ok && caught && Object.getOwnPropertyDescriptor(assigned, 'length').writable &&
  !Object.getOwnPropertyDescriptor(defined, 'length').writable &&
  !Reflect.set(defined, '2', 7) && defined.length === 2;
"#,
    );
}

#[test]
fn forward_and_reverse_operations_visit_dense_properties_after_holes() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
var array = collect(0, 1, 2, 3, 4);
delete array[0]; delete array[4];
var keys = Object.keys(array).join(',');
var values = Object.values(array).join(',');
var entries = Object.entries(array).map(function(entry) { return entry.join(':'); }).join(',');
var before = array.indexOf(2) === 2 && array.lastIndexOf(2) === 2 &&
  array.indexOf(undefined) === -1 && array.lastIndexOf(undefined) === -1;
var calls = '';
array.forEach(function(value, index, receiver) {
  calls += index + ':' + value + ';';
  if (index === 1) { delete receiver[2]; receiver[3] = 30; receiver[4] = 40; receiver[5] = 50; }
});
keys === '1,2,3' && values === '1,2,3' && entries === '1:1,2:2,3:3' &&
  before && calls === '1:1;3:30;4:40;' && array.length === 6;
"#,
    );
}

#[test]
fn index_definitions_respect_nonwritable_length_for_every_descriptor_kind() {
    assert_wasm_true(
        r#"
var descriptors = [
  {value: 9, writable: true, enumerable: true, configurable: true},
  {get: function() { return 9; }, configurable: true},
  {}
];
var ok = true;
for (var sparse = 0; sparse < 2; sparse++) {
  for (var i = 0; i < descriptors.length; i++) {
    var array = [7], length = sparse ? 2000001 : 1;
    if (sparse) array[2000000] = 7;
    Object.defineProperty(array, 'length', {writable: false});
    var accepted = Reflect.defineProperty(array, String(length), descriptors[i]);
    var caught = false;
    try { Object.defineProperty(array, String(length + 1), descriptors[i]); }
    catch (error) { caught = error instanceof TypeError; }
    ok = ok && !accepted && caught && array.length === length &&
      !Object.hasOwn(array, String(length)) && !Object.hasOwn(array, String(length + 1));
    ok = Reflect.defineProperty(array, '0', {value: 8}) && array[0] === 8 && ok;
    if (sparse) {
      ok = Reflect.defineProperty(array, '1', {value: 9}) && array[1] === 9 &&
        array.length === length && ok;
    }
  }
}
ok;
"#,
    );
}

#[test]
fn inherited_indexes_remain_observable_across_dense_holes() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
var array = collect(0, 1, 2), reads = 0, calls = '';
delete array[0]; delete array[2];
var prototype = Object.create(Array.prototype);
Object.defineProperty(prototype, '0', { get: function() { reads++; return 9; } });
Object.setPrototypeOf(array, prototype);
var found = array.indexOf(9) === 0 && array.lastIndexOf(9) === 0;
array.forEach(function(value, index) { calls += index + ':' + value + ';'; });
found && reads === 3 && calls === '0:9;1:1;' && Object.keys(array).join(',') === '1';
"#,
    );
}

#[test]
fn sparse_traversal_keeps_numeric_order_and_stops_length_shrink_without_gets() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
var array = collect(1, 2, 3, 4), reads = 0;
delete array[0];
array[4294967294] = 'high';
Object.defineProperty(array, '2000000', {
  value: 'middle', configurable: false, enumerable: true, writable: true
});
Object.defineProperty(array, '3000000', {
  get: function() { reads++; return 'accessor'; }, configurable: true, enumerable: true
});
var keys = Object.keys(array).join(',');
var accepted = Reflect.set(array, 'length', 0);
keys === '1,2,3,2000000,3000000,4294967294' && !accepted && reads === 0 &&
  array.length === 2000001 && array[2000000] === 'middle' &&
  !(3000000 in array) && !(4294967294 in array) &&
  array.lastIndexOf('middle') === 2000000 && array.indexOf(3) === 2;
"#,
    );
}

#[test]
fn length_growth_transfers_sparse_values_and_complete_accessor_descriptors() {
    assert_wasm_true(
        r#"
var array = [], retained = {}, reads = 0, writes = 0, receiver;
var getter = function() { reads++; return retained; };
var setter = function(value) { writes += value; receiver = this; };
Object.defineProperty(array, '100', { value: retained, configurable: false });
Object.defineProperty(array, '101', {
  get: getter, set: setter, configurable: true, enumerable: true
});
array[102] = 'deleted'; delete array[102];
array.length = 128;
var descriptor = Object.getOwnPropertyDescriptor(array, '101');
var before = reads === 0 && writes === 0 && array[100] === retained &&
  descriptor.get === getter && descriptor.set === setter && descriptor.configurable &&
  descriptor.enumerable && !(102 in array) && Object.keys(array).join(',') === '101';
array[101] = 7;
var value = array[101];
var accepted = Reflect.defineProperty(array, 'length', { value: 0, writable: false });
before && value === retained && reads === 1 && writes === 7 && receiver === array &&
  !accepted && array.length === 101 && array[100] === retained && !(101 in array) &&
  !Object.getOwnPropertyDescriptor(array, '100').writable &&
  !Object.getOwnPropertyDescriptor(array, 'length').writable;
"#,
    );
}

#[test]
fn sparse_deletion_preserves_flags_holes_and_reinsertion_without_accessor_calls() {
    assert_wasm_true(
        r#"
var array = [], calls = 0;
array[2000000] = 'remove';
Object.defineProperty(array, '2000001', { value: 'retain', configurable: false });
Object.defineProperty(array, '2000002', {
  get() { calls++; return 'getter'; },
  set(value) { calls++; }, configurable: true
});
var removed = Reflect.deleteProperty(array, '2000000');
var blocked = Reflect.deleteProperty(array, '2000001');
var strictFailure = false;
try { (function() { 'use strict'; delete array[2000001]; })(); }
catch (error) { strictFailure = error instanceof TypeError; }
var accessorRemoved = delete array[2000002];
var absent = Reflect.deleteProperty(array, '3000000');
var before = removed && !blocked && strictFailure && accessorRemoved && absent &&
  calls === 0 && !(2000000 in array) && !(2000002 in array) &&
  array[2000001] === 'retain' && array.length === 2000003 &&
  Object.getOwnPropertyNames(array).join(',') === '2000001,length';
array[2000000] = 'new';
Object.defineProperty(array, '2000002', { get() { return 'new getter'; }, configurable: true });
var descriptor = Object.getOwnPropertyDescriptor(array, '2000002');
before && array[2000000] === 'new' && array[2000002] === 'new getter' &&
  descriptor.set === undefined && calls === 0;
"#,
    );
}

#[test]
fn deleting_and_recreating_argument_indexes_disconnects_parameter_maps() {
    assert_wasm_true(
        r#"
function mapped(value) {
  var removed = delete arguments[0];
  value = 7;
  var absent = !(0 in arguments) && !Object.hasOwn(arguments, '0') &&
    arguments[0] === undefined;
  arguments[0] = 9;
  var descriptor = Object.getOwnPropertyDescriptor(arguments, '0');
  return removed && absent && value === 7 && arguments[0] === 9 &&
    descriptor.value === 9 && descriptor.writable && descriptor.enumerable && descriptor.configurable;
}
function unmapped(value) {
  'use strict';
  var removed = delete arguments[0];
  var absent = !(0 in arguments) && arguments[0] === undefined;
  arguments[0] = 9;
  return removed && absent && value === 1 && arguments[0] === 9;
}
mapped(1) && unmapped(1);
"#,
    );
}

#[test]
fn indexed_growth_promotes_sparse_entries_before_new_capacity_is_published() {
    assert_wasm_true(
        r#"
function collect(...values) { return values; }
var array = collect(0, 1, 2, 3), retained = {};
array[100] = retained;
array[2000000] = 'outside';
for (var i = 4; i <= 64; i++) array[i] = i;
var before = array[100] === retained && Object.hasOwn(array, '100') &&
  Object.getOwnPropertyDescriptor(array, '100').value === retained &&
  array[2000000] === 'outside';
array.length = 101;
before && array.length === 101 && array[64] === 64 && array[100] === retained &&
  !(2000000 in array) && Object.keys(array).length === 66 && array.lastIndexOf(retained) === 100;
"#,
    );
}

#[test]
fn existing_dense_capacity_owns_indexes_above_the_growth_limit() {
    assert_wasm_true(
        r#"
var array = [];
array.length = 600000;
array[600000] = 1;
array[1100000] = 7;
var descriptor = Object.getOwnPropertyDescriptor(array, '1100000');
var before = array[1100000] === 7 && Object.hasOwn(array, '1100000') &&
  descriptor.value === 7 && descriptor.writable && descriptor.enumerable && descriptor.configurable;
array.length = 600001;
before && array[600000] === 1 && !(1100000 in array) && array.length === 600001;
"#,
    );
}

#[test]
fn dense_literal_json_and_arguments_producers_preserve_own_index_enumeration() {
    assert_wasm_true(
        r#"
var literal = [, undefined, 2, 3];
var source = '';
var parsed = JSON.parse('[1,2,3,4]', function(key, value, context) {
  if (key === '1') return undefined;
  if (key === '2') source = context.source;
  return value;
});
var before = Object.keys(literal).join(',') === '1,2,3' &&
  Object.keys(parsed).join(',') === '0,2,3' && source === '3';
parsed.length = 2;
function mapped(first, second, third) {
  delete arguments[0];
  arguments[8] = 9;
  second = 7;
  return Object.keys(arguments).join(',') === '1,2,8' && arguments[1] === 7;
}
function unmapped(first, second, third) {
  'use strict';
  delete arguments[0];
  arguments[8] = 9;
  second = 7;
  return Object.keys(arguments).join(',') === '1,2,8' && arguments[1] === 2;
}
before && JSON.stringify(parsed) === '[1,null]' && mapped(1, 2, 3) && unmapped(1, 2, 3);
"#,
    );
}

#[test]
fn dense_fill_reset_and_refill_preserve_holes_values_and_key_uniqueness() {
    assert_wasm_true(
        r#"
var array = [], retained = {}, ok = true;
for (var cycle = 0; cycle < 3; cycle++) {
  for (var i = 0; i < 2048; i++) array[i] = i;
  delete array[7]; array[511] = retained;
  var keys = Object.keys(array);
  ok = ok && array.length === 2048 && keys.length === 2047 &&
    keys[0] === '0' && keys[2046] === '2047' && array[511] === retained &&
    array.indexOf(retained) === 511 && array.lastIndexOf(retained) === 511;
  array.length = 0;
  array.length = 2048;
  ok = ok && !(0 in array) && !(511 in array) && !(2047 in array) && Object.keys(array).length === 0;
}
ok;
"#,
    );
}
