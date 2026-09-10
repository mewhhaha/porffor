use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_integer_indexed_get(source: &str) {
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
        .expect("integer-indexed references must execute through Wasm AOT");
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn numeric_and_bigint_compound_references_read_live_elements() {
    assert_integer_indexed_get(
        r#"
function check(values, initial, step, expected) {
  if (Reflect.get(values, '0') !== initial) throw 'Reflect.get element';
  if ((values[0] += step) !== expected || values[0] !== expected) throw 'compound element';
  if (values[0]++ !== expected || values[0] !== expected + step) throw 'update element';
}
check(new Uint8Array([7]), 7, 1, 8);
check(new Float64Array([7]), 7, 1, 8);
check(new BigInt64Array([7n]), 7n, 1n, 8n);
check(new BigUint64Array([9223372036854775808n]),
  9223372036854775808n, 1n, 9223372036854775809n);
var floats = new Float64Array([4000]);
var bits = new BigUint64Array(floats.buffer);
var before = bits[0];
if ((bits[0] += 1n) !== before + 1n || floats[0] !== 4000.0000000000005) {
  throw 'shared buffer element';
}
true;
"#,
    );
}

#[test]
fn canonical_numeric_misses_stop_the_prototype_walk() {
    assert_integer_indexed_get(
        r#"
var values = new Uint8Array([7]), reads = 0, receiver, symbol = Symbol('named');
var prototype = {
  get 2() { reads++; return 99; },
  get '-0'() { reads++; return 99; },
  get NaN() { reads++; return 99; },
  get Infinity() { reads++; return 99; },
  get '0.5'() { reads++; return 99; },
  get '01'() { receiver = this; return 11; },
  get [symbol]() { receiver = this; return 13; }
};
Object.setPrototypeOf(values, prototype);
var child = Object.create(values);
for (var key of ['2', '-0', 'NaN', 'Infinity', '0.5']) {
  if (Reflect.get(values, key) !== undefined || Reflect.get(child, key) !== undefined) {
    throw 'canonical numeric key inherited';
  }
}
if (reads !== 0 || Reflect.get(child, '0') !== 7) throw 'integer indexed prototype';
if (Reflect.get(child, '01') !== 11 || receiver !== child) throw 'named receiver';
if (Reflect.get(child, symbol) !== 13 || receiver !== child) throw 'symbol receiver';
var proxy = new Proxy(values, {});
if (Reflect.get(proxy, '0') !== 7) throw 'proxy forwarding';
true;
"#,
    );
}

#[test]
fn compound_get_precedes_rhs_and_preserves_abrupt_values() {
    assert_integer_indexed_get(
        r#"
var values = new BigInt64Array([7n]), trace = [];
function base() { trace.push('base'); return values; }
var key = { [Symbol.toPrimitive](hint) { trace.push(hint); return '0'; } };
function right() {
  trace.push('rhs');
  values[0] = 100n;
  return { valueOf() { trace.push('coerce'); return 2n; } };
}
var result = base()[key] += right();
if (result !== 9n || values[0] !== 9n || trace.join(',') !== 'base,string,rhs,coerce') {
  throw 'compound reference order';
}
var marker = {}, received;
function abrupt() { throw marker; }
try { values[0] += abrupt(); } catch (error) { received = error; }
if (received !== marker || values[0] !== 9n) throw 'RHS abrupt identity';
var calls = 0;
try { values[0] += { valueOf() { calls++; return 1; } }; }
catch (error) { received = error; }
if (!(received instanceof TypeError) || calls !== 1 || values[0] !== 9n) {
  throw 'mixed numeric kinds';
}
true;
"#,
    );
}

#[test]
fn resize_and_detach_are_observed_at_each_integer_indexed_get() {
    assert_integer_indexed_get(
        r#"
var buffer = new ArrayBuffer(16, { maxByteLength: 32 });
var fixed = new BigInt64Array(buffer, 0, 2), tracking = new BigInt64Array(buffer);
fixed[0] = 7n;
var result = fixed[0] += (buffer.resize(0), 2n);
if (result !== 9n || Reflect.get(fixed, '0') !== undefined) throw 'resize after Get';
buffer.resize(8);
tracking[0] = 11n;
if (Reflect.get(fixed, '0') !== undefined || Reflect.get(tracking, '0') !== 11n) {
  throw 'fixed versus tracking bounds';
}
var calls = 0, received;
var key = { toString() { buffer.resize(0); return '0'; } };
try { tracking[key] += (calls++, 1n); } catch (error) { received = error; }
if (!(received instanceof TypeError) || calls !== 1) throw 'resize during key conversion';
buffer.resize(16);
fixed[0] = 7n;
result = fixed[0] += (buffer.transfer(), 2n);
if (result !== 9n || Reflect.get(fixed, '0') !== undefined) throw 'detach after Get';
try { fixed[0] += (calls++, 1n); } catch (error) { received = error; }
if (!(received instanceof TypeError) || calls !== 2) throw 'detached Get before RHS';
var numbers = new Uint8Array(new ArrayBuffer(1, { maxByteLength: 2 }));
numbers.buffer.resize(0);
if (!Number.isNaN(numbers[0] += 1) || Reflect.get(numbers, '0') !== undefined) {
  throw 'missing number element';
}
true;
"#,
    );
}
