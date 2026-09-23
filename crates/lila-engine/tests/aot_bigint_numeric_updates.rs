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

#[test]
fn immutable_symbol_updates_preserve_error_tags_and_captured_binding_values() {
    let mut source = String::from(
        r#"
function check(condition, label) { if (!condition) throw label; }
function isTypeError(error) {
  return typeof error === 'object' && error instanceof TypeError &&
    Object.getPrototypeOf(error) === TypeError.prototype;
}
"#,
    );
    for (scope, captured) in [("captured", true), ("uncaptured", false)] {
        for (label, update) in [
            ("postIncrement", "value++"),
            ("preIncrement", "++value"),
            ("postDecrement", "value--"),
            ("preDecrement", "--value"),
        ] {
            let binding = format!("{scope}_{label}");
            let update = update.replace("value", &binding);
            source.push_str(&format!(
                "const {binding} = Symbol('{binding}');\nconst original_{binding} = {binding};\n"
            ));
            if captured {
                source.push_str(&format!(
                    "function read_{binding}() {{ return {binding}; }}\n"
                ));
            }
            source.push_str(&format!(
                r#"
let caught_{binding};
try {{ {update}; }} catch (error) {{ caught_{binding} = error; }}
check(isTypeError(caught_{binding}), '{binding} error payload and tag');
check({binding} === original_{binding}, '{binding} original value');
"#,
            ));
            if captured {
                source.push_str(&format!(
                    "check(read_{binding}() === original_{binding}, '{binding} captured value');\n"
                ));
            }
        }
    }
    source.push_str("true;");
    assert_numeric_updates(&source);
}

#[test]
fn immutable_updates_preserve_numeric_coercion_order_and_arbitrary_user_throws() {
    let mut source = String::from(
        r#"
function check(condition, label) { if (!condition) throw label; }
function isTypeError(error) {
  return typeof error === 'object' && error instanceof TypeError &&
    Object.getPrototypeOf(error) === TypeError.prototype;
}
const symbol = Symbol('conversion result');
let conversions = 0;
const converted = {[Symbol.toPrimitive](hint) {
  check(hint === 'number', 'numeric conversion hint');
  conversions++;
  return symbol;
}};
function readConverted() { return converted; }
"#,
    );
    for (index, update) in ["converted++", "++converted", "converted--", "--converted"]
        .into_iter()
        .enumerate()
    {
        let count = index + 1;
        source.push_str(&format!(
            r#"
let convertedError{index};
try {{ {update}; }} catch (error) {{ convertedError{index} = error; }}
check(isTypeError(convertedError{index}) && conversions === {count}, 'object to Symbol conversion');
check(readConverted() === converted, 'coercion does not replace the const binding');
"#,
        ));
    }
    source.push_str(
        r#"
const markers = [undefined, null, 7, 'sentinel', Symbol('thrown'), 1n, NaN, {}, new TypeError('sentinel')];
for (const marker of markers) {
  let calls = 0;
  const value = {valueOf() { calls++; throw marker; }};
  function read() { return value; }
"#,
    );
    for (index, update) in ["value++", "++value", "value--", "--value"]
        .into_iter()
        .enumerate()
    {
        let count = index + 1;
        source.push_str(&format!(
            r#"
  let caught{index} = false;
  try {{ {update}; }} catch (error) {{
    caught{index} = true;
    check(Object.is(error, marker) && typeof error === typeof marker, 'arbitrary thrown value identity');
  }}
  check(caught{index} && calls === {count} && read() === value, 'throw precedes immutable publication');
"#,
        ));
    }
    source.push_str(
        r#"
}
for (const value of [7, 1n, 9223372036854775808n]) {
  let caught;
  try { value++; } catch (error) { caught = error; }
  check(isTypeError(caught), 'successful ToNumeric reaches the immutable-binding error');
}
true;
"#,
    );
    assert_numeric_updates(&source);
}

#[test]
fn primitive_numeric_throws_reach_nested_catch_finally_and_function_return_routes() {
    let mut source = String::from(
        r#"
function check(condition, label) { if (!condition) throw label; }
function isTypeError(error) {
  return typeof error === 'object' && error instanceof TypeError &&
    Object.getPrototypeOf(error) === TypeError.prototype;
}
const value = Symbol('nested update');
function read() { return value; }
const original = value;
const trace = [];
"#,
    );
    for update in ["value++", "++value", "value--", "--value"] {
        source.push_str(&format!(
            r#"
trace.length = 0;
for (let iteration = 0; iteration < 2; iteration++) {{
  try {{
    try {{
      if (iteration >= 0) {{ {update}; trace.push('after'); }}
    }} finally {{ trace.push('inner finally'); }}
  }} catch (error) {{
    check(isTypeError(error), 'nested conversion error');
    trace.push('catch');
  }} finally {{ trace.push('outer finally'); }}
}}
check(trace.join(',') === 'inner finally,catch,outer finally,inner finally,catch,outer finally',
      'numeric throw follows nested catch/finally order');
check(read() === original && value === original, 'nested update leaves binding intact');
"#,
        ));
    }
    source.push_str(
        r#"
function updateWithoutHandler() {
  const local = Symbol('function update');
  return local++;
}
let returnedError;
try { updateWithoutHandler(); } catch (error) { returnedError = error; }
check(isTypeError(returnedError), 'function completion retains the thrown object tag');
true;
"#,
    );
    assert_numeric_updates(&source);
}
