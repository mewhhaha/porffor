use super::*;

#[test]
fn script_top_level_using_declaration_rejects() {
    let source = "using x = null;";
    let err = parse(source, ParseOptions::script())
        .expect_err("a top-level Script using declaration should fail");
    assert_eq!(
        err.diagnostic().phase(),
        ParseDiagnosticPhase::Early,
        "{source:?}: {err:?}"
    );
    assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
    assert_eq!(
        err.diagnostic().code,
        early(EarlyErrorCode::ScriptTopLevelUsingDeclaration),
        "{source:?}: {err:?}"
    );
    assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
}

#[test]
fn script_nested_using_declaration_boundaries_remain_valid() {
    for source in [
        "{ using x = null; }",
        "function f() { using x = null; }",
        "async function f() { await using x = null; }",
        "for (using x = null;;) break;",
        "for (using x of [null]) {}",
        "class C { static { using x = null; } }",
    ] {
        parse(source, ParseOptions::script())
            .expect("nested and loop-head using declarations should remain valid");
    }
}

#[test]
fn for_in_using_declarations_reject_under_both_goals() {
    for source in [
        "for (using x in {}) {}",
        "async function f() { for (await using x in {}) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("a using declaration in a for-in head should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ForInUsingDeclaration),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn for_in_using_declaration_siblings_remain_valid() {
    for source in [
        "for (using x of [null]) {}",
        "async function f() { for (await using x of []) {} }",
        "for (let x in {}) {}",
        "for (const x in {}) {}",
        "for (using x = null;;) break;",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("for-of, ordinary for-in and classic-for siblings should remain valid");
        }
    }
}

#[test]
fn for_head_body_declaration_conflicts_reject_under_both_goals() {
    for source in [
        "for (let x; false;) { var x; }",
        "for (const x = 0; false;) { var x; }",
        "for (using x = null; false;) { var x; }",
        "async function f() { for (await using x = null; false;) { var x; } }",
        "for (let x in {}) { var x; }",
        "for (const x in {}) { var x; }",
        "for (let x of []) { var x; }",
        "for (const x of []) { var x; }",
        "for (using x of []) { var x; }",
        "async function f() { for (await using x of []) { var x; } }",
        "async function f() { for await (let x of []) { var x; } }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("a lexical loop head must conflict with a body var declaration");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ForHeadBodyDeclarationConflict),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("the conflicting loop declaration must retain its source span");
            assert!(span.start < span.end, "{source:?}: {err:?}");
        }
    }
}

#[test]
fn for_declaration_duplicate_bound_names_reject_under_both_goals() {
    for source in [
        "for (let [x, x] in {}) {}",
        "for (const { a: x, b: x } in {}) {}",
        "for (let { a: x, b: x } of []) {}",
        "for (const [x, x] of []) {}",
        "async function f() { for await (let [x, x] of []) {} }",
        "async function f() { for await (const { a: x, b: x } of []) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("duplicate BoundNames in a ForDeclaration should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ForDeclarationDuplicateBoundName),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("the duplicate loop binding must retain its source span");
            assert!(span.start < span.end, "{source:?}: {err:?}");
        }
    }
}

#[test]
fn lexical_bound_name_let_rejects_the_complete_parser_surface_under_both_goals() {
    for source in [
        "let let;",
        "let [let] = [];",
        "let { value: let } = {};",
        "let { outer: [let] } = {};",
        "const let = 0;",
        "const [let] = [];",
        "const { value: let } = {};",
        "const [...let] = [];",
        r#""use strict"; let [let] = [];"#,
        "function f() { using let = null; }",
        "async function f() { await using let = null; }",
        "for (let let = 0; false;) {}",
        "for (let [let] = []; false;) {}",
        "for (let { value: let } = {}; false;) {}",
        "for (let { outer: [...let] } = {}; false;) {}",
        "for (const let = 0; false;) {}",
        "for (const [let] = []; false;) {}",
        "for (const { value: let } = {}; false;) {}",
        "for (let let in {}) {}",
        "for (const [let] in {}) {}",
        "for (let { value: let } of []) {}",
        "for (const { outer: [let] } of []) {}",
        "for (const let of []) {}",
        "async function f() { for await (let let of []) {} }",
        "async function f() { for await (const [let] of []) {} }",
        "for (using let of []) {}",
        "async function f() { for (await using let of []) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("a lexical or ForDeclaration BoundName equal to let should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::LexicalBoundNameLet),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("the forbidden lexical binding must retain its source span");
            assert!(span.start < span.end, "{source:?}: {err:?}");
        }
    }
}

#[test]
fn lexical_bound_name_let_precedes_adjacent_declaration_errors() {
    for source in [
        "let [let, let] = [];",
        "for (let [let, let] = []; ; ) {}",
        "for (let [let, let] of []) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("the forbidden let binding should precede duplicate-name errors");
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::LexicalBoundNameLet),
                "{source:?}: {err:?}"
            );
        }
    }
}

#[test]
fn lexical_bound_name_let_positive_and_strict_identifier_boundaries_stay_distinct() {
    for source in [
        "let { let: x } = {};",
        "const { let: x } = {};",
        "let letter;",
        "const letter = 0;",
        "for (let letter = 0; false;) {}",
        "for (const { let: x } = {}; false;) {}",
        "for (let letter in {}) {}",
        "for (const { let: x } of []) {}",
        "async function f() { for await (let letter of []) {} }",
        "for (using letter of []) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("only an exact BoundName equal to let belongs to this condition");
        }
    }

    parse("var let;", ParseOptions::script())
        .expect("sloppy Script var let remains outside lexical BoundNames");

    for (source, options) in [
        (r#""use strict"; var let;"#, ParseOptions::script()),
        (r#""use strict"; var [let] = [];"#, ParseOptions::script()),
        ("var let;", ParseOptions::module()),
        ("var { value: let } = {};", ParseOptions::module()),
    ] {
        let err = parse(source, options)
            .expect_err("strict code should retain its existing identifier rejection");
        assert_eq!(
            err.diagnostic().code,
            ParseCode::Malformed,
            "{source:?}: {err:?}"
        );
        assert_ne!(
            err.diagnostic().code,
            early(EarlyErrorCode::LexicalBoundNameLet),
            "{source:?}: {err:?}"
        );
    }
}

#[test]
fn user_export_names_cannot_forge_lexical_bound_name_let_classification() {
    for exported_name in [
        "'let' is disallowed as a lexically bound name at line",
        "Cannot use 'let' as a lexically bound name at line",
    ] {
        let source = format!(
            "const value = 0;\nexport {{ value as \"{exported_name}\" }};\nexport {{ value as \"{exported_name}\" }};"
        );
        let err = parse(&source, ParseOptions::module())
            .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{exported_name:?}: {err:?}"
        );
    }
}

#[test]
fn for_declaration_duplicate_bound_name_boundaries_remain_distinct() {
    for source in [
        "for (var [x, x] in {}) {}",
        "for (var { a: x, b: x } of []) {}",
        "async function f() { for await (var [x, x] of []) {} }",
        "for (let [x, y] in {}) {}",
        "for (const { a: x, b: y } of []) {}",
        "async function f() { for await (let [x, y] of []) {} }",
        "for (let [x, y] = []; false;) {}",
        "for (const { a: x, b: y } = {}; false;) {}",
        "for (using [x, x] of []) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).expect(
                "positive boundaries outside duplicate ForDeclaration BoundNames should parse",
            );
        }
    }

    for (source, keyword) in [
        ("for (let [x, x] = []; false;) {}", "let"),
        ("for (const { a: x, b: x } = {}; false;) {}", "const"),
        ("for (\n    let [x, x] = [];\n    false;\n) {}", "let"),
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("classic-for lexical duplicate BoundNames should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::DuplicateLexicalDeclaration),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("classic-for revalidation must retain the lexical keyword position");
            let keyword_start = source
                .find(keyword)
                .expect("the classic-for source must contain its lexical keyword");
            assert_eq!(
                span,
                SourceSpan {
                    start: keyword_start,
                    end: keyword_start + 1,
                },
                "{source:?}: {err:?}"
            );
        }
    }

    let source = "for (let [x, x] of []) { var x; }";
    for options in [ParseOptions::script(), ParseOptions::module()] {
        let err = parse(source, options)
            .expect_err("the head/body conflict should be diagnosed before the duplicate");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ForHeadBodyDeclarationConflict),
            "{source:?}: {err:?}"
        );
    }

    for source in ["async function f() { for (await using [x, x] of []) {} }"] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("an earlier non-ForDeclaration boundary should reject");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Parse,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(err.diagnostic().code, ParseCode::Malformed);
            assert_ne!(
                err.diagnostic().code,
                early(EarlyErrorCode::ForDeclarationDuplicateBoundName),
                "{source:?}: {err:?}"
            );
        }
    }
}

#[test]
fn var_heads_and_nested_function_var_declarations_remain_valid() {
    for source in [
        "for (var x; false;) { var x; }",
        "for (var x in {}) { var x; }",
        "for (var x of []) { var x; }",
        "for (let x; false;) { (function () { var x; }); }",
        "for (let x in {}) { (function () { var x; }); }",
        "for (let x of []) { (function () { var x; }); }",
        "for (let x; false;) { function nested() { var x; } }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("var heads and declarations across a nested function boundary stay valid");
        }
    }
}

#[test]
fn known_for_declaration_semantic_producers_stay_reviewed() {
    fn count_message_in_rust_sources(root: &std::path::Path, message: &str) -> usize {
        let entries = std::fs::read_dir(root)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
        let mut count = 0;
        for entry in entries {
            let path = entry.expect("failed to read vendored Boa entry").path();
            if path.is_dir() {
                count += count_message_in_rust_sources(&path, message);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                let source = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
                count += source.matches(message).count();
            }
        }
        count
    }

    const CONFLICT_MESSAGE: &str = "For loop initializer declared in loop body";
    const DUPLICATE_MESSAGE: &str = "For loop initializer cannot contain duplicate identifiers";
    const GENERIC_DUPLICATE_MESSAGE: &str = "lexical name declared multiple times";
    const LEXICAL_BOUND_NAME_LET_MESSAGE: &str = "'let' is disallowed as a lexically bound name";
    const FOR_DECLARATION_BOUND_NAME_LET_MESSAGE: &str =
        "Cannot use 'let' as a lexically bound name";
    const LEXICAL_CONTEXT: &str = r#"enum LexicalDeclarationContext {
    Statement,
    ForHead,
}"#;
    const BINDING_IDENTIFIER_CONTEXT: &str = r#"enum BindingIdentifierContext {
    General,
    LexicalDeclaration,
}"#;
    const EXHAUSTIVE_BINDING_IDENTIFIER_CONTEXT_PROJECTION: &str = r#"fn allows_lexical_bound_name_let(self, identifier: Sym) -> bool {
        match self {
            Self::General => false,
            Self::LexicalDeclaration => identifier == Sym::LET,
        }
    }"#;
    const STRICT_RESERVED_LET_EXCEPTION: &str = r#"if cursor.strict()
            && ident.is_strict_reserved_identifier()
            && !context.allows_lexical_bound_name_let(ident)
        {"#;
    const STATEMENT_CONTEXT_CONSTRUCTOR: &str = r#"pub(in crate::parser) fn statement<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            context: LexicalDeclarationContext::Statement,
        }
    }"#;
    const FOR_HEAD_CONTEXT_CONSTRUCTOR: &str = r#"pub(in crate::parser) fn for_head<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            context: LexicalDeclarationContext::ForHead,
        }
    }"#;
    const AWAIT_RESOURCE_PATTERN_LOOKAHEAD_EXIT: &str = r#"if matches!(
            next.kind(),
            TokenKind::Punctuator(Punctuator::OpenBracket | Punctuator::OpenBlock)
        ) {
            return Ok(None);
        }"#;
    const ORDINARY_RESOURCE_PATTERN_LOOKAHEAD_EXIT: &str = r#"if matches!(
        next.kind(),
        TokenKind::Punctuator(Punctuator::OpenBracket | Punctuator::OpenBlock)
    ) {
        return Ok(None);
    }"#;
    const DEFERRED_LEXICAL_INITIALIZER: &str = r#"DeferredLexical {
        declaration: ast::declaration::LexicalDeclaration,
        keyword_position: Position,
    }"#;
    const SHARED_DUPLICATE_VALIDATOR: &str = r#"pub(in crate::parser) fn validate_duplicate_bound_names(
        declaration: &ast::declaration::LexicalDeclaration,
        position: Position,
    ) -> ParseResult<()> {
        let mut names = FxHashSet::default();
        for name in bound_names(declaration) {
            if !names.insert(name) {
                return Err(Error::general(
                    "lexical name declared multiple times",
                    position,
                ));
            }
        }
        Ok(())
    }"#;
    const SHARED_BOUND_NAME_LET_VALIDATOR: &str = r#"pub(in crate::parser) fn validate_bound_name_let(
        declaration: &ast::declaration::LexicalDeclaration,
        position: Position,
    ) -> ParseResult<()> {
        for name in bound_names(declaration) {
            if name == Sym::LET {
                return Err(Error::general(
                    "'let' is disallowed as a lexically bound name",
                    position,
                ));
            }
        }
        Ok(())
    }"#;
    const STATEMENT_CONTEXT_DECISION: &str = r#"match &self.context {
            LexicalDeclarationContext::Statement => {
                cursor.expect_semicolon("lexical declaration", interner)?;
                Self::validate_bound_name_let(&lexical_declaration, tok.span().start())?;
                Self::validate_duplicate_bound_names(&lexical_declaration, tok.span().start())?;
            }
            LexicalDeclarationContext::ForHead => {}
        }"#;
    const INITIALIZER_REQUIREMENT: &str = r#"match self.context {
                    LexicalDeclarationContext::Statement if init_is_some => decls.push(decl),
                    LexicalDeclarationContext::Statement => {
                        let next = cursor.next(interner).or_abrupt()?;
                        return Err(Error::general(
                            format!(
                                "Expected initializer for {} declaration",
                                self.declaration_kind.description()
                            ),
                            next.span().start(),
                        ));
                    }
                    LexicalDeclarationContext::ForHead => decls.push(decl),
                }"#;
    const MISSING_TERMINATOR_DECISION: &str = r#"SemicolonResult::NotFound(_) => match self.context {
                    LexicalDeclarationContext::Statement => {
                        let next = cursor.next(interner).or_abrupt()?;
                        return Err(Error::expected(
                            [";".to_owned(), "line terminator".to_owned()],
                            next.to_string(interner),
                            next.span(),
                            "lexical declaration binding list",
                        ));
                    }
                    LexicalDeclarationContext::ForHead => break,
                },"#;
    const DEFERRED_CLASSIC_ROUTE: &str = r#"(
                Some(ParsedForInitializer::DeferredLexical {
                    declaration,
                    keyword_position,
                }),
                _,
            ) => {
                LexicalDeclaration::validate_bound_name_let(
                    &declaration,
                    keyword_position,
                )?;
                LexicalDeclaration::validate_duplicate_bound_names(
                    &declaration,
                    keyword_position,
                )?;
                Some(declaration.into())
            }"#;
    const DEFERRED_ITERABLE_ROUTE: &str = r#"Some(ParsedForInitializer::DeferredLexical { declaration, .. }),
                TokenKind::Keyword((kw @ (Keyword::In | Keyword::Of), false)),
            ) => {
                let in_loop = kw == &Keyword::In;
                let init = initializer_to_iterable_loop_initializer(
                    declaration.into(),
                    position,
                    cursor.strict(),
                    in_loop,
                )?;
                return parse_iterable_loop_tail("#;
    const CLASSIC_INTERSECTION: &str = r#"if let Some(ForLoopInitializer::Lexical(initializer)) = &init {
            let vars = var_declared_names(&body);
            for name in bound_names(initializer.declaration()) {
                if vars.contains(&name) {
                    return Err(Error::general(
                        "For loop initializer declared in loop body",
"#;
    const ITERABLE_INTERSECTION: &str = r#"if matches!(
        &init,
        IterableLoopInitializer::Const(_)
            | IterableLoopInitializer::Let(_)
            | IterableLoopInitializer::Using(_)
            | IterableLoopInitializer::AwaitUsing(_)
    ) {
        let vars = var_declared_names(&body);
        let mut names = FxHashSet::default();
        for name in bound_names(&init) {
            if name == Sym::LET {
                return Err(Error::general(
                    "Cannot use 'let' as a lexically bound name",
                    position,
                ));
            }
            if vars.contains(&name) {
                return Err(Error::general(
                    "For loop initializer declared in loop body",
"#;
    const DUPLICATE_AFTER_INTERSECTION: &str = r#"if vars.contains(&name) {
                return Err(Error::general(
                    "For loop initializer declared in loop body",
                    position,
                ));
            }
            if !names.insert(name) {
                return Err(Error::general(
                    "For loop initializer cannot contain duplicate identifiers",
"#;
    const FOR_STATEMENT_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/statement/iteration/for_statement.rs"
    ));
    const LEXICAL_DECLARATION_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/lexical.rs"
    ));
    const DECLARATION_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/mod.rs"
    ));
    const BINDING_PATTERN_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/statement/mod.rs"
    ));
    const BINDING_IDENTIFIER_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/expression/identifiers.rs"
    ));
    const VARIABLE_DECLARATION_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../../../vendor/boa_parser-0.21.1/src/parser/statement/variable/mod.rs"
    ));

    let boa_package_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/boa_parser-0.21.1");
    assert_eq!(
        count_message_in_rust_sources(&boa_package_root, CONFLICT_MESSAGE),
        2
    );
    assert_eq!(
        count_message_in_rust_sources(&boa_package_root, DUPLICATE_MESSAGE),
        1
    );
    assert_eq!(
        count_message_in_rust_sources(&boa_package_root, GENERIC_DUPLICATE_MESSAGE),
        6
    );
    assert_eq!(
        count_message_in_rust_sources(&boa_package_root, LEXICAL_BOUND_NAME_LET_MESSAGE),
        1
    );
    assert_eq!(
        count_message_in_rust_sources(&boa_package_root, FOR_DECLARATION_BOUND_NAME_LET_MESSAGE,),
        1
    );
    assert_eq!(
        count_message_in_rust_sources(
            &boa_package_root,
            "BindingIdentifierContext::LexicalDeclaration",
        ),
        3,
        "only lexical binding parsers may opt into deferring the let condition"
    );
    assert_eq!(FOR_STATEMENT_SOURCE.matches(CONFLICT_MESSAGE).count(), 2);
    assert_eq!(FOR_STATEMENT_SOURCE.matches(DUPLICATE_MESSAGE).count(), 1);
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(GENERIC_DUPLICATE_MESSAGE)
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(GENERIC_DUPLICATE_MESSAGE)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(LEXICAL_BOUND_NAME_LET_MESSAGE)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(FOR_DECLARATION_BOUND_NAME_LET_MESSAGE)
            .count(),
        0
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(LEXICAL_BOUND_NAME_LET_MESSAGE)
            .count(),
        0
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(FOR_DECLARATION_BOUND_NAME_LET_MESSAGE)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(SHARED_DUPLICATE_VALIDATOR)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(SHARED_BOUND_NAME_LET_VALIDATOR)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE.matches(LEXICAL_CONTEXT).count(),
        1
    );
    assert_eq!(
        BINDING_IDENTIFIER_SOURCE
            .matches(BINDING_IDENTIFIER_CONTEXT)
            .count(),
        1
    );
    assert_eq!(
        BINDING_IDENTIFIER_SOURCE
            .matches(EXHAUSTIVE_BINDING_IDENTIFIER_CONTEXT_PROJECTION)
            .count(),
        1
    );
    assert_eq!(
        BINDING_IDENTIFIER_SOURCE
            .matches(STRICT_RESERVED_LET_EXCEPTION)
            .count(),
        1
    );
    assert_eq!(
        BINDING_IDENTIFIER_SOURCE
            .matches("context: BindingIdentifierContext::General")
            .count(),
        1
    );
    assert_eq!(
        BINDING_PATTERN_SOURCE
            .matches("binding_identifier_context: BindingIdentifierContext::General")
            .count(),
        2
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(STATEMENT_CONTEXT_CONSTRUCTOR)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(FOR_HEAD_CONTEXT_CONSTRUCTOR)
            .count(),
        1
    );
    assert_eq!(LEXICAL_DECLARATION_SOURCE.matches("loop_init").count(), 0);
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("matches!(self.context")
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("match self.context")
            .count(),
        2
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("match &self.context")
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("self.context ==")
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("self.context !=")
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("self.context.is_for_head()")
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("context: LexicalDeclarationContext,")
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("context: &'a LexicalDeclarationContext,")
            .count(),
        2
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("LexicalDeclarationContext::Statement")
            .count(),
        5
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("LexicalDeclarationContext::ForHead")
            .count(),
        4
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("Self::Statement")
            .count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE.matches("Self::ForHead").count(),
        0
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches("BindingIdentifierContext::LexicalDeclaration")
            .count(),
        3
    );
    assert_eq!(
        BINDING_PATTERN_SOURCE
            .matches(".with_context(self.binding_identifier_context)")
            .count(),
        5
    );
    assert_eq!(
        BINDING_PATTERN_SOURCE
            .matches(".with_binding_identifier_context")
            .count(),
        6
    );
    assert_eq!(
        VARIABLE_DECLARATION_SOURCE
            .matches("BindingIdentifierContext")
            .count(),
        0,
        "var declarations must retain the general strict-identifier path"
    );
    assert_eq!(
        DECLARATION_SOURCE
            .matches("LexicalDeclaration::statement(")
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches("LexicalDeclaration::for_head(")
            .count(),
        3
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches("using_declaration_kind(cursor, interner, self.allow_await.0, true)?")
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(AWAIT_RESOURCE_PATTERN_LOOKAHEAD_EXIT)
            .count(),
        1
    );
    assert_eq!(
        LEXICAL_DECLARATION_SOURCE
            .matches(ORDINARY_RESOURCE_PATTERN_LOOKAHEAD_EXIT)
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(DEFERRED_LEXICAL_INITIALIZER)
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches("keyword_position: init_token.span().start()")
            .count(),
        3
    );
    for decision in [
        STATEMENT_CONTEXT_DECISION,
        INITIALIZER_REQUIREMENT,
        MISSING_TERMINATOR_DECISION,
    ] {
        assert_eq!(LEXICAL_DECLARATION_SOURCE.matches(decision).count(), 1);
    }
    assert_eq!(
        FOR_STATEMENT_SOURCE.matches(DEFERRED_CLASSIC_ROUTE).count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches("LexicalDeclaration::validate_duplicate_bound_names(")
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches("LexicalDeclaration::validate_bound_name_let(")
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(DEFERRED_ITERABLE_ROUTE)
            .count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE.matches(CLASSIC_INTERSECTION).count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE.matches(ITERABLE_INTERSECTION).count(),
        1
    );
    assert_eq!(
        FOR_STATEMENT_SOURCE
            .matches(DUPLICATE_AFTER_INTERSECTION)
            .count(),
        1
    );
}

#[test]
fn switch_clause_using_declarations_reject_under_both_goals() {
    for source in [
        "switch (0) { case 0: using x = null; }",
        "switch (0) { default: using x = null; }",
        "async function f() { switch (0) { case 0: await using x = null; } }",
        "async function f() { switch (0) { default: await using x = null; } }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("a direct switch-clause using declaration should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::SwitchClauseUsingDeclaration),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn nested_switch_clause_using_declaration_boundaries_remain_valid() {
    for source in [
        "switch (0) { case 0: { using x = null; } }",
        "async function f() { switch (0) { default: { await using x = null; } } }",
        "switch (0) { case 0: let x; const y = null; }",
        "switch (0) { case 0: for (using x = null;;) break; }",
        "switch (0) { case 0: for (using x of []) {} }",
        "switch (0) { case 0: function f() { using x = null; } }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("nested and ordinary lexical switch-clause siblings should remain valid");
        }
    }
}

#[test]
fn parser_accepts_sloppy_let_and_static_labels_with_decoded_identifiers() {
    for source in [
        "let: 42;",
        r"l\u0065t: 42;",
        "if (true) let: 42;",
        r"if (true) l\u0065t: 42;",
        "let: { break let; }",
        r"l\u0065t: { break let; }",
        r"let: { break l\u0065t; }",
        "let: while (false) { continue let; }",
        r"l\u0065t: while (false) { continue let; }",
        r"let: while (false) { continue l\u0065t; }",
        "static: { break static; }",
        r"st\u0061tic: { break static; }",
        r"static: { break st\u0061tic; }",
        "function outer() { let: { break let; } }",
    ] {
        parse(source, ParseOptions::script())
            .unwrap_or_else(|err| panic!("sloppy label should parse: {source:?}: {err:?}"));
    }
}

#[test]
fn parser_rejects_let_and_static_labels_in_every_strict_context() {
    for label in ["let", r"l\u0065t", "static", r"st\u0061tic"] {
        let body = format!("{label}: 42;");
        for source in [
            format!("'use strict'; {body}"),
            format!("'use strict'\n{body}"),
            format!("function outer() {{ 'use strict'; {body} }}"),
            format!("function outer() {{ 'use strict'\n{body} }}"),
            format!("class C {{ method() {{ {body} }} }}"),
        ] {
            let err = parse(&source, ParseOptions::script())
                .expect_err("strict reserved word must not become a label");
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        }
        let err = parse(&body, ParseOptions::module())
            .expect_err("module labels must obey strict reserved-word rules");
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
    }
}

#[test]
fn parser_let_label_lookahead_preserves_declaration_and_expression_routes() {
    for source in [
        "let value = 42; value;",
        "let [value] = [42]; value;",
        "let { value } = { value: 42 }; value;",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).expect("lexical declarations must retain their parse route");
        }
    }
    for source in [
        "let = 42;",
        r"l\u0065t = 42;",
        "static = 42;",
        r"st\u0061tic = 42;",
    ] {
        parse(source, ParseOptions::script())
            .expect("sloppy identifier assignment must retain its expression route");
        parse(source, ParseOptions::module())
            .expect_err("strict reserved identifier assignment must still fail");
    }
    let err = parse(r"let: { l\u0065t: 42; }", ParseOptions::script())
        .expect_err("decoded duplicate labels must still be rejected");
    assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
}

#[test]
fn parser_duplicate_lexical_declaration_reports_early_phase() {
    let err = parse("let x; let x;", ParseOptions::script())
        .expect_err("duplicate lexical declaration should fail");
    assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
    assert_eq!(
        err.diagnostic().code,
        early(EarlyErrorCode::DuplicateLexicalDeclaration)
    );
}

#[test]
fn catch_body_declaration_conflicts_report_one_early_error_in_both_goals() {
    for source in [
        "try {} catch (a) { let a; }",
        "try {} catch ({ a }) { var a; }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("catch parameter/body declaration conflict should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::CatchBodyDeclarationConflict),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn simple_catch_identifier_preserves_var_redeclaration_exception_in_both_goals() {
    let source = "try {} catch (a) { var a; }";
    for options in [ParseOptions::script(), ParseOptions::module()] {
        parse(source, options)
            .expect("a simple catch identifier may be redeclared with var in its body");
    }
}

/// Drift B3, closed.
///
/// `ModuleParser::parse` words this one ``lexical name `x` declared
/// multiple times`` — with an interpolated identifier and no `names`. The
/// front end's old loose alternative required the literal substring
/// `names`, so a module-goal lexical redeclaration classified as
/// `P_PARSE_MALFORMED` here while the identical source classified as
/// `E_DUPLICATE_LEXICAL_DECLARATION` when it arrived as a *dependency*
/// module. One table, one answer.
#[test]
fn module_goal_duplicate_lexical_declaration_is_an_early_error_not_malformed() {
    let err = parse("let x; const x = 1;", ParseOptions::module())
        .expect_err("duplicate lexical declaration should fail in module goal");
    assert_eq!(
        err.diagnostic().code,
        early(EarlyErrorCode::DuplicateLexicalDeclaration),
        "{err:?}"
    );
    assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
}

#[test]
fn parser_accepts_sloppy_annex_b_block_functions() {
    for source in [
        "if (true) function then_branch() {} else function else_branch() {}",
        "label: function labelled() {}",
    ] {
        parse(source, ParseOptions::script()).expect("sloppy Annex B block function should parse");
    }
}

#[test]
fn parser_rejects_annex_b_block_functions_in_strict_and_module_code() {
    let cases = [
        (
            "'use strict'; if (true) function strict_script() {}",
            ParseOptions::script(),
        ),
        (
            "function outer() { 'use strict'; if (true) function strict_function() {} }",
            ParseOptions::script(),
        ),
        (
            "'use strict'; label: function strict_label() {}",
            ParseOptions::script(),
        ),
        ("if (true) function module_if() {}", ParseOptions::module()),
        ("label: function module_label() {}", ParseOptions::module()),
    ];

    for (source, options) in cases {
        parse(source, options).expect_err("strict and module Annex B forms should fail");
    }
}

#[test]
fn annex_b_declarations_preserve_each_eligible_function_identity() {
    let source = r#"
{
    function sibling() {}
}
{
    function sibling() {}
}
switch (0) {
    case 0:
        function switch_function() {}
        break;
    default:
        function switch_function() {}
}
{
    function protected() {}
}
{
    let protected;
    {
        function protected() {}
    }
}
"#;
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");

    let declarations = annex_b_function_declarations(&script);
    let names = declarations
        .iter()
        .map(|function| interner.resolve_expect(function.name().sym()).to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "sibling",
            "sibling",
            "switch_function",
            "switch_function",
            "protected",
        ]
    );
    assert!(declarations
        .windows(2)
        .all(|pair| pair[0].linear_span().start() < pair[1].linear_span().start()));
    assert!(!core::ptr::eq(declarations[0], declarations[1]));
    assert!(!core::ptr::eq(declarations[2], declarations[3]));
    assert_eq!(
        annex_b_function_declarations_names(&script)
            .into_iter()
            .map(|name| interner.resolve_expect(name).to_string())
            .collect::<Vec<_>>(),
        ["sibling", "switch_function", "protected"]
    );
}

#[test]
fn annex_b_script_direct_function_allows_nested_candidate_with_the_same_name() {
    let source = "function f() { return 1; } { function f() { return 2; } } f() === 2;";
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");
    let StatementListItem::Declaration(declaration) = &script.statements().statements()[0] else {
        panic!("script should begin with a function declaration");
    };
    let Declaration::FunctionDeclaration(direct_function) = declaration.as_ref() else {
        panic!("script should begin with an ordinary function declaration");
    };

    let declarations = annex_b_function_declarations(&script);

    assert_eq!(declarations.len(), 1);
    let span = declarations[0].linear_span();
    assert_eq!(
        &source[span.start().pos()..span.end().pos()],
        "function f() { return 2; }",
        "the nested Annex B declaration should update the script's var-scoped binding"
    );
    assert!(
        !core::ptr::eq(declarations[0], direct_function),
        "the direct script declaration is not itself an Annex B candidate"
    );
}

#[test]
fn annex_b_function_body_direct_function_allows_nested_candidate_with_the_same_name() {
    let source = "function outer() { function f() { return 1; } { function f() { return 2; } } return f() === 2; }";
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");
    let StatementListItem::Declaration(declaration) = &script.statements().statements()[0] else {
        panic!("script should begin with the enclosing function declaration");
    };
    let Declaration::FunctionDeclaration(outer_function) = declaration.as_ref() else {
        panic!("script should begin with an ordinary function declaration");
    };
    let StatementListItem::Declaration(declaration) = &outer_function.body().statements()[0] else {
        panic!("function body should begin with a function declaration");
    };
    let Declaration::FunctionDeclaration(direct_function) = declaration.as_ref() else {
        panic!("function body should begin with an ordinary function declaration");
    };

    let declarations = annex_b_function_declarations(outer_function.body());

    assert_eq!(declarations.len(), 1);
    let span = declarations[0].linear_span();
    assert_eq!(
        &source[span.start().pos()..span.end().pos()],
        "function f() { return 2; }",
        "the nested Annex B declaration should update the function body's var-scoped binding"
    );
    assert!(
        !core::ptr::eq(declarations[0], direct_function),
        "the direct function-body declaration is not itself an Annex B candidate"
    );
}

#[test]
fn annex_b_direct_function_blocks_only_nested_candidate_with_same_name() {
    let source = r#"
{
    { function before() {} }
    function protected() {}
    { function protected() {} }
    { function sibling() {} }
    { function after() {} }
}
"#;
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");

    let declarations = annex_b_function_declarations(&script);
    let names = declarations
        .iter()
        .map(|function| interner.resolve_expect(function.name().sym()).to_string())
        .collect::<Vec<_>>();

    assert_eq!(names, ["before", "protected", "sibling", "after"]);
    assert_eq!(
        declarations[1].linear_span().start().pos(),
        source
            .find("function protected() {}")
            .expect("source should contain the direct declaration")
    );
    assert!(declarations
        .windows(2)
        .all(|pair| pair[0].linear_span().start() < pair[1].linear_span().start()));
    assert_eq!(
        annex_b_function_declarations_names(&script)
            .into_iter()
            .map(|name| interner.resolve_expect(name).to_string())
            .collect::<Vec<_>>(),
        ["before", "protected", "sibling", "after"]
    );
}

#[test]
fn annex_b_direct_function_blocks_nested_if_candidate_with_same_name() {
    let source = "{ function f(){1} if (true) function f(){2} }";
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");

    let declarations = annex_b_function_declarations(&script);

    assert_eq!(declarations.len(), 1);
    assert_eq!(
        interner
            .resolve_expect(declarations[0].name().sym())
            .to_string(),
        "f",
        "the direct declaration should remain eligible"
    );
    let span = declarations[0].linear_span();
    assert_eq!(
        &source[span.start().pos()..span.end().pos()],
        "function f(){1}",
        "the nested if declaration must not replace the direct declaration"
    );
}

#[test]
fn annex_b_switch_direct_functions_block_nested_candidates_with_the_same_name() {
    let source = r#"
switch (0) {
    case 0:
        { function f() { 0 } }
        { function before() {} }
        function f() { 1 }
        break;
    case 1:
        { function after() {} }
        function f() { 2 }
}
"#;
    let mut interner = Interner::default();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&Scope::new_global(), &mut interner)
        .expect("sloppy Annex B declarations should parse");

    let declarations = annex_b_function_declarations(&script);
    let names = declarations
        .iter()
        .map(|function| interner.resolve_expect(function.name().sym()).to_string())
        .collect::<Vec<_>>();

    assert_eq!(names, ["before", "f", "after", "f"]);
    assert_eq!(
        declarations
            .iter()
            .map(|function| {
                let span = function.linear_span();
                &source[span.start().pos()..span.end().pos()]
            })
            .collect::<Vec<_>>(),
        [
            "function before() {}",
            "function f() { 1 }",
            "function after() {}",
            "function f() { 2 }",
        ]
    );
    assert!(declarations
        .windows(2)
        .all(|pair| pair[0].linear_span().start() < pair[1].linear_span().start()));
    assert!(!core::ptr::eq(declarations[1], declarations[3]));
}

#[test]
fn parser_accepts_annex_b_html_comments_in_scripts() {
    for source in [
        "<!-- open comment\nconst open_comment = 1;",
        "const close_comment = 1;\n--> close comment",
        "'use strict';\n<!-- strict comment\nconst strict_comment = 1;\n--> close comment",
    ] {
        parse(source, ParseOptions::script()).expect("script HTML comment should parse");
    }
}

#[test]
fn parser_rejects_annex_b_html_comments_in_modules() {
    for source in [
        "<!-- open comment\nexport const open_comment = 1;",
        "export const close_comment = 1;\n--> close comment",
    ] {
        parse(source, ParseOptions::module()).expect_err("module HTML comment should fail");
    }
}
