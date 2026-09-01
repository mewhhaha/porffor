use super::invocation_effects::{AnalyzedInvocationEffects, InvocationCallerFlowEffects};
use super::*;

pub(super) enum CallCandidateSource<'a> {
    DirectSyntax {
        source_callee: &'a Expression,
        lowered_callee: &'a TypedExpr,
        arguments: &'a [Expression],
    },
    IndirectSyntax(&'a [Expression]),
    AlreadyAccounted,
}

impl<'a> CallCandidateSource<'a> {
    fn arguments(&self) -> Option<&'a [Expression]> {
        match self {
            Self::DirectSyntax { arguments, .. } | Self::IndirectSyntax(arguments) => {
                Some(arguments)
            }
            Self::AlreadyAccounted => None,
        }
    }

    fn context(
        &self,
        lowerer: &ScriptLowerer<'_>,
        callee: &ValueInfo,
        function_id: &FunctionId,
    ) -> BuiltinCallContext {
        let Self::DirectSyntax {
            source_callee,
            lowered_callee,
            ..
        } = self
        else {
            return BuiltinCallContext::Call;
        };
        debug_assert_eq!(lowered_callee.value_info(), *callee);
        lowerer.resolved_builtin_call_context(source_callee, lowered_callee, function_id)
    }
}

pub(super) enum CallCandidateAnalysis {
    UnsupportedDynamicSource,
    Accepted {
        result: ValueInfo,
        effects: AnalyzedInvocationEffects,
    },
}

#[must_use = "dynamic-source candidate preflight must admit analysis or reject emission"]
pub(super) enum DynamicSourceCallAdmission {
    Admitted(AdmittedDynamicSourceCall),
    Rejected,
}

pub(super) struct AdmittedDynamicSourceCall {
    function_ids: Vec<FunctionId>,
    pass_through_results: BTreeMap<FunctionId, ValueInfo>,
}

impl<'a> ScriptLowerer<'a> {
    fn combine_candidate_caller_flow_effects(
        accumulated: &mut Option<InvocationCallerFlowEffects>,
        candidate: InvocationCallerFlowEffects,
    ) {
        *accumulated = Some(match accumulated.take() {
            Some(accumulated) => accumulated.combine(candidate),
            None => candidate,
        });
    }

    fn preflight_dynamic_source_call_candidates(
        &mut self,
        callee: &ValueInfo,
        arguments: &[TypedExpr],
        source: &CallCandidateSource<'_>,
    ) -> DynamicSourceCallAdmission {
        let mut function_ids = callee
            .function_targets
            .known_targets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        function_ids
            .sort_by_key(|function_id| StandardBuiltinId::from_function_id(function_id).is_some());
        let mut pass_through_results = BTreeMap::new();
        let mut rejected_dynamic_source = false;

        for function_id in &function_ids {
            let Some(signature) = self.function_signature_for_current_flow(function_id) else {
                continue;
            };
            if !signature.callable {
                continue;
            }
            let context = source.context(self, callee, function_id);
            match self.resolve_dynamic_source_call(
                function_id,
                &context,
                source.arguments(),
                arguments,
            ) {
                None => {}
                Some(ResolvedDynamicSourceCall::EvalPassThrough(proof)) => {
                    pass_through_results.insert(function_id.clone(), proof.into_result_info());
                }
                Some(ResolvedDynamicSourceCall::Unsupported(unsupported)) => {
                    if matches!(source, CallCandidateSource::AlreadyAccounted) {
                        pass_through_results.insert(function_id.clone(), ValueInfo::undefined());
                    } else {
                        self.record_unsupported_dynamic_source(unsupported);
                        rejected_dynamic_source = true;
                    }
                }
            }
        }

        if rejected_dynamic_source {
            DynamicSourceCallAdmission::Rejected
        } else {
            DynamicSourceCallAdmission::Admitted(AdmittedDynamicSourceCall {
                function_ids,
                pass_through_results,
            })
        }
    }

    pub(super) fn preflight_function_prototype_call_dynamic_source(
        &mut self,
        receiver: &ValueInfo,
        source_arguments: &[Expression],
        arguments: &[TypedExpr],
    ) -> DynamicSourceCallAdmission {
        debug_assert!(!Self::call_args_have_spread(arguments));
        debug_assert!(!source_arguments
            .iter()
            .any(|argument| matches!(argument, Expression::Spread(_))));
        self.preflight_dynamic_source_call_candidates(
            receiver,
            arguments.get(1..).unwrap_or_default(),
            &CallCandidateSource::IndirectSyntax(source_arguments.get(1..).unwrap_or_default()),
        )
    }

    pub(super) fn consume_forwarded_dynamic_source_admission(
        &mut self,
        receiver: &ValueInfo,
        admission: AdmittedDynamicSourceCall,
    ) -> Option<ValueInfo> {
        let AdmittedDynamicSourceCall {
            function_ids,
            mut pass_through_results,
        } = admission;
        for function_id in pass_through_results.keys() {
            if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                self.note_standard_builtin_call(builtin);
            }
        }
        let function_id = receiver.function_targets.exact_single_target()?;
        debug_assert!(function_ids.contains(function_id));
        pass_through_results.remove(function_id)
    }

    pub(super) fn consume_forwarded_call_flow_effects(&mut self, receiver: &TypedExpr) {
        let InvocationTargetProvenance::ProvenFunction(targets) =
            InvocationTargetProvenance::from(receiver)
        else {
            self.invalidate_unknown_user_code_effects();
            return;
        };
        if targets.is_empty() {
            self.invalidate_unknown_user_code_effects();
            return;
        }

        let mut effects: Option<InvocationCallerFlowEffects> = None;
        for function_id in targets {
            let target_effects = if StandardBuiltinId::from_function_id(function_id)
                .is_some_and(StandardBuiltinId::mutates_indexed_receiver)
            {
                InvocationCallerFlowEffects::may_invalidate()
            } else if let Some(signature) = self.function_signature_for_current_flow(function_id) {
                self.invocation_caller_flow_effects(function_id, &signature)
            } else {
                InvocationCallerFlowEffects::may_invalidate()
            };
            effects = Some(match effects {
                Some(effects) => effects.combine(target_effects),
                None => target_effects,
            });
        }

        if effects.is_none_or(InvocationCallerFlowEffects::may_invalidate_caller_flow) {
            self.invalidate_unknown_user_code_effects();
        }
    }

    pub(super) fn analyze_known_call_candidates(
        &mut self,
        callee: &ValueInfo,
        receiver: Option<&ValueInfo>,
        arguments: &[TypedExpr],
        source: CallCandidateSource<'_>,
    ) -> CallCandidateAnalysis {
        let (function_ids, mut dynamic_source_results) =
            match self.preflight_dynamic_source_call_candidates(callee, arguments, &source) {
                DynamicSourceCallAdmission::Admitted(AdmittedDynamicSourceCall {
                    function_ids,
                    pass_through_results,
                }) => (function_ids, pass_through_results),
                DynamicSourceCallAdmission::Rejected => {
                    return CallCandidateAnalysis::UnsupportedDynamicSource;
                }
            };

        let arguments_have_spread = Self::call_args_have_spread(arguments);
        let argument_values = arguments
            .iter()
            .map(TypedExpr::value_info)
            .collect::<Vec<_>>();
        let canonical_argument_values = self.canonical_exact_context_arg_infos(&argument_values);
        let mut result = None;
        let mut effects = AnalyzedInvocationEffects::must_attach();
        let mut caller_flow_effects = None;
        let mut has_unaccounted_candidate = false;

        for function_id in &function_ids {
            self.observe_live_script_global_values();
            let Some(initial_signature) = self.function_signature_for_current_flow(function_id)
            else {
                has_unaccounted_candidate = true;
                Self::merge_call_candidate_result(
                    self,
                    &mut result,
                    ValueInfo::new(ValueKind::Dynamic),
                );
                continue;
            };
            if !initial_signature.callable {
                continue;
            }
            let always_throwing_builtin = StandardBuiltinId::from_function_id(function_id)
                .filter(|builtin| builtin.always_throws());
            if let Some(builtin) = always_throwing_builtin {
                self.note_standard_builtin_call(builtin);
                continue;
            }

            self.mark_host_builtin_from_function_id(function_id);
            self.host_builtin_calls +=
                usize::from(HostBuiltinId::from_function_id(function_id).is_some());
            let this_value = match receiver {
                Some(receiver) => {
                    let receiver = TypedExpr::from_info(receiver.clone(), ExprIr::Undefined);
                    self.explicit_this_info_for_function_target(
                        function_id,
                        &receiver,
                        initial_signature.this_info.clone(),
                    )
                }
                None => self.default_this_info_for_function_target(function_id),
            };
            self.merge_function_this_info(function_id, this_value.clone());

            if let Some(dynamic_result) = dynamic_source_results.remove(function_id) {
                if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                    self.note_standard_builtin_call(builtin);
                }
                Self::merge_call_candidate_result(self, &mut result, dynamic_result);
                continue;
            }

            if arguments_have_spread {
                if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                    self.note_standard_builtin_call(builtin);
                    if builtin.mutates_indexed_receiver() {
                        Self::combine_candidate_caller_flow_effects(
                            &mut caller_flow_effects,
                            InvocationCallerFlowEffects::may_invalidate(),
                        );
                    }
                    if let Some(define_property) = Self::define_property_builtin(function_id) {
                        let (candidate_result, candidate_effects) = self
                            .unknown_forwarded_define_property_call_analysis(define_property)
                            .into_parts();
                        Self::merge_call_candidate_result(self, &mut result, candidate_result);
                        effects = effects.combine(candidate_effects);
                        continue;
                    }
                    if self.function_may_run_user_code_synchronously(function_id) {
                        Self::combine_candidate_caller_flow_effects(
                            &mut caller_flow_effects,
                            InvocationCallerFlowEffects::may_invalidate(),
                        );
                    }
                } else {
                    self.merge_unknown_spread_param_infos(function_id);
                    Self::combine_candidate_caller_flow_effects(
                        &mut caller_flow_effects,
                        self.invocation_caller_flow_effects(function_id, &initial_signature),
                    );
                }
                let candidate_result = if StandardBuiltinId::from_function_id(function_id).is_some()
                {
                    self.function_call_return_info(&initial_signature)
                } else {
                    ValueInfo::new(ValueKind::Dynamic)
                };
                Self::merge_call_candidate_result(self, &mut result, candidate_result);
                continue;
            }

            let exact_prepass_call =
                self.is_prepass && self.analysis.function_plans.contains_key(function_id);
            if !exact_prepass_call {
                self.merge_function_param_infos(function_id, &argument_values);
                if let Some(signature) = self.function_signatures.get_mut(function_id) {
                    Self::merge_omitted_signature_params_as_undefined(signature, arguments.len());
                }
            } else if let Some(helper_context_id) = self
                .exact_context_callback_targets
                .get(&self.original_exact_function_id(function_id))
                .cloned()
            {
                let original_function_id = self.original_exact_function_id(function_id);
                self.observe_exact_callback_this_info(
                    &original_function_id,
                    &helper_context_id,
                    this_value,
                );
                self.observe_exact_callback_param_infos(
                    &original_function_id,
                    &helper_context_id,
                    &canonical_argument_values,
                );
            }

            let helper_context_id =
                Self::exact_helper_context_id(function_id, &canonical_argument_values);
            let callsite_result =
                if self.is_prepass || self.analysis.function_plans.contains_key(function_id) {
                    self.propagate_direct_call_context(function_id, &canonical_argument_values)
                } else {
                    None
                };
            let effective_function_id = self
                .exact_context_function_specializations
                .get(&(function_id.clone(), helper_context_id.clone()))
                .cloned()
                .unwrap_or_else(|| function_id.clone());

            if let Some(builtin) = StandardBuiltinId::from_function_id(&effective_function_id) {
                self.note_standard_builtin_call(builtin);
                if builtin.mutates_indexed_receiver() {
                    Self::combine_candidate_caller_flow_effects(
                        &mut caller_flow_effects,
                        InvocationCallerFlowEffects::may_invalidate(),
                    );
                }
                let context = source.context(self, callee, function_id);
                let Some(candidate_analysis) =
                    self.standard_builtin_call_info(builtin, arguments, context)
                else {
                    Self::merge_call_candidate_result(
                        self,
                        &mut result,
                        self.function_call_return_info(&initial_signature),
                    );
                    if self.function_may_run_user_code_synchronously(function_id) {
                        Self::combine_candidate_caller_flow_effects(
                            &mut caller_flow_effects,
                            InvocationCallerFlowEffects::may_invalidate(),
                        );
                    }
                    continue;
                };
                let (candidate_result, candidate_effects) = candidate_analysis.into_parts();
                Self::merge_call_candidate_result(self, &mut result, candidate_result);
                effects = effects.combine(candidate_effects);
                continue;
            }

            let exact_context_key = (function_id.clone(), helper_context_id);
            let signature = self
                .exact_context_function_observations
                .get(&exact_context_key)
                .or_else(|| {
                    self.exact_context_callback_observations
                        .get(&exact_context_key)
                })
                .filter(|_| !self.function_has_untracked_captures(function_id))
                .filter(|signature| {
                    !signature.this_observed
                        || signature
                            .return_targets
                            .exact_targets()
                            .is_some_and(BTreeSet::is_empty)
                })
                .cloned()
                .or_else(|| {
                    self.function_signatures
                        .get(&effective_function_id)
                        .cloned()
                })
                .unwrap_or(initial_signature);
            let candidate_result =
                callsite_result.unwrap_or_else(|| self.function_call_return_info(&signature));
            Self::merge_call_candidate_result(self, &mut result, candidate_result);
            let candidate_flow_effects =
                self.invocation_caller_flow_effects(function_id, &signature);
            Self::combine_candidate_caller_flow_effects(
                &mut caller_flow_effects,
                candidate_flow_effects,
            );
        }

        if callee.function_targets.exact_targets().is_none() || has_unaccounted_candidate {
            self.observe_unaccounted_invocation_effects(InvocationTargetProvenance::Erased);
            Self::merge_call_candidate_result(
                self,
                &mut result,
                ValueInfo::new(ValueKind::Dynamic),
            );
        } else if function_ids
            .iter()
            .any(Self::invocation_target_requires_unknown_property_hook_observation)
        {
            self.observe_unaccounted_invocation_effects(InvocationTargetProvenance::from(callee));
        } else if caller_flow_effects
            .is_some_and(InvocationCallerFlowEffects::may_invalidate_caller_flow)
        {
            self.invalidate_unknown_user_code_effects();
        }

        CallCandidateAnalysis::Accepted {
            result: result.unwrap_or_else(ValueInfo::undefined),
            effects,
        }
    }

    pub(super) fn analyze_known_construct_candidates(
        &mut self,
        callee: &ValueInfo,
        arguments: &[TypedExpr],
        source_arguments: &[Expression],
    ) -> CallCandidateAnalysis {
        let mut function_ids = callee
            .function_targets
            .known_targets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        function_ids
            .sort_by_key(|function_id| StandardBuiltinId::from_function_id(function_id).is_some());
        let mut rejected_dynamic_source = false;

        for function_id in &function_ids {
            let Some(signature) = self.function_signature_for_current_flow(function_id) else {
                continue;
            };
            if !signature.protocol.is_constructable()
                || signature.protocol.flavor() == FunctionFlavor::Arrow
            {
                continue;
            }
            match self.resolve_dynamic_source_call(
                function_id,
                &BuiltinCallContext::Construct,
                Some(source_arguments),
                arguments,
            ) {
                None => {}
                Some(ResolvedDynamicSourceCall::EvalPassThrough(_)) => {
                    unreachable!("the intrinsic eval function is not constructable")
                }
                Some(ResolvedDynamicSourceCall::Unsupported(unsupported)) => {
                    self.record_unsupported_dynamic_source(unsupported);
                    rejected_dynamic_source = true;
                }
            }
        }

        if rejected_dynamic_source {
            return CallCandidateAnalysis::UnsupportedDynamicSource;
        }

        let arguments_have_spread = Self::call_args_have_spread(arguments);
        let argument_values = arguments
            .iter()
            .map(TypedExpr::value_info)
            .collect::<Vec<_>>();
        let canonical_argument_values = self.canonical_exact_context_arg_infos(&argument_values);
        let common_instance_prototype = match callee
            .heap_shape
            .as_deref()
            .and_then(|shape| read_heap_shape_property(shape, "prototype"))
        {
            Some(ObjectShapeProperty::Data(prototype_info))
                if matches!(
                    prototype_info.kind,
                    ValueKind::Object
                        | ValueKind::Array
                        | ValueKind::Function
                        | ValueKind::Arguments
                ) =>
            {
                prototype_info.heap_shape
            }
            Some(ObjectShapeProperty::Data(_))
            | Some(ObjectShapeProperty::Accessor { .. })
            | None => None,
        };
        let mut result = None;
        let mut effects = AnalyzedInvocationEffects::must_attach();
        let mut caller_flow_effects = None;
        let mut has_unaccounted_candidate = false;

        for function_id in &function_ids {
            self.observe_live_script_global_values();
            let Some(signature) = self.function_signature_for_current_flow(function_id) else {
                has_unaccounted_candidate = true;
                Self::merge_call_candidate_result(
                    self,
                    &mut result,
                    Self::unknown_construct_result_info(),
                );
                continue;
            };
            if !signature.protocol.is_constructable()
                || signature.protocol.flavor() == FunctionFlavor::Arrow
            {
                continue;
            }
            let always_throwing_builtin = StandardBuiltinId::from_function_id(function_id)
                .filter(|builtin| builtin.always_throws());
            if let Some(builtin) = always_throwing_builtin {
                self.note_standard_builtin_call(builtin);
                continue;
            }

            self.mark_host_builtin_from_function_id(function_id);
            self.host_builtin_calls +=
                usize::from(HostBuiltinId::from_function_id(function_id).is_some());

            if arguments_have_spread {
                if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                    self.note_standard_builtin_call(builtin);
                    if self.function_may_run_user_code_synchronously(function_id) {
                        Self::combine_candidate_caller_flow_effects(
                            &mut caller_flow_effects,
                            InvocationCallerFlowEffects::may_invalidate(),
                        );
                    }
                } else {
                    self.merge_unknown_spread_param_infos(function_id);
                    let constructed_this = Self::with_instance_prototype(
                        self.function_construct_instance_info(&signature),
                        common_instance_prototype.clone(),
                    );
                    self.merge_function_this_info(function_id, constructed_this);
                    Self::combine_candidate_caller_flow_effects(
                        &mut caller_flow_effects,
                        self.invocation_caller_flow_effects(function_id, &signature),
                    );
                }
                Self::merge_call_candidate_result(
                    self,
                    &mut result,
                    Self::unknown_construct_result_info(),
                );
                continue;
            }

            self.merge_function_param_infos(function_id, &argument_values);
            if let Some(signature) = self.function_signatures.get_mut(function_id) {
                Self::merge_omitted_signature_params_as_undefined(signature, arguments.len());
            }

            if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                self.note_standard_builtin_call(builtin);
                let Some(candidate_analysis) = self.standard_builtin_call_info(
                    builtin,
                    arguments,
                    BuiltinCallContext::Construct,
                ) else {
                    Self::merge_call_candidate_result(
                        self,
                        &mut result,
                        Self::unknown_construct_result_info(),
                    );
                    if self.function_may_run_user_code_synchronously(function_id) {
                        Self::combine_candidate_caller_flow_effects(
                            &mut caller_flow_effects,
                            InvocationCallerFlowEffects::may_invalidate(),
                        );
                    }
                    continue;
                };
                let (candidate_result, candidate_effects) = candidate_analysis.into_parts();
                Self::merge_call_candidate_result(self, &mut result, candidate_result);
                effects = effects.combine(candidate_effects);
                continue;
            }

            let constructed_this = Self::with_instance_prototype(
                self.function_construct_instance_info(&signature),
                common_instance_prototype.clone(),
            );
            self.merge_function_this_info(function_id, constructed_this.clone());
            let helper_context_id =
                Self::exact_helper_context_id(function_id, &canonical_argument_values);
            let callsite_result = {
                if self.is_prepass {
                    if let Some(helper_context_id) = self
                        .exact_context_callback_targets
                        .get(&self.original_exact_function_id(function_id))
                        .cloned()
                    {
                        let original_function_id = self.original_exact_function_id(function_id);
                        self.observe_exact_callback_this_info(
                            &original_function_id,
                            &helper_context_id,
                            constructed_this.clone(),
                        );
                        self.observe_exact_callback_param_infos(
                            &original_function_id,
                            &helper_context_id,
                            &canonical_argument_values,
                        );
                    }
                }
                if self.is_prepass || self.analysis.function_plans.contains_key(function_id) {
                    self.propagate_direct_call_context(function_id, &canonical_argument_values)
                } else {
                    None
                }
            };
            let effective_function_id = self
                .exact_context_function_specializations
                .get(&(function_id.clone(), helper_context_id.clone()))
                .cloned()
                .unwrap_or_else(|| function_id.clone());
            let exact_context_key = (function_id.clone(), helper_context_id);
            let signature = self
                .exact_context_function_observations
                .get(&exact_context_key)
                .or_else(|| {
                    self.exact_context_callback_observations
                        .get(&exact_context_key)
                })
                .filter(|_| !self.function_has_untracked_captures(function_id))
                .filter(|signature| {
                    !signature.this_observed
                        || signature
                            .return_targets
                            .exact_targets()
                            .is_some_and(BTreeSet::is_empty)
                })
                .cloned()
                .or_else(|| {
                    self.function_signatures
                        .get(&effective_function_id)
                        .cloned()
                })
                .unwrap_or(signature);
            let explicit_return =
                callsite_result.unwrap_or_else(|| self.function_call_return_info(&signature));
            let null_heritage_return_path = signature.class_heritage_kind
                == ClassHeritageKind::Null
                && !explicit_return
                    .possible_kinds
                    .is_subset_of(KindSet::PRIMITIVE_ONLY);
            let explicit_object_return =
                Self::construct_explicit_object_return_info(explicit_return);
            let candidate_result = if null_heritage_return_path {
                explicit_object_return
                    .expect("a non-primitive constructor return must have an object-like kind")
            } else {
                match explicit_object_return {
                    Some(explicit_object_return) => {
                        self.merge_value_infos(constructed_this, explicit_object_return)
                    }
                    None => constructed_this,
                }
            };
            Self::merge_call_candidate_result(self, &mut result, candidate_result);
            let candidate_flow_effects =
                self.invocation_caller_flow_effects(function_id, &signature);
            Self::combine_candidate_caller_flow_effects(
                &mut caller_flow_effects,
                candidate_flow_effects,
            );
        }

        if callee.function_targets.exact_targets().is_none() || has_unaccounted_candidate {
            self.observe_unaccounted_invocation_effects(InvocationTargetProvenance::Erased);
            Self::merge_call_candidate_result(
                self,
                &mut result,
                Self::unknown_construct_result_info(),
            );
        } else if function_ids
            .iter()
            .any(Self::invocation_target_requires_unknown_property_hook_observation)
        {
            self.observe_unaccounted_invocation_effects(InvocationTargetProvenance::from(callee));
        } else if caller_flow_effects
            .is_some_and(InvocationCallerFlowEffects::may_invalidate_caller_flow)
        {
            self.invalidate_unknown_user_code_effects();
        }

        CallCandidateAnalysis::Accepted {
            result: result.unwrap_or_else(ValueInfo::undefined),
            effects,
        }
    }

    fn merge_call_candidate_result(&self, result: &mut Option<ValueInfo>, candidate: ValueInfo) {
        *result = Some(match result.take() {
            Some(existing) => self.merge_value_infos(existing, candidate),
            None => candidate,
        });
    }
}
