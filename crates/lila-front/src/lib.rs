use boa_ast::{scope::Scope, Module, Script};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use std::ops::Deref;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

// The closed domain of pre-evaluation rejection codes and the one table that
// classifies boa's static-semantics messages into it. See
// `docs/rust-rewrite/contracts/early-error-taxonomy.md`.
mod early_error_code;

pub use early_error_code::{
    classify_parse_failure, EarlyErrorCode, ParseClassified, NO_EARLY_ERROR_CODE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseGoal {
    Script,
    Module,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    pub goal: ParseGoal,
    pub filename: Option<String>,
}

impl ParseOptions {
    pub fn script() -> Self {
        Self {
            goal: ParseGoal::Script,
            filename: None,
        }
    }

    pub fn module() -> Self {
        Self {
            goal: ParseGoal::Module,
            filename: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnit {
    pub goal: ParseGoal,
    pub filename: Option<String>,
    pub source_text: String,
}

/// A successfully parsed Script compilation unit.
///
/// The syntax tree and its interner are one allocation and cannot be separated
/// or replaced. That relationship matters: every `Sym` in Boa's tree belongs
/// to precisely this interner. Compiler stages can only borrow the pair through
/// [`ParsedScript::with_compiler_session`], so neither half can escape with a
/// longer lifetime than the other.
#[derive(Clone)]
pub struct ParsedScript {
    source: SourceUnit,
    syntax: Rc<ScriptSyntax>,
}

struct ScriptSyntax {
    ast: Script,
    interner: Interner,
}

/// A successfully parsed Module compilation unit.
///
/// Like [`ParsedScript`], this owns the AST and the exact interner that produced
/// it. A module record therefore consumes parsed syntax rather than source text
/// that it could accidentally parse a second time.
#[derive(Clone)]
pub struct ParsedModule {
    source: SourceUnit,
    syntax: Rc<ModuleSyntax>,
}

struct ModuleSyntax {
    ast: Module,
    interner: Interner,
}

/// The closed result of a successful parse.
///
/// Keeping the parse goal in the variant makes passing Script syntax to a
/// Module-only static-semantics operation a type error after the variant is
/// selected. Raw [`SourceUnit`] metadata is deliberately a different type and
/// is not accepted by the IR lowerer.
#[derive(Clone)]
pub enum ParsedSource {
    Script(ParsedScript),
    Module(ParsedModule),
}

impl ParsedSource {
    #[must_use]
    pub const fn source(&self) -> &SourceUnit {
        match self {
            Self::Script(source) => source.source(),
            Self::Module(source) => source.source(),
        }
    }

    #[must_use]
    pub const fn goal(&self) -> ParseGoal {
        match self {
            Self::Script(_) => ParseGoal::Script,
            Self::Module(_) => ParseGoal::Module,
        }
    }

    #[must_use]
    pub const fn as_script(&self) -> Option<&ParsedScript> {
        match self {
            Self::Script(source) => Some(source),
            Self::Module(_) => None,
        }
    }

    #[must_use]
    pub const fn as_module(&self) -> Option<&ParsedModule> {
        match self {
            Self::Module(source) => Some(source),
            Self::Script(_) => None,
        }
    }
}

impl ParsedScript {
    #[must_use]
    pub const fn source(&self) -> &SourceUnit {
        &self.source
    }

    /// Borrows Boa's syntax implementation as one non-escaping compiler
    /// session.
    ///
    /// This is an internal workspace seam, not Lila's syntax or IR contract.
    /// Boa types are intentionally absent from every stored public field and
    /// from all returned Lila IR. Keeping the callback here makes an AST and
    /// the wrong interner impossible to pair and gives a future parser swap one
    /// narrow adapter to replace.
    #[doc(hidden)]
    pub fn with_compiler_session<R>(
        &self,
        consume: impl for<'syntax> FnOnce(&'syntax Script, &'syntax Interner) -> R,
    ) -> R {
        consume(&self.syntax.ast, &self.syntax.interner)
    }
}

impl ParsedModule {
    #[must_use]
    pub const fn source(&self) -> &SourceUnit {
        &self.source
    }

    /// Module counterpart of [`ParsedScript::with_compiler_session`].
    #[doc(hidden)]
    pub fn with_compiler_session<R>(
        &self,
        consume: impl for<'syntax> FnOnce(&'syntax Module, &'syntax Interner) -> R,
    ) -> R {
        consume(&self.syntax.ast, &self.syntax.interner)
    }
}

impl Deref for ParsedScript {
    type Target = SourceUnit;

    fn deref(&self) -> &Self::Target {
        self.source()
    }
}

impl Deref for ParsedModule {
    type Target = SourceUnit;

    fn deref(&self) -> &Self::Target {
        self.source()
    }
}

impl core::fmt::Debug for ParsedScript {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ParsedScript").field(&self.source).finish()
    }
}

impl core::fmt::Debug for ParsedModule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ParsedModule").field(&self.source).finish()
    }
}

impl core::fmt::Debug for ParsedSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Script(source) => source.fmt(f),
            Self::Module(source) => source.fmt(f),
        }
    }
}

impl PartialEq for ParsedScript {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for ParsedScript {}

impl PartialEq for ParsedModule {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for ParsedModule {}

impl PartialEq for ParsedSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Script(left), Self::Script(right)) => left == right,
            (Self::Module(left), Self::Module(right)) => left == right,
            (Self::Script(_), Self::Module(_)) | (Self::Module(_), Self::Script(_)) => false,
        }
    }
}

impl Eq for ParsedSource {}

/// What sort of thing the front end rejected. A **return type**, not a field:
/// it is a function of [`ParseCode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiagnosticKind {
    MalformedJavaScript,
    UnsupportedParserFeature,
}

/// When the rejection was decided. A **return type**, not a field.
///
/// 16.1.4 `ParseScript` and 16.2.1.6.1 `ParseModule` fix the reporting phase per
/// producing operation; clause 17 makes it a property of *where* the rejection
/// comes from, never a free parameter of a call site. Storing it as a field was
/// the opportunity for one condition to be reported under two phases depending
/// on which path found it, and that had already happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiagnosticPhase {
    Parse,
    Early,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

/// Everything [`parse`] can report, as one closed domain.
///
/// The two `P_...` codes are **compiler-gap** codes, not spec rejections, and
/// keeping them out of [`EarlyErrorCode`] is deliberate: an `EarlyErrorCode`
/// must always name a program that ECMAScript rejects. A source boa could not
/// read, or a parse that aborted, is a fact about this front end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCode {
    /// Boa rejected the source and we do not model the wording.
    Malformed,
    /// Boa's parser aborted; the panic was caught in [`parse`].
    UnsupportedParserFeature,
    /// A modelled spec rejection, classified by
    /// [`classify_parse_failure`] — the same table the dependency-module path
    /// uses, so one source cannot report under two codes depending on whether it
    /// was the entry file or an import.
    ///
    /// The payload is [`ParseClassified`], not a bare [`EarlyErrorCode`]: this
    /// variant reports at [`ParseDiagnosticPhase::Early`], and a link-only code
    /// reported there is one condition under two phases from two paths. The
    /// witness makes that `error[E0308]` at the call site rather than a
    /// convention the table's assertion P7 can only state for the table.
    Early(ParseClassified),
}

impl ParseCode {
    /// The single spelling authority for the two `P_...` codes; an early code
    /// delegates to [`EarlyErrorCode::wire_name`], which owns all of
    /// the `E_...` spellings.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Malformed => "P_PARSE_MALFORMED",
            Self::UnsupportedParserFeature => "P_PARSE_UNSUPPORTED",
            Self::Early(code) => code.code().wire_name(),
        }
    }

    #[must_use]
    pub const fn kind(self) -> ParseDiagnosticKind {
        match self {
            Self::Malformed | Self::Early(_) => ParseDiagnosticKind::MalformedJavaScript,
            Self::UnsupportedParserFeature => ParseDiagnosticKind::UnsupportedParserFeature,
        }
    }

    #[must_use]
    pub const fn phase(self) -> ParseDiagnosticPhase {
        match self {
            Self::Early(_) => ParseDiagnosticPhase::Early,
            Self::Malformed | Self::UnsupportedParserFeature => ParseDiagnosticPhase::Parse,
        }
    }

    /// The error the program would have thrown, if the spec says it throws one.
    ///
    /// The one `"SyntaxError"` literal in this crate. 16.1.4 and 16.2.1.6.1
    /// both return "a List of **SyntaxError** objects", and every
    /// `parse`/`resolution` negative in the pinned test262 suite is a
    /// `SyntaxError` — there is no second inhabitant to choose between. It
    /// cannot be `lila_ir::NativeErrorKind` because that type lives in a
    /// crate this one is *below*; closing that requires moving
    /// `NativeErrorKind` down and is another lane's file (ledger L2).
    ///
    /// **`UnsupportedParserFeature` returns `None`, and that is a fix.** It is
    /// the caught-panic case ([`parse`]: "parser aborted while handling
    /// source") — a compiler gap, not a program ECMAScript rejects. Returning
    /// `"SyntaxError"` for it made `compile_negative_error_matches` score a
    /// **pass** for any `parse`/`SyntaxError` negative whose source merely
    /// crashed boa's parser, because `phase()` is already `Parse`. Clause 17:
    /// an implementation "must not treat other kinds of error as early errors".
    /// This is the same shape `lila_ir::IrDiagnosticKind::error_type`
    /// already has, and what `module_parse_failure_diagnostic`'s doc comment
    /// forbids in words.
    #[must_use]
    pub const fn error_type(self) -> Option<&'static str> {
        match self {
            // boa read the source and rejected it: a real syntax error, whether
            // or not the fragment table models its wording.
            Self::Malformed | Self::Early(_) => Some("SyntaxError"),
            Self::UnsupportedParserFeature => None,
        }
    }
}

/// A front-end rejection: one closed code, plus payload.
///
/// `kind`, `phase` and `error_type` used to be independent fields beside
/// `code`; they are now accessors derived from `code`, so no call site can pair
/// them inconsistently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub code: ParseCode,
    pub span: Option<SourceSpan>,
    pub message: String,
}

impl ParseDiagnostic {
    #[must_use]
    pub const fn kind(&self) -> ParseDiagnosticKind {
        self.code.kind()
    }

    #[must_use]
    pub const fn phase(&self) -> ParseDiagnosticPhase {
        self.code.phase()
    }

    #[must_use]
    pub const fn error_type(&self) -> Option<&'static str> {
        self.code.error_type()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    diagnostic: ParseDiagnostic,
    message: String,
}

impl ParseError {
    pub fn malformed(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self::new(ParseCode::Malformed, message, span)
    }

    pub fn unsupported_parser_feature(
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self::new(ParseCode::UnsupportedParserFeature, message, span)
    }

    /// A modelled spec rejection. The `error_type` parameter is gone: passing
    /// `"SyntaxError"` here was never a choice, and passing anything else was
    /// never correct.
    ///
    /// The code parameter is a [`ParseClassified`], not a bare
    /// [`EarlyErrorCode`]: this constructor reports at
    /// [`ParseDiagnosticPhase::Early`], so it must not be able to name a
    /// link-only condition. Obtain one from [`classify_parse_failure`], or —
    /// for a producer that names its code directly — from
    /// [`ParseClassified::from_parse_table`] in a `const` initializer.
    pub fn early_error(
        code: ParseClassified,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self::new(ParseCode::Early(code), message, span)
    }

    fn new(code: ParseCode, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        let message = message.into();
        Self {
            diagnostic: ParseDiagnostic {
                code,
                span,
                message: message.clone(),
            },
            message,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn diagnostic(&self) -> &ParseDiagnostic {
        &self.diagnostic
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(
    source_text: impl Into<String>,
    options: ParseOptions,
) -> Result<ParsedSource, ParseError> {
    let source_text = source_text.into();
    if source_text.contains('\0') {
        return Err(ParseError::malformed(
            "source contains NUL byte, front-end rejects this input",
            first_nul_span(&source_text),
        ));
    }

    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let source = if let Some(filename) = &options.filename {
        Source::from_bytes(source_text.as_bytes()).with_path(std::path::Path::new(filename))
    } else {
        Source::from_bytes(source_text.as_bytes())
    };

    let parsed = panic::catch_unwind(AssertUnwindSafe(|| match options.goal {
        ParseGoal::Script => Parser::new(source)
            .parse_script(&scope, &mut interner)
            .map(ParsedAst::Script),
        ParseGoal::Module => Parser::new(source)
            .parse_module(&scope, &mut interner)
            .map(ParsedAst::Module),
    }));

    let ast = match parsed {
        Ok(Ok(ast)) => ast,
        Ok(Err(err)) => {
            let err = err.to_string();
            let message = format!("parse error: {err}");
            let span = parse_error_span_from_message(&source_text, &err);
            // `&err` is Boa's bare message. Classify before adding presentation
            // context so the taxonomy depends only on the parser's wording.
            return if let Some(code) = classify_parse_failure(&err) {
                Err(ParseError::early_error(code, message, span))
            } else {
                Err(ParseError::malformed(message, span))
            };
        }
        Err(payload) => {
            return Err(ParseError::unsupported_parser_feature(
                format!(
                "parse unsupported by current frontend: parser aborted while handling source ({})",
                parser_abort_message(&payload)
            ),
                None,
            ));
        }
    };

    let source = SourceUnit {
        goal: options.goal,
        filename: options.filename,
        source_text,
    };
    Ok(match ast {
        ParsedAst::Script(ast) => ParsedSource::Script(ParsedScript {
            source,
            syntax: Rc::new(ScriptSyntax { ast, interner }),
        }),
        ParsedAst::Module(ast) => ParsedSource::Module(ParsedModule {
            source,
            syntax: Rc::new(ModuleSyntax { ast, interner }),
        }),
    })
}

enum ParsedAst {
    Script(Script),
    Module(Module),
}

fn first_nul_span(source_text: &str) -> Option<SourceSpan> {
    source_text.find('\0').map(|start| SourceSpan {
        start,
        end: start + 1,
    })
}

fn parse_error_span_from_message(source_text: &str, message: &str) -> Option<SourceSpan> {
    let (_, after_colon) = message.split_once(" at line ")?;
    let (line_text, after_line) = after_colon.split_once(", col ")?;
    let line = line_text.parse::<usize>().ok()?;
    let col_text = after_line
        .split(|ch: char| !ch.is_ascii_digit())
        .next()
        .unwrap_or_default();
    let col = col_text.parse::<usize>().ok()?;

    let start = byte_offset_for_line_col(source_text, line, col)?;
    let width = source_text[start..]
        .chars()
        .next()
        .map(char::len_utf8)
        .unwrap_or_default();
    Some(SourceSpan {
        start,
        end: start + width,
    })
}

fn byte_offset_for_line_col(source_text: &str, line: usize, col: usize) -> Option<usize> {
    let target_line = line.checked_sub(1)?;
    let target_col = col.checked_sub(1)?;
    let mut current_line = 0usize;
    let mut line_start = 0usize;

    for (idx, ch) in source_text.char_indices() {
        if current_line == target_line {
            let mut col_count = 0usize;
            for (relative_idx, _) in source_text[line_start..].char_indices() {
                if col_count == target_col {
                    return Some(line_start + relative_idx);
                }
                col_count += 1;
            }
            return if col_count == target_col {
                Some(source_text.len())
            } else {
                None
            };
        }
        if ch == '\n' {
            current_line += 1;
            line_start = idx + ch.len_utf8();
        }
    }

    if current_line == target_line {
        let mut col_count = 0usize;
        for (relative_idx, _) in source_text[line_start..].char_indices() {
            if col_count == target_col {
                return Some(line_start + relative_idx);
            }
            col_count += 1;
        }
        if col_count == target_col {
            return Some(source_text.len());
        }
    }
    None
}

fn parser_abort_message(payload: &Box<dyn core::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string abort payload".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_ast::operations::{annex_b_function_declarations, annex_b_function_declarations_names};
    use boa_ast::{Declaration, StatementListItem};

    /// The expected `ParseCode` for a modelled rejection.
    ///
    /// Goes through `ParseClassified::from_parse_table`, so a test that names a
    /// link-only code panics here instead of asserting against a `ParseCode`
    /// the product path cannot construct.
    fn early(code: EarlyErrorCode) -> ParseCode {
        ParseCode::Early(ParseClassified::from_parse_table(code))
    }

    #[test]
    fn script_rejects_module_syntax() {
        let err = parse("export const value = 1;", ParseOptions::script())
            .expect_err("script goal should reject export");
        assert!(err.message().contains("parse error"));
        assert_eq!(
            err.diagnostic().kind(),
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Parse);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(err.diagnostic().code, ParseCode::Malformed);
    }

    #[test]
    fn parser_rejects_obvious_function_syntax_error() {
        let err = parse("function {", ParseOptions::script())
            .expect_err("broken function syntax should fail");
        assert!(err.message().contains("parse error"));
        assert_eq!(
            err.diagnostic().kind(),
            ParseDiagnosticKind::MalformedJavaScript
        );
    }

    #[test]
    fn syntax_error_reports_structured_diagnostic_with_byte_span_when_available() {
        let err =
            parse("let x = ;", ParseOptions::script()).expect_err("broken initializer should fail");
        assert_eq!(
            err.diagnostic().kind(),
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Parse);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(err.diagnostic().code, ParseCode::Malformed);
        assert!(
            err.diagnostic().span.is_some(),
            "diagnostic should carry Boa's source position when available: {err:?}"
        );
    }

    #[test]
    fn parser_static_semantics_error_reports_early_phase() {
        let err = parse(
            "({ __proto__: null, __proto__: {} });",
            ParseOptions::script(),
        )
        .expect_err("duplicate __proto__ prototype setters should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ObjectDuplicateProto)
        );
        assert!(
            err.diagnostic().span.is_some(),
            "diagnostic should carry Boa's source position when available: {err:?}"
        );
    }

    #[test]
    fn object_literal_cover_initialized_name_rejections_cover_all_parser_contexts() {
        for (source, options) in [
            ("({ a = 1 });", ParseOptions::script()),
            ("function f() { ({ a = 1 }); }", ParseOptions::script()),
            ("function f() { ({ a = 1 }); }", ParseOptions::module()),
            ("export {}; ({ a = 1 });", ParseOptions::module()),
            (
                "class C { static { ({ a = 1 }); } }",
                ParseOptions::script(),
            ),
            (
                "class C { static { ({ a = 1 }); } }",
                ParseOptions::module(),
            ),
        ] {
            let err = parse(source, options)
                .expect_err("a surviving ObjectLiteral CoverInitializedName should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ObjectLiteralCoverInitializedName),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }

    #[test]
    fn object_literal_cover_initialized_name_reinterpretations_remain_valid() {
        for source in [
            "let target = {}; ({ a = 1 } = target);",
            "let { a = 1 } = {};",
            "const f = ({ a = 1 }) => a;",
            "let a; ({ a });",
            "({ a: 1 });",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "assignment/binding reinterpretations and ordinary properties are valid",
                );
            }
        }
    }

    #[test]
    fn script_top_level_new_target_rejects_direct_and_arrow_carried_uses() {
        for source in ["new.target;", "() => { new.target; };"] {
            let err = parse(source, ParseOptions::script())
                .expect_err("ScriptBody Contains NewTarget should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ScriptTopLevelNewTarget),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }

    #[test]
    fn script_top_level_new_target_function_and_class_boundaries_remain_valid() {
        for source in [
            "function F() { return new.target; }",
            "function F() { return (() => new.target)(); }",
            "class C { constructor() { new.target; } method() { new.target; } static method() { new.target; } static { new.target; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("function and class boundaries make new.target parse-valid");
            }
        }
    }

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
                parse(source, options).expect(
                    "nested and ordinary lexical switch-clause siblings should remain valid",
                );
            }
        }
    }

    #[test]
    fn generator_declaration_parameter_yield_rejects_under_both_goals() {
        for source in [
            "function* g(x = yield) {}",
            "async function* g(x = yield) {}",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("YieldExpression in generator declaration parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::GeneratorDeclarationParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn generator_declaration_parameter_yield_boundaries_remain_valid() {
        for source in [
            "function* g(x = 1) { yield x; }",
            "async function* g(x = 1) { yield x; }",
            "function* outer(x = function*(){ yield 1; }) {}",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("generator bodies and nested generators are Contains boundaries");
            }
        }
    }

    #[test]
    fn generator_expression_parameter_yield_rejects_under_both_goals() {
        for source in [
            "0, function*(x = yield) {};",
            "const g = function* named(x = yield) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("YieldExpression in generator expression parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::GeneratorExpressionParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn generator_expression_parameter_yield_boundaries_remain_valid() {
        for source in [
            "const g = function*(x = 1) { yield x; };",
            "const outer = function*(x = function*(){ yield 1; }) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("generator bodies and nested generators are Contains boundaries");
            }
        }
    }

    #[test]
    fn async_generator_expression_parameter_yield_rejects_under_both_goals() {
        for source in [
            "(async function*(x = yield) {});",
            "const g = async function* named(x = yield) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options).expect_err(
                    "YieldExpression in async generator expression parameters should fail",
                );
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_generator_expression_parameter_yield_boundaries_remain_valid() {
        for source in [
            "const g = async function*(x = 1) { yield x; };",
            "const outer = async function*(x = function*(){ yield 1; }) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("async-generator bodies and nested generators are Contains boundaries");
            }
        }
    }

    #[test]
    fn async_generator_expression_parameter_await_rejects_under_both_goals() {
        for source in [
            "(async function*(x = await 1) {});",
            "const g = async function* named(x = await 1) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options).expect_err(
                    "AwaitExpression in async generator expression parameters should fail",
                );
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncGeneratorExpressionParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_generator_expression_parameter_await_boundaries_remain_valid() {
        for source in [
            "const g = async function*(x = 1) { await 1; yield x; };",
            "const outer = async function*(x = async function(){ await 1; }) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "async-generator bodies and nested async functions are Contains boundaries",
                );
            }
        }
    }

    #[test]
    fn async_declaration_parameter_await_rejects_under_both_goals() {
        for source in [
            "async function f(x = await 1) {}",
            "async function* g(x = await 1) {}",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("AwaitExpression in async declaration parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncDeclarationParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_declaration_parameter_await_boundaries_remain_valid() {
        for source in [
            "async function f(x = 1) { await 1; }",
            "async function* g(x = 1) { await 1; yield x; }",
            "async function outer(x = async function(){ await 1; }) {}",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("async bodies and nested async functions are Contains boundaries");
            }
        }
    }

    #[test]
    fn generator_method_parameter_yield_rejects_under_both_goals() {
        for source in [
            "({ *m(x = yield) {} });",
            "class C { *m(x = yield) {} }",
            "class C { static *m(x = yield) {} }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("YieldExpression in generator method parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::GeneratorMethodParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_generator_method_parameter_yield_rejects_under_both_goals() {
        for source in [
            "({ async *m(x = yield) {} });",
            "class C { async *m(x = yield) {} }",
            "class C { static async *m(x = yield) {} }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("YieldExpression in async-generator method parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncGeneratorMethodParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_generator_method_parameter_await_rejects_under_both_goals() {
        for source in [
            "({ async *m(x = await 1) {} });",
            "class C { async *m(x = await 1) {} }",
            "class C { static async *m(x = await 1) {} }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("AwaitExpression in async-generator method parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncGeneratorMethodParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn generator_method_parameter_contains_boundaries_remain_valid() {
        for source in [
            "({ *m(x = 1) { yield x; } });",
            "class C { *m(x = function*(){ yield 1; }) {} }",
            "({ async *m(x = 1) { yield x; await 1; } });",
            "class C { async *m(x = function*(){ yield 1; }) {} }",
            "class C { static async *m(x = async function(){ await 1; }) {} }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("method bodies and nested functions are Contains boundaries");
            }
        }
    }

    #[test]
    fn arrow_parameter_yield_rejects_under_both_goals() {
        for source in [
            "function* outer() { (x = yield) => x; }",
            "function* outer() { async (x = yield) => x; }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("YieldExpression in arrow parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ArrowParametersContainYield),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn arrow_parameter_await_rejects_under_both_goals() {
        for source in [
            "async function outer() { (x = await 1) => x; }",
            "const f = async (x = await 1) => x;",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("AwaitExpression in arrow parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ArrowParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn arrow_parameter_contains_boundaries_remain_valid() {
        for source in [
            "const f = (x = function*(){ yield 1; }) => x;",
            "function* outer() { async (x = function*(){ yield 1; }) => x; }",
            "const f = async (x = async function(){ await 1; }) => await x;",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("arrow bodies and nested functions are Contains boundaries");
            }
        }

        parse(
            "var yield = 1; const f = async (x = yield) => x;",
            ParseOptions::script(),
        )
        .expect("a sloppy-script yield identifier must not enable Yield grammar globally");
    }

    #[test]
    fn async_function_expression_parameter_await_rejects_under_both_goals() {
        for source in [
            "(async function(x = await 1) {});",
            "const f = async function named(x = await 1) {};",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options).expect_err(
                    "AwaitExpression in async function expression parameters should fail",
                );
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncFunctionExpressionParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_method_parameter_await_rejects_under_both_goals() {
        for source in [
            "({ async m(x = await 1) {} });",
            "class C { async m(x = await 1) {} }",
            "class C { static async m(x = await 1) {} }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("AwaitExpression in async method parameters should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::AsyncMethodParametersContainAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn async_expression_and_method_parameter_await_boundaries_remain_valid() {
        for source in [
            "const f = async function(x = async function(){ await 1; }) { await 1; };",
            "const o = { async m(x = async function(){ await 1; }) { await 1; } };",
            "class C { static async m(x = async function(){ await 1; }) { await 1; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("async bodies and nested async functions are Contains boundaries");
            }
        }
    }

    #[test]
    fn parser_label_static_semantics_errors_report_early_phase() {
        let err = parse("break;", ParseOptions::script())
            .expect_err("unlabelled break outside breakable statement should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().code, early(EarlyErrorCode::IllegalBreak));

        let err = parse("continue missing;", ParseOptions::script())
            .expect_err("labelled continue outside iteration should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::IllegalContinue)
        );

        let err = parse(
            "while (false) { continue missing; }",
            ParseOptions::script(),
        )
        .expect_err("continue to undefined label should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::UndefinedContinueTarget)
        );
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
    fn duplicate_formal_parameter_wordings_report_one_early_error() {
        for source in [
            "function duplicate(a = 0, a) {}",
            "(a, a) => 0",
            "class Duplicate { method(a, a) {} }",
        ] {
            let err = parse(source, ParseOptions::script())
                .expect_err("duplicate formal parameters should fail in this context");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::DuplicateFormalParameter),
                "{source:?}: {err:?}"
            );
        }
    }

    #[test]
    fn duplicate_formal_parameter_fixture_preserves_the_sloppy_script_exception() {
        let source = include_str!("../tests/fixtures/duplicate_formal_parameters.js");
        parse(source, ParseOptions::script())
            .expect("sloppy ordinary function with a simple duplicate list should parse");

        let err = parse(source, ParseOptions::module())
            .expect_err("module code is strict, so duplicate formal parameters should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::DuplicateFormalParameter),
            "{err:?}"
        );
    }

    #[test]
    fn duplicate_catch_parameter_fixture_reports_one_early_error_in_both_goals() {
        let source = include_str!("../tests/fixtures/duplicate_catch_parameter.js");
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("duplicate BoundNames in a catch parameter should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::DuplicateCatchParameter),
                "{err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{err:?}");
        }
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

    #[test]
    fn duplicate_class_constructors_report_one_early_error_for_both_forms_and_goals() {
        for source in [
            "class C { constructor() {} constructor() {} }",
            "let C = class { constructor() {} constructor() {} };",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a class may not contain two ordinary constructors");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::DuplicateClassConstructor),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn duplicate_class_constructor_boundaries_preserve_static_and_computed_methods() {
        for source in [
            r#"class C {
                constructor() {}
                static constructor() {}
                ["constructor"]() {}
            }"#,
            r#"let C = class {
                constructor() {}
                static constructor() {}
                ["constructor"]() {}
            };"#,
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "static and computed constructor methods are not constructor definitions",
                );
            }
        }
    }

    #[test]
    fn class_constructor_generator_methods_report_one_early_error_for_both_forms_and_goals() {
        for source in [
            "class C { *constructor() {} }",
            "let C = class { async *constructor() {} };",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a non-static class constructor may not be a generator method");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassConstructorGeneratorMethod),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_constructor_generator_boundaries_preserve_static_and_computed_methods() {
        for source in [
            r#"class C {
                constructor() {}
                static *constructor() {}
                *["constructor"]() {}
            }"#,
            r#"let C = class {
                constructor() {}
                static async *constructor() {}
                async *["constructor"]() {}
            };"#,
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "static and computed generator methods are not constructor definitions",
                );
            }
        }
    }

    #[test]
    fn remaining_class_constructor_restrictions_report_specific_early_errors_in_both_goals() {
        for (source, code) in [
            (
                "class C { async constructor() {} }",
                EarlyErrorCode::ClassConstructorAsyncMethod,
            ),
            (
                "let C = class { async constructor() {} };",
                EarlyErrorCode::ClassConstructorAsyncMethod,
            ),
            (
                "class C { get constructor() {} }",
                EarlyErrorCode::ClassConstructorGetter,
            ),
            (
                "let C = class { get constructor() {} };",
                EarlyErrorCode::ClassConstructorGetter,
            ),
            (
                "class C { set constructor(value) {} }",
                EarlyErrorCode::ClassConstructorSetter,
            ),
            (
                "let C = class { set constructor(value) {} };",
                EarlyErrorCode::ClassConstructorSetter,
            ),
            (
                "class C { #constructor; }",
                EarlyErrorCode::ClassPrivateConstructorName,
            ),
            (
                "let C = class { static async *#constructor() {} };",
                EarlyErrorCode::ClassPrivateConstructorName,
            ),
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a forbidden class constructor form should fail before evaluation");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(err.diagnostic().code, early(code), "{source:?}: {err:?}");
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn remaining_class_constructor_boundaries_preserve_static_and_computed_public_names() {
        for source in [
            r##"class C {
                constructor() {}
                static async constructor() {}
                async ["constructor"]() {}
                static get constructor() { return 1; }
                static set constructor(value) {}
                get ["constructor"]() { return 1; }
                set ["constructor"](value) {}
                ["#constructor"] = 1;
            }"##,
            r##"let C = class {
                constructor() {}
                static async constructor() {}
                static get constructor() { return 1; }
                static set constructor(value) {}
                ["#constructor"];
            };"##,
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("static and computed public names are not forbidden constructor forms");
            }
        }
    }

    #[test]
    fn duplicate_class_private_names_report_one_early_error_for_both_forms_and_goals() {
        for source in [
            "class C { #x; #x; }",
            "let C = class { #x() {} static #x; };",
            "class C { get #x() {} get #x() {} }",
            "let C = class { set #x(value) {} #x; };",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a class may not declare the same private name twice");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassDuplicatePrivateName),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn duplicate_class_private_name_boundaries_preserve_accessor_pairs_and_nested_classes() {
        for source in [
            "class C { get #x() {} set #x(value) {} }",
            "let C = class { #x() {} method() { return class { #x() {} }; } };",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "a getter/setter pair and a nested class have valid private-name domains",
                );
            }
        }
    }

    #[test]
    fn class_field_literal_name_restrictions_cover_fields_accessors_forms_and_goals() {
        for (source, code) in [
            (
                "class C { constructor = 1; }",
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                r#"let C = class { "constructor"; };"#,
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                "class C { accessor constructor = 1; }",
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                r#"let C = class { accessor "constructor"; };"#,
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                "class C { static constructor = 1; }",
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
            (
                r#"let C = class { static "prototype"; };"#,
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
            (
                "class C { static accessor constructor = 1; }",
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
            (
                r#"let C = class { static accessor "prototype"; };"#,
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options).expect_err(
                    "a forbidden literal class-field name should fail before evaluation",
                );
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(
                    err.diagnostic().error_type(),
                    Some("SyntaxError"),
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().code, early(code), "{source:?}: {err:?}");
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_field_name_restrictions_preserve_computed_names_and_constructor_methods() {
        for source in [
            r#"class C {
                constructor() {}
                prototype;
                accessor "prototype";
                ["constructor"];
                static ["constructor"];
                static ["prototype"] = 1;
            }"#,
            r#"let C = class {
                accessor ["constructor"] = 1;
                static accessor ["constructor"];
                static accessor ["prototype"] = 1;
            };"#,
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "computed names, non-static prototype fields and constructor methods remain valid",
                );
            }
        }
    }

    #[test]
    fn strict_mode_with_statements_report_one_early_error_across_strict_contexts() {
        for source in [
            r#""use strict"; with ({}) {}"#,
            r#"function f() { "use strict"; with ({}) {} }"#,
            "class C { method() { with ({}) {} } }",
        ] {
            let err = parse(source, ParseOptions::script())
                .expect_err("with statements in strict Script code should fail before evaluation");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::StrictModeWithStatement),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }

        let err = parse("with ({}) {}", ParseOptions::module())
            .expect_err("Module code is strict without a directive");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::StrictModeWithStatement),
            "{err:?}"
        );
        assert!(err.diagnostic().span.is_some(), "{err:?}");
    }

    #[test]
    fn sloppy_with_statements_remain_valid_without_a_strict_context() {
        for source in ["with ({}) {}", "function f() { with ({}) {} }"] {
            parse(source, ParseOptions::script())
                .expect("sloppy Script code permits WithStatement");
        }
    }

    #[test]
    fn class_static_block_arguments_rejections_cover_both_forms_and_goals() {
        for source in [
            r"class C { static { (class { [argument\u0073]() {} }); } }",
            "const C = class { static { (() => arguments); } };",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("lexical arguments use in a class static block should fail");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassStaticBlockContainsArguments),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_static_block_arguments_stop_at_function_and_method_boundaries() {
        for source in [
            r#"class C {
                static {
                    function nested(value = arguments) { return arguments; }
                }
            }"#,
            r#"const C = class {
                static {
                    class Nested {
                        method(value = arguments) { return arguments; }
                    }
                }
            };"#,
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("ordinary function and method bodies own their arguments bindings");
            }
        }
    }

    #[test]
    fn class_static_block_await_rejections_cover_both_forms_and_goals() {
        for source in [
            "async function outer() { class C { static { await 0; } } }",
            "async function outer() { const C = class { static { await 0; } }; }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("an AwaitExpression in a class static block should fail");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassStaticBlockContainsAwait),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_static_block_await_stops_at_nested_function_boundaries() {
        let source = r#"class C {
            static {
                async function nested() { await 0; }
                const arrow = async () => await 0;
            }
        }"#;
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("nested async ordinary and arrow functions own their AwaitExpressions");
        }
    }

    #[test]
    fn class_static_block_await_rule_does_not_absorb_declaration_parameter_errors() {
        assert_eq!(
            classify_parse_failure(
                "invalid await usage in generator function parameters at line 1, col 1"
            )
            .map(ParseClassified::code),
            Some(EarlyErrorCode::AsyncDeclarationParametersContainAwait)
        );
    }

    #[test]
    fn class_static_method_prototype_rejections_cover_all_forms_and_goals() {
        for element in [
            "prototype() {}",
            "*prototype() {}",
            "async prototype() {}",
            "async *prototype() {}",
            "get prototype() {}",
            "set prototype(value) {}",
            r#""prototype"() {}"#,
            r"prototyp\u0065() {}",
        ] {
            for source in [
                format!("class C {{ static {element} }}"),
                format!("const C = class {{ static {element} }};"),
            ] {
                for options in [ParseOptions::script(), ParseOptions::module()] {
                    let err = parse(&source, options)
                        .expect_err("a literal public static prototype method should fail");
                    assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                    assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                    assert_eq!(
                        err.diagnostic().code,
                        early(EarlyErrorCode::ClassStaticMethodPrototypeName),
                        "{source:?}: {err:?}"
                    );
                    assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
                }
            }
        }
    }

    #[test]
    fn class_static_method_prototype_computed_private_and_instance_names_remain_valid() {
        for element in [
            "prototype() {}",
            "*prototype() {}",
            "async prototype() {}",
            "async *prototype() {}",
            "get prototype() {}",
            "set prototype(value) {}",
        ] {
            let source = format!("class C {{ {element} }}");
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(&source, options).expect("non-static literal prototype methods are valid");
            }
        }

        for element in [
            r#"["prototype"]() {}"#,
            r#"*["prototype"]() {}"#,
            r#"async ["prototype"]() {}"#,
            r#"async *["prototype"]() {}"#,
            r#"get ["prototype"]() {}"#,
            r#"set ["prototype"](value) {}"#,
            "#prototype() {}",
            "*#prototype() {}",
            "async #prototype() {}",
            "async *#prototype() {}",
            "get #prototype() {}",
            "set #prototype(value) {}",
        ] {
            let source = format!("class C {{ static {element} }}");
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(&source, options)
                    .expect("computed and private static prototype names are parse-valid");
            }
        }
    }

    #[test]
    fn class_field_arguments_rejections_cover_all_field_forms_and_goals() {
        for source in [
            "class C { value = arguments; }",
            "const C = class { static value = arguments; };",
            "class C { #value = arguments; }",
            "const C = class { static #value = arguments; };",
            "class C { accessor value = arguments; }",
            "const C = class { static accessor value = arguments; };",
            "class C { accessor #value = arguments; }",
            "const C = class { static accessor #value = arguments; };",
            "class C { value = () => arguments; }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("lexical arguments use in a class field should fail");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassFieldContainsArguments),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_field_arguments_stop_at_function_and_method_boundaries() {
        for source in [
            "class C { value = function () { return arguments; }; }",
            "const C = class { static value = async function () { return arguments; }; };",
            "class C { #value = function* () { yield arguments; }; }",
            "const C = class { static #value = async function* () { yield arguments; }; };",
            "class C { accessor value = ({ method() { return arguments; } }); }",
            "const C = class { static accessor #value = ({ get value() { return arguments; } }); };",
            "class C { value = ({ arguments: 1, ['arguments']: 2 }); }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("nested functions, methods and property names own no lexical arguments use");
            }
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

    /// The two goals agree on the *same* source, which is the property the two
    /// deleted tables could only promise in a doc comment.
    #[test]
    fn both_goals_classify_one_source_identically() {
        for source in [
            "({ __proto__: null, __proto__: {} });",
            "let x; const x = 1;",
            "break;",
        ] {
            let script = parse(source, ParseOptions::script())
                .expect_err("source is rejected in script goal");
            let module = parse(source, ParseOptions::module())
                .expect_err("source is rejected in module goal");
            assert_eq!(
                script.diagnostic().code,
                module.diagnostic().code,
                "goals disagree on {source:?}"
            );
        }
    }

    #[test]
    fn parser_rejects_unbalanced_delimiters() {
        let err = parse("if (true {", ParseOptions::script())
            .expect_err("unbalanced delimiters should fail");
        assert!(err.message().contains("parse error"));
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Parse);
    }

    #[test]
    fn nul_byte_reports_structured_malformed_diagnostic_with_span() {
        let err = parse("let x = 0;\0", ParseOptions::script())
            .expect_err("NUL byte should be rejected before Boa parsing");
        assert_eq!(
            err.diagnostic().kind(),
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().code, ParseCode::Malformed);
        assert_eq!(
            err.diagnostic().span,
            Some(SourceSpan { start: 10, end: 11 })
        );
    }

    #[test]
    fn parser_accepts_async_arrow_heads_longer_than_thirty_two_tokens() {
        let source = r#"
var ref = async (aFalse = falseCount +=1, aString = stringCount += 1, aNaN = nanCount += 1, a0 = zeroCount += 1, aNull = nullCount += 1, aObj = objCount +=1) => {};
"#;

        parse(source, ParseOptions::script()).expect("long async arrow head should parse");
    }

    #[test]
    fn parser_accepts_simple_module_syntax() {
        parse("export const value = 1;", ParseOptions::module())
            .expect("module goal should accept export");
    }

    #[test]
    fn parser_accepts_sloppy_annex_b_block_functions() {
        for source in [
            "if (true) function then_branch() {} else function else_branch() {}",
            "label: function labelled() {}",
        ] {
            parse(source, ParseOptions::script())
                .expect("sloppy Annex B block function should parse");
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
    fn parser_rejects_labelled_functions_nested_under_if_and_loop() {
        for source in [
            "if (true) label: function nested_if() {}",
            "while (false) label: function nested_loop() {}",
        ] {
            parse(source, ParseOptions::script())
                .expect_err("labelled function nested under a control-flow statement should fail");
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
        let StatementListItem::Declaration(declaration) = &script.statements().statements()[0]
        else {
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
        let StatementListItem::Declaration(declaration) = &script.statements().statements()[0]
        else {
            panic!("script should begin with the enclosing function declaration");
        };
        let Declaration::FunctionDeclaration(outer_function) = declaration.as_ref() else {
            panic!("script should begin with an ordinary function declaration");
        };
        let StatementListItem::Declaration(declaration) = &outer_function.body().statements()[0]
        else {
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

    /// Ledger L1's injection channel, closed.
    ///
    /// boa renders a `TokenKind::StringLiteral` as its raw contents
    /// (`boa_parser/src/lexer/token.rs:313`) and interpolates the found token
    /// into `Error::Unexpected` / `Error::Expected`, so a program can put a
    /// whole fragment set of the one table into the message boa produces for an
    /// ordinary syntax error. `classify_parse_failure` refuses the two
    /// interpolating shapes, so this stays `Malformed` — a syntax error we do
    /// not model — rather than becoming a forged `E_ILLEGAL_BREAK`.
    #[test]
    fn user_source_text_cannot_forge_an_early_error_classification() {
        let err = parse(
            "var x = \"illegal break statement\" \"y\";",
            ParseOptions::script(),
        )
        .expect_err("two adjacent string literals are a syntax error");
        assert_eq!(err.diagnostic().code, ParseCode::Malformed, "{err}");
    }

    /// MC4's call-site half. A code the fragment table cannot produce is not a
    /// `ParseClassified`, so it cannot be reported at
    /// `ParseDiagnosticPhase::Early` by any parse-stage producer.
    #[test]
    fn only_parse_table_codes_are_parse_classified() {
        assert!(ParseClassified::from_early(EarlyErrorCode::ObjectDuplicateProto).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ObjectLiteralCoverInitializedName)
                .is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ScriptTopLevelNewTarget).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ScriptTopLevelUsingDeclaration).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ForInUsingDeclaration).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::SwitchClauseUsingDeclaration).is_some()
        );
        assert!(ParseClassified::from_early(
            EarlyErrorCode::GeneratorDeclarationParametersContainYield,
        )
        .is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncDeclarationParametersContainAwait,
        )
        .is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::GeneratorExpressionParametersContainYield,
        )
        .is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield,
        )
        .is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncGeneratorExpressionParametersContainAwait,
        )
        .is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::GeneratorMethodParametersContainYield,)
                .is_some()
        );
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncGeneratorMethodParametersContainYield,
        )
        .is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncGeneratorMethodParametersContainAwait,
        )
        .is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ArrowParametersContainYield).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ArrowParametersContainAwait).is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::AsyncFunctionExpressionParametersContainAwait,
        )
        .is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::AsyncMethodParametersContainAwait)
                .is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::DuplicateFormalParameter).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::DuplicateCatchParameter).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::CatchBodyDeclarationConflict).is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassConstructorGeneratorMethod).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassConstructorAsyncMethod).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassConstructorGetter).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassConstructorSetter).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassPrivateConstructorName).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassStaticMethodPrototypeName).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassDuplicatePrivateName).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassFieldConstructorName).is_some());
        assert!(ParseClassified::from_early(
            EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
        )
        .is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassStaticBlockContainsArguments)
                .is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassStaticBlockContainsAwait).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassFieldContainsArguments).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::StrictModeWithStatement).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleDuplicateExport).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleMissingExport).is_none());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleUnresolved).is_none());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleTooManyUnits).is_none());
    }
}
