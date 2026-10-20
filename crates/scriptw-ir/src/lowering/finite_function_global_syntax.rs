//! Source grammar hypotheses survive effects that erase live method shapes.
//! This projection never contributes an IR value or a runtime callable target.

use super::*;
use crate::prepared_source_cache::{FunctionGlobalConstructorHints, MAX_SOURCE_CANDIDATES};

#[derive(Clone, Copy)]
enum GrammarSource {
    Function(DynamicFunctionKind),
    Prototype(DynamicFunctionKind),
    Constructor(DynamicFunctionKind),
    PrototypeReader,
}

struct SourceGrammar<'a, 'syntax> {
    analysis: &'a Analysis<'syntax>,
    interner: &'a Interner,
    aliases: BTreeMap<String, GrammarSource>,
}

impl SourceGrammar<'_, '_> {
    fn source(&self, expression: &Expression) -> Option<GrammarSource> {
        let source = ScriptLowerer::unwrap_parenthesized_expr(expression);
        let kind = match source {
            Expression::FunctionExpression(_) | Expression::ArrowFunction(_) => {
                Some(DynamicFunctionKind::Ordinary)
            }
            Expression::GeneratorExpression(_) => Some(DynamicFunctionKind::Generator),
            Expression::AsyncFunctionExpression(_) | Expression::AsyncArrowFunction(_) => {
                Some(DynamicFunctionKind::Async)
            }
            Expression::AsyncGeneratorExpression(_) => Some(DynamicFunctionKind::AsyncGenerator),
            _ => None,
        };
        if let Some(kind) = kind {
            return Some(GrammarSource::Function(kind));
        }
        let root = &self.analysis.owner_plans[SCRIPT_OWNER_ID];
        match source {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                if let Some(source) = self.aliases.get(&name) {
                    return Some(*source);
                }
                if name == "Function" && !root.root_bindings.contains(&name) {
                    return Some(GrammarSource::Constructor(DynamicFunctionKind::Ordinary));
                }
                let function_id = root.function_bindings.get(&name)?;
                let plan = self.analysis.function_plans.get(function_id)?;
                Some(GrammarSource::Function(
                    match plan.protocol.source_execution_kind() {
                        FunctionExecutionKind::Ordinary => DynamicFunctionKind::Ordinary,
                        FunctionExecutionKind::Generator => DynamicFunctionKind::Generator,
                        FunctionExecutionKind::Async => DynamicFunctionKind::Async,
                        FunctionExecutionKind::AsyncGenerator => {
                            DynamicFunctionKind::AsyncGenerator
                        }
                    },
                ))
            }
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let PropertyAccessField::Const(field) = access.field() else {
                    return None;
                };
                let field = self.interner.resolve_expect(field.sym()).to_string();
                if field == "getPrototypeOf" {
                    let Expression::Identifier(identifier) =
                        ScriptLowerer::unwrap_parenthesized_expr(access.target())
                    else {
                        return None;
                    };
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    // Only the standard global namespace patterns supply this
                    // hypothesis. A declaration named Object/Reflect cannot.
                    return (matches!(name.as_str(), "Object" | "Reflect")
                        && !root.root_bindings.contains(&name))
                    .then_some(GrammarSource::PrototypeReader);
                }
                if field != "constructor" {
                    return None;
                }
                match self.source(access.target())? {
                    GrammarSource::Function(kind) | GrammarSource::Prototype(kind) => {
                        Some(GrammarSource::Constructor(kind))
                    }
                    GrammarSource::Constructor(_) | GrammarSource::PrototypeReader => None,
                }
            }
            Expression::Call(call) => {
                if !matches!(
                    self.source(call.function())?,
                    GrammarSource::PrototypeReader
                ) {
                    return None;
                }
                match self.source(call.args().first()?)? {
                    GrammarSource::Function(kind) => Some(GrammarSource::Prototype(kind)),
                    GrammarSource::Prototype(_)
                    | GrammarSource::Constructor(_)
                    | GrammarSource::PrototypeReader => None,
                }
            }
            _ => None,
        }
    }
}

pub(super) fn source_function_global_hypotheses(
    script: &Script,
    interner: &Interner,
    analysis: &Analysis<'_>,
) -> FunctionGlobalConstructorHints {
    let mut hints = FunctionGlobalConstructorHints::default();
    let root = &analysis.owner_plans[SCRIPT_OWNER_ID];
    if !matches!(
        analysis.script_instantiation,
        ScriptInstantiation::FreshEntry
    ) || !analysis
        .script_instantiation
        .has_global_variable_environment(root.strict)
    {
        return hints;
    }
    let mut source = SourceGrammar {
        analysis,
        interner,
        aliases: BTreeMap::new(),
    };
    // Only actual top-level var initializers are projected. Blocks, with,
    // function activations, global lexical cells and eval contexts are absent.
    for item in script.statements().statements() {
        let StatementListItem::Statement(statement) = item else {
            continue;
        };
        let Statement::Var(declaration) = statement.as_ref() else {
            continue;
        };
        for variable in declaration.0.as_ref() {
            let Binding::Identifier(identifier) = variable.binding() else {
                continue;
            };
            let Some(value) = variable.init().and_then(|value| source.source(value)) else {
                continue;
            };
            let name = interner.resolve_expect(identifier.sym()).to_string();
            if analysis.module_execution.is_private_dispatcher_name(&name) {
                continue;
            }
            if let GrammarSource::Constructor(kind) = value {
                hints.observe(&name, kind);
            }
            if source.aliases.len() < MAX_SOURCE_CANDIDATES || source.aliases.contains_key(&name) {
                source.aliases.insert(name, value);
            }
        }
    }
    hints
}
