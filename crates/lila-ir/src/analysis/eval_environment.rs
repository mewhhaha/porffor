use super::*;
use boa_ast::expression::Identifier;
use boa_ast::visitor::{VisitWith, Visitor};
use core::ops::ControlFlow;

pub(super) fn source_identifiers(script: &Script, interner: &Interner) -> BTreeSet<String> {
    struct SourceIdentifiers<'a> {
        interner: &'a Interner,
        names: BTreeSet<String>,
    }
    impl<'ast> Visitor<'ast> for SourceIdentifiers<'_> {
        type BreakTy = core::convert::Infallible;
        fn visit_identifier(&mut self, identifier: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            self.names
                .insert(self.interner.resolve_expect(identifier.sym()).to_string());
            ControlFlow::Continue(())
        }
    }
    let mut visitor = SourceIdentifiers {
        interner,
        names: BTreeSet::new(),
    };
    let _ = script.visit_with(&mut visitor);
    visitor.names
}

impl AnalysisBuilder<'_> {
    pub(super) fn finalize_eval_environment_roles(&mut self, interner: &Interner) {
        let global_variable_environment = self
            .script_instantiation
            .has_global_variable_environment(self.owner_plans[SCRIPT_OWNER_ID].strict);
        let borrowed_variable_environment = matches!(
            &self.script_instantiation,
            ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(_))
        ) && !self.owner_plans[SCRIPT_OWNER_ID].strict;
        for environment in self
            .environment_plans
            .values_mut()
            .filter(|environment| environment.eval_visible)
        {
            if environment.kind == EnvironmentKind::WithObject {
                let object = &self.with_object_environment_plans[&environment.id];
                environment.eval_environment = Some(EvalEnvironmentRoleIr::WithObject {
                    object_slot: environment.owned_env_slots[object.binding_name.as_str()],
                });
                continue;
            }
            let parameter_names = &self.owner_plans[&environment.owner_id].parameter_names;
            let bindings = environment
                .owned_env_slots
                .iter()
                .filter_map(|(storage_name, slot)| {
                    let source_name = if storage_name == LEXICAL_ARGUMENTS_NAME {
                        if environment.binding_storage_names.contains("arguments") {
                            return None;
                        }
                        "arguments".to_string()
                    } else if let Some(source_name) = self.source_names_by_storage.get(storage_name)
                    {
                        source_name.clone()
                    } else if self.source_identifiers.contains(storage_name) {
                        storage_name.clone()
                    } else {
                        return None;
                    };
                    let mode = environment.binding_modes[storage_name];
                    if matches!(
                        environment.kind,
                        EnvironmentKind::Activation | EnvironmentKind::FunctionParameters
                    ) && mode == BindingMode::Const
                        && self
                            .function_plans
                            .get(&environment.owner_id)
                            .is_some_and(|plan| {
                                plan.self_binding_name.as_deref() == Some(source_name.as_str())
                                    && (self.owner_plans[&environment.owner_id]
                                        .body_environment_id
                                        .is_some()
                                        || !lexically_declared_names(plan.body).into_iter().any(
                                            |name| {
                                                interner.resolve_expect(name).to_string()
                                                    == source_name
                                            },
                                        ))
                            })
                    {
                        return None;
                    }
                    let declaration =
                        if environment.kind == EnvironmentKind::NamedFunctionExpression {
                            EvalBindingDeclarationIr::NamedFunctionExpression
                        } else if matches!(
                            environment.kind,
                            EnvironmentKind::Activation | EnvironmentKind::FunctionParameters
                        ) && parameter_names.contains(&source_name)
                        {
                            EvalBindingDeclarationIr::Parameter
                        } else if mode == BindingMode::Var
                            || storage_name == LEXICAL_ARGUMENTS_NAME
                            || (matches!(
                                environment.kind,
                                EnvironmentKind::Activation
                                    | EnvironmentKind::FunctionParameters
                                    | EnvironmentKind::FunctionBody
                            ) && self.owner_plans[&environment.owner_id]
                                .function_bindings
                                .contains_key(&source_name))
                        {
                            EvalBindingDeclarationIr::Variable
                        } else {
                            EvalBindingDeclarationIr::Lexical
                        };
                    if environment.owner_id == SCRIPT_OWNER_ID
                        && (global_variable_environment || borrowed_variable_environment)
                        && declaration == EvalBindingDeclarationIr::Variable
                    {
                        return None;
                    }
                    Some(EvalVisibleBindingIr {
                        source_name,
                        slot: *slot,
                        mode,
                        declaration,
                    })
                })
                .collect();
            let kind = match environment.kind {
                EnvironmentKind::Activation
                    if environment.owner_id == SCRIPT_OWNER_ID && borrowed_variable_environment =>
                {
                    EvalDeclarativeEnvironmentKindIr::Lexical
                }
                EnvironmentKind::Activation
                    if environment.owner_id == SCRIPT_OWNER_ID && global_variable_environment =>
                {
                    EvalDeclarativeEnvironmentKindIr::GlobalLexical
                }
                EnvironmentKind::Activation | EnvironmentKind::FunctionParameters
                    if self.owner_plans[&environment.owner_id]
                        .parameter_eval_environment_id
                        .is_some() =>
                {
                    EvalDeclarativeEnvironmentKindIr::Parameters
                }
                EnvironmentKind::Activation
                | EnvironmentKind::FunctionParameters
                | EnvironmentKind::FunctionBody
                | EnvironmentKind::ParameterEvalVariable => {
                    EvalDeclarativeEnvironmentKindIr::Variable
                }
                EnvironmentKind::CatchParameter => EvalDeclarativeEnvironmentKindIr::Catch,
                EnvironmentKind::SimpleCatchParameter => {
                    EvalDeclarativeEnvironmentKindIr::SimpleCatch
                }
                EnvironmentKind::Block
                | EnvironmentKind::NamedFunctionExpression
                | EnvironmentKind::ClassName
                | EnvironmentKind::SwitchCaseBlock
                | EnvironmentKind::ForLexicalHead
                | EnvironmentKind::ForInOfTdzHead
                | EnvironmentKind::ForInOfIteration => EvalDeclarativeEnvironmentKindIr::Lexical,
                EnvironmentKind::WithObject => unreachable!("object record handled above"),
            };
            environment.eval_environment =
                Some(EvalEnvironmentRoleIr::Declarative { kind, bindings });
        }
    }
}

impl Analysis<'_> {
    pub(crate) fn owner_eval_environment(&self, owner_id: &str) -> Option<EvalEnvironmentRoleIr> {
        self.owner_plans.get(owner_id).and_then(|owner| {
            self.environment_plans[&owner.activation_environment_id]
                .eval_environment
                .clone()
        })
    }
}

pub(super) fn parameter_names(
    interner: &Interner,
    parameters: &[boa_ast::function::FormalParameter],
) -> BTreeSet<String> {
    parameters
        .iter()
        .flat_map(|parameter| {
            let mut names = Vec::new();
            collect_binding_names(interner, parameter.variable().binding(), &mut names);
            names
        })
        .collect()
}
