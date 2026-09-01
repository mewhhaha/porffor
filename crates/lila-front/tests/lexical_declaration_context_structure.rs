use std::fs;
use std::path::{Path, PathBuf};

const OWNER_SOURCE: &str =
    include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/lexical.rs");
const DECLARATION_SOURCE: &str =
    include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/mod.rs");
const FOR_STATEMENT_SOURCE: &str = include_str!(
    "../../../vendor/boa_parser-0.21.1/src/parser/statement/iteration/for_statement.rs"
);
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/lexical-declaration-context.md");
const TASK: &str = include_str!("../../../tasks/07-parser-grammar-early-errors.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn parser_source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/boa_parser-0.21.1/src")
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn lexical_declaration_context_is_the_exact_private_no_capability_domain() {
    let context_region = bounded(
        OWNER_SOURCE,
        "pub(crate) enum UsingDeclarationKind {\n    Using,\n    AwaitUsing,\n}\n",
        "pub(in crate::parser) struct LexicalDeclaration",
    );
    let expected_context_region = r#"

/// Parses a lexical declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalDeclaration
enum LexicalDeclarationContext {
    Statement,
    ForHead,
}

"#;
    assert_eq!(
        normalized(context_region),
        normalized(expected_context_region),
        "the exact adjacent context item must remain private and attribute-free"
    );

    let lexical_carrier = bounded(
        OWNER_SOURCE,
        "enum LexicalDeclarationContext {\n    Statement,\n    ForHead,\n}\n",
        "impl LexicalDeclaration",
    );
    let expected_lexical_carrier = r#"

pub(in crate::parser) struct LexicalDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    context: LexicalDeclarationContext,
}

"#;
    assert_eq!(
        normalized(lexical_carrier),
        normalized(expected_lexical_carrier),
        "the exact adjacent lexical carrier item must remain attribute-free"
    );

    let allowed_token_after_using = OWNER_SOURCE
        .split_once("pub(crate) fn allowed_token_after_using")
        .expect("allowed-token-after-using item")
        .1;
    let binding_carrier = allowed_token_after_using
        .split_once("\n}\n")
        .expect("end of allowed-token-after-using item")
        .1
        .split_once("enum BindingDeclarationKind")
        .expect("binding-declaration-kind item after binding carrier")
        .0;
    let expected_binding_carrier = r#"

/// Parses a binding list.
///
/// It will return an error if a `const` declaration is being parsed and there is no
/// initializer.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingList
struct BindingList<'a> {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    declaration_kind: BindingDeclarationKind,
    context: &'a LexicalDeclarationContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
"#;
    assert_eq!(
        normalized(binding_carrier),
        normalized(expected_binding_carrier),
        "the exact adjacent binding carrier item must remain attribute-free"
    );

    assert!(!OWNER_SOURCE.contains("pub enum LexicalDeclarationContext"));
    assert!(!OWNER_SOURCE.contains("pub(crate) enum LexicalDeclarationContext"));
    assert!(!OWNER_SOURCE.contains("pub(super) enum LexicalDeclarationContext"));
    assert!(!OWNER_SOURCE.contains("impl LexicalDeclarationContext"));
    assert!(!OWNER_SOURCE.contains("for LexicalDeclarationContext"));

    let normalized_owner = normalized(OWNER_SOURCE);
    assert_eq!(normalized_owner.matches("forLexicalDeclaration").count(), 1);
    assert_eq!(
        normalized_owner
            .matches("impl<R>TokenParser<R>forLexicalDeclaration")
            .count(),
        1
    );
    assert_eq!(normalized_owner.matches("forBindingList").count(), 1);
    assert_eq!(
        normalized_owner
            .matches("impl<R>TokenParser<R>forBindingList<'_>")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("context: &'a LexicalDeclarationContext,")
            .count(),
        2
    );

    assert_eq!(
        count_in_rust_sources(&parser_source_root(), "LexicalDeclarationContext"),
        13,
        "one declaration, three carrier types, two producers and seven exhaustive arms must own every mention"
    );
}

#[test]
fn exact_statement_and_for_head_producers_borrow_the_context_into_binding_lists() {
    let statement = normalized(bounded(
        OWNER_SOURCE,
        "pub(in crate::parser) fn statement<I, Y, A>(",
        "/// Creates a `LexicalDeclaration` parser for an undifferentiated for-head.",
    ));
    assert_eq!(
        statement
            .matches("LexicalDeclarationContext::Statement")
            .count(),
        1
    );
    assert!(statement.ends_with(
        "Self{allow_in:allow_in.into(),allow_yield:allow_yield.into(),allow_await:allow_await.into(),context:LexicalDeclarationContext::Statement,}}"
    ));

    let for_head = normalized(bounded(
        OWNER_SOURCE,
        "pub(in crate::parser) fn for_head<I, Y, A>(",
        "/// Applies the generic LexicalDeclaration duplicate-name rule.",
    ));
    assert_eq!(
        for_head
            .matches("LexicalDeclarationContext::ForHead")
            .count(),
        1
    );
    assert!(for_head.ends_with(
        "Self{allow_in:allow_in.into(),allow_yield:allow_yield.into(),allow_await:allow_await.into(),context:LexicalDeclarationContext::ForHead,}}"
    ));

    assert_eq!(
        count_in_rust_sources(&parser_source_root(), "LexicalDeclaration::statement("),
        1
    );
    assert_eq!(
        count_in_rust_sources(&parser_source_root(), "LexicalDeclaration::for_head("),
        3
    );

    let statement_route = r#"
            TokenKind::Keyword((Keyword::Const | Keyword::Let, _))
            | TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Await, false))
                if matches!(tok.kind(), TokenKind::Keyword((Keyword::Const | Keyword::Let, _)))
                    || using_decl.is_some() =>
            {
                LexicalDeclaration::statement(true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Into::into)
            }
"#;
    assert_eq!(DECLARATION_SOURCE.matches(statement_route).count(), 1);

    let let_for_head_route = r#"
                TokenKind::Keyword((Keyword::Let, false))
                    if allowed_token_after_let(cursor.peek(1, interner)?) =>
                {
                    Some(ParsedForInitializer::DeferredLexical {
                        declaration: LexicalDeclaration::for_head(
                            false,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                        keyword_position: init_token.span().start(),
                    })
                }
"#;
    let const_for_head_route = r#"
                TokenKind::Keyword((Keyword::Const, _)) => {
                    Some(ParsedForInitializer::DeferredLexical {
                        declaration: LexicalDeclaration::for_head(
                            false,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                        keyword_position: init_token.span().start(),
                    })
                }
"#;
    let resource_for_head_route = r#"
                TokenKind::IdentifierName(_)
                | TokenKind::Keyword((Keyword::Await, false))
                    if using_declaration_kind(cursor, interner, self.allow_await.0, true)?
                        .is_some() =>
                {
                    Some(ParsedForInitializer::DeferredLexical {
                        declaration: LexicalDeclaration::for_head(
                            false,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                        keyword_position: init_token.span().start(),
                    })
                }
"#;
    for route in [
        let_for_head_route,
        const_for_head_route,
        resource_for_head_route,
    ] {
        assert_eq!(FOR_STATEMENT_SOURCE.matches(route).count(), 1);
    }
    assert_eq!(OWNER_SOURCE.matches("BindingList::new(").count(), 4);
    assert_eq!(OWNER_SOURCE.matches("&self.context,").count(), 4);
}

#[test]
fn three_exhaustive_decisions_preserve_statement_and_for_head_semantics() {
    let parse_declaration = normalized(bounded(
        OWNER_SOURCE,
        "impl<R> TokenParser<R> for LexicalDeclaration",
        "/// Check if the given token is valid after the `let` keyword",
    ));
    assert!(parse_declaration.contains(
        "match&self.context{LexicalDeclarationContext::Statement=>{cursor.expect_semicolon(\"lexicaldeclaration\",interner)?;Self::validate_bound_name_let(&lexical_declaration,tok.span().start())?;Self::validate_duplicate_bound_names(&lexical_declaration,tok.span().start())?;}LexicalDeclarationContext::ForHead=>{}}"
    ));

    let binding_parse = normalized(bounded(
        OWNER_SOURCE,
        "impl<R> TokenParser<R> for BindingList<'_>",
        "impl BindingDeclarationKind",
    ));
    let initializer_decision = "matchself.context{LexicalDeclarationContext::Statementifinit_is_some=>decls.push(decl),LexicalDeclarationContext::Statement=>{letnext=cursor.next(interner).or_abrupt()?;returnErr(Error::general(format!(\"Expectedinitializerfor{}declaration\",self.declaration_kind.description()),next.span().start(),));}LexicalDeclarationContext::ForHead=>decls.push(decl),}";
    let missing_terminator_decision = "SemicolonResult::NotFound(_)=>matchself.context{LexicalDeclarationContext::Statement=>{letnext=cursor.next(interner).or_abrupt()?;returnErr(Error::expected([\";\".to_owned(),\"lineterminator\".to_owned()],next.to_string(interner),next.span(),\"lexicaldeclarationbindinglist\",));}LexicalDeclarationContext::ForHead=>break,},";
    let initializer_offset = binding_parse
        .find(initializer_decision)
        .expect("initializer requirement decision");
    let peek_semicolon_offset = binding_parse
        .find("matchcursor.peek_semicolon(interner)?{")
        .expect("semicolon probe");
    let missing_terminator_offset = binding_parse
        .find(missing_terminator_decision)
        .expect("missing-terminator decision");
    assert!(initializer_offset < peek_semicolon_offset);
    assert!(peek_semicolon_offset < missing_terminator_offset);

    assert_eq!(OWNER_SOURCE.matches("match &self.context").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("match self.context").count(), 2);
    for forbidden in [
        "is_for_head",
        "self.context ==",
        "self.context !=",
        "matches!(self.context",
        "Default for LexicalDeclarationContext",
    ] {
        assert!(!OWNER_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_t07_record_the_borrowed_exhaustive_boundary() {
    for marker in [
        "private two-row authority",
        "grammar decisions match both rows directly and exhaustively",
        "changes no grammar",
        "remains red on pre-existing formatting",
        "three touched exhaustive-match regions are clean",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("LexicalDeclarationContext::{Statement, ForHead}"));
    assert!(TASK.contains("This is source-equivalent parser"));
    assert!(TASK.contains("invariant closure"));
    assert!(TASK.contains("Direct vendor-file `rustfmt --check`"));
    assert!(TASK.contains("touched match\nregions are clean"));
}
