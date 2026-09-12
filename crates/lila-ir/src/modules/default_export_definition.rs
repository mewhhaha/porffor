//! Anonymous default export definitions retain their name and instantiation phase.

use crate::*;

use super::evaluation_mode::ModuleMaterializationModeIr;
use super::record::DefaultExportFormIr;

/// Exact definition spans in the linked Script, measured in Boa's UTF-16 offsets.
#[derive(Debug, Default)]
pub(crate) struct DefaultExportDefinitions(BTreeMap<(usize, usize), DefaultExportEvaluation>);

#[derive(Debug, Clone)]
enum DefaultExportEvaluation {
    Expression,
    HoistedFunction { binding_name: String },
}

impl DefaultExportDefinitions {
    pub(super) fn rewrite_body(
        source: &str,
        rewrite: super::source::DefaultExportRewrite<'_>,
    ) -> Result<String, String> {
        let declaration = if matches!(rewrite, super::source::DefaultExportRewrite::Bind { .. }) {
            let parsed = lila_front::parse(source, lila_front::ParseOptions::module())
                .map_err(|error| format!("default export module did not parse: {error}"))?;
            let ParsedSource::Module(parsed) = parsed else {
                unreachable!("Module parse options produce Module syntax")
            };
            parsed.with_compiler_session(|module, _| {
                module.items().items().iter().find_map(|item| {
                    let ModuleItem::ExportDeclaration(export) = item else {
                        return None;
                    };
                    let span = match export.as_ref() {
                        ExportDeclaration::DefaultFunctionDeclaration(function) => {
                            function.linear_span()
                        }
                        ExportDeclaration::DefaultGeneratorDeclaration(function) => {
                            function.linear_span()
                        }
                        ExportDeclaration::DefaultAsyncFunctionDeclaration(function) => {
                            function.linear_span()
                        }
                        ExportDeclaration::DefaultAsyncGeneratorDeclaration(function) => {
                            function.linear_span()
                        }
                        ExportDeclaration::DefaultClassDeclaration(class) => class.linear_span(),
                        _ => return None,
                    };
                    Some(span)
                })
            })
        } else {
            None
        };
        let mut body =
            super::source::strip_module_syntax(source, rewrite).map_err(|error| error.reason)?;
        if let Some(declaration) = declaration {
            // A declaration needs no trailing semicolon, but its rewritten
            // variable initializer does. The scanner keeps byte offsets stable;
            // insert only after it finishes, at the parsed definition boundary.
            // The later dynamic-import pass rescans this resulting text.
            let byte = source_byte_range_from_utf16_span(source, declaration).end;
            body.insert(byte, ';');
        }
        Ok(body)
    }

    pub(super) fn record_body(
        &mut self,
        body: &str,
        module: ModuleUnitId,
        mode: ModuleMaterializationModeIr,
        form: DefaultExportFormIr,
        preceding_source: &str,
    ) -> Result<(), String> {
        let evaluation = match form {
            DefaultExportFormIr::Absent | DefaultExportFormIr::Named => return Ok(()),
            DefaultExportFormIr::Anonymous { hoisted: false } => {
                DefaultExportEvaluation::Expression
            }
            DefaultExportFormIr::Anonymous { hoisted: true } => {
                DefaultExportEvaluation::HoistedFunction {
                    binding_name: MergedName::anonymous_default(module).as_str().to_string(),
                }
            }
        };
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
            self.0.insert(
                (span.start().pos() + offset, span.end().pos() + offset),
                evaluation,
            );
        } else if matches!(evaluation, DefaultExportEvaluation::HoistedFunction { .. }) {
            return Err("hoistable default export must retain its anonymous definition".into());
        }
        Ok(())
    }

    pub(super) fn prepend(&mut self, prefix: &str) {
        let offset = prefix.encode_utf16().count();
        self.0 = self
            .0
            .iter()
            .map(|(&(start, end), evaluation)| ((start + offset, end + offset), evaluation.clone()))
            .collect();
    }

    pub(crate) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        if self.0.is_empty() {
            return;
        }
        struct Definitions<'a, 'b> {
            remaining: BTreeMap<(usize, usize), DefaultExportEvaluation>,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for Definitions<'a, '_> {
            type BreakTy = ();

            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                if let Some((_, span, key)) = definition(expression) {
                    if let Some(evaluation) = self
                        .remaining
                        .remove(&(span.start().pos(), span.end().pos()))
                    {
                        match key {
                            DefinitionKey::Function(key) => {
                                let id = self.analysis.function_expr_ids[&key].clone();
                                self.analysis
                                    .function_plans
                                    .get_mut(&id)
                                    .expect("default export function is analyzed")
                                    .name = "default".into();
                                if let DefaultExportEvaluation::HoistedFunction { binding_name } =
                                    evaluation
                                {
                                    self.analysis
                                        .hoist_default_export_function(id, binding_name);
                                }
                            }
                            DefinitionKey::Class(key) => {
                                assert!(
                                    matches!(evaluation, DefaultExportEvaluation::Expression),
                                    "class default exports initialize during evaluation"
                                );
                                let id = self.analysis.class_execution_ids[&key].clone();
                                self.analysis.default_export_class_ids.insert(id);
                            }
                        }
                    }
                }
                expression.visit_with(self)
            }
        }
        let mut definitions = Definitions {
            remaining: self.0.clone(),
            analysis,
        };
        let _ = script.visit_with(&mut definitions);
        assert!(
            definitions.remaining.is_empty(),
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
    fn hoist_default_export_function(&mut self, id: FunctionId, binding_name: String) {
        let plan = self
            .function_plans
            .get_mut(&id)
            .expect("default export is analyzed");
        assert!(
            matches!(
                plan.protocol,
                FunctionProtocolIr::OrdinaryCallAndConstruct
                    | FunctionProtocolIr::Generator
                    | FunctionProtocolIr::Async
                    | FunctionProtocolIr::AsyncGenerator
            ),
            "only hoistable declarations enter module instantiation"
        );
        // The rewritten var already owns the binding and capture storage. Move
        // initialization into that owner's existing declaration-instantiation
        // list without changing the anonymous callable's exact source text.
        plan.is_expression = false;
        let owner = plan.parent_owner_id.clone();
        let function = PendingFunction {
            id: id.clone(),
            name: binding_name.clone(),
            to_string_representation: plan.to_string_representation.clone(),
            protocol: plan.protocol,
            strict: plan.strict,
            self_binding_name: plan.self_binding_name.clone(),
            parameters: plan.parameters,
            body: plan.body,
            is_expression: false,
            capture_aliases: BTreeMap::new(),
        };
        self.owner_plans
            .get_mut(&owner)
            .expect("default export owner is analyzed")
            .function_bindings
            .insert(binding_name, id.clone());
        if owner == SCRIPT_OWNER_ID {
            self.script_root_functions.push(function);
        } else {
            self.function_plans
                .get_mut(&owner)
                .expect("default export wrapper is analyzed")
                .root_functions
                .push(function);
        }
        self.hoisted_default_export_function_ids.insert(id);
    }

    pub(crate) fn is_hoisted_default_export_initializer(&self, item: &StatementListItem) -> bool {
        if self.hoisted_default_export_function_ids.is_empty() {
            return false;
        }
        let StatementListItem::Statement(statement) = item else {
            return false;
        };
        let Statement::Var(declaration) = statement.as_ref() else {
            return false;
        };
        let [variable] = declaration.0.as_ref() else {
            return false;
        };
        let Some(expression) = variable.init() else {
            return false;
        };
        let Some((_, _, DefinitionKey::Function(key))) = definition(expression.flatten()) else {
            return false;
        };
        self.function_expr_ids
            .get(&key)
            .is_some_and(|id| self.hoisted_default_export_function_ids.contains(id))
    }

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
