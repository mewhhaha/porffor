use super::super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_non_property_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> TypedExpr {
        let source_callee = callee;
        let callee = self.lower_expression(callee);
        let callee = match callee {
            TypedExpr {
                expr: ExprIr::OptionalPropertyChain { target, mut chain },
                ..
            } => {
                let source_args = args;
                let call_sources = already_accounted_optional_calls(&chain);
                let mut analysis =
                    self.analyze_optional_property_chain(target.as_ref(), &chain, &call_sources);
                let mut receiver = self.take_optional_chain_call_receiver(
                    &mut analysis,
                    OptionalChainCallReceiverIr::ReferenceOrUndefined,
                );
                let lowered_args = self.lower_call_args_expanding_spread(args);
                let args = match receiver.as_mut() {
                    Some(receiver) => lowered_args.into_arguments_after_value(receiver),
                    None => lowered_args.into_arguments_without_predecessor(),
                };
                self.analyze_optional_chain_call(
                    &mut analysis,
                    receiver.as_ref(),
                    &args,
                    false,
                    true,
                    &OptionalCallSource::Syntax(source_args),
                );
                chain.push(OptionalChainOperationIr::Call {
                    args,
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false,
                    boundary_before: true,
                });
                let (result, effects) = self.finish_optional_chain_analysis(analysis);
                let chain =
                    TypedExpr::from_info(result, ExprIr::OptionalPropertyChain { target, chain });
                return effects.attach_to_emitted_call(chain);
            }
            callee => callee,
        };
        let lower_generic_indirect_call = |this: &mut Self, mut callee: TypedExpr| {
            let erased_direct_eval = callee
                .function_targets
                .exact_targets()
                .is_none()
                .then(|| this.capture_erased_direct_eval_call(source_callee, &callee))
                .flatten();
            let lowered_args = this
                .lower_call_args_expanding_spread(args)
                .into_arguments_after_expression(&mut callee);
            if let Some(erased_direct_eval) = erased_direct_eval {
                let resolved = erased_direct_eval.resolve(this, args, &lowered_args);
                match resolved {
                    ResolvedDynamicSourceCall::EvalPassThrough(_) => {}
                    ResolvedDynamicSourceCall::Unsupported(unsupported) => {
                        this.record_unsupported_dynamic_source(unsupported);
                        return TypedExpr::undefined();
                    }
                }
            }
            let analysis = this.analyze_known_call_candidates(
                &callee.value_info(),
                None,
                &lowered_args,
                CallCandidateSource::DirectSyntax {
                    source_callee,
                    lowered_callee: &callee,
                    arguments: args,
                },
            );
            let CallCandidateAnalysis::Accepted { result, effects } = analysis else {
                return TypedExpr::undefined();
            };
            let result = TypedExpr::from_info(
                result,
                ExprIr::CallIndirect {
                    callee: Box::new(callee),
                    this_arg: None,
                    args: lowered_args,
                    static_regexp_compilation: None,
                },
            );
            effects.attach_to_emitted_call(result)
        };
        if callee.kind != ValueKind::Function {
            return lower_generic_indirect_call(self, callee);
        }
        if matches!(
            InvocationTargetProvenance::from(&callee),
            InvocationTargetProvenance::Erased
        ) {
            return lower_generic_indirect_call(self, callee);
        }
        let mut callee = callee;
        if matches!(&callee.expr, ExprIr::FunctionValue(_))
            && !self.exact_context_callback_targets.is_empty()
            && callee
                .function_targets
                .exact_targets()
                .is_some_and(|targets| !targets.is_empty())
        {
            let mut rewritten_targets = BTreeSet::new();
            for target_id in callee
                .function_targets
                .exact_targets()
                .expect("proven function value targets must remain exact")
            {
                let original_target_id = self.original_exact_function_id(target_id);
                if let Some(helper_context_id) = self
                    .exact_context_callback_targets
                    .get(&original_target_id)
                    .cloned()
                {
                    if let Some(synthetic_id) = self
                        .exact_context_callback_specializations
                        .get(&(original_target_id, helper_context_id))
                        .cloned()
                    {
                        rewritten_targets.insert(synthetic_id);
                        continue;
                    }
                }
                rewritten_targets.insert(target_id.clone());
            }
            if &rewritten_targets
                != callee
                    .function_targets
                    .exact_targets()
                    .expect("proven function value targets must remain exact")
            {
                callee.function_targets = FunctionTargetKnowledge::exact_many(rewritten_targets);
                if let Some(single_target) = self.resolve_single_function_target(&callee) {
                    callee = self.function_value_expr(single_target);
                }
            }
        }
        let Some(mut function_id) = self.resolve_single_function_target(&callee) else {
            return lower_generic_indirect_call(self, callee);
        };
        if let Some(helper_context_id) = self
            .exact_context_callback_targets
            .get(&self.original_exact_function_id(&function_id))
            .cloned()
        {
            let original_function_id = self.original_exact_function_id(&function_id);
            if let Some(synthetic_id) = self
                .exact_context_callback_specializations
                .get(&(original_function_id, helper_context_id))
                .cloned()
            {
                function_id = synthetic_id.clone();
                if matches!(&callee.expr, ExprIr::FunctionValue(_)) {
                    callee = self.function_value_expr(synthetic_id);
                }
            }
        }
        let Some(signature) = self.function_signatures.get(&function_id) else {
            return lower_generic_indirect_call(self, callee);
        };
        if !signature.callable && signature.protocol.class_kind() != ClassFunctionKind::Constructor
        {
            return self.unsupported_expr("indirect call");
        }
        self.mark_host_builtin_from_function_id(&function_id);
        self.host_builtin_calls +=
            usize::from(HostBuiltinId::from_function_id(&function_id).is_some());
        let context = self.resolved_builtin_call_context(source_callee, &callee, &function_id);
        let prepared_static_json_parse_reviver =
            self.prepare_static_json_parse_reviver(&function_id, args);
        let (effective_function_id, args, info, invocation_effects) = self
            .lower_call_args_with_target(
                &function_id,
                args,
                context,
                InvocationThisObservation::Default,
            );
        if let Some(builtin) = StandardBuiltinId::from_function_id(&effective_function_id) {
            if let Some(folded) = Self::fold_standard_builtin_literal_call(builtin, &args) {
                return folded;
            }
        }
        // A context-specialized body is only safe to materialize directly when
        // the source expression already creates that function object here.
        // Replacing an identifier/property callee would discard the original
        // closure object's captured environment and function identity.
        let callee = if effective_function_id != function_id
            && matches!(&callee.expr, ExprIr::FunctionValue(_))
        {
            self.function_value_expr(effective_function_id.clone())
        } else {
            callee
        };
        if let Some(static_json_parse_reviver) = prepared_static_json_parse_reviver
            .and_then(|prepared| self.finish_static_json_parse_reviver(prepared, &callee, &args))
        {
            return invocation_effects.attach_to_emitted_call(static_json_parse_reviver);
        }
        let static_regexp_compilation =
            self.static_regexp_compilation_for_direct_call(&callee, &effective_function_id, &args);
        let call = TypedExpr::from_info(
            info,
            ExprIr::CallIndirect {
                callee: Box::new(callee),
                this_arg: None,
                args,
                static_regexp_compilation,
            },
        );
        invocation_effects.attach_to_emitted_call(call)
    }
}
