use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn run_numeric_updates(source: &str) -> String {
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
        .expect("numeric updates must execute through Wasm AOT");
    outcome.note
}

fn assert_numeric_updates(source: &str) {
    let note = run_numeric_updates(source);
    assert!(note.contains("boolean(true)"), "{note}\n{source}");
}

#[test]
fn standalone_updates_reach_bigint_arithmetic_without_binary_operators() {
    for source in [
        "let value = 9223372036854775807n; ++value;",
        "function next(value) { return ++value; } next(9223372036854775807n);",
    ] {
        assert_eq!(
            run_numeric_updates(source),
            "wasm-aot completion: bigint(9223372036854775808n)",
            "{source}",
        );
    }
}

#[test]
fn local_updates_preserve_old_and_new_bigint_representations() {
    assert_numeric_updates(
        r#"
function check() {
  let positive = 9223372036854775807n;
  const oldInline = positive++;
  if (oldInline !== 9223372036854775807n || positive !== 9223372036854775808n) return false;
  const oldHeap = positive--;
  if (oldHeap !== 9223372036854775808n || positive !== 9223372036854775807n) return false;
  let negative = -9223372036854775808n;
  if (--negative !== -9223372036854775809n) return false;
  if (++negative !== -9223372036854775808n) return false;
  let wide = 18446744073709551615n;
  if (++wide !== 18446744073709551616n || --wide !== 18446744073709551615n) return false;
  let number = 7;
  return number++ === 7 && --number === 7;
}
check();
"#,
    );
}

#[test]
fn property_and_super_updates_publish_the_selected_numeric_pair() {
    assert_numeric_updates(
        r#"
var trace = [], stored = 9223372036854775807n;
var value = {
  get count() { trace.push('get'); return {valueOf() { trace.push('coerce'); return stored; }}; },
  set count(next) { trace.push('set'); stored = next; }
};
var previous = value.count++;
if (previous !== 9223372036854775807n || stored !== 9223372036854775808n ||
    trace.join(',') !== 'get,coerce,set') throw 'property numeric update';
if (--value.count !== 9223372036854775807n || stored !== 9223372036854775807n) {
  throw 'property prefix tag';
}
var prototype = {get count() { return this.current; }, set count(next) { this.current = next; }};
var child = {
  current: 9223372036854775807n,
  increment() { return super.count++; },
  decrement() { return --super.count; }
};
Object.setPrototypeOf(child, prototype);
if (child.increment() !== 9223372036854775807n || child.current !== 9223372036854775808n ||
    child.decrement() !== 9223372036854775807n) throw 'super numeric update';
var number = {count: -0};
Object.is(number.count++, -0) && number.count === 1;
"#,
    );
}

#[test]
fn environment_updates_preserve_postfix_tags_across_inline_boundaries() {
    assert_numeric_updates(
        r#"
var scope = {count: 9223372036854775807n}, previous, next;
with (scope) { previous = count++; next = --count; }
if (previous !== 9223372036854775807n || next !== 9223372036854775807n ||
    scope.count !== 9223372036854775807n) throw 'with environment update';
function borrowed() {
  let count = 9223372036854775808n;
  const oldHeap = eval('count--');
  const newHeap = eval('++count');
  return oldHeap === 9223372036854775808n && newHeap === 9223372036854775808n &&
    count === 9223372036854775808n;
}
globalThis.updateCount = 9223372036854775807n;
var oldGlobal = updateCount++;
var newGlobal = --updateCount;
borrowed() && oldGlobal === 9223372036854775807n && newGlobal === 9223372036854775807n &&
  globalThis.updateCount === 9223372036854775807n;
"#,
    );
}

#[test]
fn typed_array_prefix_results_precede_integer_element_conversion() {
    assert_numeric_updates(
        r#"
var signed = new BigInt64Array([9223372036854775807n]);
if (++signed[0] !== 9223372036854775808n || signed[0] !== -9223372036854775808n) {
  throw 'signed increment wrapping';
}
if (--signed[0] !== -9223372036854775809n || signed[0] !== 9223372036854775807n) {
  throw 'signed decrement wrapping';
}
var unsigned = new BigUint64Array([18446744073709551615n]);
if (++unsigned[0] !== 18446744073709551616n || unsigned[0] !== 0n) {
  throw 'unsigned increment wrapping';
}
if (unsigned[0]-- !== 0n || unsigned[0] !== 18446744073709551615n) {
  throw 'unsigned postfix wrapping';
}
true;
"#,
    );
}

#[test]
fn abrupt_coercions_and_setters_do_not_publish_numeric_results() {
    assert_numeric_updates(
        r#"
var marker = {}, received, sets = 0, conversions = 0;
var coercion = {get count() { return {valueOf() { throw marker; }}; }, set count(v) { sets++; }};
try { coercion.count++; } catch (error) { received = error; }
if (received !== marker || sets !== 0) throw 'coercion abrupt identity';
var setter = {
  get count() { return {valueOf() { conversions++; return 9223372036854775807n; }}; },
  set count(v) { sets++; if (v !== 9223372036854775808n) throw 'wrong setter value'; throw marker; }
};
try { ++setter.count; } catch (error) { received = error; }
if (received !== marker || conversions !== 1 || sets !== 1) throw 'setter abrupt identity';
var readOnly = {};
Object.defineProperty(readOnly, 'count', {value: {valueOf() { conversions++; return 9223372036854775807n; }}});
try { (function() { 'use strict'; readOnly.count++; })(); } catch (error) { received = error; }
received instanceof TypeError && conversions === 2;
"#,
    );
}
