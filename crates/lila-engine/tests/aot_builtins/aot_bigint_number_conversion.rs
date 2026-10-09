use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_bigint_number_conversion(source: &str) {
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
        .unwrap_or_else(|error| panic!("BigInt Number conversion failed: {error}\n{source}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn computed_bigints_round_to_nearest_even_for_both_signs() {
    let cases = [
        (63, 1024_u64, 1_u128 << 63),
        (63, 1025, (1_u128 << 63) + 2048),
        (63, 3072, (1_u128 << 63) + 4096),
        (64, 2048, 1_u128 << 64),
        (64, 2049, (1_u128 << 64) + 4096),
        (95, 1_u64 << 42, 1_u128 << 95),
        (95, (1_u64 << 42) + 1, (1_u128 << 95) + (1_u128 << 43)),
    ];
    let mut source = String::from(
        "function make(bits, extra) { return (1n << BigInt(bits)) + BigInt(extra); }\n\
         function convert(value) { return Number(value); }\n",
    );
    for (bits, extra, expected) in cases {
        source.push_str(&format!(
            "convert(make({bits}, {extra})) === {expected} &&\n\
             convert(-make({bits}, {extra})) === -{expected} &&\n"
        ));
    }
    source.push_str("convert(0n) === 0 && !Object.is(convert(0n), -0);");
    assert_bigint_number_conversion(&source);
}

#[test]
fn computed_bigints_cover_maximum_finite_and_overflow_rounding() {
    assert_bigint_number_conversion(
        r#"
function power(bits) { return 1n << BigInt(bits); }
function convert(value) { return Number(value); }
var maximum = (power(53) - 1n) << 971n;
var overflowTie = power(1024) - power(970);
convert(maximum) === Number.MAX_VALUE &&
  convert(-maximum) === -Number.MAX_VALUE &&
  convert(overflowTie - 1n) === Number.MAX_VALUE &&
  convert(overflowTie) === Infinity && convert(-overflowTie) === -Infinity &&
  convert(power(1024)) === Infinity && convert(-power(1024)) === -Infinity;
"#,
    );
}

#[test]
fn number_accepts_bigint_primitive_results_without_relaxing_to_number() {
    assert_bigint_number_conversion(
        r#"
function convert(value) { return Number(value); }
function make(bits) { return 1n << BigInt(bits); }
var value = make(70);
var trace = [];
var object = {
  [Symbol.toPrimitive](hint) { trace.push(hint); return value; },
  valueOf() { throw 'unexpected valueOf'; },
  toString() { throw 'unexpected toString'; }
};
var converted = convert(object);
var marker = {};
var received;
try { convert({ valueOf() { throw marker; } }); }
catch (error) { received = error; }
var rejected = 0;
try { +value; } catch (error) { if (error instanceof TypeError) rejected++; }
try { value - 1; } catch (error) { if (error instanceof TypeError) rejected++; }
converted === 1180591620717411303424 && trace.join(',') === 'number' &&
  convert(Object(value)) === converted && received === marker && rejected === 2;
"#,
    );
}

#[test]
fn object_constructor_boxes_runtime_primitives_and_preserves_object_identity() {
    assert_bigint_number_conversion(
        r#"
function box(value) { return Object(value); }
function make(bits) { return 1n << BigInt(bits); }
var bigint = make(70);
var symbol = Symbol();
var object = {};
var array = [];
var callable = function () { return 9; };
var args = (function () { return arguments; })(1);
typeof box(undefined) === 'object' && typeof box(null) === 'object' &&
  Object.getPrototypeOf(box(undefined)) === Object.prototype &&
  box(17).valueOf() === 17 && box(false).valueOf() === false &&
  box('value').valueOf() === 'value' && box(symbol).valueOf() === symbol &&
  BigInt.prototype.valueOf.call(box(bigint)) === bigint &&
  box(object) === object && box(array) === array &&
  box(callable) === callable && box(callable)() === 9 && box(args) === args;
"#,
    );
}
