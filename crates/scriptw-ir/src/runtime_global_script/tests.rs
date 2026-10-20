use super::*;
use crate::{
    ExprIr, GlobalDeclarationSetIr, PreparedScriptOutcome, PreparedScriptUnit, StatementIr,
    TypedExpr, ValueKind,
};
use scriptw_front::{decode_js_string_utf16, ParseDiagnosticKind, ParseGoal};

fn lower(kind: RuntimeGlobalScriptKind, source: &str) -> RuntimeGlobalScriptProgram {
    lower_runtime_global_script(
        kind,
        &source.encode_utf16().collect::<Vec<_>>(),
        HostSurfacePolicy::default(),
    )
    .expect("independent global source compiles")
}

fn entry(compiled: &RuntimeGlobalScriptProgram) -> &PreparedScriptUnit {
    compiled
        .program()
        .script
        .as_ref()
        .unwrap()
        .prepared_script_units()
        .find(|unit| unit.id == compiled.script_id())
        .unwrap()
}

fn assert_number_completion(unit: &PreparedScriptUnit, expected: f64) {
    assert!(matches!(unit.body.statements.last(),
        Some(StatementIr::Expression(TypedExpr { kind: ValueKind::Number, expr: ExprIr::Number(bits), .. }))
            if *bits == expected.to_bits()));
}

#[test]
fn strictness_selects_the_correct_variable_environment_without_a_fresh_entry() {
    for kind in [
        RuntimeGlobalScriptKind::IndirectEval,
        RuntimeGlobalScriptKind::RealmScript,
    ] {
        for strict in [false, true] {
            let source = format!(
                "{}var local = 17; function f() {{ return local; }} let lexical = 23; 42;",
                if strict { "'use strict'; " } else { "" }
            );
            let compiled = lower(kind, &source);
            assert_eq!(compiled.kind(), kind);
            assert_eq!(compiled.program().goal, ParseGoal::Script);
            assert!(compiled.program().modules.is_none());
            assert!(compiled.program().diagnostics.is_empty());
            let script = compiled.program().script.as_ref().unwrap();
            assert!(script.body.statements.is_empty());
            assert!(script.owned_env_bindings.is_empty());
            assert!(script.global_bindings.iter().next().is_none());
            assert_eq!(script.global_bindings.lexical_names().len(), 0);
            assert!(script.runtime_declarations.var_names.is_empty());
            let unit = entry(&compiled);
            assert_eq!(unit.kind, kind.prepared_kind());
            assert_eq!(unit.strict, strict);
            assert_eq!(
                unit.has_global_variable_environment(),
                kind == RuntimeGlobalScriptKind::RealmScript || !strict
            );
            assert_eq!(unit.declarations.var_names, ["local"]);
            assert_eq!(unit.declarations.lexical_names_in_source_order, ["lexical"]);
            assert_eq!(unit.declarations.functions_in_reverse_order[0].name, "f");
            assert!(unit
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == "lexical"));
            assert!(!unit.function_ids.is_empty());
            for id in &unit.function_ids {
                assert!(script.functions.iter().any(|function| &function.id == id));
            }
            assert_number_completion(unit, 42.0);
        }
    }
}

#[test]
fn restricted_global_admission_stays_in_the_runtime_declaration_plan() {
    // Fresh-entry lowering rejects this declaration. Prepared Script lowering
    // must retain the normal runtime admission and its actual target Realm.
    for kind in [
        RuntimeGlobalScriptKind::IndirectEval,
        RuntimeGlobalScriptKind::RealmScript,
    ] {
        let compiled = lower(kind, "function undefined() {} var undefined; undefined;");
        let unit = entry(&compiled);
        assert_eq!(
            unit.global_bindings.get("undefined").unwrap().declarations,
            GlobalDeclarationSetIr::FunctionAndVar
        );
        assert_eq!(
            unit.declarations.functions_in_reverse_order[0].name,
            "undefined"
        );
        assert_eq!(unit.declarations.var_names, ["undefined"]);
        assert!(matches!(unit.body.statements.last(),
            Some(StatementIr::Expression(TypedExpr { expr: ExprIr::GlobalIdentifierRead { name }, .. }))
                if name == "undefined"));
    }
}

#[test]
fn source_and_literal_transport_preserves_lone_surrogates_and_the_codec_sentinel() {
    for kind in [
        RuntimeGlobalScriptKind::IndirectEval,
        RuntimeGlobalScriptKind::RealmScript,
    ] {
        for literal in [vec![0xd800], vec![0xdfff], vec![0xdb80, 0xdc00]] {
            let source = [vec![0x27], literal.clone(), vec![0x27, 0x3b]].concat();
            let compiled =
                lower_runtime_global_script(kind, &source, HostSurfacePolicy::default()).unwrap();
            let script = compiled.program().script.as_ref().unwrap();
            let prepared = script
                .prepared_scripts
                .iter()
                .find(|prepared| {
                    matches!(&prepared.outcome, PreparedScriptOutcome::Executable(unit)
                    if unit.id == compiled.script_id())
                })
                .unwrap();
            assert_eq!(decode_js_string_utf16(&prepared.source), source);
            let unit = entry(&compiled);
            let Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::String(value),
                ..
            })) = unit.body.statements.last()
            else {
                panic!("literal expression must remain real string IR")
            };
            assert_eq!(decode_js_string_utf16(value), literal);
        }
    }
}

#[test]
fn syntax_failures_retain_the_exact_parser_diagnostic_and_no_caller_authority() {
    for kind in [
        RuntimeGlobalScriptKind::IndirectEval,
        RuntimeGlobalScriptKind::RealmScript,
    ] {
        for source in [
            "new.target;",
            "super.x;",
            "return 1;",
            "let duplicate; let duplicate;",
            "class C { method() { return this.#missing; } }",
        ] {
            let source: Vec<u16> = source.encode_utf16().collect();
            let direct = match kind {
                RuntimeGlobalScriptKind::IndirectEval => {
                    prepare_eval_source_utf16(&source, &EvalParseContext::Indirect)
                }
                RuntimeGlobalScriptKind::RealmScript => parse_script_utf16(&source),
            }
            .unwrap_err();
            let error = lower_runtime_global_script(kind, &source, HostSurfacePolicy::default())
                .unwrap_err();
            let RuntimeGlobalScriptCompileError::Parse(error) = error else {
                panic!("syntax must retain its parser rejection")
            };
            assert_eq!(error, direct);
            assert_eq!(
                error.diagnostic().kind(),
                ParseDiagnosticKind::MalformedJavaScript
            );
        }
        // A lone surrogate used as an identifier is invalid syntax. Never feed
        // U+FFFD to the parser via a lossy UTF-16-to-UTF-8 conversion.
        let source = vec![0x6c, 0x65, 0x74, 0x20, 0xd800, 0x3b];
        assert!(matches!(
            lower_runtime_global_script(kind, &source, HostSurfacePolicy::default()),
            Err(RuntimeGlobalScriptCompileError::Parse(_))
        ));
    }
}

#[test]
fn nested_scripts_and_functions_keep_complete_bodies_and_distinct_entry_ids() {
    let compiled = lower(
        RuntimeGlobalScriptKind::IndirectEval,
        "function holder() { return 17; } (0, eval)('23;'); Function('return 42;'); 99;",
    );
    let script = compiled.program().script.as_ref().unwrap();
    let outer = entry(&compiled);
    assert_number_completion(outer, 99.0);
    let nested = script
        .prepared_script_units()
        .find(|unit| unit.id != compiled.script_id())
        .unwrap();
    assert_eq!(nested.kind, PreparedScriptKind::IndirectEval);
    assert_number_completion(nested, 23.0);
    assert_ne!(nested.id.function_id(), outer.id.function_id());
    assert!(!script.prepared_dynamic_functions.is_empty());
    assert!(script
        .functions
        .iter()
        .any(|function| function.name == "holder"));
    assert!(script.functions.len() >= 2);
}

#[test]
fn each_request_beyond_the_prepared_catalogue_limit_has_its_own_real_ir() {
    for number in 0..257 {
        let compiled = lower(RuntimeGlobalScriptKind::IndirectEval, &format!("{number};"));
        assert_number_completion(entry(&compiled), f64::from(number));
        assert_eq!(
            compiled
                .program()
                .script
                .as_ref()
                .unwrap()
                .prepared_scripts
                .len(),
            1
        );
    }
}
