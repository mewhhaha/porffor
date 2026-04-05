use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions as OxcParseOptions, Parser};
use oxc_span::SourceType;

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
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
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
        return Err(ParseError::new(
            "source contains NUL byte, front-end rejects this input",
        ));
    }

    if matches!(options.goal, ParseGoal::Script) {
        let trimmed = source_text.trim_start();
        if trimmed.starts_with("export ") || trimmed.starts_with("import ") {
            return Err(ParseError::new(
                "parse error: import/export syntax requires module goal",
            ));
        }
    }

    let allocator = Allocator::default();
    let source_type = source_type_for(&options);
    let parser_return = Parser::new(&allocator, &source_text, source_type)
        .with_options(OxcParseOptions {
            parse_regular_expression: true,
            ..OxcParseOptions::default()
        })
        .parse();

    if parser_return.panicked {
        return Err(ParseError::new("parse error: parser panicked"));
    }

    if let Some(error) = parser_return.errors.into_iter().next() {
        return Err(ParseError::new(format!("parse error: {error}")));
    }

    Ok(SourceUnit {
        goal: options.goal,
        filename: options.filename,
        source_text,
    })
}

fn source_type_for(options: &ParseOptions) -> SourceType {
    if let Some(filename) = &options.filename {
        if let Ok(source_type) = SourceType::from_path(filename) {
            return match options.goal {
                ParseGoal::Script => source_type.with_script(true),
                ParseGoal::Module => source_type.with_module(true),
            };
        }
    }

    match options.goal {
        ParseGoal::Script => SourceType::script(),
        ParseGoal::Module => SourceType::mjs(),
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
    }

    #[test]
    fn parser_rejects_obvious_function_syntax_error() {
        let err = parse("function {", ParseOptions::script())
            .expect_err("broken function syntax should fail");
        assert!(err.message().contains("parse error"));
    }

    #[test]
    fn parser_rejects_unbalanced_delimiters() {
        let err = parse("if (true {", ParseOptions::script())
            .expect_err("unbalanced delimiters should fail");
        assert!(err.message().contains("parse error"));
    }

    #[test]
    fn parser_accepts_simple_module_syntax() {
        parse("export const value = 1;", ParseOptions::module())
            .expect("module goal should accept export");
    }
}
