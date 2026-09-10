use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_numeric_conversion(source: &str) {
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
        .expect("string numeric conversion compiles and executes through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
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
