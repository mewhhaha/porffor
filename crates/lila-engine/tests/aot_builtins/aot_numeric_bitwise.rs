use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_numeric_bitwise(source: &str) {
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
        .unwrap_or_else(|error| panic!("bitwise execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn number_bitwise_operations_preserve_wrapping_truncation_and_shift_counts() {
    assert_numeric_bitwise(
        r#"
(2147483649 << 1) === 2 && (2147483649 >> 1) === -1073741824 &&
(2147483649 >>> 1) === 1073741824 && (4294967297 << 33) === 2 &&
(4294967295 >> 0) === -1 && (4294967295 >>> 0) === 4294967295 &&
(7.9 << 1.9) === 14 && (7.9 >> 1.9) === 3 && (7.9 >>> 1.9) === 3 &&
(1 << 31) === -2147483648 && (1 << 32) === 1 && (1 << -1) === -2147483648 &&
(-1 >>> 1) === 2147483647 && (NaN << 4) === 0 && (Infinity >>> 1) === 0 &&
(9007199254740991 >>> 0) === 4294967295 && (9007199254740992 >>> 0) === 0 &&
(4294967295 & 2147483648) === -2147483648 && (2147483648 | 1) === -2147483647 &&
(4294967295 ^ 2147483648) === 2147483647 && Object.is(-0 >>> 0, 0);
"#,
    );
}

#[test]
fn statically_number_valued_operands_still_evaluate_left_then_right_once() {
    assert_numeric_bitwise(
        r#"
var log = [], sentinel = {}, caught;
var shifted = (log.push('left'), 5) << (log.push('right'), 3);
if (shifted !== 40 || log.join('|') !== 'left|right') throw 'number evaluation';
log = [];
function fail() { log.push('throw'); throw sentinel; }
try { (log.push('left'), 5) >>> (fail(), 3); } catch (error) { caught = error; }
if (caught !== sentinel || log.join('|') !== 'left|throw') throw 'number abrupt right';
log = [];
try { (fail(), 5) & (log.push('right'), 3); } catch (error) { caught = error; }
caught === sentinel && log.join('|') === 'throw';
"#,
    );
}

#[test]
fn dynamic_operands_are_both_evaluated_before_ordered_numeric_conversion() {
    assert_numeric_bitwise(
        r#"
var log = [];
var left = {valueOf() { log.push('old conversion'); return 1; }};
function evaluateLeft() { log.push('evaluate left'); return left; }
function evaluateRight() {
  log.push('evaluate right');
  left.valueOf = function() { log.push('convert left'); return 8; };
  return {[Symbol.toPrimitive](hint) { log.push('convert right ' + hint); return 1; }};
}
var shifted = evaluateLeft() >> evaluateRight();
shifted === 4 && log.join('|') === 'evaluate left|evaluate right|convert left|convert right number';
"#,
    );
}

#[test]
fn abrupt_evaluation_and_conversion_preserve_values_and_skip_later_steps() {
    assert_numeric_bitwise(
        r#"
var log = [], sentinel = {}, caught;
var left = {valueOf() { log.push('convert left'); throw sentinel; }};
var right = {valueOf() { log.push('convert right'); return 1; }};
function fail() { log.push('evaluate right'); throw sentinel; }
try { left << fail(); } catch (error) { caught = error; }
if (caught !== sentinel || log.join('|') !== 'evaluate right') throw 'evaluation abrupt';
log = [];
try { left >>> (log.push('evaluate right'), right); } catch (error) { caught = error; }
if (caught !== sentinel || log.join('|') !== 'evaluate right|convert left') throw 'conversion abrupt';
log = [];
try { Symbol('left') | (log.push('evaluate right'), right); } catch (error) { caught = error; }
caught instanceof TypeError && log.join('|') === 'evaluate right';
"#,
    );
}

#[test]
fn bigint_dispatch_preserves_large_values_negative_counts_and_mixed_type_errors() {
    assert_numeric_bitwise(
        r#"
if ((1n << 64n) !== 18446744073709551616n ||
    (18446744073709551616n >> 64n) !== 1n ||
    (8n << -1n) !== 4n || (8n >> -1n) !== 16n ||
    (-5n >> 1n) !== -3n || (18446744073709551617n & 3n) !== 1n ||
    (18446744073709551616n | 3n) !== 18446744073709551619n ||
    (18446744073709551619n ^ 3n) !== 18446744073709551616n) throw 'BigInt arithmetic';
var log = [], caught;
var left = {valueOf() { log.push('left'); return 1n; }};
var right = {valueOf() { log.push('right'); return 1; }};
try { left << right; } catch (error) { caught = error; }
if (!(caught instanceof TypeError) || log.join('|') !== 'left|right') throw 'mixed types';
log = [];
right.valueOf = function() { log.push('right'); return 1n; };
try { left >>> right; } catch (error) { caught = error; }
caught instanceof TypeError && log.join('|') === 'left|right';
"#,
    );
}

#[test]
fn mutable_storage_and_callable_hooks_keep_runtime_numeric_dispatch() {
    assert_numeric_bitwise(
        r#"
function captured() {
  let value = 2;
  function replace() { value = 2n; }
  replace();
  return value << 1n;
}
var stored = 3, reads = 0;
var object = {get value() { reads++; return stored; }};
if ((object.value << 1) !== 6) throw 'Number property';
stored = 3n;
if ((object.value << 1n) !== 6n || reads !== 2 || captured() !== 4n) throw 'dynamic storage';
function callable() {}
callable[Symbol.toPrimitive] = function(hint) { if (hint !== 'number') throw 'hint'; return 8; };
var caught, value = 2;
try { value << (value = 1n); } catch (error) { caught = error; }
(callable >>> 1) === 4 && caught instanceof TypeError && value === 1n;
"#,
    );
}

#[test]
fn unary_number_projection_preserves_nested_values_and_dynamic_bigints() {
    assert_numeric_bitwise(
        r#"
if (!Object.is(-0, -0) || !Object.is(-(-0), 0) || -(-Infinity) !== Infinity ||
    !Number.isNaN(-NaN) || ~2147483649 !== 2147483646 ||
    (-2147483649 << -1) !== -2147483648 ||
    ((2147483649 << 1) | ~7) !== -6 || ((-1 >>> 1) ^ 3) !== 2147483644) throw 'Number projection';
var log = [], token = {}, caught;
var object = {[Symbol.toPrimitive](hint) { log.push(hint); return 18446744073709551616n; }};
if (-object !== -18446744073709551616n || ~object !== -18446744073709551617n || log.join() !== 'number,number') throw 'BigInt projection';
var value = 1;
function change() { value = 2n; }
change();
if (-value !== -2n || ~value !== -3n) throw 'mutable Number fact';
object[Symbol.toPrimitive] = function() { throw token; };
try { -(log.push('evaluate'), object); } catch(error) { caught = error; }
if (caught !== token || log.join() !== 'number,number,evaluate') throw 'object abrupt';
function fail() { throw token; }
for (var operation of [() => -(fail(), 1), () => ~(fail(), 1)]) {
    caught = undefined;
    try { operation(); } catch(error) { caught = error; }
    if (caught !== token) throw 'static Number evaluation abrupt';
}
for (var operation of [() => -(1n >>> 0n), () => ~(1n >>> 0n)]) {
    caught = undefined;
    try { operation(); } catch(error) { caught = error; }
    if (!(caught instanceof TypeError)) throw 'inner unsigned BigInt shift';
}
true;
"#,
    );
}
