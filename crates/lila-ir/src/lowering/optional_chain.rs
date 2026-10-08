//! Optional-chain References, acquired values and caller-flow analysis.

use super::*;

/// A property Reference acquired before the first optional Call. Its base is
/// retained as the chain target, so GetValue cannot discard the Call receiver.
enum OptionalChainInitialProperty<'source> {
    Ordinary(&'source PropertyAccessField),
    Private(PrivateNameId),
}
enum OptionalChainDestination {
    Value,
    DeleteProperty,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_optional_property_chain(&mut self, optional: &Optional) -> TypedExpr {
        self.lower_optional_chain_to(optional, OptionalChainDestination::Value)
    }

    pub(super) fn lower_optional_delete_property(&mut self, optional: &Optional) -> TypedExpr {
        match optional.chain().last().map(|operation| operation.kind()) {
            Some(OptionalOperationKind::SimplePropertyAccess { .. }) => {
                if contains(optional, ContainsSymbol::AwaitExpression) {
                    self.register_optional_source_candidates(optional);
                    let Some(source) =
                        async_expression_prefix::CheckedAwaitedOptionalChainSource::new_delete(
                            self, optional,
                        )
                    else {
                        return self.unsupported_expr("awaited optional Delete Reference");
                    };
                    return self.lower_awaited_optional_chain_delete(source);
                }
                self.lower_optional_chain_to(optional, OptionalChainDestination::DeleteProperty)
            }
            Some(OptionalOperationKind::Call { .. }) => TypedExpr::from_info(
                ValueInfo::new(ValueKind::Boolean),
                ExprIr::DeleteValue {
                    expr: Box::new(self.lower_optional_property_chain(optional)),
                },
            ),
            Some(OptionalOperationKind::PrivatePropertyAccess { .. }) | None => {
                self.unsupported_expr("private optional Delete Reference")
            }
        }
    }

    fn lower_optional_chain_to(
        &mut self,
        optional: &Optional,
        destination: OptionalChainDestination,
    ) -> TypedExpr {
        self.register_optional_source_candidates(optional);
        let delete_terminal = matches!(destination, OptionalChainDestination::DeleteProperty);
        if self.async_expression_prefix.is_some()
            && self.has_plain_async_value_branch_owner()
            && (self.has_checked_async_source_value_branch_owner()
                || !self.is_statically_nullish_optional_target(optional.target()))
            && optional
                .chain()
                .iter()
                .any(|operation| contains(operation, ContainsSymbol::AwaitExpression))
        {
            let Some(source) =
                async_expression_prefix::CheckedAwaitedOptionalChainSource::new(self, optional)
            else {
                return self.unsupported_expr("awaited optional Property/Call source");
            };
            return self.lower_awaited_optional_chain_value(source);
        }
        let starts_with_call = optional.chain().first().is_some_and(|operation| {
            matches!(operation.kind(), OptionalOperationKind::Call { .. })
        });
        let mut first_call_receiver = OptionalChainCallReceiverIr::ReferenceOrUndefined;
        let (target, initial_property) = if starts_with_call {
            match Self::unwrap_parenthesized_expr(optional.target()) {
                Expression::PropertyAccess(PropertyAccess::Simple(access)) => (
                    self.lower_property_target(access.target()),
                    Some(OptionalChainInitialProperty::Ordinary(access.field())),
                ),
                Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                    let Some(private_name_id) = self.current_private_name_id(access.field()) else {
                        return self.unsupported_expr("private class element");
                    };
                    (
                        self.lower_property_target(access.target()),
                        Some(OptionalChainInitialProperty::Private(private_name_id)),
                    )
                }
                Expression::PropertyAccess(PropertyAccess::Super(access)) => {
                    first_call_receiver = OptionalChainCallReceiverIr::CurrentThis;
                    (self.lower_super_property_access(access), None)
                }
                target => (self.lower_property_target(target), None),
            }
        } else {
            (self.lower_property_target(optional.target()), None)
        };

        let nullish_kinds =
            KindSet::from_kind(ValueKind::Undefined).union(KindSet::from_kind(ValueKind::Null));
        if initial_property.is_none()
            && contains(optional, ContainsSymbol::AwaitExpression)
            && target.possible_kinds.is_subset_of(nullish_kinds)
            && optional
                .chain()
                .first()
                .is_some_and(|operation| operation.shorted())
        {
            let target_name = self.alloc_temp_binding_name("optional.short.circuit.target.");
            return TypedExpr::from_info(
                ValueInfo::undefined(),
                ExprIr::MaterializeBinding {
                    name: target_name,
                    value: Box::new(target),
                    body: Box::new(TypedExpr::undefined()),
                },
            );
        }

        let mut boundary_before_first_call = false;
        let (target, mut chain, mut analysis) = if starts_with_call && initial_property.is_none() {
            match target {
                TypedExpr {
                    expr: ExprIr::OptionalPropertyChain { target, chain },
                    ..
                } => {
                    boundary_before_first_call = true;
                    let call_sources = already_accounted_optional_calls(&chain);
                    let target = *target;
                    let analysis =
                        self.analyze_optional_property_chain(&target, &chain, &call_sources);
                    (target, chain, analysis)
                }
                target => {
                    let analysis = OptionalChainAnalysisState::from_target(&target);
                    (
                        target,
                        Vec::with_capacity(
                            optional.chain().len() + usize::from(initial_property.is_some()),
                        ),
                        analysis,
                    )
                }
            }
        } else {
            let analysis = OptionalChainAnalysisState::from_target(&target);
            (
                target,
                Vec::with_capacity(
                    optional.chain().len() + usize::from(initial_property.is_some()),
                ),
                analysis,
            )
        };
        match initial_property {
            Some(OptionalChainInitialProperty::Ordinary(field)) => {
                let before_key_effect_epoch = self.intervening_effect_epoch;
                let Some(key) = self.lower_optional_chain_property_key(field) else {
                    return self.unsupported_expr("unsupported optional computed property key");
                };
                if self.intervening_effect_epoch != before_key_effect_epoch {
                    analysis.current.heap_shape = None;
                }
                self.analyze_optional_chain_property(&mut analysis, &key, false);
                chain.push(OptionalChainOperationIr::Property {
                    key,
                    shorted: false,
                });
            }
            Some(OptionalChainInitialProperty::Private(private_name_id)) => {
                self.analyze_optional_chain_private_property(&mut analysis, private_name_id, false);
                chain.push(OptionalChainOperationIr::PrivateProperty {
                    private_name_id,
                    shorted: false,
                });
            }
            None => {}
        }

        for (index, operation) in optional.chain().iter().enumerate() {
            match operation.kind() {
                OptionalOperationKind::SimplePropertyAccess { field } => {
                    let before_key_effect_epoch = self.intervening_effect_epoch;
                    let Some(key) = self.lower_optional_chain_property_key(field) else {
                        return self.unsupported_expr("unsupported optional computed property key");
                    };
                    if self.intervening_effect_epoch != before_key_effect_epoch {
                        analysis.current.heap_shape = None;
                    }
                    if delete_terminal && index + 1 == optional.chain().len() {
                        // [[Delete]] and computed key conversion may run user code;
                        // no terminal Get or accessor-result analysis is performed.
                        self.invalidate_unknown_user_code_effects();
                        analysis.current = ValueInfo::new(ValueKind::Boolean);
                        analysis.property_receiver = None;
                    } else {
                        self.analyze_optional_chain_property(
                            &mut analysis,
                            &key,
                            operation.shorted(),
                        );
                    }
                    chain.push(OptionalChainOperationIr::Property {
                        key,
                        shorted: operation.shorted(),
                    });
                }
                OptionalOperationKind::Call { args } => {
                    let source_args = args;
                    let receiver = std::mem::replace(
                        &mut first_call_receiver,
                        OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    );
                    let mut call_receiver =
                        self.take_optional_chain_call_receiver(&mut analysis, receiver);
                    let lowered_args = self.lower_call_args_expanding_spread(source_args);
                    let args = match call_receiver.as_mut() {
                        Some(receiver) => lowered_args.into_arguments_after_value(receiver),
                        None => lowered_args.into_arguments_without_predecessor(),
                    };
                    let shorted = operation.shorted();
                    let boundary_before = std::mem::take(&mut boundary_before_first_call);
                    self.analyze_optional_chain_call(
                        &mut analysis,
                        call_receiver.as_ref(),
                        &args,
                        shorted,
                        boundary_before,
                        &OptionalCallSource::Syntax(source_args),
                    );
                    chain.push(OptionalChainOperationIr::Call {
                        args,
                        receiver,
                        shorted,
                        boundary_before,
                    });
                }
                OptionalOperationKind::PrivatePropertyAccess { field } => {
                    let Some(private_name_id) = self.current_private_name_id(*field) else {
                        return self.unsupported_expr("private class element");
                    };
                    self.analyze_optional_chain_private_property(
                        &mut analysis,
                        private_name_id,
                        operation.shorted(),
                    );
                    chain.push(OptionalChainOperationIr::PrivateProperty {
                        private_name_id,
                        shorted: operation.shorted(),
                    });
                }
            }
        }

        let (result, effects) = self.finish_optional_chain_analysis(analysis);
        let chain = if delete_terminal {
            let reference =
                DeleteOptionalPropertyChainIr::new(target, chain, self.reference_strictness())
                    .expect("actual optional terminal Property supplies the Delete Reference");
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::Boolean),
                ExprIr::DeleteOptionalPropertyChain(Box::new(reference)),
            )
        } else {
            TypedExpr::from_info(
                result,
                ExprIr::OptionalPropertyChain {
                    target: Box::new(target),
                    chain,
                },
            )
        };
        effects.attach_to_emitted_call(chain)
    }

    pub(super) fn analyze_optional_property_chain(
        &mut self,
        target: &TypedExpr,
        chain: &[OptionalChainOperationIr],
        call_sources: &[OptionalCallSource<'_>],
    ) -> OptionalChainAnalysisState {
        let mut analysis = OptionalChainAnalysisState::from_target(target);
        let mut call_sources = call_sources.iter();

        for operation in chain {
            match operation {
                OptionalChainOperationIr::Property { key, shorted } => {
                    self.analyze_optional_chain_property(&mut analysis, key, *shorted);
                }
                OptionalChainOperationIr::PrivateProperty {
                    private_name_id,
                    shorted,
                } => {
                    self.analyze_optional_chain_private_property(
                        &mut analysis,
                        *private_name_id,
                        *shorted,
                    );
                }
                OptionalChainOperationIr::Call {
                    args,
                    receiver,
                    shorted,
                    boundary_before,
                } => {
                    let source = call_sources.next().expect("missing optional call source");
                    let mut call_receiver =
                        self.take_optional_chain_call_receiver(&mut analysis, *receiver);
                    if matches!(source, OptionalCallSource::AlreadyAccounted) && !args.is_empty() {
                        if let Some(receiver) = &mut call_receiver {
                            receiver.heap_shape = None;
                        }
                    }
                    self.analyze_optional_chain_call(
                        &mut analysis,
                        call_receiver.as_ref(),
                        args,
                        *shorted,
                        *boundary_before,
                        source,
                    );
                }
            }
        }

        assert!(call_sources.next().is_none(), "extra optional call source");
        analysis
    }

    pub(super) fn analyze_optional_chain_property(
        &mut self,
        analysis: &mut OptionalChainAnalysisState,
        key: &PropertyKeyIr,
        shorted: bool,
    ) {
        analysis.short_circuit_reaches_result |= shorted;
        let mut receiver = analysis.current.clone();
        analysis.current = match self.optional_chain_property_analysis(&receiver, key) {
            OptionalPropertyReadAnalysis::ProvenEffectFree(result) => result,
            OptionalPropertyReadAnalysis::MayRunUserCode(result) => {
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                receiver.heap_shape = None;
                result
            }
        };
        analysis.property_receiver = Some(receiver);
    }

    pub(super) fn analyze_optional_chain_private_property(
        &mut self,
        analysis: &mut OptionalChainAnalysisState,
        private_name_id: PrivateNameId,
        shorted: bool,
    ) {
        analysis.short_circuit_reaches_result |= shorted;
        let mut receiver = analysis.current.clone();
        let receiver_expr = TypedExpr::from_info(receiver.clone(), ExprIr::Undefined);
        let read = match self
            .read_object_shape_property(&receiver_expr, &private_data_key(private_name_id))
        {
            Some(ObjectShapeProperty::Data(info)) => {
                OptionalPropertyReadAnalysis::ProvenEffectFree(info)
            }
            Some(ObjectShapeProperty::Accessor {
                getter: Some(getter),
                ..
            }) => OptionalPropertyReadAnalysis::MayRunUserCode(
                self.accessor_return_info(&getter.function_id),
            ),
            Some(ObjectShapeProperty::Accessor { getter: None, .. }) => {
                OptionalPropertyReadAnalysis::ProvenEffectFree(ValueInfo::undefined())
            }
            None => OptionalPropertyReadAnalysis::MayRunUserCode(unknown_runtime_value_info()),
        };
        analysis.current = match read {
            OptionalPropertyReadAnalysis::ProvenEffectFree(info) => info,
            OptionalPropertyReadAnalysis::MayRunUserCode(mut info) => {
                self.invalidate_unknown_user_code_effects();
                receiver.heap_shape = None;
                info.heap_shape = None;
                info
            }
        };
        analysis.property_receiver = Some(receiver);
    }

    pub(super) fn take_optional_chain_call_receiver(
        &self,
        analysis: &mut OptionalChainAnalysisState,
        receiver: OptionalChainCallReceiverIr,
    ) -> Option<ValueInfo> {
        match receiver {
            OptionalChainCallReceiverIr::ReferenceOrUndefined => analysis.property_receiver.take(),
            OptionalChainCallReceiverIr::CurrentThis => {
                analysis.property_receiver = None;
                Some(self.current_this_info())
            }
        }
    }

    pub(super) fn analyze_optional_chain_call(
        &mut self,
        analysis: &mut OptionalChainAnalysisState,
        receiver: Option<&ValueInfo>,
        args: &[TypedExpr],
        shorted: bool,
        boundary_before: bool,
        source: &OptionalCallSource<'_>,
    ) {
        if boundary_before {
            // A grouped chain has already produced its value. A nullish path
            // reaching an ordinary call now throws instead of flowing through
            // to the enclosing expression as `undefined`.
            analysis.short_circuit_reaches_result = false;
        }
        analysis.short_circuit_reaches_result |= shorted;
        let (next, effects) = self.optional_call_info(&analysis.current, receiver, args, source);
        analysis.current = next;
        let previous = std::mem::replace(
            &mut analysis.invocation_effects,
            AnalyzedInvocationEffects::already_applied(),
        );
        analysis.invocation_effects = previous.combine(effects);
    }

    pub(super) fn finish_optional_chain_analysis(
        &mut self,
        analysis: OptionalChainAnalysisState,
    ) -> (ValueInfo, AnalyzedInvocationEffects) {
        let result = if analysis.short_circuit_reaches_result {
            self.merge_value_infos(analysis.current, ValueInfo::undefined())
        } else {
            analysis.current
        };
        (result, analysis.invocation_effects)
    }

    fn optional_chain_property_analysis(
        &self,
        receiver: &ValueInfo,
        key: &PropertyKeyIr,
    ) -> OptionalPropertyReadAnalysis {
        match key {
            PropertyKeyIr::StaticString(key) => {
                self.optional_chain_static_property_analysis(receiver, key)
            }
            PropertyKeyIr::ArrayLength => {
                OptionalPropertyReadAnalysis::ProvenEffectFree(ValueInfo::new(ValueKind::Number))
            }
            PropertyKeyIr::ArrayIndex(index) => {
                let receiver = TypedExpr::from_info(receiver.clone(), ExprIr::Undefined);
                self.read_array_shape(&receiver, index).map_or_else(
                    || {
                        OptionalPropertyReadAnalysis::MayRunUserCode(ValueInfo::new(
                            ValueKind::Dynamic,
                        ))
                    },
                    OptionalPropertyReadAnalysis::ProvenEffectFree,
                )
            }
            PropertyKeyIr::StringExpr(key) => match &key.expr {
                ExprIr::WellKnownSymbol(symbol) => {
                    self.optional_chain_symbol_property_analysis(receiver, *symbol)
                }
                _ => OptionalPropertyReadAnalysis::MayRunUserCode(unknown_runtime_value_info()),
            },
        }
    }

    fn optional_chain_static_property_analysis(
        &self,
        receiver: &ValueInfo,
        key: &str,
    ) -> OptionalPropertyReadAnalysis {
        let non_nullish_kinds = receiver
            .possible_kinds
            .without(ValueKind::Null)
            .without(ValueKind::Undefined);
        if key == "length"
            && matches!(
                non_nullish_kinds.as_value_kind(),
                ValueKind::String | ValueKind::Array
            )
        {
            return OptionalPropertyReadAnalysis::ProvenEffectFree(ValueInfo::new(
                ValueKind::Number,
            ));
        }
        let receiver = self.optional_chain_lookup_receiver(receiver);
        let property = self.read_current_object_shape_property(&receiver, key);
        self.optional_chain_descriptor_analysis(&receiver, key, property)
    }

    /// Primitive Get uses its intrinsic prototype, while object Get retains
    /// the receiver's actual prototype chain. These snapshots supply possible
    /// callees only; the common live owner must license any exact descriptor.
    fn optional_chain_lookup_receiver(&self, receiver: &ValueInfo) -> TypedExpr {
        let mut receiver = receiver.clone();
        let primitive = match receiver
            .possible_kinds
            .without(ValueKind::Null)
            .without(ValueKind::Undefined)
            .as_value_kind()
        {
            ValueKind::String => Some(BoxedPrimitiveKind::String),
            ValueKind::Number => Some(BoxedPrimitiveKind::Number),
            ValueKind::Boolean => Some(BoxedPrimitiveKind::Boolean),
            ValueKind::BigInt => Some(BoxedPrimitiveKind::BigInt),
            ValueKind::Symbol => Some(BoxedPrimitiveKind::Symbol),
            _ => None,
        };
        if let Some(primitive) = primitive {
            receiver.heap_shape = Some(Self::standard_boxed_prototype_shape(primitive));
        }
        TypedExpr::from_info(receiver, ExprIr::Undefined)
    }

    fn optional_chain_descriptor_analysis(
        &self,
        receiver: &TypedExpr,
        name: &str,
        property: Option<ObjectShapeProperty>,
    ) -> OptionalPropertyReadAnalysis {
        match property {
            Some(ObjectShapeProperty::Data(info)) => {
                OptionalPropertyReadAnalysis::ProvenEffectFree(info)
            }
            Some(ObjectShapeProperty::Accessor {
                getter: Some(getter),
                ..
            }) => OptionalPropertyReadAnalysis::MayRunUserCode(
                self.accessor_return_info(&getter.function_id),
            ),
            Some(ObjectShapeProperty::Accessor { getter: None, .. }) => {
                OptionalPropertyReadAnalysis::ProvenEffectFree(ValueInfo::undefined())
            }
            None => OptionalPropertyReadAnalysis::MayRunUserCode(
                self.unproven_object_property_info(receiver, name),
            ),
        }
    }

    fn optional_chain_symbol_property_analysis(
        &self,
        receiver: &ValueInfo,
        symbol: WellKnownSymbol,
    ) -> OptionalPropertyReadAnalysis {
        let receiver = self.optional_chain_lookup_receiver(receiver);
        let property = self.read_current_object_symbol_shape_property(&receiver, symbol);
        self.optional_chain_descriptor_analysis(&receiver, &shape_namespace_key(symbol), property)
    }

    pub(super) fn optional_chain_well_known_symbol_property_info(
        &self,
        receiver: &ValueInfo,
        key: WellKnownSymbol,
    ) -> ValueInfo {
        match self.optional_chain_symbol_property_analysis(receiver, key) {
            OptionalPropertyReadAnalysis::ProvenEffectFree(info)
            | OptionalPropertyReadAnalysis::MayRunUserCode(info) => info,
        }
    }

    fn optional_call_info(
        &mut self,
        callee: &ValueInfo,
        receiver: Option<&ValueInfo>,
        args: &[TypedExpr],
        source: &OptionalCallSource<'_>,
    ) -> (ValueInfo, AnalyzedInvocationEffects) {
        let source = match source {
            OptionalCallSource::AlreadyAccounted => CallCandidateSource::AlreadyAccounted,
            OptionalCallSource::Syntax(arguments) => CallCandidateSource::IndirectSyntax(arguments),
        };
        match self.analyze_known_call_candidates(callee, receiver, args, source) {
            CallCandidateAnalysis::UnsupportedDynamicSource => (
                ValueInfo::undefined(),
                AnalyzedInvocationEffects::already_applied(),
            ),
            CallCandidateAnalysis::Accepted { result, effects } => (result, effects),
        }
    }

    pub(super) fn lower_optional_chain_property_key(
        &mut self,
        field: &PropertyAccessField,
    ) -> Option<PropertyKeyIr> {
        match field {
            PropertyAccessField::Const(name) => Some(PropertyKeyIr::StaticString(
                self.interner.resolve_expect(name.sym()).to_string(),
            )),
            PropertyAccessField::Expr(expr) => {
                self.lower_dynamic_object_property_key(expr.as_ref())
            }
        }
    }
}
