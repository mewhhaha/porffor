use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

const UNITS: &str = include_str!("../fixtures/regexp_runtime_gap/from_units.js");
const HELPERS: &str = r#"
function textUnits(text) {
  var units = [];
  for (var i = 0; i < text.length; i++) units.push(text.charCodeAt(i));
  return units;
}
function computed(source, flags) {
  return new RegExp(fromUnits(textUnits(source)), fromUnits(textUnits(flags)));
}
function codePoint(value) {
  if (value <= 0xffff) return fromUnits([value]);
  value -= 0x10000;
  return fromUnits([0xd800 + (value >> 10), 0xdc00 + (value & 1023)]);
}
function codePoints(values) {
  var text = '';
  for (var i = 0; i < values.length; i++) text += codePoint(values[i]);
  return text;
}
function checkMembers(source, flags, members, nonmembers) {
  var expression = computed('^(?:' + source + ')$', flags);
  for (var member of members) require(expression.test(member), 'finite member: ' + source);
  for (var nonmember of nonmembers) require(!expression.test(nonmember), 'finite nonmember: ' + source);
}
function rejects(source, flags) {
  var caught;
  try { computed(source, flags); } catch (error) { caught = error; }
  require(caught instanceof SyntaxError, 'finite syntax: ' + source);
}
"#;

fn assert_finite(source: &str, output: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{UNITS}\n{HELPERS}\n{source}");
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("computed finite strings execute through actual Wasm AOT");
        assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            result.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            result.output_events,
            vec![HostOutputEvent::PrintLine(output.into())],
            "{source}"
        );
    }
}

#[test]
fn computed_finite_strings_preserve_algebra_priority_and_empty_progress() {
    assert_finite(
        include_str!("../fixtures/regexp_computed_finite_strings/algebra.js"),
        "regexp-computed-strings-algebra:ok",
    );
}

#[test]
fn all_seven_computed_properties_of_strings_use_complete_finite_payloads() {
    assert_finite(
        include_str!("../fixtures/regexp_computed_finite_strings/properties.js"),
        "regexp-computed-strings-properties:ok",
    );
}

#[test]
fn finite_folding_reverse_matching_and_recompile_are_transactional() {
    assert_finite(
        include_str!("../fixtures/regexp_computed_finite_strings/folding_reverse_transactions.js"),
        "regexp-computed-strings-folding:ok",
    );
}
