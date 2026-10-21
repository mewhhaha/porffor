use boa_interner::Interner;
use boa_parser::{Parser, Source};

use crate::{
    parse, parse_script_utf16, parse_utf16_with_boundary, parse_with_boundary, ParseError,
    ParseOptions, ParsedScript, ParsedSource,
};

/// The four independent grammar goals of CreateDynamicFunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionParseKind {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}

impl FunctionParseKind {
    const fn grammar_flags(self) -> (bool, bool) {
        match self {
            Self::Ordinary => (false, false),
            Self::Generator => (true, false),
            Self::Async => (false, true),
            Self::AsyncGenerator => (true, true),
        }
    }

    const fn prefix(self) -> &'static str {
        match self {
            Self::Ordinary => "function",
            Self::Generator => "function*",
            Self::Async => "async function",
            Self::AsyncGenerator => "async function*",
        }
    }

    /// Exact source units retained after a UTF-16 constructor preparation.
    #[must_use]
    pub fn source_text_utf16(self, arguments: &[Vec<u16>]) -> Vec<u16> {
        let (body, parameters) = arguments
            .split_last()
            .map_or((&[][..], &[][..]), |(body, parameters)| {
                (body.as_slice(), parameters)
            });
        let mut source: Vec<u16> = format!("{} anonymous(", self.prefix())
            .encode_utf16()
            .collect();
        append_parameters_utf16(&mut source, parameters);
        source.extend("\n) {\n".encode_utf16());
        source.extend_from_slice(body);
        source.extend("\n}".encode_utf16());
        source
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
    let (allow_yield, allow_await) = kind.grammar_flags();
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

fn append_parameters_utf16(destination: &mut Vec<u16>, parameters: &[Vec<u16>]) {
    for (index, parameter) in parameters.iter().enumerate() {
        if index != 0 {
            destination.push(u16::from(b','));
        }
        destination.extend_from_slice(parameter);
    }
}

/// Parses already-coerced UTF-16 arguments in each independent constructor
/// grammar. No transport markers or replacements enter the lexical grammar.
pub fn prepare_dynamic_function_utf16(
    kind: FunctionParseKind,
    arguments: &[Vec<u16>],
) -> Result<ParsedScript, ParseError> {
    let (body, parameters) = arguments
        .split_last()
        .map_or((&[][..], &[][..]), |(body, parameters)| {
            (body.as_slice(), parameters)
        });
    let mut parameter_units = Vec::new();
    append_parameters_utf16(&mut parameter_units, parameters);
    let mut body_units = vec![u16::from(b'\n')];
    body_units.extend_from_slice(body);
    body_units.push(u16::from(b'\n'));
    let (allow_yield, allow_await) = kind.grammar_flags();
    let mut interner = Interner::default();
    parse_utf16_with_boundary(&parameter_units, || {
        Parser::new(Source::from_utf16(&parameter_units)).parse_formal_parameters(
            &mut interner,
            allow_yield,
            allow_await,
        )
    })?;
    parse_utf16_with_boundary(&body_units, || {
        Parser::new(Source::from_utf16(&body_units)).parse_function_body(
            &mut interner,
            allow_yield,
            allow_await,
        )
    })?;
    let mut expression: Vec<u16> = format!("({} (", kind.prefix()).encode_utf16().collect();
    expression.extend(parameter_units);
    expression.extend("\n) {".encode_utf16());
    expression.extend(body_units);
    expression.extend("});".encode_utf16());
    parse_script_utf16(&expression)
}
