use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_lines(source: &str, expected: &[&str]) {
    scriptw_engine::configure_compilation_jobs(1).unwrap();
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

fn assert_named_environment_lines(source: &str, expected: &[&str]) {
    for directive in ["", "'use strict';"] {
        assert_lines(&format!("{directive}\n{source}"), expected);
    }
}

#[test]
fn generator_named_condition_preserves_external_captured_writes_across_resume() {
    let original =
        include_str!("../fixtures/named_generator_environment/captured_condition_resume.js");
    for directive in ["", "'use strict';"] {
        scriptw_engine::configure_compilation_jobs(1).unwrap();
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}\n{original}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .expect("the original captured-cell diagnostic executes");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(scriptw_engine::ObservedJsValue::Boolean(true)),
            "the original second result must be1 after the external closure's write"
        );
        assert_eq!(
            observation.output_events,
            vec![
                HostOutputEvent::PrintLine("first=1,done=false".into()),
                HostOutputEvent::PrintLine("second=1,done=true".into()),
            ]
        );
    }
}

#[test]
fn generator_named_conditional_skips_its_yield_and_preserves_original_branch_record() {
    assert_named_environment_lines(
        r#"
function* activation(parameter) { let local = parameter; if (local) { yield local; local += arguments[0]; } }
var skipped = activation(0), last = skipped.next();
if (last.value !== undefined || !last.done) throw 'skipped conditional resumed';
var selected = activation(3), first = selected.next(), second = selected.next();
if (first.value !== 3 || first.done || second.value !== undefined || !second.done) throw 'original activation yield';
print('ok');
"#,
        &["ok"],
    );
}

#[test]
fn named_async_arrow_parameter_reads_reject_before_body_and_preserve_lexical_owners() {
    assert_named_environment_lines(
        r#"
function make(marker) {
 const expression = async delta => this.value + arguments[0] + delta + (new.target === undefined ? 1 : 0);
 const block = async (value = later, later) => {
  await Promise.resolve();
  return this.value + arguments[0] + value + (new.target === undefined ? 1 : 0);
 };
 return [expression, block];
}
async function verify() {
 var arrows = make.call({value: 10}, 3);
 if (await arrows[0](4) !== 18) throw 'expression lexical captures';
 var rejected = false;
 try { await arrows[1](undefined, 7); } catch (error) { rejected = error instanceof ReferenceError; }
 if (!rejected) throw 'forward parameter default lost TDZ';
 if (await arrows[1](4, 7) !== 18) throw 'initialized parameter or lexical captures';
 print('ok');
}
verify().catch(function(error) { print(error); });
void 0;
"#,
        &["ok"],
    );
}

#[test]
fn named_root_update_replaces_an_earlier_arguments_write_in_source_order() {
    assert_named_environment_lines(
        r#"
var args = null; var close = function() { args = arguments; }; close(); args = 1; args;
print(args);
close(5, 6); args = 2; print(args);
"#,
        &["1", "2"],
    );
}
