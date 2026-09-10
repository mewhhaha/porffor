use super::*;
use boa_ast::operations::{
    all_private_identifiers_valid, contains, contains_arguments, ContainsSymbol,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalInvocationContext {
    Script,
    Function,
    Method,
    DerivedConstructor,
    ClassFieldInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectEvalParseContext {
    pub strict_caller: bool,
    pub invocation: EvalInvocationContext,
    pub private_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalParseContext {
    Indirect,
    Direct(DirectEvalParseContext),
}

pub fn prepare_eval_source(
    source_text: String,
    context: &EvalParseContext,
) -> Result<ParsedScript, ParseError> {
    let mut interner = Interner::default();
    let mut parser = Parser::new(Source::from_bytes(source_text.as_bytes()));
    if matches!(context, EvalParseContext::Direct(context) if context.strict_caller) {
        parser.set_strict();
    }
    let direct = matches!(context, EvalParseContext::Direct(_));
    let (mut ast, _) =
        parse_with_boundary(&source_text, || parser.parse_eval(direct, &mut interner))?;
    if let EvalParseContext::Direct(context) = context {
        let invalid_syntax = match context.invocation {
            EvalInvocationContext::Script if contains(&ast, ContainsSymbol::NewTarget) => {
                Some("invalid new.target usage in Script caller")
            }
            EvalInvocationContext::ClassFieldInitializer if contains_arguments(&ast) => {
                Some("arguments is not allowed in a class field initializer")
            }
            _ => None,
        };
        let invalid_syntax = invalid_syntax
            .or_else(|| {
                (!matches!(
                    context.invocation,
                    EvalInvocationContext::Method
                        | EvalInvocationContext::DerivedConstructor
                        | EvalInvocationContext::ClassFieldInitializer
                ) && contains(&ast, ContainsSymbol::SuperProperty))
                .then_some("invalid super property in eval caller")
            })
            .or_else(|| {
                (context.invocation != EvalInvocationContext::DerivedConstructor
                    && contains(&ast, ContainsSymbol::SuperCall))
                .then_some("invalid super call in eval caller")
            });
        if let Some(message) = invalid_syntax {
            return Err(ParseError::malformed(message, None));
        }
        let names = context
            .private_names
            .iter()
            .map(|name| interner.get_or_intern(name.as_str()))
            .collect();
        if !all_private_identifiers_valid(&ast, names) {
            return Err(ParseError::malformed(
                "invalid private identifier usage",
                None,
            ));
        }
    }
    // Lila owns caller binding resolution; Boa's local scope facts still need
    // initializing for nested functions in this independently parsed Script.
    ast.analyze_scope(&Scope::new_global(), &interner)
        .map_err(|reason| {
            ParseError::malformed(format!("invalid scope analysis: {reason}"), None)
        })?;
    Ok(ParsedScript {
        source: SourceUnit {
            goal: ParseGoal::Script,
            filename: None,
            source_text,
        },
        syntax: Rc::new(ScriptSyntax { ast, interner }),
    })
}
