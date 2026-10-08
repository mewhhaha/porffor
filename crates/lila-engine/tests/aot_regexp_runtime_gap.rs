use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

const PRELUDE: &str = include_str!("fixtures/regexp_runtime_gap/from_units.js");

fn assert_normal(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{PRELUDE}\n{source}");
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("implemented patterns and genuine errors retain normal JS behavior");
        assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            result.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            result.output_events,
            vec![HostOutputEvent::PrintLine("ok".into())]
        );
    }
}

fn compile_options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    }
}

#[test]
fn computed_unicode_property_constructor_uses_real_matching() {
    assert_normal("var expression = new RegExp(fromUnits([92,112,123,65,83,67,73,73,125]), fromUnits([117])); require(expression.test('a') && expression.test('~') && !expression.test(fromUnits([233])), 'computed ASCII property'); print('ok'); 262;");
}

#[test]
fn computed_set_strings_exec_uses_real_finite_matching() {
    assert_normal("var expression = new RegExp(fromUnits([91,92,113,123,97,98,125,93]), fromUnits([118])); var match = expression.exec('ab'); require(match && match[0] === 'ab' && !expression.test('ac'), 'computed finite string matching'); print('ok'); 262;");
}

#[test]
fn computed_names_and_lookbehind_use_real_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_named_captures.js"
    ));
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_lookbehind.js"
    ));
    assert_normal(
        r"var expression = /[\q{a}&&A]/iv;
          require(expression.test('a') && expression.test('A'), 'finite literal folds both operands');
          require(!expression.test('b'), 'finite literal excludes nonmembers');
          print('ok');
          262;",
    );
}

#[test]
fn recompile_set_strings_installs_the_completed_descriptor() {
    assert_normal(
        "var expression = /a/g; expression.lastIndex = 7; \
         var source = fromUnits([91,92,113,123,97,98,125,93]); \
         expression.compile(source, fromUnits([118])); \
         require(expression.source === source && expression.flags === 'v' && expression.lastIndex === 0, 'finite recompile slots'); \
         require(expression.test('ab') && !expression.test('ac'), 'finite recompile matcher'); print('ok'); 262;",
    );
}

#[test]
fn compiled_unicode_properties_and_set_literals_keep_real_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/compiled_literals.js"
    ));
}

#[test]
fn literal_and_computed_classes_preserve_whitespace_and_octal_character_domains() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/class_character_domains.js"
    ));
}

#[test]
fn clean_computed_unicode_modes_keep_real_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_simple_modes.js"
    ));
}

#[test]
fn computed_unicode_character_domains_folding_and_grammar_use_real_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_unicode_character_domain.js"
    ));
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_unicode_grammar.js"
    ));
}

#[test]
fn literal_and_computed_non_ascii_identity_atoms_preserve_utf16_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/non_ascii_identity_atoms.js"
    ));
}

#[test]
fn computed_braced_unicode_escapes_use_real_matching_and_syntax_routes() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_braced_unicode_escape.js"
    ));
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_braced_unicode_syntax.js"
    ));
}

#[test]
fn computed_unicode_named_references_use_completed_names_and_real_matching() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_named_unicode_references.js"
    ));
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/computed_named_unicode_syntax.js"
    ));
}

#[test]
fn unicode_named_references_compose_with_computed_finite_strings() {
    for tail in [
        "[91,92,113,123,97,125,93]",
        "[91,91,92,113,123,97,125,93,93]",
        "[91,92,113,123,97,125,38,38,97,93]",
    ] {
        assert_normal(&format!(
            "var expression = new RegExp(fromUnits([40,63,60,110,62,97,41,92,107,60,110,62].concat({tail})), fromUnits([118])); var match = expression.exec('aaa'); require(match && match[0] === 'aaa' && match.groups.n === 'a' && !expression.test('aab'), 'computed finite string named reference'); print('ok'); 262;",
        ));
    }
}

#[test]
fn invalid_computed_unicode_syntax_is_catchable_and_recompile_is_transactional() {
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/syntax_transaction.js"
    ));
}

#[test]
fn source_instruction_capacity_is_catchable_and_recompile_is_transactional() {
    // Exact decimal bounds are admitted. A genuinely excessive source-sized
    // instruction domain remains a resource failure with transactional compile.
    assert_normal(include_str!(
        "fixtures/regexp_runtime_gap/resource_transaction.js"
    ));
}
