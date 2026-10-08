use super::*;
use crate::generator_for_of_iterator::{
    GeneratorForOfAssignmentIr, GeneratorForOfIteratorHeadInputIr, GeneratorForOfLexicalPatternIr,
};

pub(super) enum GeneratorForOfHeadSource<'a> {
    Binding {
        source_name: &'a str,
        binding: ForOfAssignmentIr,
        head_environment: Option<ForInOfEnvironmentIr>,
    },
    IdentifierAssignment {
        source_name: &'a str,
        value_name: String,
        head_environment: Option<ForInOfEnvironmentIr>,
    },
    OrdinaryProperty {
        value_name: String,
        head_environment: Option<ForInOfEnvironmentIr>,
    },
    LexicalPattern(ValidatedResumableSyncForOfLexicalPatternIr),
}

impl<'a> ScriptLowerer<'a> {
    pub(in crate::lowering) fn plain_generator_entry_state(&self) -> Option<u32> {
        self.function_signatures
            .get(&self.current_owner_id)
            .filter(|signature| {
                signature.protocol.execution_kind() == FunctionExecutionKind::Generator
            })
            .and(self.current_generator_resume_state)
    }

    pub(super) fn lower_generator_for_of_iterator(
        &mut self,
        source: GeneratorForOfHeadSource<'_>,
        iterable: TypedExpr,
        body: StatementIr,
        body_kind: ValueKind,
        entry_state: u32,
    ) -> ForOfLoweringIr {
        // Keep lexical blocks intact: suspended closures must retain their cells.
        let statements = match body {
            StatementIr::Block(block) if block.lexical_environment.is_none() => block.statements,
            statement => vec![statement],
        };
        let body = match GeneratorForOfBodyIr::new(statements, entry_state) {
            Ok(body) => body,
            Err(error) => {
                self.unsupported(&format!(
                    "invalid generator for-of body continuation: {error:?}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
        };
        if self.current_generator_resume_state != Some(body.exit_state()) {
            self.unsupported("generator for-of lowered body and state plan disagree");
            return ForOfLoweringIr::no_iteration();
        }
        let persistent = self
            .generated_owned_env_bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .chain(
                self.analysis
                    .owner_plans
                    .values()
                    .flat_map(|owner| owner.owned_env_slots.keys().map(String::as_str)),
            )
            .chain(self.captured_binding_positions.keys().map(String::as_str));
        let head = match source {
            GeneratorForOfHeadSource::Binding {
                source_name,
                binding,
                head_environment,
            } => {
                match ValidatedResumableSyncForOfBindingIr::new(
                    source_name,
                    binding,
                    head_environment,
                ) {
                    Ok(head) => GeneratorForOfIteratorHeadInputIr::Binding(head),
                    Err(error) => {
                        self.unsupported(&format!(
                            "invalid generator for-of binding head: {error:?}"
                        ));
                        return ForOfLoweringIr::no_iteration();
                    }
                }
            }
            GeneratorForOfHeadSource::LexicalPattern(pattern) => {
                match GeneratorForOfLexicalPatternIr::new(
                    pattern,
                    persistent,
                    &self.generated_functions,
                ) {
                    Ok(head) => GeneratorForOfIteratorHeadInputIr::LexicalPattern(head),
                    Err(error) => {
                        self.unsupported(&format!(
                            "invalid generator lexical-pattern prefix: {error:?}"
                        ));
                        return ForOfLoweringIr::no_iteration();
                    }
                }
            }
            GeneratorForOfHeadSource::IdentifierAssignment {
                source_name,
                value_name,
                head_environment,
            } => {
                let Some(prefix) = body.statements().first() else {
                    self.unsupported("generator for-of assignment prefix is absent");
                    return ForOfLoweringIr::no_iteration();
                };
                let ignored = self
                    .lookup_binding(source_name)
                    .filter(|binding| {
                        binding.mode == BindingMode::Const
                            && self
                                .sloppy_immutable_binding_storage_names
                                .contains(&binding.storage_name)
                            && !self.reference_strictness().throws_on_failed_set()
                    })
                    .map(|_| {
                        IdentifierWriteReferenceIr::ignored_immutable_binding(
                            source_name.to_string(),
                        )
                    });
                match GeneratorForOfAssignmentIr::identifier(
                    source_name,
                    value_name,
                    prefix,
                    head_environment.as_ref(),
                    ignored.as_ref(),
                    persistent,
                    &self.generated_functions,
                ) {
                    Ok(head) => GeneratorForOfIteratorHeadInputIr::Assignment(head),
                    Err(error) => {
                        self.unsupported(&format!(
                            "invalid generator for-of assignment: {error:?}"
                        ));
                        return ForOfLoweringIr::no_iteration();
                    }
                }
            }
            GeneratorForOfHeadSource::OrdinaryProperty {
                value_name,
                head_environment,
            } => {
                let Some(prefix) = body.statements().first() else {
                    self.unsupported("generator for-of assignment prefix is absent");
                    return ForOfLoweringIr::no_iteration();
                };
                match GeneratorForOfAssignmentIr::ordinary_property(
                    value_name,
                    prefix,
                    head_environment.as_ref(),
                    persistent,
                    &self.generated_functions,
                ) {
                    Ok(head) => GeneratorForOfIteratorHeadInputIr::Assignment(head),
                    Err(error) => {
                        self.unsupported(&format!(
                            "invalid generator for-of assignment: {error:?}"
                        ));
                        return ForOfLoweringIr::no_iteration();
                    }
                }
            }
        };
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        let plan = match GeneratorForOfIteratorPlanIr::new(head, record, body) {
            Ok(plan) => plan,
            Err(error) => {
                self.unsupported(&format!("invalid generator for-of exit state: {error:?}"));
                return ForOfLoweringIr::no_iteration();
            }
        };
        match plan.value_storage() {
            GeneratorForOfIteratorValueStorageIr::Activation(binding) => {
                self.add_suspension_owned_binding(binding.name.clone())
            }
            GeneratorForOfIteratorValueStorageIr::IterationEnvironment(_)
            | GeneratorForOfIteratorValueStorageIr::EntryLocal { .. } => {}
        }
        self.current_generator_resume_state = Some(plan.exit_state());
        ForOfLoweringIr::generator_iterator(iterable, plan, body_kind)
    }
}
