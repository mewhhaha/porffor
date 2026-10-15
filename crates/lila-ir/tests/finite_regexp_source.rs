use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, EnvironmentIdentifierResolutionStart, ExprIr,
    PreparedScriptKind, PreparedScriptOutcome, ProgramIr, SpecOperationIr, StatementIr, TypedExpr,
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
        ("", r"RegExp('\\\n')", r"/\n/"),
        ("", r"RegExp('\\\r')", r"/\r/"),
        ("", r"RegExp('\\\u2028')", r"/\u2028/"),
        ("", r"RegExp('\\\u2029')", r"/\u2029/"),
        ("", r"RegExp('[\\\n]')", r"/[\n]/"),
        ("", r"RegExp('[/]')", r"/[/]/"),
        ("", r"RegExp('[\\/]')", r"/[\/]/"),
        ("", r"/[/]/", r"/[/]/"),
        ("", r"/[\\/]/", r"/[\\/]/"),
        ("", r"/\[\/\]/", r"/\[\/\]/"),
        ("", r"RegExp('\\[/\\]')", r"/\[\/\]/"),
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

fn assert_named_environment_read(expression: &TypedExpr, expected_name: &str) {
    let ExprIr::EnvironmentIdentifier(reference) = &expression.expr else {
        panic!("{expected_name} retains its live Reference: {expression:?}")
    };
    assert_eq!(reference.name, expected_name);
    assert_eq!(
        reference.resolution_start(),
        EnvironmentIdentifierResolutionStart::CurrentEnvironment
    );
    assert!(matches!(
        reference.operation,
        EnvironmentIdentifierOperationIr::Read
    ));
}

#[test]
fn source_hints_retain_the_original_constructor_get_and_direct_eval_argument() {
    let program =
        prepare("var constructor = RegExp; eval('/' + new constructor('x').source + '/');");
    assert_prepared(&program, "/x/");
    let script = program.script.as_ref().unwrap();
    let StatementIr::Var(declarators) = &script.body.statements[0] else {
        panic!("the constructor alias declaration remains emitted");
    };
    assert_eq!(declarators.len(), 1);
    assert_eq!(declarators[0].name, "constructor");
    assert_named_environment_read(declarators[0].init.as_ref().unwrap(), "RegExp");
    let Some(StatementIr::Expression(expression)) = script.body.statements.last() else {
        panic!("the original eval call remains an expression");
    };
    let ExprIr::EnvironmentIdentifier(identifier) = &expression.expr else {
        panic!("the original eval Reference remains runtime-owned");
    };
    assert_eq!(identifier.name, "eval");
    assert_eq!(
        identifier.resolution_start(),
        EnvironmentIdentifierResolutionStart::CurrentEnvironment
    );
    let EnvironmentIdentifierOperationIr::Call {
        args,
        direct_eval: Some(_),
        ..
    } = &identifier.operation
    else {
        panic!("the original call retains direct-eval context");
    };
    assert_eq!(args.len(), 1);
    let ExprIr::StringConcat { lhs, rhs } = &args[0].expr else {
        panic!("the original eval argument retains its suffix concatenation: {args:?}");
    };
    assert!(matches!(&rhs.expr, ExprIr::String(text) if text == "/"));
    let ExprIr::CoerciveAdd { lhs, rhs } = &lhs.expr else {
        panic!("the source Get retains its live coercion boundary: {lhs:?}");
    };
    assert!(matches!(&lhs.expr, ExprIr::String(text) if text == "/"));
    let ExprIr::SpecOperation {
        operation: SpecOperationIr::GetV,
        operands,
    } = &rhs.expr
    else {
        panic!("a hint replaced the live source GetV: {rhs:?}");
    };
    assert_eq!(operands.len(), 2);
    assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == "source"));
    let ExprIr::Construct { callee, args, .. } = &operands[0].expr else {
        panic!("a hint replaced live construction: {:?}", operands[0]);
    };
    assert_named_environment_read(callee, "constructor");
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0].expr, ExprIr::String(pattern) if pattern == "x"));
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
