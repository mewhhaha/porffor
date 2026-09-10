use boa_ast::{Expression, Statement, StatementListItem};
use lila_front::{prepare_dynamic_function, FunctionParseKind, ParseDiagnosticKind};

#[test]
fn prepared_functions_parse_each_execution_grammar_without_a_name_binding() {
    for (kind, body) in [
        (FunctionParseKind::Ordinary, "return anonymous + value;"),
        (FunctionParseKind::Generator, "yield anonymous + value;"),
        (FunctionParseKind::Async, "return await value;"),
        (FunctionParseKind::AsyncGenerator, "yield await value;"),
    ] {
        let arguments = ["value".to_string(), body.to_string()];
        let parsed =
            prepare_dynamic_function(kind, &arguments).expect("constructor grammar parses");
        parsed.with_compiler_session(|script, _| {
            let StatementListItem::Statement(statement) = &script.statements().statements()[0]
            else {
                panic!("prepared script contains a statement");
            };
            let Statement::Expression(expression) = statement.as_ref() else {
                panic!("prepared script contains one function expression");
            };
            let expression = match expression {
                Expression::Parenthesized(expression) => expression.expression(),
                expression => expression,
            };
            let name = match expression {
                Expression::FunctionExpression(function) => function.name(),
                Expression::GeneratorExpression(function) => function.name(),
                Expression::AsyncFunctionExpression(function) => function.name(),
                Expression::AsyncGeneratorExpression(function) => function.name(),
                expression => panic!("unexpected prepared expression {expression:?}"),
            };
            assert!(name.is_none(), "display name must not introduce a binding");
        });
        assert!(kind
            .source_text(&arguments)
            .contains(" anonymous(value\n) {\n"));
    }
}

#[test]
fn constructor_parameters_and_body_cannot_complete_each_others_tokens() {
    for kind in [
        FunctionParseKind::Ordinary,
        FunctionParseKind::Generator,
        FunctionParseKind::Async,
        FunctionParseKind::AsyncGenerator,
    ] {
        for arguments in [
            ["/*", "*/) {}"],
            ["a) {", "return 1; } //"],
            ["a", "} function injected() {} //"],
            ["a", "/*"],
        ] {
            let arguments = arguments.map(str::to_string);
            let error = prepare_dynamic_function(kind, &arguments)
                .expect_err("fragments must parse separately");
            assert_eq!(error.diagnostic().error_type(), Some("SyntaxError"));
            assert_ne!(
                error.diagnostic().kind(),
                ParseDiagnosticKind::UnsupportedParserFeature
            );
        }
    }
}

#[test]
fn combined_constructor_early_errors_follow_the_body_strictness() {
    for arguments in [
        ["a,a", "'use strict'; return a;"],
        ["a = 1", "'use strict'; return a;"],
        ["eval", "'use strict'; return eval;"],
        ["a", "let a;"],
        ["a", "return super.value;"],
        ["a", "return this.#secret;"],
    ] {
        let error =
            prepare_dynamic_function(FunctionParseKind::Ordinary, &arguments.map(str::to_string))
                .expect_err("combined constructor early error");
        assert_eq!(error.diagnostic().error_type(), Some("SyntaxError"));
    }
    prepare_dynamic_function(
        FunctionParseKind::Ordinary,
        &["a,a".to_string(), "return a;".to_string()],
    )
    .expect("sloppy duplicate parameters remain valid");
}

#[test]
fn constructor_source_text_preserves_arguments_and_required_newlines() {
    let arguments = [
        "a /* é */".to_string(),
        "b".to_string(),
        "return a + b; // 🌱".to_string(),
    ];
    assert_eq!(
        FunctionParseKind::Ordinary.source_text(&arguments),
        "function anonymous(a /* é */,b\n) {\nreturn a + b; // 🌱\n}",
    );
    assert_eq!(
        FunctionParseKind::AsyncGenerator.source_text(&[]),
        "async function* anonymous(\n) {\n\n}",
    );
    prepare_dynamic_function(FunctionParseKind::Ordinary, &arguments).expect("comments preserved");
}
