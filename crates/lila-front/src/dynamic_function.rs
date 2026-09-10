use boa_interner::Interner;
use boa_parser::{Parser, Source};

use crate::{parse, parse_with_boundary, ParseError, ParseOptions, ParsedScript, ParsedSource};

/// The four independent grammar goals of CreateDynamicFunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionParseKind {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}

impl FunctionParseKind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Ordinary => "function",
            Self::Generator => "function*",
            Self::Async => "async function",
            Self::AsyncGenerator => "async function*",
        }
    }

    /// Source retained by Function.prototype.toString after successful preparation.
    #[must_use]
    pub fn source_text(self, arguments: &[String]) -> String {
        let (body, parameters) = arguments
            .split_last()
            .map_or(("", &[][..]), |(body, parameters)| {
                (body.as_str(), parameters)
            });
        format!(
            "{} anonymous({}\n) {{\n{body}\n}}",
            self.prefix(),
            parameters.join(",")
        )
    }
}

/// Parses already-coerced constructor arguments without inheriting caller syntax.
///
/// Parameters and body must each parse in isolation before the expression is
/// parsed for combined early errors. The expression is unnamed: `anonymous` is
/// its eventual display name, not a lexical binding visible to its body.
pub fn prepare_dynamic_function(
    kind: FunctionParseKind,
    arguments: &[String],
) -> Result<ParsedScript, ParseError> {
    let (body, parameters) = arguments
        .split_last()
        .map_or(("", &[][..]), |(body, parameters)| {
            (body.as_str(), parameters)
        });
    let parameters = parameters.join(",");
    let body = format!("\n{body}\n");
    let (allow_yield, allow_await) = match kind {
        FunctionParseKind::Ordinary => (false, false),
        FunctionParseKind::Generator => (true, false),
        FunctionParseKind::Async => (false, true),
        FunctionParseKind::AsyncGenerator => (true, true),
    };
    let mut interner = Interner::default();
    parse_with_boundary(&parameters, || {
        Parser::new(Source::from_bytes(parameters.as_bytes())).parse_formal_parameters(
            &mut interner,
            allow_yield,
            allow_await,
        )
    })?;
    parse_with_boundary(&body, || {
        Parser::new(Source::from_bytes(body.as_bytes())).parse_function_body(
            &mut interner,
            allow_yield,
            allow_await,
        )
    })?;

    let expression = format!("({} ({parameters}\n) {{{body}}});", kind.prefix());
    match parse(expression, ParseOptions::script())? {
        ParsedSource::Script(script) => Ok(script),
        ParsedSource::Module(_) => unreachable!("Script parse goal produces Script syntax"),
    }
}
