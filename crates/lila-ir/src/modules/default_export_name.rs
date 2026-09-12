//! NamedEvaluation of anonymous default exports, independent of their merged binding names.

use crate::*;

use super::evaluation_mode::ModuleMaterializationModeIr;

/// Exact definition spans in the linked Script, measured in Boa's UTF-16 offsets.
#[derive(Debug, Default)]
pub(crate) struct DefaultExportNames(BTreeSet<(usize, usize)>);

impl DefaultExportNames {
    pub(super) fn record_body(
        &mut self,
        body: &str,
        module: ModuleUnitId,
        mode: ModuleMaterializationModeIr,
        preceding_source: &str,
    ) -> Result<(), String> {
        // Rewrites of import.meta/import() may change offsets. Inspect the final
        // body, and only its module scope (or the known deferred-module thunk).
        let parsed = lila_front::parse(body, lila_front::ParseOptions::module())
            .map_err(|error| format!("rewritten default export did not parse: {error}"))?;
        let ParsedSource::Module(parsed) = parsed else {
            unreachable!("Module parse options produce Module syntax")
        };
        let span = parsed.with_compiler_session(|module_ast, interner| {
            let binding = MergedName::anonymous_default(module);
            let deferred = MergedName::minted(module, UnitCellRole::DeferEvaluate);
            let mut initializer = None;
            for item in module_ast.items().items() {
                let ModuleItem::StatementListItem(statement) = item else {
                    continue;
                };
                match mode {
                    ModuleMaterializationModeIr::Eager => {
                        initializer = default_initializer(statement, binding.as_str(), interner)
                            .or(initializer);
                    }
                    ModuleMaterializationModeIr::Deferred => {
                        if let StatementListItem::Declaration(declaration) = statement {
                            if let Declaration::FunctionDeclaration(function) = declaration.as_ref()
                            {
                                if interner.resolve_expect(function.name().sym()).to_string()
                                    == deferred.as_str()
                                {
                                    initializer =
                                        function.body().statements().iter().find_map(|statement| {
                                            default_initializer(
                                                statement,
                                                binding.as_str(),
                                                interner,
                                            )
                                        });
                                }
                            }
                        }
                    }
                }
            }
            let (binding, expression) = initializer
                .ok_or_else(|| "rewritten default export has no module binding".to_string())?;
            // Boa gives an anonymous initializer the declaration's Identifier,
            // including its span. A same-spelled explicit name is distinct.
            Ok::<_, String>(
                definition(expression)
                    .and_then(|(name, span, _)| (name == Some(*binding)).then_some(span)),
            )
        })?;
        if let Some(span) = span {
            let offset = preceding_source.encode_utf16().count();
            self.0
                .insert((span.start().pos() + offset, span.end().pos() + offset));
        }
        Ok(())
    }

    pub(super) fn prepend(&mut self, prefix: &str) {
        let offset = prefix.encode_utf16().count();
        self.0 = self
            .0
            .iter()
            .map(|(start, end)| (start + offset, end + offset))
            .collect();
    }

    pub(crate) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        if self.0.is_empty() {
            return;
        }
        struct Naming<'a, 'b> {
            remaining: BTreeSet<(usize, usize)>,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for Naming<'a, '_> {
            type BreakTy = ();

            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                if let Some((_, span, key)) = definition(expression) {
                    if self
                        .remaining
                        .remove(&(span.start().pos(), span.end().pos()))
                    {
                        match key {
                            DefinitionKey::Function(key) => {
                                let id = &self.analysis.function_expr_ids[&key];
                                self.analysis
                                    .function_plans
                                    .get_mut(id)
                                    .expect("default export function is analyzed")
                                    .name = "default".into();
                            }
                            DefinitionKey::Class(key) => {
                                let id = self.analysis.class_execution_ids[&key].clone();
                                self.analysis.default_export_class_ids.insert(id);
                            }
                        }
                    }
                }
                expression.visit_with(self)
            }
        }
        let mut naming = Naming {
            remaining: self.0.clone(),
            analysis,
        };
        let _ = script.visit_with(&mut naming);
        assert!(
            naming.remaining.is_empty(),
            "linked default export definition spans must survive Script parsing"
        );
    }
}

fn default_initializer<'a>(
    statement: &'a StatementListItem,
    name: &str,
    interner: &Interner,
) -> Option<(&'a boa_ast::expression::Identifier, &'a Expression)> {
    let variables = match statement {
        StatementListItem::Declaration(declaration) => match declaration.as_ref() {
            Declaration::Lexical(declaration) => declaration.variable_list(),
            _ => return None,
        },
        StatementListItem::Statement(statement) => match statement.as_ref() {
            Statement::Var(declaration) => &declaration.0,
            _ => return None,
        },
    };
    variables.as_ref().iter().find_map(|variable| {
        let Binding::Identifier(identifier) = variable.binding() else {
            return None;
        };
        (interner.resolve_expect(identifier.sym()).to_string() == name)
            .then(|| {
                variable
                    .init()
                    .map(|expression| (identifier, expression.flatten()))
            })
            .flatten()
    })
}

enum DefinitionKey {
    Function(String),
    Class(String),
}

fn definition(
    expression: &Expression,
) -> Option<(
    Option<boa_ast::expression::Identifier>,
    boa_ast::LinearSpan,
    DefinitionKey,
)> {
    let (name, span, key) = match expression {
        Expression::FunctionExpression(function) => (
            function.name(),
            function.linear_span()?,
            function_expression_key(function),
        ),
        Expression::GeneratorExpression(function) => (
            function.name(),
            function.linear_span(),
            generator_expression_key(function),
        ),
        Expression::AsyncFunctionExpression(function) => (
            function.name(),
            function.linear_span(),
            async_function_expression_key(function),
        ),
        Expression::AsyncGeneratorExpression(function) => (
            function.name(),
            function.linear_span(),
            async_generator_expression_key(function),
        ),
        Expression::ArrowFunction(function) => (
            function.name(),
            function.linear_span(),
            arrow_function_key(function),
        ),
        Expression::AsyncArrowFunction(function) => (
            function.name(),
            function.linear_span(),
            async_arrow_function_key(function),
        ),
        Expression::ClassExpression(class) => {
            return Some((
                class.name(),
                class.linear_span(),
                DefinitionKey::Class(
                    class
                        .constructor()
                        .map(class_constructor_key)
                        .unwrap_or_else(|| class_default_constructor_key(class.linear_span())),
                ),
            ))
        }
        _ => return None,
    };
    Some((name, span, DefinitionKey::Function(key)))
}

impl Analysis<'_> {
    pub(crate) fn class_display_name<'a>(
        &self,
        constructor: &FunctionId,
        binding_name: &'a Option<String>,
    ) -> Option<&'a str> {
        if self.default_export_class_ids.contains(constructor) {
            Some("default")
        } else {
            binding_name.as_deref()
        }
    }
}
