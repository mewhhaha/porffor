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
function checkSet(source, flags, members, nonmembers) {
  var expression = computed('^' + source + '$', flags);
  for (var member of members) require(expression.test(member), 'set member: ' + source);
  for (var nonmember of nonmembers) require(!expression.test(nonmember), 'set nonmember: ' + source);
}
function rejects(source, flags) {
  var caught;
  try { computed(source, flags); } catch (error) { caught = error; }
  require(caught instanceof SyntaxError, 'computed set syntax: ' + source);
}
"#;

fn options() -> (CompileOptions, RunOptions) {
    (
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
}

fn assert_normal(source: &str, output: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{UNITS}\n{HELPERS}\n{source}");
        let (compile, run) = options();
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(&source, compile, run)
            .expect("computed UnicodeSets executes through Wasm AOT");
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
fn computed_nested_codepoint_sets_keep_algebra_and_full_character_domain() {
    assert_normal(
        include_str!("../fixtures/regexp_computed_unicode_sets/algebra.js"),
        "regexp-computed-sets-algebra:ok",
    );
}

#[test]
fn folding_is_owned_by_each_operand_before_algebra_and_nested_complement() {
    assert_normal(
        include_str!("../fixtures/regexp_computed_unicode_sets/folding.js"),
        "regexp-computed-sets-folding:ok",
    );
}

#[test]
fn complete_set_syntax_precedes_recompile_publication() {
    assert_normal(
        include_str!("../fixtures/regexp_computed_unicode_sets/syntax_and_transactions.js"),
        "regexp-computed-sets-syntax:ok",
    );
}

#[test]
fn valid_string_syntax_and_static_may_contain_strings_are_admitted() {
    let patterns = [
        r"[\q{ab}]",
        r"[\q{a}]",
        r"[\q{a|b}]",
        r"[\q{a}--\q{a}]",
        r"[\q{\}|a}]",
        r"[^\q{a}]",
        r"[^\q{ab}&&[a]]",
        r"[^\q{\uD83D\uDE00}]",
        "[^\\q{😀}]",
        r"[^\p{Basic_Emoji}&&[a]]",
        r"[^\p{Basic_Emoji}&&\q{a}]",
        r"[[\p{Basic_Emoji}]--[\p{Basic_Emoji}]]",
    ];
    let literals = patterns
        .iter()
        .map(|pattern| format!("{pattern:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "var patterns = [{literals}]; for (var pattern of patterns) {{ \
         require(computed(pattern, 'v').flags === 'v', 'valid finite class admission'); \
         }} print('regexp-computed-string-admission:ok'); 262;"
    );
    assert_normal(&source, "regexp-computed-string-admission:ok");
}
