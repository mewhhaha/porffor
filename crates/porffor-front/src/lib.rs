use boa_ast::scope::Scope;
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use std::panic::{self, AssertUnwindSafe};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDiagnosticKind {
    MalformedJavaScript,
    UnsupportedParserFeature,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub phase: ParseDiagnosticPhase,
    pub code: &'static str,
    pub error_type: &'static str,
    pub span: Option<SourceSpan>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    diagnostic: ParseDiagnostic,
    message: String,
}

impl ParseError {
    pub fn malformed(message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        let message = message.into();
        Self {
            diagnostic: ParseDiagnostic {
                kind: ParseDiagnosticKind::MalformedJavaScript,
                phase: ParseDiagnosticPhase::Parse,
                code: "P_PARSE_MALFORMED",
                error_type: "SyntaxError",
                span,
                message: message.clone(),
            },
            message,
        }
    }

    pub fn unsupported_parser_feature(
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        let message = message.into();
        Self {
            diagnostic: ParseDiagnostic {
                kind: ParseDiagnosticKind::UnsupportedParserFeature,
                phase: ParseDiagnosticPhase::Parse,
                code: "P_PARSE_UNSUPPORTED",
                error_type: "SyntaxError",
                span,
                message: message.clone(),
            },
            message,
        }
    }

    pub fn early_error(
        code: &'static str,
        error_type: &'static str,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        let message = message.into();
        Self {
            diagnostic: ParseDiagnostic {
                kind: ParseDiagnosticKind::MalformedJavaScript,
                phase: ParseDiagnosticPhase::Early,
                code,
                error_type,
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
) -> Result<SourceUnit, ParseError> {
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
            .map(|_| ()),
        ParseGoal::Module => Parser::new(source)
            .parse_module(&scope, &mut interner)
            .map(|_| ()),
    }));

    match parsed {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            let err = err.to_string();
            let message = format!("parse error: {err}");
            let span = parse_error_span_from_message(&source_text, &err);
            return if let Some(code) = parser_static_semantics_error_code(&err) {
                Err(ParseError::early_error(code, "SyntaxError", message, span))
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
    }

    Ok(SourceUnit {
        goal: options.goal,
        filename: options.filename,
        source_text,
    })
}

fn parser_static_semantics_error_code(message: &str) -> Option<&'static str> {
    if message.contains("Duplicate __proto__ fields are not allowed in object literals") {
        return Some("E_OBJECT_DUPLICATE_PROTO");
    }
    if message.contains("duplicate lexical declaration")
        || message.contains("lexical name declared multiple times")
        || (message.contains("lexical")
            && message.contains("declared")
            && message.contains("names"))
    {
        return Some("E_DUPLICATE_LEXICAL_DECLARATION");
    }
    if message.contains("undefined break target") {
        return Some("E_UNDEFINED_BREAK_TARGET");
    }
    if message.contains("undefined continue target") {
        return Some("E_UNDEFINED_CONTINUE_TARGET");
    }
    if message.contains("illegal break statement") {
        return Some("E_ILLEGAL_BREAK");
    }
    if message.contains("illegal continue statement") {
        return Some("E_ILLEGAL_CONTINUE");
    }
    None
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

    #[test]
    fn script_rejects_module_syntax() {
        let err = parse("export const value = 1;", ParseOptions::script())
            .expect_err("script goal should reject export");
        assert!(err.message().contains("parse error"));
        assert_eq!(
            err.diagnostic().kind,
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Parse);
        assert_eq!(err.diagnostic().error_type, "SyntaxError");
        assert_eq!(err.diagnostic().code, "P_PARSE_MALFORMED");
    }

    #[test]
    fn parser_rejects_obvious_function_syntax_error() {
        let err = parse("function {", ParseOptions::script())
            .expect_err("broken function syntax should fail");
        assert!(err.message().contains("parse error"));
        assert_eq!(
            err.diagnostic().kind,
            ParseDiagnosticKind::MalformedJavaScript
        );
    }

    #[test]
    fn syntax_error_reports_structured_diagnostic_with_byte_span_when_available() {
        let err =
            parse("let x = ;", ParseOptions::script()).expect_err("broken initializer should fail");
        assert_eq!(
            err.diagnostic().kind,
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Parse);
        assert_eq!(err.diagnostic().error_type, "SyntaxError");
        assert_eq!(err.diagnostic().code, "P_PARSE_MALFORMED");
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
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type, "SyntaxError");
        assert_eq!(err.diagnostic().code, "E_OBJECT_DUPLICATE_PROTO");
        assert!(
            err.diagnostic().span.is_some(),
            "diagnostic should carry Boa's source position when available: {err:?}"
        );
    }

    #[test]
    fn parser_label_static_semantics_errors_report_early_phase() {
        let err = parse("break;", ParseOptions::script())
            .expect_err("unlabelled break outside breakable statement should fail");
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().code, "E_ILLEGAL_BREAK");

        let err = parse("continue missing;", ParseOptions::script())
            .expect_err("labelled continue outside iteration should fail");
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().code, "E_ILLEGAL_CONTINUE");

        let err = parse(
            "while (false) { continue missing; }",
            ParseOptions::script(),
        )
        .expect_err("continue to undefined label should fail");
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().code, "E_UNDEFINED_CONTINUE_TARGET");
    }

    #[test]
    fn parser_duplicate_lexical_declaration_reports_early_phase() {
        let err = parse("let x; let x;", ParseOptions::script())
            .expect_err("duplicate lexical declaration should fail");
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().code, "E_DUPLICATE_LEXICAL_DECLARATION");
    }

    #[test]
    fn parser_rejects_unbalanced_delimiters() {
        let err = parse("if (true {", ParseOptions::script())
            .expect_err("unbalanced delimiters should fail");
        assert!(err.message().contains("parse error"));
        assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Parse);
    }

    #[test]
    fn nul_byte_reports_structured_malformed_diagnostic_with_span() {
        let err = parse("let x = 0;\0", ParseOptions::script())
            .expect_err("NUL byte should be rejected before Boa parsing");
        assert_eq!(
            err.diagnostic().kind,
            ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(err.diagnostic().code, "P_PARSE_MALFORMED");
        assert_eq!(
            err.diagnostic().span,
            Some(SourceSpan { start: 10, end: 11 })
        );
    }

    #[test]
    fn parser_abort_becomes_parse_error() {
        let source = r#"
var ref = async (aFalse = falseCount +=1, aString = stringCount += 1, aNaN = nanCount += 1, a0 = zeroCount += 1, aNull = nullCount += 1, aObj = objCount +=1) => {};
"#;

        let result = std::panic::catch_unwind(|| parse(source, ParseOptions::script()));
        let parsed = result.expect("frontend parser boundary should not unwind");
        if let Err(err) = parsed {
            assert!(err.message().contains("parse unsupported"));
            assert_eq!(
                err.diagnostic().kind,
                ParseDiagnosticKind::UnsupportedParserFeature
            );
            assert_eq!(err.diagnostic().code, "P_PARSE_UNSUPPORTED");
            assert_eq!(err.diagnostic().phase, ParseDiagnosticPhase::Parse);
        }
    }

    #[test]
    fn parser_accepts_simple_module_syntax() {
        parse("export const value = 1;", ParseOptions::module())
            .expect("module goal should accept export");
    }
}
