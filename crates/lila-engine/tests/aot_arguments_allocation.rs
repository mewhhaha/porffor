use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("arguments allocation must preserve Wasm AOT execution");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}\n{source}",
        outcome.completion,
        outcome.note,
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}",
    );
}

#[test]
fn unobserved_strict_arguments_do_not_allocate_per_call() {
    assert_trace(
        r#"
'use strict';
function add(left, right) { return left + right; }
var result = 0;
for (var index = 0; index < 4000000; index++) result = add(result, 1);
print(result);
"#,
        &["4000000"],
    );
}

#[test]
fn observed_arguments_keep_values_identity_and_unmapped_parameters() {
    assert_trace(
        r#"
'use strict';
function read(first) {
  var original = arguments;
  arguments[0] = 7;
  print(first, arguments[0], original === arguments, arguments.length);
  return arguments;
}
var saved = read(3, 9);
print(saved[0], saved[1]);
var holder = { method(first) { return arguments[0] === first; } };
print(holder.method(saved));
"#,
        &["3 7 true 2", "7 9", "true"],
    );
}

#[test]
fn captured_arguments_remain_available_after_return_and_later_calls() {
    assert_trace(
        r#"
'use strict';
function capture(value) { return () => arguments; }
var first = capture(17), second = capture(29);
print(first()[0], second()[0], first() === first(), first() !== second());
first()[0] = 31;
print(first()[0], second()[0]);
"#,
        &["17 29 true true", "31 29"],
    );
}

#[test]
fn parameter_initializers_and_prepared_eval_can_observe_arguments() {
    assert_trace(
        r#"
'use strict';
function defaults(first = arguments[1]) { return first + arguments.length; }
function evaluated(first) { return eval('arguments[0]'); }
function capturedDefault(first = () => arguments[1]) { return first; }
print(defaults(undefined, 40), evaluated(39));
var saved = capturedDefault(undefined, 23);
print(saved());
"#,
        &["42 39", "23"],
    );
}

#[test]
fn sloppy_methods_keep_argument_reassignment_and_parameter_aliases() {
    assert_trace(
        r#"
var holder = {
  replace(first) {
    var original = arguments;
    first = 8;
    arguments = [13];
    print(original[0], arguments[0], original !== arguments);
  },
  declared() { var arguments; return arguments[0]; },
  captured() { var arguments; return () => arguments[0]; },
  capturedScalar() { var arguments = 113; return () => [typeof arguments, arguments]; },
  initialized() { var arguments = [43]; return arguments[0]; },
  split(first = 0) { var arguments; return arguments[1]; },
  capturedSplit(first = 0) { var arguments; return () => arguments[1]; },
  capturedInitialized(first = 0) { var arguments = [79]; return () => arguments[0]; },
  shadow(arguments = 71) { var arguments; return arguments; },
  capturedShadow(arguments = 83) { return () => arguments; },
  capturedFunction(first = 0) { function arguments() { return 89; } return () => arguments(); },
  simpleCapturedFunction() { function arguments() { return 97; } return () => arguments(); },
  mutatedDefault(first = (arguments = 101)) { var arguments; return [typeof arguments, arguments]; },
  separate(first = () => arguments[1]) { var arguments = [103]; return [first, () => arguments[0]]; }
};
function ordinary() { var arguments; return arguments[0]; }
holder.replace(5);
print(holder.declared(19), holder.captured(37)(), holder.initialized(), ordinary(53));
print(holder.split(undefined, 61), holder.capturedSplit(undefined, 67)(), holder.shadow(), holder.capturedInitialized()());
print(holder.capturedShadow()(), holder.capturedFunction()(), holder.simpleCapturedFunction()());
var mutated = holder.mutatedDefault();
print(mutated[0], mutated[1]);
var preserved = holder.mutatedDefault(109);
print(preserved[0], preserved[1][0]);
var separated = holder.separate(undefined, 107);
print(separated[0](), separated[1]());
var scalar = holder.capturedScalar()();
print(scalar[0], scalar[1]);
"#,
        &[
            "8 13 true",
            "19 37 43 53",
            "61 67 71 79",
            "83 89 97",
            "number 101",
            "object 109",
            "107 103",
            "number 113",
        ],
    );
}
