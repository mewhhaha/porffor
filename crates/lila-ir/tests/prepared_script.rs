use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower_with_host_surface_policy, ExprIr, GlobalDeclarationSetIr, GlobalLexicalBindingModeIr,
    HostSurfacePolicy, PreparedScriptKind, PreparedScriptOutcome, ProgramIr, StatementIr,
    TypedExpr, ValueKind,
};

fn lower(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("outer Script parses");
    lower_with_host_surface_policy(&source, HostSurfacePolicy::Test262)
}

#[test]
fn entry_global_function_instantiation_has_no_second_body_allocation() {
    let program = lower("function declared() {} declared.extra = 1;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert!(matches!(
        script.global_bindings.get("declared").unwrap().initializer,
        lila_ir::GlobalPropertyInitializerIr::SourceFunction(_)
    ));
    assert!(!script.body.statements.iter().any(|statement| {
        matches!(statement, StatementIr::Lexical { name, .. } if name == "declared")
    }));
}

#[test]
fn entry_function_and_var_references_read_the_canonical_global_property() {
    let program = lower("function declared() {} var declared; declared;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert_eq!(
        script.global_bindings.get("declared").unwrap().declarations,
        GlobalDeclarationSetIr::FunctionAndVar,
    );
    assert!(matches!(
        script.body.statements.last(),
        Some(StatementIr::Expression(TypedExpr {
            kind: ValueKind::Function,
            expr: ExprIr::GlobalIdentifierRead { name },
            ..
        })) if name == "declared"
    ));
}

#[test]
fn prepared_global_function_and_var_references_do_not_require_a_local_seed() {
    for source in [
        "eval.call(undefined, 'function declared() {} var declared; declared;');",
        "__lilaRealmEvalScript('function declared() {} var declared; declared;');",
    ] {
        let program = lower(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let unit = script.prepared_script_units().next().unwrap();
        assert!(unit.has_global_variable_environment());
        assert_eq!(
            unit.global_bindings.get("declared").unwrap().declarations,
            GlobalDeclarationSetIr::FunctionAndVar,
        );
        assert!(matches!(
            unit.body.statements.last(),
            Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::GlobalIdentifierRead { name },
                ..
            })) if name == "declared"
        ));
        assert!(!unit.body.statements.iter().any(|statement| {
            matches!(statement, StatementIr::Lexical { name, .. } if name == "declared")
        }));
    }
}

#[test]
fn deleted_eval_function_references_use_runtime_global_resolution() {
    let program =
        lower("eval.call(undefined, 'function deleted() {} delete globalThis.deleted; deleted;');");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script.prepared_script_units().next().unwrap();
    assert!(matches!(
        unit.body.statements.last(),
        Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::GlobalIdentifierRead { name },
            ..
        })) if name == "deleted"
    ));
}

#[test]
fn root_function_overrides_host_spelling_value_facts() {
    let program = lower("function parseInt() { return 'replacement'; } parseInt('17');");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
        panic!("the source function call must remain an expression");
    };
    assert_eq!(result.kind, ValueKind::String);
}

#[test]
fn prepared_var_deletion_keeps_descriptor_and_presence_checks_in_runtime_ir() {
    for deletion in ["delete declared", "delete globalThis.declared"] {
        let source = format!(
            "eval.call(undefined, 'var declared = 17; {deletion}; declared; typeof declared;');"
        );
        let program = lower(&source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let unit = script.prepared_script_units().next().unwrap();
        assert!(unit.body.statements.iter().any(|statement| {
            matches!(statement, StatementIr::Expression(TypedExpr {
                expr: ExprIr::DeleteGlobalProperty { name, .. }, ..
            }) if name == "declared")
        }));
        assert!(unit.body.statements.iter().any(|statement| {
            matches!(statement, StatementIr::Expression(TypedExpr {
                expr: ExprIr::GlobalIdentifierRead { name }, ..
            }) if name == "declared")
        }));
        let Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::TypeOf { expr },
            ..
        })) = unit.body.statements.last()
        else {
            panic!("typeof a possibly deleted var requires runtime lookup");
        };
        assert!(matches!(&expr.expr, ExprIr::GlobalPropertyRead { name } if name == "declared"));
    }
}

#[test]
fn root_function_references_use_global_properties_in_every_function_kind() {
    let program = lower(
        r#"
function declaration() {}
function ordinary() { return declaration; }
async function asynchronous() { return declaration; }
function* generator() { yield declaration; }
async function* asynchronousGenerator() { yield declaration; }
class Container { method() { return declaration; } }
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert!(script.functions.len() >= 6);
    for function in &script.functions {
        assert!(
            function
                .captured_bindings
                .iter()
                .all(|capture| capture.source_name != "declaration"),
            "{} must read the canonical global function property",
            function.name,
        );
    }
}

#[test]
fn global_lexicals_and_strict_eval_functions_retain_real_closure_cells() {
    let program = lower(
        r#"
let lexical = 1;
function readLexical() { return lexical; }
(0, eval)('"use strict"; function local() {} function readLocal() { return local; } readLocal;');
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    for (function_name, captured_name) in [("readLexical", "lexical"), ("readLocal", "local")] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap();
        assert!(function
            .captured_bindings
            .iter()
            .any(|capture| capture.source_name == captured_name));
    }
}

#[test]
fn prepared_sources_are_independent_units_with_exact_declaration_metadata() {
    let program = lower(
        r#"__lilaRealmEvalScript("let z; const a = 1; var v; function f() { return 1; } function g() {} function f() { return 2; } { function block() {} }");"#,
    );
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.expect("outer Script IR");
    assert!(script.global_bindings.get("v").is_none());
    assert_eq!(script.global_bindings.lexical_names().len(), 0);
    let unit = script
        .prepared_script_units()
        .next()
        .expect("separate Script unit");
    assert_eq!(unit.kind, PreparedScriptKind::RealmScript);
    assert_eq!(unit.declarations.lexical_names_in_source_order, ["z", "a"]);
    assert_eq!(
        unit.global_bindings.lexical_bindings()["a"],
        GlobalLexicalBindingModeIr::Immutable
    );
    assert_eq!(unit.declarations.var_names, ["v"]);
    assert_eq!(
        unit.declarations
            .functions_in_reverse_order
            .iter()
            .map(|function| function.name.as_str())
            .collect::<Vec<_>>(),
        ["f", "g"]
    );
    assert_eq!(unit.declarations.annex_b_candidates.len(), 1);
    let candidate = &unit.declarations.annex_b_candidates[0];
    assert_eq!(candidate.name, "block");
    assert!(unit.owned_env_bindings.contains(&candidate.admission));
    assert!(!unit
        .body
        .statements
        .iter()
        .any(|statement| matches!(statement,
        StatementIr::Lexical { name, .. } if name == "f" || name == "g")));
}

#[test]
fn parsing_and_early_errors_are_deferred_without_replacing_outer_statements() {
    let program = lower(r#"__lilaRealmEvalScript("let x; let x;"); 7;"#);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.expect("outer Script IR");
    assert_eq!(script.prepared_scripts.len(), 1);
    assert!(matches!(
        script.prepared_scripts[0].outcome,
        PreparedScriptOutcome::DeferredSyntaxError { .. }
    ));
    assert_eq!(script.prepared_script_units().count(), 0);
    assert_eq!(script.body.statements.len(), 2);
}

#[test]
fn literal_template_and_pure_concatenation_share_the_syntax_boundary() {
    for source in [
        "eval.call(undefined, '1 + 2');",
        "eval.call(undefined, `1 + 2`);",
        "eval.call(undefined, ('1' + ' + ' + '2'));",
    ] {
        let program = lower(source);
        assert!(
            program.diagnostics.is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("outer Script IR");
        assert_eq!(script.prepared_scripts.len(), 1, "{source}");
        assert_eq!(script.prepared_scripts[0].source, "1 + 2");
        assert_eq!(
            script.prepared_scripts[0].kind,
            PreparedScriptKind::IndirectEval
        );
    }
    let program = lower("eval.call(undefined, String('1 + 2'));");
    assert!(!program.is_wasm_supported());
    assert!(program
        .script
        .expect("outer Script IR")
        .prepared_scripts
        .is_empty());
}

#[test]
fn nested_source_units_share_unique_function_and_script_ids() {
    let program = lower(
        r#"__lilaRealmEvalScript("function owner() { return eval.call(undefined, 'function child() {} child;'); } owner();");"#,
    );
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.expect("outer Script IR");
    assert_eq!(script.prepared_script_units().count(), 2);
    let ids = script
        .functions
        .iter()
        .map(|function| function.id.clone())
        .chain(
            script
                .prepared_script_units()
                .map(|unit| unit.id.function_id()),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        ids.len()
    );
}

#[test]
fn forwarded_literal_candidates_keep_the_original_runtime_invocation() {
    for source in [
        "Reflect.construct(other.Function, ['return 3;'], newTarget);",
        "Reflect.apply(other.Function, null, ['return 3;']);",
        "Function.apply(null, ['return 3;']);",
    ] {
        let program = lower(source);
        assert!(
            program.diagnostics.is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("outer Script IR");
        assert_eq!(script.prepared_dynamic_functions.len(), 1, "{source}");
        assert_eq!(
            script.prepared_dynamic_functions[0].arguments,
            ["return 3;"]
        );
        assert_eq!(script.body.statements.len(), 1, "{source}");
        assert!(
            matches!(script.body.statements[0], StatementIr::Expression(_)),
            "{source}"
        );
    }
    for source in [
        "Reflect.apply(other.eval, null, ['6;']);",
        "eval.apply(null, ['6;']);",
        "Reflect.apply(other.evalScript, null, ['6;']);",
    ] {
        let program = lower(source);
        assert!(
            program.diagnostics.is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("outer Script IR");
        assert_eq!(script.prepared_scripts.len(), 1, "{source}");
        assert_eq!(script.prepared_scripts[0].source, "6;");
        assert_eq!(script.body.statements.len(), 1, "{source}");
        assert!(
            matches!(script.body.statements[0], StatementIr::Expression(_)),
            "{source}"
        );
    }
}
