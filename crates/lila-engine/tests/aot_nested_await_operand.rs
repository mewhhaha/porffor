//! `await f(await p)` evaluates the inner Await, then the call, then the outer
//! Await, in every statement shape that lowers an Await directly.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn nested_await_operands_resume_in_evaluation_order() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var log = [];
var assigned;
function step(name, value) { log.push(name); return value; }
async function plusOne(value) { log.push('call ' + value); return value + 1; }
async function run() {
  await plusOne(await step('statement', 1));
  assigned = await plusOne(await step('assignment', 10));
  const declared = await plusOne(await step('declaration', 20));
  log.push(assigned + ',' + declared);
  return await plusOne(await step('return', 30));
}
run().then(result => {
  print(log.join(' | '));
  print(String(result));
});
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
        .expect("nested awaits compile and execute through Wasm AOT");
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        [
            "statement | call 1 | assignment | call 10 | declaration | call 20 | 11,21 | return | call 30",
            "31",
        ]
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>()
    );
}
