use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, ExprIr, PreparedScriptKind, PreparedScriptOutcome,
    ProgramIr, PropertyKeyIr, StatementIr, TypedExpr,
};

fn prepare(source: &str) -> ProgramIr {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
}

fn assert_prepared(program: &ProgramIr, text: &str) {
    for direct in [false, true] {
        assert!(
            program
                .script
                .as_ref()
                .unwrap()
                .prepared_scripts
                .iter()
                .any(|entry| entry.source == text
                    && matches!(entry.kind, PreparedScriptKind::DirectEval(_)) == direct
                    && matches!(entry.outcome, PreparedScriptOutcome::Executable(_))),
            "missing {text:?}, direct={direct}"
        );
    }
}

#[test]
fn literal_and_resolved_constructor_sources_prepare_escaped_script_candidates() {
    for (definitions, expression, expected) in [
        ("", "new RegExp('')", "/(?:)/"),
        ("", "new RegExp()", "/(?:)/"),
        ("", "new RegExp(undefined)", "/(?:)/"),
        ("", r"RegExp('\n')", r"/\n/"),
        ("", "new RegExp('/')", r"/\//"),
        ("", r"/\//", r"/\//"),
        (
            "var constructor = RegExp; var pattern = 'a/b';",
            "new constructor(pattern)",
            r"/a\/b/",
        ),
        (
            "var constructor = RegExp; var regexp = constructor('c/d');",
            "regexp",
            r"/c\/d/",
        ),
    ] {
        let program = prepare(&format!(
            "{definitions} eval('/' + ({expression}).source + '/');"
        ));
        assert_prepared(&program, expected);
    }
}

fn has_live_regexp_source_get(expression: &TypedExpr) -> bool {
    match &expression.expr {
        ExprIr::CoerciveAdd { lhs, rhs } => {
            has_live_regexp_source_get(lhs) || has_live_regexp_source_get(rhs)
        }
        ExprIr::MaterializeBinding { value, .. } => has_live_regexp_source_get(value),
        ExprIr::PropertyRead { target, key } => {
            matches!(key, PropertyKeyIr::StaticString(name) if name == "source")
                && matches!(
                    target.expr,
                    ExprIr::Construct { .. } | ExprIr::RegExpLiteral { .. }
                )
        }
        _ => false,
    }
}

#[test]
fn source_hints_retain_the_original_constructor_get_and_direct_eval_argument() {
    let program =
        prepare("var constructor = RegExp; eval('/' + new constructor('x').source + '/');");
    assert_prepared(&program, "/x/");
    let Some(StatementIr::Expression(expression)) =
        program.script.as_ref().unwrap().body.statements.last()
    else {
        panic!("the original eval call remains an expression");
    };
    let ExprIr::EnvironmentIdentifier(identifier) = &expression.expr else {
        panic!("the original eval Reference remains runtime-owned");
    };
    let EnvironmentIdentifierOperationIr::Call {
        args,
        direct_eval: Some(_),
        ..
    } = &identifier.operation
    else {
        panic!("the original call retains direct-eval context");
    };
    assert!(
        has_live_regexp_source_get(&args[0]),
        "a hint replaced live construction or Get"
    );
}

#[test]
fn direct_eval_keeps_all_256_primary_sources_before_regexp_conversion_views() {
    let texts = (0..256)
        .map(|index| format!("'{index};'"))
        .collect::<Vec<_>>()
        .join(",");
    let program = prepare(&format!(
        "var texts = [{texts}]; eval(...texts, '/' + /hint/.source + '/');"
    ));
    for index in 0..256 {
        assert_prepared(&program, &format!("{index};"));
    }
    let prepared = &program.script.as_ref().unwrap().prepared_scripts;
    assert_eq!(
        prepared.len(),
        512,
        "each of the two eval kinds retains exactly 256 sources"
    );
    assert!(!prepared.iter().any(|entry| entry.source == "/hint/"));
}

#[test]
fn unknown_runtime_patterns_do_not_acquire_prepared_source_authority() {
    let program =
        prepare("var pattern = 'z' + Math.random(); eval('/' + new RegExp(pattern).source + '/');");
    assert!(program.script.unwrap().prepared_scripts.is_empty());
}
