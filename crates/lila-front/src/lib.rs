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
            // or not the message-pattern table models its wording.
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

    const SCRIPT_BODY_SUPER_SOURCES: [&str; 19] = [
        "super();",
        "super.value;",
        r#""use strict"; super.value;"#,
        "() => { super(); };",
        "() => super.value;",
        "async () => { super(); };",
        "async () => super.value;",
        "async (value = super()) => value;",
        "async (value = super.value) => value;",
        "() => () => super.value;",
        "(value = super.value) => value;",
        "class C extends super.value {}",
        "(class extends super.value {});",
        "class C { [super.value]() {} }",
        "class C { [super.value]; }",
        "class C { static [super.value]; }",
        "({ [super.value]() {} });",
        "({ get [super.value]() { return 0; } });",
        "({ [super.value]: 0 });",
    ];

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
    fn script_top_level_super_rejects_each_pinned_contains_boundary() {
        for source in SCRIPT_BODY_SUPER_SOURCES {
            let err = parse(source, ParseOptions::script())
                .expect_err("ScriptBody Contains super should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ScriptTopLevelSuper),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("the fixed ScriptBody position must retain a source span");
            assert!(span.start < span.end, "{source:?}: {err:?}");
        }
    }

    #[test]
    fn module_top_level_super_keeps_its_distinct_goal_owner() {
        for source in SCRIPT_BODY_SUPER_SOURCES {
            let err = parse(source, ParseOptions::module())
                .expect_err("ModuleItemList Contains super should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ModuleTopLevelSuper),
                "{source:?}: {err:?}"
            );
            let span = err
                .diagnostic()
                .span
                .expect("the fixed Module position must retain a source span");
            assert!(span.start < span.end, "{source:?}: {err:?}");
        }
    }

    #[test]
    fn method_owned_super_forms_remain_valid_under_both_goals() {
        for source in [
            "class Base {}; class Derived extends Base { constructor() { super(); } }",
            "class Base {}; class Derived extends Base { method() { return super.value; } }",
            "class Base {}; class Derived extends Base { method() { return () => super.value; } }",
            "class Base {}; class Derived extends Base { field = super.value; }",
            "class Base {}; class Derived extends Base { static field = super.value; }",
            "class Base {}; class Derived extends Base { static { void super.value; } }",
            "({ method() { return super.value; } });",
            "({ get value() { return super.value; } });",
            "({ set value(input) { void super.value; } });",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("method-owned super references should remain parse-valid");
            }
        }
    }

    #[test]
    fn base_constructor_direct_super_rejections_cover_both_forms_inputs_arrows_and_goals() {
        for source in [
            "class C { constructor() { super(); } }",
            "(class { constructor() { super(); } });",
            "class C { constructor(value = super()) {} }",
            "class C { constructor() { (() => super())(); } }",
            "class C { constructor(value = (() => super())()) {} }",
            "class C { constructor() { (() => () => super())()(); } }",
            "class C { constructor() { (async () => super())(); } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a base constructor HasDirectSuper should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassBaseConstructorHasDirectSuper),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the class-owned rejection must retain a source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_static_block_super_call_rejections_cover_forms_heritage_arrows_and_goals() {
        for source in [
            "class C { static { super(); } }",
            "(class { static { super(); } });",
            "class B {}; class C extends B { static { super(); } }",
            "class C { static { { super(); } } }",
            "class C { static { (() => super())(); } }",
            "class C { static { ((value = super()) => value)(); } }",
            "class C { static { (() => () => super())()(); } }",
            "class C { static { (async () => super())(); } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a class static block Contains SuperCall should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassStaticBlockContainsSuperCall),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the class-owned rejection must retain a source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_field_initializer_super_call_rejections_cover_every_field_shape_and_goal() {
        for source in [
            "class C { field = super(); }",
            "class C { static field = super(); }",
            "class C { #field = super(); }",
            "class C { static #field = super(); }",
            "class C { accessor field = super(); }",
            "class C { static accessor field = super(); }",
            "(class { field = super(); });",
            "class B {}; class C extends B { field = super(); }",
            "class C { field = () => super(); }",
            "class C { static #field = async () => super(); }",
            "class C { field = () => () => super(); }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a class field initializer Contains SuperCall should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ClassFieldInitializerContainsSuperCall),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the field-owned rejection must retain a source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn function_expression_super_rejections_cover_parameters_bodies_arrows_and_goals() {
        for source in [
            "(function() { super(); });",
            "(function() { void super.value; });",
            "(function(value = super()) {});",
            "(function(value = super.value) {});",
            "(function named() { super(); });",
            "(function named(value = super.value) {});",
            "(function() { (() => super())(); });",
            "(function(value = (() => super.value)()) {});",
            "(function() { (async () => super())(); });",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("FunctionExpression parameters and body may not contain super");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::FunctionExpressionContainsSuper),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the function-expression rejection must retain a source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn function_expression_super_positive_boundaries_remain_valid_under_both_goals() {
        for source in [
            "(function() {});",
            "(function named(value) { return value; });",
            "(function() { class D extends Object { constructor() { super(); } } });",
            "(function() { return \"super() and super.value\"; });",
            "class B {}; class C extends B { method() { return function() {}; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("nested class ownership and ordinary text must remain parse-valid");
            }
        }
    }

    #[test]
    fn function_expression_super_precedence_preserves_prior_callable_checks() {
        for (source, code) in [
            (
                "(function(value = super(), value) {});",
                EarlyErrorCode::DuplicateFormalParameter,
            ),
            (
                "(function(value = super()) { \"use strict\"; });",
                EarlyErrorCode::CallableNonSimpleParametersContainUseStrict,
            ),
            (
                "(function(value = super()) { let value; });",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("an earlier FunctionExpression check should own the rejection");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(err.diagnostic().code, early(code), "{source:?}: {err:?}");
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn function_expression_super_classifier_is_anchored_and_injection_safe() {
        assert_eq!(
            classify_parse_failure("function expression cannot contain super at line 1, col 11")
                .map(ParseClassified::code),
            Some(EarlyErrorCode::FunctionExpressionContainsSuper)
        );
        for message in [
            "invalid super usage at line 1, col 11",
            "invalid super call usage at line 1, col 11",
        ] {
            assert_eq!(classify_parse_failure(message), None);
        }

        let exported_name = "function expression cannot contain super at line";
        let source = format!(
            "const value = 0; export {{ value as \"{exported_name}\" }}; export {{ value as \"{exported_name}\" }};"
        );
        let err = parse(&source, ParseOptions::module())
            .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err:?}"
        );
    }

    #[test]
    fn function_declaration_super_rejections_cover_parameters_bodies_arrows_and_goals() {
        for source in [
            "function invalid() { super(); }",
            "function invalid() { void super.value; }",
            "function invalid(value = super()) {}",
            "function invalid(value = super.value) {}",
            "function invalid() { (() => super())(); }",
            "function invalid(value = (() => super.value)()) {}",
            "function invalid() { (async () => super())(); }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("FunctionDeclaration parameters and body may not contain super");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::FunctionDeclarationContainsSuper),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the function-declaration rejection must retain a source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn anonymous_default_function_declaration_super_rejects_under_module_goal() {
        let source = "export default function(value = super()) {}";
        let err = parse(source, ParseOptions::module())
            .expect_err("an anonymous default FunctionDeclaration may not contain super");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::FunctionDeclarationContainsSuper),
            "{err:?}"
        );
        let span = err
            .diagnostic()
            .span
            .expect("the default FunctionDeclaration rejection must retain a source span");
        assert!(span.start < span.end, "{source:?}: {err:?}");
    }

    #[test]
    fn function_declaration_super_positive_boundaries_remain_valid_under_both_goals() {
        for source in [
            "function valid() {}",
            "function valid(value) { return value; }",
            "function valid() { function nested() {} }",
            "function valid() { class D extends Object { constructor() { super(); } } }",
            "function valid() { return \"super() and super.value\"; }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options)
                    .expect("nested ownership and ordinary text must remain parse-valid");
            }
        }
    }

    #[test]
    fn function_declaration_super_precedence_preserves_prior_callable_checks() {
        for (source, code) in [
            (
                "function invalid(value = super(), value) {}",
                EarlyErrorCode::DuplicateFormalParameter,
            ),
            (
                "function invalid(value = super()) { \"use strict\"; }",
                EarlyErrorCode::CallableNonSimpleParametersContainUseStrict,
            ),
            (
                "function invalid(value = super()) { let value; }",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("an earlier FunctionDeclaration check should own the rejection");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(err.diagnostic().code, early(code), "{source:?}: {err:?}");
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn function_declaration_super_classifier_is_anchored_and_injection_safe() {
        for (message, expected) in [
            (
                "function declaration cannot contain super at line 1, col 12",
                Some(EarlyErrorCode::FunctionDeclarationContainsSuper),
            ),
            (
                "function expression cannot contain super at line 1, col 11",
                Some(EarlyErrorCode::FunctionExpressionContainsSuper),
            ),
            ("invalid super usage at line 1, col 12", None),
            ("invalid super call usage at line 1, col 12", None),
        ] {
            assert_eq!(
                classify_parse_failure(message).map(ParseClassified::code),
                expected
            );
        }

        let exported_name = "function declaration cannot contain super at line";
        let source = format!(
            "const value = 0; export {{ value as \"{exported_name}\" }}; export {{ value as \"{exported_name}\" }};"
        );
        let err = parse(&source, ParseOptions::module())
            .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err:?}"
        );
    }

    #[test]
    fn class_super_call_positive_boundaries_remain_valid_under_both_goals() {
        for source in [
            "class C {}",
            "class B {}; class C extends B { constructor() { super(); } }",
            "class C extends null { constructor() { super(); } }",
            "class C { constructor() { void super.value; } }",
            "class C { constructor() { class D extends C { constructor() { super(); } } } }",
            "class C { static {} }",
            "class B {}; class C extends B { static { void super.value; } }",
            "class C { static { class D extends C { constructor() { super(); } } } }",
            "class C { static { const text = \"super()\"; } }",
            "class B {}; class C extends B { field = super.value; static other = super.value; }",
            "class C { field = class D extends C { constructor() { super(); } }; }",
            "class C { field = \"super()\"; }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "heritage, SuperProperty and nested class ownership must stay distinct",
                );
            }
        }
    }

    #[test]
    fn class_super_call_precedence_preserves_pinned_boa_check_order() {
        for (source, code) in [
            (
                "class C { constructor() { super(); } static { super(); } }",
                EarlyErrorCode::ClassStaticBlockContainsSuperCall,
            ),
            (
                "class C { static { super(); } constructor() { super(); } }",
                EarlyErrorCode::ClassStaticBlockContainsSuperCall,
            ),
            (
                "class C { static { arguments; super(); } }",
                EarlyErrorCode::ClassStaticBlockContainsArguments,
            ),
            (
                "class C { static { super(); await 0; } }",
                EarlyErrorCode::ClassStaticBlockContainsSuperCall,
            ),
            (
                "class C { constructor() { super(); } constructor() {} }",
                EarlyErrorCode::DuplicateClassConstructor,
            ),
            (
                "class C { field = super(); constructor() { super(); } }",
                EarlyErrorCode::ClassFieldInitializerContainsSuperCall,
            ),
            (
                "class C { field = arguments && super(); }",
                EarlyErrorCode::ClassFieldContainsArguments,
            ),
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("the earlier pinned class check should own the rejection");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(err.diagnostic().code, early(code), "{source:?}: {err:?}");
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn class_super_call_classifier_rows_are_distinct_and_injection_safe() {
        for (message, code) in [
            (
                "base class constructor cannot contain direct super call at line 2, col 1",
                EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
            ),
            (
                "class static block cannot contain super call at line 2, col 1",
                EarlyErrorCode::ClassStaticBlockContainsSuperCall,
            ),
            (
                "class field initializer cannot contain super call at line 2, col 1",
                EarlyErrorCode::ClassFieldInitializerContainsSuperCall,
            ),
        ] {
            assert_eq!(
                classify_parse_failure(message).map(ParseClassified::code),
                Some(code)
            );
        }
        for message in [
            "invalid super usage at line 1, col 2",
            "invalid super call usage at line 1, col 1",
        ] {
            assert_eq!(classify_parse_failure(message), None);
        }

        for exported_name in [
            "base class constructor cannot contain direct super call at line",
            "class static block cannot contain super call at line",
            "class field initializer cannot contain super call at line",
        ] {
            let source = format!(
                "const value = 0; export {{ value as \"{exported_name}\" }}; export {{ value as \"{exported_name}\" }};"
            );
            let err = parse(&source, ParseOptions::module())
                .expect_err("the user-chosen exported name is duplicated");
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ModuleDuplicateExport),
                "{err:?}"
            );
        }
    }

    #[test]
    fn adjacent_invalid_super_producers_remain_unclassified_by_this_lane() {
        for source in [
            "function* f() { super.value; }",
            "async function f() { super.value; }",
            "async function* f() { super.value; }",
            "(function*() { super.value; });",
            "(async function() { super.value; });",
            "(async function*() { super.value; });",
            "class C { method() { super(); } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("the adjacent callable or class condition should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Parse,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(err.diagnostic().code, ParseCode::Malformed);
                assert_ne!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ScriptTopLevelSuper),
                    "{source:?}: {err:?}"
                );
                assert_ne!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ModuleTopLevelSuper),
                    "{source:?}: {err:?}"
                );
            }
        }
    }

    #[test]
    fn script_top_level_super_precedes_new_target_and_resists_message_injection() {
        let err = parse("super.value; new.target;", ParseOptions::script())
            .expect_err("the ScriptBody super check should run before NewTarget");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ScriptTopLevelSuper),
            "{err:?}"
        );

        for message in [
            "invalid super usage at line 1, col 2",
            "invalid super usage at line 1, col 10",
        ] {
            assert_eq!(
                classify_parse_failure(message),
                None,
                "a distinct source position must not acquire the ScriptBody code"
            );
        }

        let err = parse(
            concat!(
                "const value = 0;\n",
                "export { value as \"invalid super usage at line 1, col 1\" };\n",
                "export { value as \"invalid super usage at line 1, col 1\" };",
            ),
            ParseOptions::module(),
        )
        .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err:?}"
        );
    }

    #[test]
    fn known_script_and_class_super_producers_stay_structurally_reviewed() {
        fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            source
                .split_once(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"))
                .1
                .split_once(end)
                .unwrap_or_else(|| panic!("missing end marker after: {start}"))
                .0
        }

        fn count_in_rust_sources(root: &std::path::Path, fragment: &str) -> usize {
            let entries = std::fs::read_dir(root)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
            let mut count = 0;
            for entry in entries {
                let path = entry.expect("failed to read vendored Boa entry").path();
                if path.is_dir() {
                    count += count_in_rust_sources(&path, fragment);
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                        panic!("failed to read {}: {error}", path.display())
                    });
                    count += source.matches(fragment).count();
                }
            }
            count
        }

        fn collect_rust_sources(
            root: &std::path::Path,
            sources: &mut Vec<(std::path::PathBuf, String)>,
        ) {
            let entries = std::fs::read_dir(root)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
            for entry in entries {
                let path = entry.expect("failed to read workspace source entry").path();
                if path.is_dir() {
                    collect_rust_sources(&path, sources);
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                        panic!("failed to read {}: {error}", path.display())
                    });
                    sources.push((path, source));
                }
            }
        }

        let compact = |source: &str| {
            source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        };

        const RAW_MESSAGE: &str = "invalid super usage";
        const METHOD_MESSAGE: &str = "invalid super call usage";
        const BASE_CONSTRUCTOR_MESSAGE: &str =
            "base class constructor cannot contain direct super call";
        const STATIC_BLOCK_MESSAGE: &str = "class static block cannot contain super call";
        const FIELD_INITIALIZER_MESSAGE: &str = "class field initializer cannot contain super call";
        const FUNCTION_EXPRESSION_MESSAGE: &str = "function expression cannot contain super";
        const FUNCTION_DECLARATION_MESSAGE: &str = "function declaration cannot contain super";
        const MODULE_MESSAGE: &str = "module cannot contain `super` on the top-level";
        const SCRIPT_BRANCH: &str = r#"if contains(&body, ContainsSymbol::Super) {
                return Err(Error::general("invalid super usage", Position::new(1, 1)));
            }"#;
        const MODULE_BRANCH: &str = r#"if contains(&module, ContainsSymbol::Super) {
            return Err(Error::general(
                "module cannot contain `super` on the top-level",
                Position::new(1, 1),
            ));
        }"#;
        const PARAMETER_POSITION: &str = r#""invalid super usage".into(),
                params_start_position,"#;
        const FUNCTION_EXPRESSION_SUPER_BRANCH: &str = r#"if contains(&function, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "function expression cannot contain super".into(),
                params_start_position,
            )));
        }"#;
        const CALLABLE_DECLARATION_DEFAULT_SUPER_MESSAGE: &str = r#"fn contains_super_error_message(&self) -> &'static str {
        "invalid super usage"
    }"#;
        const CALLABLE_DECLARATION_SUPER_BRANCH: &str = r#"if contains(&body, ContainsSymbol::Super) || contains(&params, ContainsSymbol::Super) {
        return Err(Error::lex(LexError::Syntax(
            c.contains_super_error_message().into(),
            params_start_position,
        )));
    }"#;
        const FUNCTION_DECLARATION_SUPER_MESSAGE_METHOD: &str = r#"fn contains_super_error_message(&self) -> &'static str {
        "function declaration cannot contain super"
    }"#;
        const BASE_CONSTRUCTOR_BRANCH: &str = r#"if super_ref.is_none()
                && let Some(constructor) = &constructor
                && contains(constructor, ContainsSymbol::SuperCall)
            {
                return Err(Error::lex(LexError::Syntax(
                    "base class constructor cannot contain direct super call".into(),
                    body_start,"#;
        const FIELD_INITIALIZER_POSITION: &str = r#""class field initializer cannot contain super call".into(),
                            position,"#;
        const FIELD_NODE_SUPER_BRANCH: &str = r#"if let Some(node) = field.initializer()
                        && contains(node, ContainsSymbol::SuperCall)
                    {
                        return Err(Error::lex(LexError::Syntax(
                            "class field initializer cannot contain super call".into(),
                            position,
                        )));
                    }"#;
        const FIELD_SUPER_BRANCH: &str = r#"if let Some(field) = field.initializer()
                        && contains(field, ContainsSymbol::SuperCall)
                    {
                        return Err(Error::lex(LexError::Syntax(
                            "class field initializer cannot contain super call".into(),
                            position,
                        )));
                    }"#;
        const CLASS_METHOD_NAME_SUPER_BRANCH: &str = r#"if let ClassElementName::PropertyName(name) = m.name()
                        && contains(name, ContainsSymbol::SuperCall)
                    {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid super call usage".into(),
                            position,
                        )));
                    }"#;
        const CLASS_METHOD_BODY_SUPER_BRANCH: &str = r#"if contains(m.parameters(), ContainsSymbol::SuperCall)
                        || contains(m.body(), ContainsSymbol::SuperCall)
                    {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid super call usage".into(),
                            position,
                        )));
                    }"#;
        const OBJECT_METHOD_POSITION_SUPER_BRANCH: &str = r#"if has_direct_super_new(&params, &body) {
                    return Err(Error::lex(LexError::Syntax(
                        "invalid super call usage".into(),
                        position,
                    )));
                }"#;
        const OBJECT_GETTER_SUPER_BRANCH: &str = r#"if has_direct_super_new(&FormalParameterList::default(), &body) {
                    return Err(Error::lex(LexError::Syntax(
                        "invalid super call usage".into(),
                        position,
                    )));
                }"#;
        const OBJECT_METHOD_PARAMS_SUPER_BRANCH: &str = r#"if has_direct_super_new(&params, &body) {
                    return Err(Error::lex(LexError::Syntax(
                        "invalid super call usage".into(),
                        params_start_position,
                    )));
                }"#;
        const OBJECT_METHOD_BODY_SUPER_BRANCH: &str = r#"if has_direct_super_new(&params, &body) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super call usage".into(),
                body_start,
            )));
        }"#;
        const STATIC_BLOCK_BRANCH: &str = r#"if contains(&statement_list, ContainsSymbol::SuperCall) {
                        return Err(Error::general(
                            "class static block cannot contain super call",
                            position,"#;
        const PARSER_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/mod.rs");
        const HOISTABLE_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs"
        );
        const FUNCTION_DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/function_decl/mod.rs"
        );
        const GENERATOR_DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/generator_decl/mod.rs"
        );
        const ASYNC_FUNCTION_DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/async_function_decl/mod.rs"
        );
        const ASYNC_GENERATOR_DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/async_generator_decl/mod.rs"
        );
        const FUNCTION_EXPRESSION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/function_expression/mod.rs"
        );
        const GENERATOR_EXPRESSION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/generator_expression/mod.rs"
        );
        const ASYNC_FUNCTION_EXPRESSION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/async_function_expression/mod.rs"
        );
        const ASYNC_GENERATOR_EXPRESSION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/async_generator_expression/mod.rs"
        );
        const OBJECT_INITIALIZER_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/object_initializer/mod.rs"
        );
        const CLASS_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs"
        );
        const FRONT_SOURCE: &str = include_str!("lib.rs");

        let boa_package_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/boa_parser-0.21.1");
        assert_eq!(count_in_rust_sources(&boa_package_root, RAW_MESSAGE), 5);
        assert_eq!(
            count_in_rust_sources(&boa_package_root, FUNCTION_EXPRESSION_MESSAGE),
            1
        );
        assert_eq!(
            count_in_rust_sources(&boa_package_root, FUNCTION_DECLARATION_MESSAGE),
            1
        );
        assert_eq!(count_in_rust_sources(&boa_package_root, METHOD_MESSAGE), 11);
        assert_eq!(
            count_in_rust_sources(&boa_package_root, BASE_CONSTRUCTOR_MESSAGE),
            1
        );
        assert_eq!(
            count_in_rust_sources(&boa_package_root, STATIC_BLOCK_MESSAGE),
            1
        );
        assert_eq!(
            count_in_rust_sources(&boa_package_root, FIELD_INITIALIZER_MESSAGE),
            4
        );
        assert_eq!(count_in_rust_sources(&boa_package_root, MODULE_MESSAGE), 1);

        let script_start = PARSER_SOURCE
            .find("impl<R> TokenParser<R> for ScriptBody")
            .expect("the ScriptBody parser remains present");
        let module_start = PARSER_SOURCE
            .find("/// Parses a full module.")
            .expect("the Module parser remains after ScriptBody");
        let script_body = &PARSER_SOURCE[script_start..module_start];
        assert_eq!(script_body.matches(RAW_MESSAGE).count(), 1);
        assert_eq!(script_body.matches(SCRIPT_BRANCH).count(), 1);
        assert_eq!(script_body.matches("if !self.direct_eval {").count(), 1);

        let super_check = script_body
            .find(SCRIPT_BRANCH)
            .expect("the fixed ScriptBody super branch remains present");
        let direct_eval_guard = script_body
            .find("if !self.direct_eval {")
            .expect("the direct-eval exclusion remains present");
        let direct_eval_end = script_body[direct_eval_guard..]
            .find("\n        }\n\n        if let Err(error) = check_labels(&body)")
            .map(|offset| direct_eval_guard + offset)
            .expect("the direct-eval guard must still enclose the Script checks");
        assert_eq!(
            script_body[direct_eval_guard..direct_eval_end]
                .matches(SCRIPT_BRANCH)
                .count(),
            1,
            "the Script super producer must remain inside the direct-eval exclusion"
        );
        let new_target_check = script_body
            .find("if contains(&body, ContainsSymbol::NewTarget)")
            .expect("the adjacent NewTarget check remains present");
        let private_name_check = script_body
            .find("if !all_private_identifiers_valid(&body, Vec::new())")
            .expect("the adjacent private-name check remains present");
        let label_check = script_body
            .find("if let Err(error) = check_labels(&body)")
            .expect("the adjacent label check remains present");
        let cover_check = script_body
            .find("if contains_invalid_object_literal(&body)")
            .expect("the adjacent cover-grammar check remains present");
        assert!(
            direct_eval_guard < super_check
                && super_check < new_target_check
                && new_target_check < private_name_check
                && private_name_check < label_check
                && label_check < cover_check,
            "the ScriptBody semantic-check order or direct-eval boundary changed"
        );

        let module_body = &PARSER_SOURCE[module_start..];
        assert_eq!(module_body.matches(MODULE_MESSAGE).count(), 1);
        assert_eq!(module_body.matches(MODULE_BRANCH).count(), 1);
        assert!(
            module_body
                .find(MODULE_BRANCH)
                .expect("the Module super branch remains present")
                < module_body
                    .find("if contains(&module, ContainsSymbol::NewTarget)")
                    .expect("the Module NewTarget branch remains present"),
            "Module must retain its separate super owner and ordering"
        );

        for source in [
            GENERATOR_EXPRESSION_SOURCE,
            ASYNC_FUNCTION_EXPRESSION_SOURCE,
            ASYNC_GENERATOR_EXPRESSION_SOURCE,
        ] {
            assert_eq!(source.matches(RAW_MESSAGE).count(), 1);
            let compact_source = compact(source);
            assert_eq!(
                compact_source
                    .matches(compact(PARAMETER_POSITION).as_str())
                    .count(),
                1,
                "each callable owner must retain its parameter-start position"
            );
            assert_eq!(
                source
                    .matches(r#"Error::general("invalid super usage", Position::new(1, 1))"#,)
                    .count(),
                0
            );
        }
        assert_eq!(HOISTABLE_SOURCE.matches(RAW_MESSAGE).count(), 1);
        assert_eq!(
            HOISTABLE_SOURCE
                .matches(FUNCTION_DECLARATION_MESSAGE)
                .count(),
            0
        );
        assert_eq!(
            compact(HOISTABLE_SOURCE)
                .matches(compact(CALLABLE_DECLARATION_DEFAULT_SUPER_MESSAGE).as_str())
                .count(),
            1
        );
        assert_eq!(
            compact(HOISTABLE_SOURCE)
                .matches(compact(CALLABLE_DECLARATION_SUPER_BRANCH).as_str())
                .count(),
            1,
            "the shared declaration predicate must select a production-owned message and retain the parameter-start position"
        );
        let declaration_lexical_name_check = HOISTABLE_SOURCE
            .find("name_in_lexically_declared_names(")
            .expect("the declaration parameter/body lexical-name check remains present");
        let declaration_super_check = HOISTABLE_SOURCE
            .find(CALLABLE_DECLARATION_SUPER_BRANCH)
            .expect("the shared declaration Contains Super branch remains present");
        let declaration_yield_check = HOISTABLE_SOURCE
            .find("if c.parameters_yield_is_early_error()")
            .expect("the adjacent generator parameter check remains present");
        assert!(
            declaration_lexical_name_check < declaration_super_check
                && declaration_super_check < declaration_yield_check,
            "the shared callable-declaration early-error order changed"
        );

        assert_eq!(FUNCTION_DECLARATION_SOURCE.matches(RAW_MESSAGE).count(), 0);
        assert_eq!(
            FUNCTION_DECLARATION_SOURCE
                .matches(FUNCTION_DECLARATION_MESSAGE)
                .count(),
            1
        );
        assert_eq!(
            compact(FUNCTION_DECLARATION_SOURCE)
                .matches(compact(FUNCTION_DECLARATION_SUPER_MESSAGE_METHOD).as_str())
                .count(),
            1,
            "only ordinary FunctionDeclaration may override the shared Contains Super diagnostic"
        );
        assert_eq!(
            FUNCTION_DECLARATION_SOURCE
                .matches("parse_callable_declaration(&self")
                .count(),
            1
        );
        for source in [
            GENERATOR_DECLARATION_SOURCE,
            ASYNC_FUNCTION_DECLARATION_SOURCE,
            ASYNC_GENERATOR_DECLARATION_SOURCE,
        ] {
            assert_eq!(source.matches(RAW_MESSAGE).count(), 0);
            assert_eq!(source.matches(FUNCTION_DECLARATION_MESSAGE).count(), 0);
            assert_eq!(source.matches("contains_super_error_message").count(), 0);
            assert_eq!(
                source.matches("parse_callable_declaration(&self").count(),
                1
            );
        }
        assert_eq!(FUNCTION_EXPRESSION_SOURCE.matches(RAW_MESSAGE).count(), 0);
        assert_eq!(
            FUNCTION_EXPRESSION_SOURCE
                .matches(FUNCTION_EXPRESSION_MESSAGE)
                .count(),
            1
        );
        assert_eq!(
            compact(FUNCTION_EXPRESSION_SOURCE)
                .matches(compact(FUNCTION_EXPRESSION_SUPER_BRANCH).as_str())
                .count(),
            1,
            "the unique FunctionExpression message must remain on the completed-node Contains Super branch"
        );

        let compact_class = compact(CLASS_SOURCE);
        assert_eq!(CLASS_SOURCE.matches(RAW_MESSAGE).count(), 0);
        assert_eq!(CLASS_SOURCE.matches(METHOD_MESSAGE).count(), 2);
        assert_eq!(OBJECT_INITIALIZER_SOURCE.matches(METHOD_MESSAGE).count(), 9);
        assert_eq!(CLASS_SOURCE.matches(BASE_CONSTRUCTOR_MESSAGE).count(), 1);
        assert_eq!(CLASS_SOURCE.matches(STATIC_BLOCK_MESSAGE).count(), 1);
        assert_eq!(CLASS_SOURCE.matches(FIELD_INITIALIZER_MESSAGE).count(), 4);
        assert_eq!(
            compact_class
                .matches(compact(BASE_CONSTRUCTOR_BRANCH).as_str())
                .count(),
            1
        );
        assert_eq!(
            compact_class
                .matches(compact(FIELD_INITIALIZER_POSITION).as_str())
                .count(),
            4
        );

        let class_elements = bounded(
            CLASS_SOURCE,
            "match &element {",
            "            elements.push(element);",
        );
        let private_field = bounded(
            class_elements,
            "function::ClassElement::PrivateFieldDefinition(field) => {",
            "function::ClassElement::PrivateStaticFieldDefinition(field) => {",
        );
        let private_static_field = bounded(
            class_elements,
            "function::ClassElement::PrivateStaticFieldDefinition(field) => {",
            "function::ClassElement::FieldDefinition(field)",
        );
        let grouped_public_fields = bounded(
            class_elements,
            "function::ClassElement::FieldDefinition(field)",
            "function::ClassElement::StaticAccessorFieldDefinition(field) => {",
        );
        let static_accessor_field = bounded(
            class_elements,
            "function::ClassElement::StaticAccessorFieldDefinition(field) => {",
            "function::ClassElement::StaticBlock(_) => {}",
        );
        let compact_node_field_branch = compact(FIELD_NODE_SUPER_BRANCH);
        let compact_field_branch = compact(FIELD_SUPER_BRANCH);
        for (owner, branch) in [
            (private_field, compact_node_field_branch.as_str()),
            (private_static_field, compact_node_field_branch.as_str()),
            (grouped_public_fields, compact_field_branch.as_str()),
            (static_accessor_field, compact_field_branch.as_str()),
        ] {
            assert_eq!(owner.matches(FIELD_INITIALIZER_MESSAGE).count(), 1);
            assert_eq!(compact(owner).matches(branch).count(), 1);
            assert_eq!(owner.matches(RAW_MESSAGE).count(), 0);
            assert_eq!(owner.matches(METHOD_MESSAGE).count(), 0);
        }

        assert_eq!(
            compact_class
                .matches(compact(CLASS_METHOD_NAME_SUPER_BRANCH).as_str())
                .count(),
            1
        );
        assert_eq!(
            compact_class
                .matches(compact(CLASS_METHOD_BODY_SUPER_BRANCH).as_str())
                .count(),
            1
        );
        let compact_object_initializer = compact(OBJECT_INITIALIZER_SOURCE);
        for (branch, count) in [
            (OBJECT_METHOD_POSITION_SUPER_BRANCH, 3),
            (OBJECT_GETTER_SUPER_BRANCH, 1),
            (OBJECT_METHOD_PARAMS_SUPER_BRANCH, 2),
            (OBJECT_METHOD_BODY_SUPER_BRANCH, 3),
        ] {
            assert_eq!(
                compact_object_initializer
                    .matches(compact(branch).as_str())
                    .count(),
                count
            );
        }
        assert_eq!(
            CLASS_SOURCE.matches(METHOD_MESSAGE).count()
                + OBJECT_INITIALIZER_SOURCE.matches(METHOD_MESSAGE).count(),
            count_in_rust_sources(&boa_package_root, METHOD_MESSAGE),
            "all eleven method-message producers must remain in the two reviewed parser owners"
        );
        assert_eq!(
            compact_class
                .matches(compact(STATIC_BLOCK_BRANCH).as_str())
                .count(),
            1
        );
        assert_eq!(
            CLASS_SOURCE
                .matches(r#"Error::general("invalid super usage", Position::new(1, 1))"#)
                .count(),
            0
        );

        let class_tail_start = CLASS_SOURCE
            .find("impl<R> TokenParser<R> for ClassTail")
            .expect("the ClassTail parser remains present");
        let class_heritage_start = CLASS_SOURCE[class_tail_start..]
            .find("/// `ClassHeritage` parsing.")
            .map(|offset| class_tail_start + offset)
            .expect("ClassHeritage remains after ClassTail");
        let class_tail = &CLASS_SOURCE[class_tail_start..class_heritage_start];
        let class_body_parse = class_tail
            .find("ClassBody::new(self.name, self.allow_yield, self.allow_await)")
            .expect("ClassTail must parse one complete ClassBody");
        let base_constructor_check = class_tail
            .find("if super_ref.is_none()")
            .expect("the absent-heritage constructor check remains present");
        assert!(class_body_parse < base_constructor_check);
        assert_eq!(class_tail.matches(BASE_CONSTRUCTOR_MESSAGE).count(), 1);
        assert_eq!(class_tail.matches("body_start").count(), 2);

        let static_block_start = CLASS_SOURCE
            .find("TokenKind::Punctuator(Punctuator::OpenBlock) if r#static => {")
            .expect("the class static-block parser remains present");
        let static_block_end = CLASS_SOURCE[static_block_start..]
            .find("function::ClassElement::StaticBlock(")
            .map(|offset| static_block_start + offset)
            .expect("the class static-block AST construction remains after validation");
        let static_block = &CLASS_SOURCE[static_block_start..static_block_end];
        let arguments_check = static_block
            .find("if contains_arguments(&statement_list)")
            .expect("ContainsArguments remains the first adjacent static-block check");
        let super_call_check = static_block
            .find("if contains(&statement_list, ContainsSymbol::SuperCall)")
            .expect("the static-block Contains SuperCall check remains present");
        let await_check = static_block
            .find("if contains(&statement_list, ContainsSymbol::AwaitExpression)")
            .expect("the static-block Contains AwaitExpression check remains present");
        let object_literal_check = static_block
            .find("if contains_invalid_object_literal(&statement_list)")
            .expect("the static-block cover-grammar check remains present");
        assert!(
            arguments_check < super_call_check
                && super_call_check < await_check
                && await_check < object_literal_check,
            "the class static-block early-error ordering changed"
        );
        assert_eq!(static_block.matches(STATIC_BLOCK_MESSAGE).count(), 1);

        let front_product = FRONT_SOURCE
            .split_once("#[cfg(test)]\nmod tests {")
            .map(|(product, _)| product)
            .expect("the product/test boundary remains explicit");
        let parser_new = ["Parser", "::new("].concat();
        let parse_script = [".parse_", "script("].concat();
        let parse_module = [".parse_", "module("].concat();
        let classifier = ["classify_parse_", "failure(&err)"].concat();
        assert_eq!(front_product.matches(parser_new.as_str()).count(), 2);
        assert_eq!(front_product.matches(parse_script.as_str()).count(), 1);
        assert_eq!(front_product.matches(parse_module.as_str()).count(), 1);
        assert_eq!(front_product.matches(classifier.as_str()).count(), 1);

        let front_source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        assert_eq!(
            count_in_rust_sources(&front_source_root, parser_new.as_str()),
            8,
            "every direct Boa parser construction, including test-only probes, requires review"
        );
        assert_eq!(
            count_in_rust_sources(&front_source_root, parse_script.as_str()),
            7,
            "every direct Script parse, including test-only probes, requires review"
        );
        assert_eq!(
            count_in_rust_sources(&front_source_root, parse_module.as_str()),
            1,
            "only the product Module route may invoke Boa directly"
        );

        fn product_dependency_section(header: &str) -> bool {
            header == "[dependencies]"
                || header.starts_with("[dependencies.")
                || (header.starts_with("[target.")
                    && (header.contains(".dependencies]") || header.contains(".dependencies.")))
        }

        fn has_product_dependency(manifest: &str, dependency: &str) -> bool {
            let mut in_product_dependencies = false;
            for line in manifest.lines() {
                let line = line.trim();
                if line.starts_with('[') && line.ends_with(']') {
                    in_product_dependencies = product_dependency_section(line);
                    if in_product_dependencies && line.contains(dependency) {
                        return true;
                    }
                } else if in_product_dependencies && line.contains(dependency) {
                    return true;
                }
            }
            false
        }

        for manifest in [
            "[dependencies]\nboa_parser = \"0.21.1\"",
            "[dependencies.boa_parser]\nversion = \"0.21.1\"",
            "[target.'cfg(unix)'.dependencies.parser]\npackage = \"boa_parser\"",
        ] {
            assert!(has_product_dependency(manifest, "boa_parser"));
        }
        assert!(!has_product_dependency(
            "[dev-dependencies.boa_parser]\nversion = \"0.21.1\"",
            "boa_parser"
        ));

        let crates_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("lila-front remains under the workspace crates directory");
        let mut parser_dependency_owners = Vec::new();
        for entry in std::fs::read_dir(crates_root).expect("failed to read workspace crates") {
            let path = entry.expect("failed to read workspace crate entry").path();
            let manifest_path = path.join("Cargo.toml");
            if !manifest_path.is_file() {
                continue;
            }
            let manifest = std::fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", manifest_path.display())
            });
            if has_product_dependency(&manifest, "boa_parser") {
                parser_dependency_owners.push(
                    path.file_name()
                        .expect("workspace crate has a directory name")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        parser_dependency_owners.sort();
        assert_eq!(parser_dependency_owners, vec!["lila-front".to_string()]);

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("lila-front remains two levels below the workspace root");
        let mut workspace_sources = Vec::new();
        collect_rust_sources(&workspace_root.join("crates"), &mut workspace_sources);

        fn identifier_mentions(source: &str, identifier: &str) -> usize {
            let is_identifier_continue = |character: Option<char>| {
                character.is_some_and(|character| character.is_alphanumeric() || character == '_')
            };

            source
                .match_indices(identifier)
                .filter(|(start, _)| {
                    let end = *start + identifier.len();
                    !is_identifier_continue(source[..*start].chars().next_back())
                        && !is_identifier_continue(source[end..].chars().next())
                })
                .count()
        }

        let classifier_identifier = ["classify_parse_", "failure"].concat();
        let classifier_call = ["classify_parse_", "failure("].concat();
        let classifier_definition = ["pub const fn classify_parse_", "failure("].concat();
        let product_parse_call = ["classify_parse_", "failure(&err)"].concat();
        let dependency_test_helper_call =
            ["lila_front::classify_parse_", "failure(message)"].concat();
        let classifier_reexport = [
            "pubuseearly_error_code::{classify_parse_",
            "failure,EarlyErrorCode,ParseClassified,NO_EARLY_ERROR_CODE,};",
        ]
        .concat();
        let front_glob_import = ["lila_front", "::*"].concat();
        let classifier_module_glob_import = ["early_error_code", "::*"].concat();
        let mut all_classifier_identifier_owners = Vec::new();
        let mut product_classifier_definitions = Vec::new();
        let mut product_classifier_calls = Vec::new();
        let mut product_parse_call_owners = Vec::new();
        let mut dependency_test_helper_owners = Vec::new();
        let mut classifier_reexport_owners = Vec::new();
        let mut classifier_glob_import_owners = Vec::new();
        for (path, source) in workspace_sources {
            let relative = path
                .strip_prefix(workspace_root)
                .expect("workspace Rust source must remain below the workspace root")
                .to_string_lossy()
                .into_owned();
            let all_count = identifier_mentions(&source, &classifier_identifier);
            if all_count != 0 {
                all_classifier_identifier_owners.push((relative.clone(), all_count));
            }

            let (product, tests) = source
                .split_once("#[cfg(test)]")
                .map_or((source.as_str(), ""), |(product, tests)| (product, tests));
            let definition_count = product.matches(classifier_definition.as_str()).count();
            if definition_count != 0 {
                product_classifier_definitions.push((relative.clone(), definition_count));
            }
            let call_count = product.matches(classifier_call.as_str()).count() - definition_count;
            if call_count != 0 {
                product_classifier_calls.push((relative.clone(), call_count));
            }
            let product_parse_call_count = product.matches(product_parse_call.as_str()).count();
            if product_parse_call_count != 0 {
                product_parse_call_owners.push((relative.clone(), product_parse_call_count));
            }
            let dependency_test_helper_count =
                tests.matches(dependency_test_helper_call.as_str()).count();
            if dependency_test_helper_count != 0 {
                dependency_test_helper_owners
                    .push((relative.clone(), dependency_test_helper_count));
            }
            let reexport_count = compact(product)
                .matches(classifier_reexport.as_str())
                .count();
            if reexport_count != 0 {
                classifier_reexport_owners.push((relative.clone(), reexport_count));
            }
            let compact_source = compact(&source);
            let glob_count = compact_source.matches(front_glob_import.as_str()).count()
                + compact_source
                    .matches(classifier_module_glob_import.as_str())
                    .count();
            if glob_count != 0 {
                classifier_glob_import_owners.push((relative, glob_count));
            }
        }
        all_classifier_identifier_owners.sort();
        product_classifier_definitions.sort();
        product_classifier_calls.sort();
        product_parse_call_owners.sort();
        dependency_test_helper_owners.sort();
        classifier_reexport_owners.sort();
        classifier_glob_import_owners.sort();
        assert_eq!(
            all_classifier_identifier_owners,
            vec![
                (
                    "crates/lila-front/src/early_error_code.rs".to_string(),
                    32,
                ),
                ("crates/lila-front/src/lib.rs".to_string(), 13),
                ("crates/lila-ir/src/modules/early.rs".to_string(), 2),
            ],
            "every classifier identifier, including imports, re-exports and aliases, requires review"
        );
        assert_eq!(
            product_classifier_definitions,
            vec![("crates/lila-front/src/early_error_code.rs".to_string(), 1,)],
            "the workspace must retain one classifier definition"
        );
        assert_eq!(
            product_classifier_calls,
            vec![
                ("crates/lila-front/src/early_error_code.rs".to_string(), 27,),
                ("crates/lila-front/src/lib.rs".to_string(), 1),
            ],
            "only classifier self-proofs and the product parse boundary may call the classifier"
        );
        assert_eq!(
            classifier_reexport_owners,
            vec![("crates/lila-front/src/lib.rs".to_string(), 1)],
            "the public classifier surface must remain the one reviewed lila-front re-export"
        );
        assert_eq!(
            classifier_glob_import_owners,
            Vec::<(String, usize)>::new(),
            "workspace glob imports must not expose or alias the classifier without naming it"
        );
        assert_eq!(
            product_parse_call_owners,
            vec![("crates/lila-front/src/lib.rs".to_string(), 1)],
            "the sole product classifier consumer must remain lila-front's direct Boa parse boundary"
        );
        assert_eq!(
            dependency_test_helper_owners,
            vec![("crates/lila-ir/src/modules/early.rs".to_string(), 1)],
            "only the retained lila-ir test helper may call the exported classifier directly"
        );
    }

    #[test]
    fn pinned_contains_super_traversal_stays_structurally_reviewed() {
        fn visitor_method<'a>(source: &'a str, signature: &str) -> &'a str {
            let start = source
                .find(signature)
                .unwrap_or_else(|| panic!("missing visitor method {signature}"));
            let remaining_source = &source[start + signature.len()..];
            let next_method = remaining_source.find("\n        fn ");
            let next_operation_comment = remaining_source.find("\n        // `");
            let end_offset = match (next_method, next_operation_comment) {
                (Some(method), Some(comment)) => method.min(comment),
                (Some(method), None) => method,
                (None, Some(comment)) => comment,
                (None, None) => remaining_source.len(),
            };
            let end = start + signature.len() + end_offset;
            &source[start..end]
        }

        fn visit_with_method<'a>(source: &'a str, implementation: &str) -> &'a str {
            let implementation_start = source
                .find(implementation)
                .unwrap_or_else(|| panic!("missing VisitWith implementation {implementation}"));
            let method_start = source[implementation_start..]
                .find("fn visit_with<'a, V>")
                .map(|offset| implementation_start + offset)
                .expect("VisitWith implementation must retain its shared visitor method");
            let method_end = source[method_start..]
                .find("\n    fn visit_with_mut")
                .map(|offset| method_start + offset)
                .expect("shared visitor method must remain before its mutable counterpart");
            &source[method_start..method_end]
        }

        let compact = |source: &str| {
            source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
                .replace(",)", ")")
        };

        const OPERATIONS_SOURCE: &str =
            include_str!("../../../vendor/boa_ast-0.21.1/src/operations/mod.rs");
        const PROPERTY_SOURCE: &str =
            include_str!("../../../vendor/boa_ast-0.21.1/src/property.rs");
        const CLASS_SOURCE: &str =
            include_str!("../../../vendor/boa_ast-0.21.1/src/function/class.rs");

        for signature in [
            "fn visit_function_expression(",
            "fn visit_function_declaration(",
            "fn visit_async_function_expression(",
            "fn visit_async_function_declaration(",
            "fn visit_generator_expression(",
            "fn visit_generator_declaration(",
            "fn visit_async_generator_expression(",
            "fn visit_async_generator_declaration(",
        ] {
            let method = visitor_method(OPERATIONS_SOURCE, signature);
            assert!(method.contains("self.visit_contains_eval(node.contains_direct_eval)?;"));
            assert!(method.contains("ControlFlow::Continue(())"));
            assert!(
                !method.contains("node.visit_with(self)"),
                "ordinary callable definitions must remain Contains boundaries: {signature}"
            );
        }

        const ARROW_VISITOR: &str = r#"fn visit_arrow_function(
            &mut self,
            node: &'ast ArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
                ContainsSymbol::DirectEval,
            ]
            .contains(&self.0)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }"#;
        const ASYNC_ARROW_VISITOR: &str = r#"fn visit_async_arrow_function(
            &mut self,
            node: &'ast AsyncArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
                ContainsSymbol::DirectEval,
            ]
            .contains(&self.0)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }"#;
        for (signature, expected) in [
            ("fn visit_arrow_function(", ARROW_VISITOR),
            ("fn visit_async_arrow_function(", ASYNC_ARROW_VISITOR),
        ] {
            let method = compact(visitor_method(OPERATIONS_SOURCE, signature));
            assert_eq!(
                method,
                compact(expected),
                "lexical arrow traversal changed: {signature}"
            );
            assert_eq!(method.matches("returnControlFlow::Continue(())").count(), 1);
            assert_eq!(method.matches("ContainsSymbol::SuperCall").count(), 1);
            assert!(method.ends_with("node.visit_with(self)}"));
        }

        const CLASS_EXPRESSION_VISITOR: &str = r#"fn visit_class_expression(
            &mut self,
            node: &'ast ClassExpression,
        ) -> ControlFlow<Self::BreakTy> {
            if !node.elements().is_empty() && self.0 == ContainsSymbol::ClassBody {
                return ControlFlow::Break(());
            }

            if node.super_ref().is_some() && self.0 == ContainsSymbol::ClassHeritage {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }"#;
        const CLASS_DECLARATION_VISITOR: &str = r#"fn visit_class_declaration(
            &mut self,
            node: &'ast ClassDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            if !node.elements().is_empty() && self.0 == ContainsSymbol::ClassBody {
                return ControlFlow::Break(());
            }

            if node.super_ref().is_some() && self.0 == ContainsSymbol::ClassHeritage {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }"#;
        for (signature, expected) in [
            ("fn visit_class_expression(", CLASS_EXPRESSION_VISITOR),
            ("fn visit_class_declaration(", CLASS_DECLARATION_VISITOR),
        ] {
            let method = compact(visitor_method(OPERATIONS_SOURCE, signature));
            assert_eq!(
                method,
                compact(expected),
                "class Contains boundary changed: {signature}"
            );
            assert!(
                !method.contains("ContainsSymbol::SuperCall"),
                "a nested class must not stop SuperCall before computed names are visited: {signature}"
            );
            assert!(method.ends_with("node.visit_with(self)}"));
        }

        const CLASS_ELEMENT_VISITOR: &str = r#"fn visit_class_element(
            &mut self,
            node: &'ast ClassElement
        ) -> ControlFlow<Self::BreakTy> {
            match node {
                ClassElement::MethodDefinition(m) => {
                    if self.0 == ContainsSymbol::DirectEval {
                        return ControlFlow::Continue(());
                    }

                    if let ClassElementName::PropertyName(name) = m.name() {
                        name.visit_with(self)
                    } else {
                        ControlFlow::Continue(())
                    }
                }
                ClassElement::FieldDefinition(field)
                | ClassElement::StaticFieldDefinition(field) => field.name.visit_with(self),
                _ => ControlFlow::Continue(()),
            }
        }"#;
        let class_element = compact(visitor_method(OPERATIONS_SOURCE, "fn visit_class_element("));
        assert_eq!(class_element, compact(CLASS_ELEMENT_VISITOR));
        assert!(!class_element.contains("StaticBlock"));
        assert!(!class_element.contains(".body"));
        assert!(!class_element.contains("initializer"));

        const CLASS_DECLARATION_VISIT_WITH: &str = r#"fn visit_with<'a, V>(
            &'a self,
            visitor: &mut V,
        ) -> ControlFlow<V::BreakTy>
        where
            V: Visitor<'a>,
        {
            visitor.visit_identifier(&self.name)?;
            for decorator in &*self.decorators {
                visitor.visit_expression(decorator)?;
            }
            if let Some(expr) = &self.super_ref {
                visitor.visit_expression(expr)?;
            }
            if let Some(func) = &self.constructor {
                visitor.visit_function_expression(func)?;
            }
            for elem in &*self.elements {
                visitor.visit_class_element(elem)?;
            }
            ControlFlow::Continue(())
        }"#;
        const CLASS_EXPRESSION_VISIT_WITH: &str = r#"fn visit_with<'a, V>(
            &'a self,
            visitor: &mut V,
        ) -> ControlFlow<V::BreakTy>
        where
            V: Visitor<'a>,
        {
            if let Some(ident) = &self.name {
                visitor.visit_identifier(ident)?;
            }
            for decorator in &*self.decorators {
                visitor.visit_expression(decorator)?;
            }
            if let Some(expr) = &self.super_ref {
                visitor.visit_expression(expr)?;
            }
            if let Some(func) = &self.constructor {
                visitor.visit_function_expression(func)?;
            }
            for elem in &*self.elements {
                visitor.visit_class_element(elem)?;
            }
            ControlFlow::Continue(())
        }"#;
        for (implementation, expected) in [
            (
                "impl VisitWith for ClassDeclaration",
                CLASS_DECLARATION_VISIT_WITH,
            ),
            (
                "impl VisitWith for ClassExpression",
                CLASS_EXPRESSION_VISIT_WITH,
            ),
        ] {
            assert_eq!(
                compact(visit_with_method(CLASS_SOURCE, implementation)),
                compact(expected),
                "nested-class traversal changed: {implementation}"
            );
        }

        let property_definition = compact(visitor_method(
            OPERATIONS_SOURCE,
            "fn visit_property_definition(",
        ));
        assert!(property_definition.contains("ifletPropertyDefinition::MethodDefinition(m)=node"));
        assert!(property_definition.contains("returnm.name().visit_with(self);"));
        assert!(property_definition.ends_with("node.visit_with(self)}"));

        const PROPERTY_NAME_VISIT_WITH: &str = r#"fn visit_with<'a, V>(
            &'a self,
            visitor: &mut V,
        ) -> ControlFlow<V::BreakTy>
        where
            V: Visitor<'a>,
        {
            match self {
                Self::Literal(ident) => visitor.visit_sym(ident.sym_ref()),
                Self::Computed(expr) => visitor.visit_expression(expr),
            }
        }"#;
        let property_visit = compact(visit_with_method(
            PROPERTY_SOURCE,
            "impl VisitWith for PropertyName",
        ));
        assert_eq!(property_visit, compact(PROPERTY_NAME_VISIT_WITH));
        assert!(
            class_element.contains("m.name()")
                && class_element.contains("field.name.visit_with(self)")
                && property_visit.contains("Self::Computed(expr)=>visitor.visit_expression(expr)"),
            "nested classes must traverse computed element names without traversing class bodies"
        );
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
    fn import_meta_outside_module_rejects_at_every_script_nesting_boundary() {
        for source in [
            "import.meta;",
            "\"use strict\";\nimport.meta;",
            "function f() { return import.meta; }",
            "function* f() { return import.meta; }",
            "async function f() { return import.meta; }",
            "const f = () => import.meta;",
            "class C { m() { return import.meta; } }",
            "class C { field = import.meta; }",
            "class C { static field = import.meta; }",
            "class C { static { void import.meta; } }",
            "\n  import.meta;",
        ] {
            let err = parse(source, ParseOptions::script())
                .expect_err("ImportMeta requires the Module syntactic goal");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ImportMetaOutsideModule),
                "{source:?}: {err:?}"
            );

            let start = source
                .find("import.meta")
                .expect("the rejection witness contains ImportMeta");
            assert_eq!(
                err.diagnostic().span,
                Some(SourceSpan {
                    start,
                    end: start + 1,
                }),
                "the parser position is the initial import token: {source:?}: {err:?}"
            );
        }
    }

    #[test]
    fn import_meta_module_goal_and_import_call_boundaries_remain_valid() {
        for source in [
            "import.meta;",
            "function f() { return import.meta; }",
            "function* f() { return import.meta; }",
            "async function f() { return import.meta; }",
            "const f = () => import.meta;",
            "class C { m() { return import.meta; } }",
            "class C { field = import.meta; }",
            "class C { static field = import.meta; }",
            "class C { static { void import.meta; } }",
        ] {
            parse(source, ParseOptions::module())
                .expect("lexical nesting does not replace the Module syntactic goal");
        }

        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse("import('./dep.mjs');", options)
                .expect("ImportCall is valid under both static source goals");
        }
    }

    #[test]
    fn import_meta_adjacent_failures_and_goal_precedence_remain_distinct() {
        let mixed = "import.meta; let x; let x;";
        let script = parse(mixed, ParseOptions::script())
            .expect_err("Script rejects ImportMeta before whole-source declaration analysis");
        assert_eq!(
            script.diagnostic().code,
            early(EarlyErrorCode::ImportMetaOutsideModule)
        );

        let module = parse(mixed, ParseOptions::module())
            .expect_err("Module accepts ImportMeta and reaches duplicate declaration analysis");
        assert_eq!(
            module.diagnostic().code,
            early(EarlyErrorCode::DuplicateLexicalDeclaration)
        );

        for source in [r"imp\u006frt.meta;", r"import.m\u0065ta;", "import.foo;"] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a spelling error must reject before the ImportMeta goal check");
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_ne!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ImportMetaOutsideModule),
                    "{source:?}: {err:?}"
                );
            }
        }

        let err = parse("import.meta = 0;", ParseOptions::module())
            .expect_err("ImportMeta is not a valid assignment target");
        assert_ne!(
            err.diagnostic().code,
            early(EarlyErrorCode::ImportMetaOutsideModule),
            "a Module assignment-target error must keep its distinct owner: {err:?}"
        );
    }

    #[test]
    fn user_export_names_cannot_forge_import_meta_goal_classification() {
        let err = parse(
            concat!(
                "const value = 0;\n",
                "export { value as \"invalid `import.meta` expression outside a module at line\" };\n",
                "export { value as \"invalid `import.meta` expression outside a module at line\" };",
            ),
            ParseOptions::module(),
        )
        .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err}"
        );
    }

    #[test]
    fn known_import_meta_goal_producer_stays_structurally_reviewed() {
        fn count_in_rust_sources(root: &std::path::Path, fragment: &str) -> usize {
            let entries = std::fs::read_dir(root)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
            let mut count = 0;
            for entry in entries {
                let path = entry.expect("failed to read vendored Boa entry").path();
                if path.is_dir() {
                    count += count_in_rust_sources(&path, fragment);
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                        panic!("failed to read {}: {error}", path.display())
                    });
                    count += source.matches(fragment).count();
                }
            }
            count
        }

        const MESSAGE: &str = "invalid `import.meta` expression outside a module";
        const POSITION_CAPTURE: &str = "let position = token.span().start();";
        const INITIAL_TOKEN_CAPTURE: &str = r#"
            cursor.set_goal(InputElement::RegExp);

            let token = cursor.peek(0, interner).or_abrupt()?;
            let position = token.span().start();
            let mut lhs = match token.kind() {
        "#;
        const GOAL_BRANCH: &str = r#"if !cursor.module() {
                    return Err(Error::general(
                        "invalid `import.meta` expression outside a module",
                        position,
                    ));
                }"#;
        const MODULE_ENTRY: &str = r#"
            #[derive(Debug, Clone, Copy)]
            struct ModuleParser;

            impl<R> TokenParser<R> for ModuleParser
            where
                R: ReadChar,
            {
                type Output = ModuleParseOutput;

                fn parse(
                    self,
                    cursor: &mut Cursor<R>,
                    interner: &mut Interner
                ) -> ParseResult<Self::Output> {
                    cursor.set_module();

                    let module = boa_ast::Module::new(ModuleItemList.parse(cursor, interner)?);
        "#;
        const MODULE_TRUE_PROJECTION: &str = r#"
            pub(super) fn set_module(&mut self) {
                self.buffered_lexer.set_module(true);
            }

            /// Returns `true` if the cursor is currently parsing a `Module`.
            pub(super) const fn module(&self) -> bool {
                self.buffered_lexer.module()
            }
        "#;
        const BUFFERED_LEXER_MODULE_PROJECTION: &str = r#"
            pub(super) const fn module(&self) -> bool {
                self.lexer.module()
            }

            pub(super) fn set_module(&mut self, module: bool) {
                self.lexer.set_module(module);
            }
        "#;
        const LEXER_MODULE_READ: &str = r#"
            pub(super) const fn module(&self) -> bool {
                self.cursor.module()
            }
        "#;
        const LEXER_MODULE_WRITE: &str = r#"
            pub(super) fn set_module(&mut self, module: bool) {
                self.cursor.set_module(module);
            }
        "#;
        const TERMINAL_MODULE_READ: &str = r#"
            pub(super) const fn module(&self) -> bool {
                self.module
            }
        "#;
        const TERMINAL_MODULE_WRITE: &str = r#"
            pub(super) fn set_module(&mut self, module: bool) {
                self.module = module;
                self.strict = module;
            }
        "#;
        const INITIAL_LEXER_STATE: &str = r#"
            Self {
                iter: inner,
                pos: Position::new(1, 1),
                strict: false,
                module: false,
                peeked: [None; 4],
                source_collector: SourceText::default(),
            }
        "#;
        const ESCAPED_META_MESSAGE: &str = "`import.meta` cannot contain escaped characters";
        const MEMBER_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/left_hand_side/member.rs"
        );
        const PARSER_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/mod.rs");
        const PARSER_CURSOR_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/cursor/mod.rs");
        const BUFFERED_LEXER_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/cursor/buffered_lexer/mod.rs"
        );
        const LEXER_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/lexer/mod.rs");
        const LEXER_CURSOR_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/lexer/cursor.rs");

        let without_whitespace = |source: &str| {
            source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        };

        let boa_package_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/boa_parser-0.21.1");
        assert_eq!(count_in_rust_sources(&boa_package_root, MESSAGE), 1);
        assert_eq!(MEMBER_SOURCE.matches(MESSAGE).count(), 1);
        assert_eq!(MEMBER_SOURCE.matches(POSITION_CAPTURE).count(), 1);
        assert_eq!(
            without_whitespace(MEMBER_SOURCE)
                .matches(without_whitespace(INITIAL_TOKEN_CAPTURE).as_str())
                .count(),
            1,
            "the diagnostic position must come from the initial member-expression token",
        );
        assert_eq!(MEMBER_SOURCE.matches("cursor.module()").count(), 1);
        assert_eq!(MEMBER_SOURCE.matches(GOAL_BRANCH).count(), 1);
        assert!(
            MEMBER_SOURCE
                .find(ESCAPED_META_MESSAGE)
                .expect("the escaped-meta producer remains present")
                < MEMBER_SOURCE
                    .find(GOAL_BRANCH)
                    .expect("the ImportMeta goal branch remains present"),
            "escaped meta must reject before the syntactic-goal condition"
        );
        assert_eq!(LEXER_CURSOR_SOURCE.matches("module: false,").count(), 1);
        assert_eq!(
            without_whitespace(LEXER_CURSOR_SOURCE)
                .matches(without_whitespace(INITIAL_LEXER_STATE).as_str())
                .count(),
            1,
            "a fresh lexer cursor must start in Script rather than Module mode",
        );
        assert_eq!(PARSER_SOURCE.matches("cursor.set_module();").count(), 1);
        assert_eq!(
            without_whitespace(PARSER_SOURCE)
                .matches(without_whitespace(MODULE_ENTRY).as_str())
                .count(),
            1,
            "ModuleParser must enter Module mode before parsing ModuleItemList",
        );
        assert_eq!(
            without_whitespace(PARSER_CURSOR_SOURCE)
                .matches(without_whitespace(MODULE_TRUE_PROJECTION).as_str())
                .count(),
            1,
            "the parser's Module transition must project the true lexer state",
        );
        assert_eq!(
            without_whitespace(BUFFERED_LEXER_SOURCE)
                .matches(without_whitespace(BUFFERED_LEXER_MODULE_PROJECTION).as_str())
                .count(),
            1,
            "the buffered lexer must forward Module reads and writes unchanged",
        );
        assert_eq!(
            without_whitespace(LEXER_SOURCE)
                .matches(without_whitespace(LEXER_MODULE_READ).as_str())
                .count(),
            1,
            "the lexer must forward Module reads unchanged",
        );
        assert_eq!(
            without_whitespace(LEXER_SOURCE)
                .matches(without_whitespace(LEXER_MODULE_WRITE).as_str())
                .count(),
            1,
            "the lexer must forward Module writes unchanged",
        );
        assert_eq!(
            without_whitespace(LEXER_CURSOR_SOURCE)
                .matches(without_whitespace(TERMINAL_MODULE_READ).as_str())
                .count(),
            1,
            "the terminal cursor must read the Module state bit directly",
        );
        assert_eq!(
            without_whitespace(LEXER_CURSOR_SOURCE)
                .matches(without_whitespace(TERMINAL_MODULE_WRITE).as_str())
                .count(),
            1,
            "the terminal cursor must write the Module state bit and matching strictness directly",
        );
        assert_eq!(LEXER_CURSOR_SOURCE.matches("self.module =").count(), 1);
        assert_eq!(count_in_rust_sources(&boa_package_root, "set_module("), 8);
        assert_eq!(
            count_in_rust_sources(&boa_package_root, ".set_module(false)"),
            0
        );
        assert_eq!(
            count_in_rust_sources(&boa_package_root, ".set_module(true)"),
            1
        );
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
                parse(source, options).expect(
                    "var heads and declarations across a nested function boundary stay valid",
                );
            }
        }
    }

    #[test]
    fn for_in_using_conflicts_keep_their_existing_early_error_owner() {
        for source in [
            "for (using x in {}) { var x; }",
            "async function f() { for (await using x in {}) { var x; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("for-in using syntax must reject before body conflict analysis");
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::ForInUsingDeclaration),
                    "{source:?}: {err:?}"
                );
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
                    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                        panic!("failed to read {}: {error}", path.display())
                    });
                    count += source.matches(message).count();
                }
            }
            count
        }

        const CONFLICT_MESSAGE: &str = "For loop initializer declared in loop body";
        const DUPLICATE_MESSAGE: &str = "For loop initializer cannot contain duplicate identifiers";
        const GENERIC_DUPLICATE_MESSAGE: &str = "lexical name declared multiple times";
        const LEXICAL_BOUND_NAME_LET_MESSAGE: &str =
            "'let' is disallowed as a lexically bound name";
        const FOR_DECLARATION_BOUND_NAME_LET_MESSAGE: &str =
            "Cannot use 'let' as a lexically bound name";
        const LEXICAL_CONTEXT: &str = r#"enum LexicalDeclarationContext {
    Statement,
    ForHead,
}"#;
        const EXHAUSTIVE_CONTEXT_PROJECTION: &str = r#"const fn is_for_head(self) -> bool {
        match self {
            Self::Statement => false,
            Self::ForHead => true,
        }
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
        const STATEMENT_TERMINATOR: &str = r#"if !self.context.is_for_head() {
            cursor.expect_semicolon("lexical declaration", interner)?;
        }"#;
        const ORDINARY_VALIDATION: &str = r#"if !self.context.is_for_head() {
            Self::validate_bound_name_let(&lexical_declaration, tok.span().start())?;
            Self::validate_duplicate_bound_names(&lexical_declaration, tok.span().start())?;
        }"#;
        const FOR_HEAD_MISSING_INITIALIZER: &str = r#"if init_is_some || self.context.is_for_head() {
                    decls.push(decl);
                } else {"#;
        const FOR_HEAD_BINDING_TERMINATOR: &str = r#"SemicolonResult::NotFound(_) if self.context.is_for_head() => {
                    break;
                }"#;
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
        const FOR_STATEMENT_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/iteration/for_statement.rs"
        );
        const LEXICAL_DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/lexical.rs"
        );
        const DECLARATION_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/mod.rs"
        );
        const BINDING_PATTERN_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/mod.rs");
        const BINDING_IDENTIFIER_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/expression/identifiers.rs");
        const VARIABLE_DECLARATION_SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/variable/mod.rs");

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
            count_message_in_rust_sources(
                &boa_package_root,
                FOR_DECLARATION_BOUND_NAME_LET_MESSAGE,
            ),
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
            LEXICAL_DECLARATION_SOURCE
                .matches(EXHAUSTIVE_CONTEXT_PROJECTION)
                .count(),
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
            0
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
            4
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches("context: LexicalDeclarationContext,")
                .count(),
            3
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches("LexicalDeclarationContext::Statement")
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches("LexicalDeclarationContext::ForHead")
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches("Self::Statement")
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE.matches("Self::ForHead").count(),
            1
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
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches(STATEMENT_TERMINATOR)
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches(ORDINARY_VALIDATION)
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches(FOR_HEAD_MISSING_INITIALIZER)
                .count(),
            1
        );
        assert_eq!(
            LEXICAL_DECLARATION_SOURCE
                .matches(FOR_HEAD_BINDING_TERMINATOR)
                .count(),
            1
        );
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
    fn callable_non_simple_parameters_with_use_strict_reject_at_every_producer() {
        // One source per remaining fixed-message site. Shared declaration and
        // method parsers need one caller here; the pinned cohort covers their
        // other grammar forms without duplicating this source-level contract.
        const PRODUCER_SOURCES: [&str; 16] = [
            "function f(a = 0) { 'use strict'; }",
            "(function(a = 0) { 'use strict'; });",
            "(function*(a = 0) { 'use strict'; });",
            "(async function(a = 0) { 'use strict'; });",
            "(async function*(a = 0) { 'use strict'; });",
            "(a = 0) => { 'use strict'; };",
            "async (a = 0) => { 'use strict'; };",
            "({ set x(a = 0) { 'use strict'; } });",
            "({ x(a = 0) { 'use strict'; } });",
            "({ *x(a = 0) { 'use strict'; } });",
            "({ async *x(a = 0) { 'use strict'; } });",
            "({ async x(a = 0) { 'use strict'; } });",
            "class C { set #x(a = 0) { 'use strict'; } }",
            "class C { set x({ a }) { 'use strict'; } }",
            "class C { #x(a = 0) { 'use strict'; } }",
            "class C { x(a = 0) { 'use strict'; } }",
        ];
        for source in PRODUCER_SOURCES {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("non-simple parameters plus an own directive should fail");
                assert_eq!(
                    err.diagnostic().phase(),
                    ParseDiagnosticPhase::Early,
                    "{source:?}: {err:?}"
                );
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::CallableNonSimpleParametersContainUseStrict),
                    "{source:?}: {err:?}"
                );
                assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn callable_use_strict_message_inventory_stays_at_sixteen_reviewed_sites() {
        const MESSAGE: &str =
            "Illegal 'use strict' directive in function with non-simple parameter list";
        const SOURCES: [(&str, usize); 10] = [
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs"
                ),
                4,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/object_initializer/mod.rs"
                ),
                5,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/function_expression/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/generator_expression/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/async_function_expression/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/primary/async_generator_expression/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/assignment/mod.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/assignment/async_arrow_function.rs"
                ),
                1,
            ),
            (
                include_str!(
                    "../../../vendor/boa_parser-0.21.1/src/parser/expression/assignment/arrow_function.rs"
                ),
                0,
            ),
        ];

        let mut total = 0;
        for (source, expected) in SOURCES {
            let count = source.matches(MESSAGE).count();
            assert_eq!(count, expected);
            total += count;
        }
        assert_eq!(total, 16);
    }

    #[test]
    fn callable_use_strict_conjunction_and_containment_boundaries_remain_valid() {
        for source in [
            "function simple(a) { 'use strict'; }",
            "function non_simple(a = 0) {}",
            "a => { 'use strict'; };",
            "(a = 0) => 0;",
            "function nested(a = 0) { function inner() { 'use strict'; } }",
            "function after_prologue(a = 0) { 0; 'use strict'; }",
            "class C { method(a = 0) {} }",
            "class C { get #x() { 'use strict'; } }",
            "class C { set #x(a = 0) {} }",
            "class C { set x({ a }) {} }",
            "class C { set x(a) { 'use strict'; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "ambient strictness, one false conjunct, or a nested directive must remain valid",
                );
            }
        }
    }

    #[test]
    fn private_getter_and_class_setter_grammar_errors_stay_unclassified() {
        for source in [
            "class C { get #x(a = 0) { 'use strict'; } }",
            "class C { set #x() {} }",
            "class C { set #x(a, b) {} }",
            "class C { set #x(a,) {} }",
            "class C { set #x(...a) { 'use strict'; } }",
            "class C { set x() {} }",
            "class C { set x(a, b) {} }",
            "class C { set x(a,) {} }",
            "class C { set x(...a) { 'use strict'; } }",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("getter/setter parameter grammar must reject before body analysis");
                assert_eq!(
                    err.diagnostic().code,
                    ParseCode::Malformed,
                    "{source:?}: {err:?}"
                );
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
    fn delete_reference_early_errors_keep_distinct_typed_conditions() {
        for (source, options, code) in [
            (
                r#""use strict"; delete identifier;"#,
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeleteIdentifierReference,
            ),
            (
                r#""use strict"; delete (identifier);"#,
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeleteIdentifierReference,
            ),
            (
                r#"function f() { "use strict"; delete ((identifier)); }"#,
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeleteIdentifierReference,
            ),
            (
                "delete identifier;",
                ParseOptions::module(),
                EarlyErrorCode::StrictModeDeleteIdentifierReference,
            ),
            (
                "class C { #x; m(o) { delete o.#x; } }",
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "const C = class { #x; field = delete ((this.#x)); };",
                ParseOptions::module(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "class C { #m() {} m(o) { delete o().#m; } }",
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "const C = class { get #x() {} m(o) { delete (o.#x); } };",
                ParseOptions::module(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "class C { m(o) { delete o.#missing; } }",
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "class C { #x; m(o) { delete o?.#x; } }",
                ParseOptions::script(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
            (
                "const C = class { #x; m(o) { delete ((o?.c.#x)); } };",
                ParseOptions::module(),
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
        ] {
            let err = parse(source, options)
                .expect_err("a forbidden delete-reference operand should fail before evaluation");
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

    #[test]
    fn sloppy_private_delete_stays_owned_by_private_name_validation() {
        for source in [
            "delete object.#missing;",
            "delete ((object.#missing));",
            "object.#missing;",
        ] {
            let err = parse(source, ParseOptions::script())
                .expect_err("an undeclared private name should fail whole-source validation");
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
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::InvalidPrivateIdentifier),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }

    #[test]
    fn delete_reference_early_error_boundaries_remain_valid() {
        for (source, options) in [
            ("delete identifier;", ParseOptions::script()),
            ("delete ((identifier));", ParseOptions::script()),
            (
                r#""use strict"; delete object.property;"#,
                ParseOptions::script(),
            ),
            ("delete object.property;", ParseOptions::module()),
            (
                r#""use strict"; delete (0, identifier);"#,
                ParseOptions::script(),
            ),
            ("delete object[key];", ParseOptions::module()),
            (
                r#""use strict"; delete object?.property;"#,
                ParseOptions::script(),
            ),
            ("class C { #x; m(o) { o.#x; } }", ParseOptions::script()),
            (
                "class C { #x; m(o) { return o?.#x; } }",
                ParseOptions::script(),
            ),
            (
                "const C = class { #x; m(o) { return o?.c.#x; } };",
                ParseOptions::module(),
            ),
            (
                "class C { #x; m(o) { delete o?.#x.property; } }",
                ParseOptions::script(),
            ),
            (
                "class C { #m() {} m(o) { delete o?.#m(); } }",
                ParseOptions::script(),
            ),
        ] {
            parse(source, options).expect(
                "sloppy identifiers, public properties, values and private reads remain valid syntax",
            );
        }
    }

    #[test]
    fn delete_reference_message_inventory_stays_at_one_reviewed_site_per_condition() {
        const SOURCE: &str =
            include_str!("../../../vendor/boa_parser-0.21.1/src/parser/expression/unary.rs");

        assert_eq!(
            SOURCE
                .matches("cannot delete variables in strict mode")
                .count(),
            1
        );
        assert_eq!(SOURCE.matches("cannot delete private fields").count(), 1);
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

    #[test]
    fn optional_chain_tagged_templates_report_one_early_error_in_both_goals() {
        for source in [
            "const value = null; value?.`x`;",
            "const value = null; value?.`x${1}`;",
            "const value = null; value?.\n`x`;",
            "const value = { tag() {} }; value?.tag`x`;",
            "const value = { tag() {} }; value?.tag`x${1}`;",
            "const value = { tag() {} }; value?.tag\n`x`;",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                let err = parse(source, options)
                    .expect_err("a TemplateLiteral directly on an optional chain should fail");
                assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
                assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
                assert_eq!(
                    err.diagnostic().code,
                    early(EarlyErrorCode::OptionalChainTaggedTemplate),
                    "{source:?}: {err:?}"
                );
                let span = err
                    .diagnostic()
                    .span
                    .expect("the rejected TemplateLiteral must retain its source span");
                assert!(span.start < span.end, "{source:?}: {err:?}");
            }
        }
    }

    #[test]
    fn ordinary_tags_and_completed_optional_chains_remain_parse_valid() {
        for source in [
            "const tag = () => {}; tag`x`;",
            "const tag = () => {}; tag`x${1}`;",
            "const value = {}; value?.property;",
            "const callable = () => 0; callable?.();",
            "const value = { tag() {} }; (value?.tag)`x`;",
            "const value = { tag() {} }; (value?.tag)`x${1}`;",
        ] {
            for options in [ParseOptions::script(), ParseOptions::module()] {
                parse(source, options).expect(
                    "only a TemplateLiteral directly in the OptionalChain production is forbidden",
                );
            }
        }
    }

    #[test]
    fn user_export_names_cannot_forge_optional_chain_tagged_template_classification() {
        let err = parse(
            concat!(
                "const value = 0;\n",
                "export { value as \"Invalid tagged template on optional chain at line\" };\n",
                "export { value as \"Invalid tagged template on optional chain at line\" };",
            ),
            ParseOptions::module(),
        )
        .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err}"
        );
    }

    #[test]
    fn known_optional_chain_tagged_template_message_producers_stay_reviewed() {
        const MESSAGE: &str = "Invalid tagged template on optional chain";
        const TEMPLATE_TOKEN_PAIR: &str =
            "TokenKind::TemplateMiddle(_) | TokenKind::TemplateNoSubstitution(_)";
        const OPTIONAL_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/expression/left_hand_side/optional/mod.rs"
        );

        assert_eq!(OPTIONAL_SOURCE.matches(MESSAGE).count(), 2);
        assert_eq!(OPTIONAL_SOURCE.matches(TEMPLATE_TOKEN_PAIR).count(), 2);
    }

    #[test]
    fn duplicate_import_attribute_keys_report_one_module_early_error() {
        for source in [
            r#"import "./dep.mjs" with { type: "json", "type": "css" };"#,
            r#"export * from "./dep.mjs" with { mode: "first", "mode": "second" };"#,
        ] {
            let err = parse(source, ParseOptions::module())
                .expect_err("duplicate static import-attribute keys should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ModuleDuplicateImportAttributeKey),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }

    #[test]
    fn distinct_import_attribute_keys_and_trailing_commas_remain_valid() {
        for source in [
            r#"import "./dep.mjs" with { type: "json", mode: "strict", };"#,
            r#"export * from "./dep.mjs" with { type: "json", mode: "strict", };"#,
        ] {
            parse(source, ParseOptions::module())
                .expect("distinct static import-attribute keys should parse");
        }
    }

    #[test]
    fn user_export_names_cannot_forge_duplicate_import_attribute_classification() {
        let err = parse(
            r#"export { "duplicate import attribute key at line" };"#,
            ParseOptions::module(),
        )
        .expect_err("a string literal cannot be a local referenced binding");
        assert_eq!(err.diagnostic().code, ParseCode::Malformed, "{err}");

        let err = parse(
            concat!(
                "const value = 0;\n",
                "export { value as \"duplicate import attribute key at line\" };\n",
                "export { value as \"duplicate import attribute key at line\" };",
            ),
            ParseOptions::module(),
        )
        .expect_err("the user-chosen exported name is duplicated");
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::ModuleDuplicateExport),
            "{err}"
        );
    }

    #[test]
    fn known_duplicate_import_attribute_message_producers_stay_reviewed() {
        const MESSAGE: &str = "duplicate import attribute key";
        const IMPORT_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/import.rs"
        );
        const EXPORT_SOURCE: &str = include_str!(
            "../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/export.rs"
        );

        assert_eq!(IMPORT_SOURCE.matches(MESSAGE).count(), 1);
        assert_eq!(EXPORT_SOURCE.matches(MESSAGE).count(), 1);
        assert!(classify_parse_failure("duplicate import attribute key: type").is_none());
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
    /// whole `ContainsAll` fragment set of the one table into the message boa produces for an
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

    /// MC4's call-site half. A code the message-pattern table cannot produce is not a
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
        assert!(ParseClassified::from_early(EarlyErrorCode::ScriptTopLevelSuper).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ImportMetaOutsideModule).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ScriptTopLevelUsingDeclaration).is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ForHeadBodyDeclarationConflict).is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ForDeclarationDuplicateBoundName).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::LexicalBoundNameLet).is_some());
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
        assert!(ParseClassified::from_early(
            EarlyErrorCode::CallableNonSimpleParametersContainUseStrict,
        )
        .is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::DuplicateFormalParameter).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::DuplicateCatchParameter).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::CatchBodyDeclarationConflict).is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassBaseConstructorHasDirectSuper)
                .is_some()
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
            ParseClassified::from_early(EarlyErrorCode::ClassStaticBlockContainsSuperCall)
                .is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ClassStaticBlockContainsAwait).is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ClassFieldContainsArguments).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::StrictModeWithStatement).is_some());
        assert!(
            ParseClassified::from_early(EarlyErrorCode::StrictModeDeleteIdentifierReference,)
                .is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::StrictModeDeletePrivateReference,)
                .is_some()
        );
        assert!(
            ParseClassified::from_early(EarlyErrorCode::ModuleDuplicateImportAttributeKey,)
                .is_some()
        );
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleDuplicateExport).is_some());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleMissingExport).is_none());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleUnresolved).is_none());
        assert!(ParseClassified::from_early(EarlyErrorCode::ModuleTooManyUnits).is_none());
    }
}
