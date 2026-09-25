use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

#[test]
fn math_log10_covers_the_real_domain_and_tonumber() {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let source = r#"
function rejects(value) {
  try { Math.log10(value); return false; }
  catch (error) { return error instanceof TypeError; }
}
// Expected results are rounded from Decimal.from_float(x).ln()/Decimal(10).ln()
// at 100 decimal digits. The third column is one ULP of the expected result.
var samples = [
  [0.9999999999999999, -4.821637332766436e-17, 6.162975822039155e-33],
  [1.0000000000000002, 9.64327466553287e-17, 1.232595164407831e-32],
  [0.7071067811865475, -0.15051499783199063, 2.7755575615628914e-17],
  [0.7071067811865476, -0.15051499783199057, 2.7755575615628914e-17],
  [0.7071067811865477, -0.1505149978319905, 2.7755575615628914e-17],
  [1.414213562373095, 0.15051499783199057, 2.7755575615628914e-17],
  [1.4142135623730951, 0.15051499783199063, 2.7755575615628914e-17],
  [1.4142135623730954, 0.15051499783199068, 2.7755575615628914e-17],
  [2, 0.3010299956639812, 5.551115123125783e-17],
  [3.2, 0.505149978319906, 1.1102230246251565e-16],
  [0.2, -0.6989700043360187, 1.1102230246251565e-16],
  [1e-300, -300, 5.684341886080802e-14],
  [Number.MIN_VALUE, -323.3062153431158, 5.684341886080802e-14],
  [2.225073858507201e-308, -307.6526555685888, 5.684341886080802e-14],
  [2.2250738585072014e-308, -307.6526555685888, 5.684341886080802e-14],
  [Number.MAX_VALUE, 308.25471555991675, 5.684341886080802e-14],
];
for (var i = 0; i < samples.length; i++) {
  var sample = samples[i];
  if (!(Math.abs(Math.log10(sample[0]) - sample[1]) <= 2 * sample[2])) {
    throw new Error('Math.log10 ULP sample ' + i);
  }
}
var hints = [];
var value = { [Symbol.toPrimitive](hint) { hints.push(hint); return '10000'; } };
Object.is(Math.log10(1), 0) &&
  Math.log10(10) === 1 && Math.log10(100) === 2 &&
  Math.log10(1000) === 3 && Math.log10(10000) === 4 &&
  Math.log10(3600000000000) + Math.log10(1e11) - Math.log10(36) === 22 &&
  Math.log10(0) === -Infinity && Math.log10(-0) === -Infinity &&
  Math.log10(Infinity) === Infinity &&
  Number.isNaN(Math.log10(-1)) && Number.isNaN(Math.log10(-Infinity)) &&
  Number.isNaN(Math.log10(NaN)) && Number.isNaN(Math.log10()) &&
  Math.log10(value) === 4 && hints.length === 1 && hints[0] === 'number' &&
  Math.log10([100]) === 2 && Math.log10(null) === -Infinity &&
  rejects(1n) && rejects(Symbol('number'));
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("Math.log10 compiles and runs through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.output_events.is_empty());
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
    );
}
