use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_relational_comparison(source: &str) {
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
        .unwrap_or_else(|error| panic!("BigInt relational comparison failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn string_operands_use_exact_bigint_ordering_in_both_directions() {
    assert_relational_comparison(
        r#"
function ordered(a, b, order) {
  return (a < b) === (order < 0) && (a <= b) === (order <= 0) &&
    (a > b) === (order > 0) && (a >= b) === (order >= 0) &&
    (b < a) === (order > 0) && (b <= a) === (order >= 0) &&
    (b > a) === (order < 0) && (b >= a) === (order <= 0);
}
var cases = [[0n, '', 0], [0n, '-0', 0], [1n, '+1', 0], [-1n, '-2', 1],
  [255n, '0xFF', 0], [8n, '0o11', -1], [3n, '0b10', 1],
  [0n, '\uFEFF\u00A0\n', 0], [42n, ' \t42\r ', 0],
  [9007199254740992n, '9007199254740993', -1],
  [-9007199254740992n, '-9007199254740993', 1],
  [9223372036854775807n, '9223372036854775808', -1],
  [-9223372036854775808n, '-9223372036854775809', 1],
  [18446744073709551616n, '18446744073709551616', 0],
  [18446744073709551617n, '18446744073709551616', 1],
  [-18446744073709551617n, '-18446744073709551616', -1],
  [1n << 100n, '0x10000000000000000000000000', 0]];
for (const entry of cases) {
  if (!ordered(entry[0], entry[1], entry[2])) throw 'exact BigInt string ordering';
}
ordered(1n << 70n, '118059162071741' + '1303425', -1);
"#,
    );
}

#[test]
fn invalid_bigint_strings_are_unordered_for_all_four_operators() {
    assert_relational_comparison(
        r#"
function unordered(a, b) {
  return !(a < b) && !(a <= b) && !(a > b) && !(a >= b) &&
    !(b < a) && !(b <= a) && !(b > a) && !(b >= a);
}
var invalid = ['foo', '-', '+', '1.0', '.1', '1e0', '1n', '1_0', '+0x1', '-0x1',
  '0x', '0o8', '0b2', '0x-1', '0x+1', '1 0', 'Infinity', 'NaN', '\u200B0', '\uD800'];
for (const text of invalid) {
  if (!unordered(0n, text) || !unordered(1n << 100n, text)) throw 'invalid string ordering';
}
true;
"#,
    );
}

#[test]
fn nonstring_primitives_convert_only_the_non_bigint_operand_to_number() {
    assert_relational_comparison(
        r#"
function ordered(a, b, order) {
  return (a < b) === (order < 0) && (a <= b) === (order <= 0) &&
    (a > b) === (order > 0) && (a >= b) === (order >= 0) &&
    (b < a) === (order > 0) && (b <= a) === (order >= 0) &&
    (b > a) === (order < 0) && (b >= a) === (order <= 0);
}
function unordered(a, b) {
  return !(a < b) && !(a <= b) && !(a > b) && !(a >= b) &&
    !(b < a) && !(b <= a) && !(b > a) && !(b >= a);
}
var cases = [[0n, false, 0], [0n, true, -1], [1n, false, 1], [1n, true, 0],
  [-3n, true, -1], [31n, true, 1], [0n, null, 0], [-1n, null, -1],
  [0n, -0, 0], [10n, 10.5, -1], [-10n, -10.5, 1],
  [9007199254740993n, 9007199254740992, 1],
  [18446744073709551617n, 18446744073709551616, 1],
  [1n << 100n, Infinity, -1], [-(1n << 100n), -Infinity, 1]];
for (const entry of cases) {
  if (!ordered(entry[0], entry[1], entry[2])) throw 'BigInt numeric ordering';
}
unordered(0n, undefined) && unordered(1n << 100n, undefined) &&
  unordered(0n, NaN) && unordered(1n << 100n, NaN) &&
  ordered('10', '2', -1) && ordered('10', 2, 1) && ordered(1n << 100n, 1n << 100n, 0);
"#,
    );
}

#[test]
fn operand_evaluation_and_number_hint_coercion_remain_left_to_right() {
    assert_relational_comparison(
        r#"
var trace = [];
function left() {
  trace.push('left');
  return { [Symbol.toPrimitive](hint) { trace.push('coerce-left-' + hint); return 1n << 70n; } };
}
function right() {
  trace.push('right');
  return { [Symbol.toPrimitive](hint) { trace.push('coerce-right-' + hint); return '1180591620717411303425'; } };
}
var comparisons = [function() { return left() < right(); }, function() { return left() <= right(); },
  function() { return left() > right(); }, function() { return left() >= right(); }];
for (var i = 0; i < comparisons.length; i++) {
  trace = [];
  if (comparisons[i]() !== (i < 2)) throw 'coerced ordering';
  if (trace.join(',') !== 'left,right,coerce-left-number,coerce-right-number') throw 'coercion order';
}
var fallback = { valueOf() { trace.push('valueOf'); return {}; },
  toString() { trace.push('toString'); return '0'; } };
trace = [];
var fallbackResult = fallback < 1n;
fallbackResult && trace.join(',') === 'valueOf,toString' &&
  Object(1n << 70n) < '1180591620717411303425';
"#,
    );
}

#[test]
fn symbol_and_coercion_errors_propagate_without_reordering_hooks() {
    assert_relational_comparison(
        r#"
var comparisons = [function(a, b) { return a < b; }, function(a, b) { return a <= b; },
  function(a, b) { return a > b; }, function(a, b) { return a >= b; }];
var symbol = Symbol(), marker = {}, caught = 0, trace = [];
for (const compare of comparisons) {
  try { compare(0n, symbol); } catch (e) { if (e instanceof TypeError) caught++; }
  try { compare(symbol, 1n << 100n); } catch (e) { if (e instanceof TypeError) caught++; }
  var left = { [Symbol.toPrimitive]() { trace.push('left'); throw marker; } };
  var right = { [Symbol.toPrimitive]() { trace.push('right'); return '0'; } };
  trace = [];
  try { compare(left, right); } catch (e) { if (e === marker && trace.join(',') === 'left') caught++; }
  left = { [Symbol.toPrimitive]() { trace.push('left'); return symbol; } };
  right = { [Symbol.toPrimitive]() { trace.push('right'); throw marker; } };
  trace = [];
  try { compare(left, right); } catch (e) { if (e === marker && trace.join(',') === 'left,right') caught++; }
}
caught === 16;
"#,
    );
}

#[test]
fn statically_known_symbols_wait_for_the_rhs_then_throw_type_error() {
    assert_relational_comparison(
        r#"
var calls = 0, caught = 0;
var symbol = Symbol();
function rhs() { calls++; return 0; }
try { symbol < rhs(); } catch (error) { if (error instanceof TypeError) caught++; }
try { symbol <= rhs(); } catch (error) { if (error instanceof TypeError) caught++; }
try { symbol > rhs(); } catch (error) { if (error instanceof TypeError) caught++; }
try { symbol >= rhs(); } catch (error) { if (error instanceof TypeError) caught++; }
calls === 4 && caught === 4;
"#,
    );
}
