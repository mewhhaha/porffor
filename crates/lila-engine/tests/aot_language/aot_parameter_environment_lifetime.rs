use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_lines(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn generator_retains_distinct_parameter_and_body_cells_across_yields() {
    assert_lines(
        r#"
function* values(parameter = 3, read = () => parameter) {
  var parameter;
  parameter = 4;
  yield read();
  yield parameter;
  parameter = 5;
  return read() + parameter;
}
var iterator = values();
print(iterator.next().value);
print(iterator.next().value);
print(iterator.next().value);
"#,
        &["3", "4", "8"],
    );
}

#[test]
fn async_body_record_retains_mutations_and_iteration_closures() {
    assert_lines(
        r#"
async function values(parameter = 3, read = () => parameter) {
  var parameter;
  parameter = 4;
  await 0;
  var observed = parameter;
  var closures = [];
  for (let index of [0, 1]) {
    closures.push(() => parameter + index);
    await 0;
  }
  print(read() === 3 && observed === 4 && closures[0]() === 4 && closures[1]() === 5);
}
values().catch(function(error) { print(error); });
void 0;
"#,
        &["true"],
    );
}

#[test]
fn async_generator_retains_body_cells_across_yields() {
    assert_lines(
        r#"
async function* values(parameter = 3, read = () => parameter) {
  var parameter;
  parameter = 4;
  yield read();
  yield parameter;
  return read() + parameter;
}
async function observe() {
  var iterator = values();
  print((await iterator.next()).value);
  print((await iterator.next()).value);
  print((await iterator.next()).value);
}
observe().catch(function(error) { print(error); });
void 0;
"#,
        &["3", "4", "7"],
    );
}
