use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_numeric_conversion(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("numeric conversion control failed: {error}\n{source}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(outcome.output_events.is_empty(), "{source}");
    }
}

#[test]
fn runtime_numeric_strings_accept_both_cases_of_each_radix_prefix() {
    assert_numeric_conversion(
        r#"
function convert(prefix, digits) { return Number(prefix + digits); }
function unary(value) { return +value; }
function subtract(value) { return value - 1; }
convert('0b', '101101') === 45 && convert('0B', '101101') === 45 &&
  convert('0o', '173') === 123 && convert('0O', '173') === 123 &&
  convert('0x', 'aF') === 175 && convert('0X', 'Af') === 175 &&
  convert('0B', '0000') === 0 && convert('0O', '007') === 7 &&
  convert('\t\u00a00B', '101\u2029') === 5 &&
  convert('\uFEFF0O', '17\n') === 15 &&
  convert('', '') === 0 && convert('  ', '\t') === 0 &&
  convert('-', '17.5e1') === -175 &&
  Object.is(convert('-', '0'), -0) &&
  unary({ valueOf() { return '0B1010'; } }) === 10 &&
  subtract({ toString() { return '0O21'; } }) === 16;
"#,
    );
}

#[test]
fn runtime_numeric_strings_reject_signs_missing_digits_and_out_of_radix_digits() {
    assert_numeric_conversion(
        r#"
function invalid(prefix, digits) {
  var result = Number(prefix + digits);
  return result !== result;
}
invalid('0b', '') && invalid('0B', '') &&
  invalid('0o', '') && invalid('0O', '') &&
  invalid('0x', '') && invalid('0X', '') &&
  invalid('0b', '102') && invalid('0B', '1a') &&
  invalid('0o', '78') && invalid('0O', '7F') && invalid('0x', '1g') &&
  invalid('+0B', '1') && invalid('-0b', '1') &&
  invalid('+0O', '7') && invalid('-0o', '7') &&
  invalid('+0X', 'f') && invalid('-0x', 'f') &&
  invalid('0B', '1 0') && invalid('0O', '1_0') &&
  invalid('0B', '1n') && invalid('0O', '1.0') && invalid('0B', '1e1');
"#,
    );
}

#[test]
fn power_of_two_radices_round_once_with_guard_and_sticky_digits() {
    let cases = [
        ((1_u128 << 54) + 2, 18_014_398_509_481_984_u64),
        ((1_u128 << 54) + 3, 18_014_398_509_481_988),
        ((1_u128 << 54) + 6, 18_014_398_509_481_992),
        ((1_u128 << 58) + 33, 288_230_376_151_711_808),
        ((1_u128 << 61) + 256, 2_305_843_009_213_693_952),
        ((1_u128 << 61) + 257, 2_305_843_009_213_694_464),
        ((1_u128 << 61) + 768, 2_305_843_009_213_694_976),
    ];
    let mut source = String::from("function convert(value) { return Number(value); }\n");
    for (integer, expected) in cases {
        for literal in [
            format!("0b{integer:b}"),
            format!("0B000{integer:b}"),
            format!("0o{integer:o}"),
            format!("0O000{integer:o}"),
            format!("0x{integer:x}"),
            format!("0X000{integer:X}"),
        ] {
            source.push_str(&format!("convert({literal:?}) === {expected} &&\n"));
        }
    }
    source.push_str(
        r#"
convert('0B' + '1'.repeat(53) + '0'.repeat(971)) === Number.MAX_VALUE &&
  convert('0Xfffffffffffff8' + '0'.repeat(242)) === Number.MAX_VALUE &&
  convert('0B1' + '0'.repeat(1024)) === Infinity &&
  convert('0O1' + '0'.repeat(342)) === Infinity &&
  convert('0X1' + '0'.repeat(256)) === Infinity &&
  convert('0B' + '0'.repeat(1200)) === 0 &&
  Number.isNaN(convert('0X1' + '0'.repeat(256) + 'g'));
"#,
    );
    assert_numeric_conversion(&source);
}

#[test]
fn global_numeric_predicates_use_ecmascript_numeric_string_rules_for_literals() {
    assert_numeric_conversion(
        r#"
!isNaN('0x10') && isFinite('0x10') &&
  !isNaN('0B101') && isFinite('0O17') &&
  !isNaN('\u00a0\uFEFF1\u2029') && isFinite('\u00a0\uFEFF1\u2029') &&
  !isNaN('') && isFinite('  \t') &&
  !isNaN('-0') && isFinite('-0') &&
  !isNaN('Infinity') && !isFinite('Infinity') &&
  !isNaN('-Infinity') && !isFinite('-Infinity') &&
  isNaN('inf') && !isFinite('inf') &&
  isNaN('-inf') && !isFinite('-inf') &&
  isNaN('+0x10') && isNaN('0b102') && isNaN('1_0') &&
  !Number.isFinite('0x10') && !Number.isNaN('not numeric') &&
  Number.isFinite(16) && Number.isNaN(NaN);
"#,
    );
}

#[test]
fn literal_arrays_keep_live_to_primitive_hooks_and_original_abrupt_values() {
    assert_numeric_conversion(
        r#"
const previous = Object.getOwnPropertyDescriptor(Array.prototype, Symbol.toPrimitive);
const seen = [];
let gets = 0;
let calls = 0;
Object.defineProperty(Array.prototype, Symbol.toPrimitive, {configurable: true, get() {
  gets++;
  if (!Array.isArray(this)) throw 'array numeric Get receiver';
  return function(hint) {
    'use strict';
    calls++;
    if (hint !== 'number' || !Array.isArray(this)) throw 'array numeric conversion receiver';
    seen.push(this);
    return this.length === 0 ? 'not numeric' : '0x10';
  };
}});
const values = !isNaN([1]) && isFinite([1]) && isNaN([]) && !isFinite([]);
const identity = gets === 4 && calls === 4 && seen.length === 4 &&
  seen[0] !== seen[1] && seen[2] !== seen[3] && seen[0][0] === 1;
const marker = {};
let assigned = 'kept';
let caught = false;
Object.defineProperty(Array.prototype, Symbol.toPrimitive, {configurable: true, value() { throw marker; }});
try { assigned = isNaN([1]); } catch (error) { caught = error === marker; }
if (previous === undefined) delete Array.prototype[Symbol.toPrimitive];
else Object.defineProperty(Array.prototype, Symbol.toPrimitive, previous);
values && identity && caught && assigned === 'kept';
"#,
    );
}

#[test]
fn global_numeric_calls_preserve_argument_conversion_and_intrinsic_error_order() {
    assert_numeric_conversion(
        r#"
const extraTrace = [];
function extraArgument() {
  extraTrace.push('extra');
  return {[Symbol.toPrimitive]() { throw 'ignored argument must not be converted'; }};
}
const extraValues = !isNaN('0x10', extraArgument()) && isFinite('0x10', extraArgument()) &&
  !globalThis.isNaN('0x10', extraArgument()) && globalThis.isFinite('0x10', extraArgument());
const extraMarker = {};
let extraAssigned = 'kept';
let extraCaught = false;
function abruptExtra() { extraTrace.push('abrupt-extra'); throw extraMarker; }
try {
  extraAssigned = isFinite({[Symbol.toPrimitive]() { throw 'conversion must follow all arguments'; }}, abruptExtra());
} catch (error) { extraCaught = error === extraMarker; }
const trace = [];
const intrinsicPrototype = TypeError.prototype;
TypeError = undefined;
let assigned = 'kept';
let errors = 0;
function argument() {
  trace.push('argument');
  return {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'global numeric conversion hint';
    trace.push('primitive');
    return 1n;
  }, toString() { throw 'primitive result must not fall back'; }};
}
try { assigned = isFinite(argument()); }
catch (error) {
  if (Object.getPrototypeOf(error) !== intrinsicPrototype) throw 'global isFinite intrinsic error';
  errors++;
  trace.push('finite-error');
}
try { assigned = isNaN(1n); }
catch (error) {
  if (Object.getPrototypeOf(error) !== intrinsicPrototype) throw 'global isNaN intrinsic error';
  errors++;
  trace.push('nan-error');
}
try { assigned = isFinite(Symbol('numeric')); }
catch (error) {
  if (Object.getPrototypeOf(error) !== intrinsicPrototype) throw 'global Symbol intrinsic error';
  errors++;
  trace.push('symbol-error');
}
extraValues && extraCaught && extraAssigned === 'kept' &&
  extraTrace.join(',') === 'extra,extra,extra,extra,abrupt-extra' &&
  errors === 3 && assigned === 'kept' &&
  trace.join(',') === 'argument,primitive,finite-error,nan-error,symbol-error';
"#,
    );
}
