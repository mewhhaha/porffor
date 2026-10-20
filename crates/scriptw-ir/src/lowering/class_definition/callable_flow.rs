use super::*;

#[derive(Clone, Copy)]
pub(super) enum ClassConstructorInvocationRole {
    Base,
    ExplicitDerived,
    SyntheticDerived,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn finalize_class_callable_flow(
        &mut self,
        prior_class_callable_flow_effects: BTreeMap<FunctionId, SourceCallFlowEffects>,
        constructor_id: &FunctionId,
        constructor_invocation_role: ClassConstructorInvocationRole,
        class_instance_element_plan: Option<&ClassInstanceElementPlanIr>,
    ) {
        for (function_id, prior_flow_effects) in prior_class_callable_flow_effects {
            let signature = self
                .function_signatures
                .get_mut(&function_id)
                .unwrap_or_else(|| {
                    panic!(
                        "class callable signature `{function_id}` must exist after body lowering"
                    )
                });
            signature.source_call_flow_effects = signature
                .source_call_flow_effects
                .merge_observation(prior_flow_effects);
        }

        let mut constructor_flow_effects = self
            .function_signatures
            .get(constructor_id)
            .unwrap_or_else(|| {
                panic!("class constructor signature `{constructor_id}` must be lowered")
            })
            .source_call_flow_effects;
        match constructor_invocation_role {
            ClassConstructorInvocationRole::Base => {
                if let Some(instance_element_plan) = class_instance_element_plan {
                    for element in &instance_element_plan.elements {
                        let init_function_id = match element {
                            ClassInstanceElementIr::Field(field) => &field.init_function_id,
                            ClassInstanceElementIr::AutoAccessorBacking(accessor) => {
                                &accessor.init_function_id
                            }
                        };
                        let Some(init_function_id) = init_function_id else {
                            continue;
                        };
                        let initializer_flow_effects = self
                            .function_signatures
                            .get(init_function_id)
                            .unwrap_or_else(|| {
                                panic!(
                                    "class instance initializer signature `{init_function_id}` must exist before constructor finalization"
                                )
                            })
                            .source_call_flow_effects;
                        constructor_flow_effects =
                            constructor_flow_effects.combine_caller_flow(initializer_flow_effects);
                    }
                }
            }
            ClassConstructorInvocationRole::ExplicitDerived => {}
            ClassConstructorInvocationRole::SyntheticDerived => {
                constructor_flow_effects = SourceCallFlowEffects::may_invalidate_caller_flow();
            }
        }
        self.function_signatures
            .get_mut(constructor_id)
            .expect("class constructor signature must remain present during finalization")
            .source_call_flow_effects = constructor_flow_effects;
    }
}
