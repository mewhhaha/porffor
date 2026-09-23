use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, ObservedNumber,
    RealmBuilder, RunOptions,
};

fn observe(source: &str) -> ObservedCompletion {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("runtime-planned script compiles and executes through Wasm AOT");
    assert!(observed.output_events.is_empty());
    observed.completion
}

fn number(value: f64) -> ObservedJsValue {
    ObservedJsValue::Number(ObservedNumber::from_f64(value))
}

fn text(value: &str) -> ObservedJsValue {
    ObservedJsValue::String(value.encode_utf16().collect())
}

#[test]
fn scalar_expressions_preserve_values_and_statement_list_completion() {
    for (source, value) in [
        ("", ObservedJsValue::Undefined),
        (";;", ObservedJsValue::Undefined),
        ("null;", ObservedJsValue::Null),
        ("true;", ObservedJsValue::Boolean(true)),
        ("'A😃B';", text("A😃B")),
        ("262;", number(262.0)),
        ("1 + 1;", number(2.0)),
        ("(7 - 2) * 3 / 2;", number(7.5)),
        ("5 % 2;", number(1.0)),
        ("17 % (8 % 3);", number(1.0)),
        ("1.7976931348623157e308 % 3;", number(f64::MAX % 3.0)),
        ("1.5e-323 % 1e-323;", number(f64::from_bits(1))),
        ("-5 % 2;", number(-1.0)),
        ("-4 % 2;", number(-0.0)),
        ("-0 + -0;", number(-0.0)),
        ("-0 - 0;", number(-0.0)),
        ("-0 * 3;", number(-0.0)),
        ("+(1 + 2);", number(3.0)),
        ("-(1 + 2);", number(-3.0)),
        ("~(1 + 2);", number(-4.0)),
        ("(1 + 2) ? (3 * 4) : (5 % 2);", number(12.0)),
        ("2 ** (1 + 2);", number(8.0)),
        ("-0;", number(-0.0)),
        ("+7;", number(7.0)),
        ("~7;", number(-8.0)),
        ("1 / 0;", number(f64::INFINITY)),
        ("0 / 0;", number(f64::NAN)),
        ("void (1 + 2);", ObservedJsValue::Undefined),
        ("delete 1;", ObservedJsValue::Boolean(true)),
        ("!0;", ObservedJsValue::Boolean(true)),
        ("1 < 2;", ObservedJsValue::Boolean(true)),
        ("false || 'text';", text("text")),
        ("null ?? 7;", number(7.0)),
        ("+(null ?? 7);", number(7.0)),
        ("!(null ?? 7);", ObservedJsValue::Boolean(false)),
        ("+(0 ?? 7);", number(0.0)),
        ("+(3 ?? 7);", number(3.0)),
        ("true ? 'yes' : 3;", text("yes")),
        ("(1, 2, 3);", number(3.0)),
        ("{ 7; { 8; } }", number(8.0)),
        ("7; ; {}", number(7.0)),
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(value),
            "{source}"
        );
    }
}

#[test]
fn primitive_prototype_formatters_remain_observable_and_callable() {
    for source in [
        "(123).toLocaleString('en-US', {minimumFractionDigits:2});",
        "(123n)['toLocaleString']('en-US', {minimumFractionDigits:2});",
        "Object.getPrototypeOf(123).toLocaleString.call(123, 'en-US', {minimumFractionDigits:2});",
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(text("123.00")),
            "{source}"
        );
    }
}

#[test]
fn runtime_side_effects_and_abrupt_completions_remain_in_the_product_path() {
    for (source, value) in [
        (
            "let count=0; const value={valueOf(){count++; return 7;}}; value + 1; count;",
            number(1.0),
        ),
        ("var answer=7; answer;", number(7.0)),
        ("function answer(){return 7;} answer();", number(7.0)),
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(value),
            "{source}"
        );
    }
    assert_eq!(observe("throw 7;"), ObservedCompletion::Throw(number(7.0)));
}

#[test]
fn scalar_numeric_operands_preserve_effects_and_abrupt_completions() {
    assert_eq!(
        observe(
            r#"
function check(condition, message) {
  if (!condition) throw new Error(message);
}
const trace = [];
function checked(value, expected) {
  check(value === expected, 'numeric result');
  check(trace.join(',') === 'left,right', 'operand order');
  trace.length = 0;
}
checked((trace.push('left'), 7) + (trace.push('right'), 3), 10);
checked((trace.push('left'), 7) - (trace.push('right'), 3), 4);
checked((trace.push('left'), 7) * (trace.push('right'), 3), 21);
checked((trace.push('left'), 8) / (trace.push('right'), 2), 4);
checked((trace.push('left'), 17) % (trace.push('right'), 8 % 3), 1);
checked((trace.push('left'), 2) ** (trace.push('right'), 3), 8);

const marker = {};
function abrupt() { trace.push('throw'); throw marker; }
let caught;
try { (abrupt(), 7) + (trace.push('right'), 3); }
catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'throw', 'left abrupt');
trace.length = 0;
caught = undefined;
try { (trace.push('left'), 7) * (abrupt(), 3); }
catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'left,throw', 'right abrupt');
trace.length = 0;

const left = {[Symbol.toPrimitive](hint) { trace.push('left:' + hint); return 7; }};
const right = {[Symbol.toPrimitive](hint) { trace.push('right:' + hint); return 3; }};
const sum = +left + +(trace.push('right-eval'), right);
check(sum === 10 && trace.join(',') === 'left:number,right-eval,right:number',
      'unary conversions remain in operand evaluation');
trace.length = 0;
const throwing = {[Symbol.toPrimitive]() { trace.push('coerce-throw'); throw marker; }};
caught = undefined;
try { +throwing % (trace.push('right'), 3); }
catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'coerce-throw', 'unary abrupt');
true;
"#,
        ),
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
    );
}

#[test]
fn runtime_numeric_values_preserve_dynamic_dispatch_and_conversion_order() {
    assert_eq!(
        observe(
            r#"
function check(condition, message) {
  if (!condition) throw new Error(message);
}
let stored = 7;
function replace(value) { stored = value; }
function read() { return stored; }
const box = {get value() { return stored; }};
replace('9');
check(stored + 1 === '91' && read() + 1 === '91' && box.value + 1 === '91',
      'mutable Number facts do not bypass string dispatch');
replace(9n);
check(stored + 1n === 10n && read() - 1n === 8n && box.value * 2n === 18n,
      'mutable Number facts do not bypass BigInt dispatch');
check(stored / 2n === 4n && read() % 2n === 1n && box.value ** 2n === 81n,
      'BigInt division remainder and power');
let caught;
try { box.value + 1; } catch (error) { caught = error; }
check(caught instanceof TypeError, 'mixed numeric types');
replace(7);
const captured = stored + (replace(3n), 2);
check(captured === 9 && stored === 3n, 'lhs captured before rhs mutation');

const trace = [];
const left = {[Symbol.toPrimitive](hint) { trace.push('left:' + hint); return 7; }};
const right = {[Symbol.toPrimitive](hint) { trace.push('right:' + hint); return 3; }};
const operands = {
  get left() { trace.push('get-left'); return left; },
  get right() { trace.push('get-right'); return right; }
};
check(operands.left + operands.right === 10, 'coercive addition');
check(trace.join(',') === 'get-left,get-right,left:default,right:default',
      'both getters before default coercion');
trace.length = 0;
check(operands.left - operands.right === 4, 'coercive subtraction');
check(trace.join(',') === 'get-left,get-right,left:number,right:number',
      'both getters before numeric coercion');
trace.length = 0;

const marker = {};
left[Symbol.toPrimitive] = function() { trace.push('left-throw'); throw marker; };
caught = undefined;
try { operands.left - operands.right; } catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'get-left,get-right,left-throw',
      'left coercion abrupt after both getters');
trace.length = 0;
left[Symbol.toPrimitive] = function(hint) { trace.push('left:' + hint); return 7; };
right[Symbol.toPrimitive] = function() { trace.push('right-throw'); throw marker; };
caught = undefined;
try { operands.left + operands.right; } catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'get-left,get-right,left:default,right-throw',
      'right coercion abrupt identity');
true;
"#,
        ),
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
    );
}

#[test]
fn typeof_number_results_preserve_operand_effects_and_abrupt_completions() {
    assert_eq!(
        observe(
            r#"
function check(condition, message) {
  if (!condition) throw new Error(message);
}
const trace = [];
const operand = {valueOf() { trace.push('unary'); return 7; }};
check(typeof +operand === 'number' && trace.join(',') === 'unary',
      'typeof unary Number evaluates coercion');
trace.length = 0;
check(typeof ((trace.push('left'), 1) + (trace.push('right'), 2)) === 'number',
      'typeof arithmetic result');
check(trace.join(',') === 'left,right', 'typeof arithmetic evaluates each operand');
trace.length = 0;
check(typeof -((trace.push('left'), 1) + (trace.push('right'), 2)) === 'number',
      'typeof nested unary arithmetic result');
check(trace.join(',') === 'left,right', 'typeof nested arithmetic evaluates once');
trace.length = 0;
check(typeof (trace.push('comma'), 7) === 'number' && trace.join(',') === 'comma',
      'typeof comma preserves effects');
trace.length = 0;

const marker = {};
const throwing = {valueOf() { trace.push('throw'); throw marker; }};
let caught;
try { typeof +throwing; trace.push('after'); } catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'throw', 'typeof unary abrupt identity');
trace.length = 0;
caught = undefined;
try {
  typeof (+throwing + (trace.push('right'), 2));
  trace.push('after');
} catch (error) { caught = error; }
check(caught === marker && trace.join(',') === 'throw', 'typeof arithmetic left abrupt');
true;
"#,
        ),
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
    );
}
