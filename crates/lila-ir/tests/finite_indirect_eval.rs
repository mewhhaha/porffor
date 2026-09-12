use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower_with_host_surface_policy, DynamicSourceGap, DynamicSourceKind, HostSurfacePolicy,
    PreparedScriptAdmission, PreparedScriptKind, PreparedScriptOutcome, ProgramIr,
    UnsupportedFeature,
};

fn lower(source: &str) -> ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("outer Script parses");
    lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262)
}

fn assert_prepared(program: &ProgramIr, text: &str, kind: PreparedScriptKind) {
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let prepared = &program.script.as_ref().expect("Script IR").prepared_scripts;
    assert!(
        prepared.iter().any(|source| {
            source.source == text
                && source.kind == kind
                && source.admission == PreparedScriptAdmission::RuntimeCandidate
                && matches!(source.outcome, PreparedScriptOutcome::Executable(_))
        }),
        "missing finite source {text:?}: {prepared:?}"
    );
    assert!(!prepared
        .iter()
        .any(|source| source.source == text
            && matches!(source.kind, PreparedScriptKind::DirectEval(_))));
}

#[test]
fn forwarded_eval_prepares_captured_class_source_without_a_direct_eval_context() {
    for target in ["eval", "$262.createRealm().global.eval"] {
        let program = lower(&format!(
            "let text = `(class {{ #x = 1; read(o) {{ return o.#x; }} }})`;\n\
             let create = function(target) {{ return new (target(text)); }};\n\
             create({target});"
        ));
        assert_prepared(
            &program,
            "(class { #x = 1; read(o) { return o.#x; } })",
            PreparedScriptKind::IndirectEval,
        );
    }
}

#[test]
fn property_and_forwarding_routes_prepare_finite_bound_text() {
    for (call, kind) in [
        ("(0, eval)(text)", PreparedScriptKind::IndirectEval),
        (
            "eval.call(undefined, text)",
            PreparedScriptKind::IndirectEval,
        ),
        (
            "eval.apply(undefined, [text])",
            PreparedScriptKind::IndirectEval,
        ),
        (
            "Reflect.apply(eval, undefined, [text])",
            PreparedScriptKind::IndirectEval,
        ),
        (
            "$262.createRealm().evalScript(text)",
            PreparedScriptKind::RealmScript,
        ),
    ] {
        let program = lower(&format!("let text = '23;'; {call};"));
        assert_prepared(&program, "23;", kind);
    }
}

#[test]
fn finite_mutation_candidates_and_empty_spreads_preserve_the_possible_first_argument() {
    let program = lower(
        "let text = '31;'; (0, eval)(text);\n\
         text = '32;'; (0, eval)(...[], text, 'not a source');",
    );
    assert_prepared(&program, "31;", PreparedScriptKind::IndirectEval);
    assert_prepared(&program, "32;", PreparedScriptKind::IndirectEval);
    assert!(!program
        .script
        .unwrap()
        .prepared_scripts
        .iter()
        .any(|source| source.source == "not a source"));
}

#[test]
fn finite_expression_sources_and_syntax_errors_are_prepared_for_runtime_dispatch() {
    let program = lower("eval.call(undefined, String('23;'));");
    assert_prepared(&program, "23;", PreparedScriptKind::IndirectEval);
    let program = lower("let text = 'let = ;'; (0, eval)(text);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program
        .script
        .unwrap()
        .prepared_scripts
        .iter()
        .any(|source| {
            source.source == "let = ;"
                && source.kind == PreparedScriptKind::IndirectEval
                && source.admission == PreparedScriptAdmission::RuntimeCandidate
                && matches!(
                    source.outcome,
                    PreparedScriptOutcome::DeferredSyntaxError { .. }
                )
        }));
}

#[test]
fn source_without_a_finite_candidate_retains_its_typed_gap() {
    let program = lower("eval.call(undefined, globalThis.unknownSource);");
    let gaps = program
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.unsupported_feature() {
            Some(UnsupportedFeature::DynamicSource(gap)) => Some(gap),
            None => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        gaps,
        vec![DynamicSourceGap::runtime_source(
            DynamicSourceKind::IndirectEval
        )]
    );
    assert!(program
        .script
        .as_ref()
        .is_none_or(|script| script.prepared_scripts.is_empty()));
}
