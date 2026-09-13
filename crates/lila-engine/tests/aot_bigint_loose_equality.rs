use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_equality(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("BigInt equality failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn string_comparisons_are_exact_across_inline_and_heap_bigints() {
    assert_equality(
        r#"
function same(integer, text) {
  return integer == text && text == integer && !(integer != text) && !(text != integer) &&
    integer !== text && text !== integer;
}
var cases = [[0n, ''], [0n, '-0'], [1n, '+1'], [-1n, '-1'],
  [900719925474099101n, '900719925474099101'],
  [9223372036854775807n, '9223372036854775807'],
  [-9223372036854775808n, '-9223372036854775808'],
  [9223372036854775808n, '9223372036854775808'],
  [18446744073709551616n, '18446744073709551616'],
  [-18446744073709551617n, '-18446744073709551617']];
for (const pair of cases) if (!same(pair[0], pair[1])) throw 'exact equality';
function different(a, b) { return a != b && b != a && !(a == b) && !(b == a); }
different(900719925474099102n, '900719925474099101') &&
  different(18446744073709551617n, '18446744073709551616') &&
  different(-18446744073709551617n, '18446744073709551617');
"#,
    );
}

#[test]
fn grammar_failures_are_normal_inequality_while_bigint_conversion_throws() {
    assert_equality(
        r#"
function equal(a, b) { return a == b && b == a && !(a != b) && !(b != a); }
var valid = [[0n, '\uFEFF\u00A0\n'], [42n, ' \t42\r '], [255n, '0xFF'],
  [255n, '0Xff'], [63n, '0o77'], [63n, '0O77'], [5n, '0b101'], [5n, '0B101'],
  [18446744073709551616n, '0x10000000000000000']];
for (const pair of valid) if (!equal(pair[0], pair[1])) throw 'valid grammar';
var invalid = ['foo', '-', '+', '1.0', '1e0', '1n', '1_0', '+0x1', '-0x1',
  '0x', '0o8', '0b2', '0x-1', '0x+1', '1 0', '\u200B0', '\uD800'];
var caught = 0;
function unequal(a, b) { return a != b && b != a && !(a == b) && !(b == a); }
for (const text of invalid) {
  if (!unequal(0n, text) || !unequal(1n << 70n, text)) throw 'invalid equality';
  try { BigInt(text); } catch (e) { if (e instanceof SyntaxError) caught++; }
}
caught === invalid.length;
"#,
    );
}

#[test]
fn object_coercion_uses_default_hint_once_and_preserves_thrown_values() {
    assert_equality(
        r#"
var trace = [], marker = {}, integer = 1n << 70n;
var object = {
  [Symbol.toPrimitive](hint) { trace.push(hint); return '1180591620717411303424'; },
  valueOf() { throw 'unexpected valueOf'; }, toString() { throw 'unexpected toString'; }
};
function compare(a, b) { return a == b; }
var first = compare(integer, object), second = compare(object, integer), received;
try { compare(integer, { [Symbol.toPrimitive]() { throw marker; } }); } catch (e) { received = e; }
var fallback = { valueOf() { trace.push('valueOf'); return {}; }, toString() { trace.push('toString'); return '0'; } };
var third = compare(0n, fallback), boxed = compare(Object(integer), '1180591620717411303424');
first && second && third && boxed && received === marker &&
  trace.join(',') === 'default,default,valueOf,toString';
"#,
    );
}

#[test]
fn operand_evaluation_precedes_coercion_and_nonstring_neighbors_remain_exact() {
    assert_equality(
        r#"
var trace = [];
function left() { trace.push('left'); return { valueOf() { trace.push('coerce'); return '1'; } }; }
function right() { trace.push('right'); return 1n; }
var equal = left() == right();
function same(a, b) { return a == b; }
function different(a, b) { return a != b; }
var object = { valueOf() { throw 'object identity must not coerce'; } };
equal && trace.join(',') === 'left,right,coerce' && same(1n, true) && same(0n, false) &&
  same(1n, 1) && different(1n, 1.5) && different(1n, Infinity) && different(1n, NaN) &&
  different(0n, null) && different(0n, undefined) && different(1n, Symbol()) &&
  same(object, object) && different(object, {});
"#,
    );
}
