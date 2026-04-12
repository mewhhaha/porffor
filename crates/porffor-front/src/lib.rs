use boa_ast::scope::Scope;
use boa_interner::Interner;
use boa_parser::{Parser, Source};

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

    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let source = if let Some(filename) = &options.filename {
        Source::from_bytes(source_text.as_bytes()).with_path(std::path::Path::new(filename))
    } else {
        Source::from_bytes(source_text.as_bytes())
    };

    match options.goal {
        ParseGoal::Script => {
            Parser::new(source)
                .parse_script(&scope, &mut interner)
                .map_err(|err| ParseError::new(format!("parse error: {err}")))?;
        }
        ParseGoal::Module => {
            Parser::new(source)
                .parse_module(&scope, &mut interner)
                .map_err(|err| ParseError::new(format!("parse error: {err}")))?;
        }
    }

    Ok(SourceUnit {
        goal: options.goal,
        filename: options.filename,
        source_text,
    })
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
