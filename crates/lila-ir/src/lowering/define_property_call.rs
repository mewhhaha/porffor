use super::invocation_effects::StandardBuiltinCallAnalysis;
use super::*;

#[derive(Clone, Copy)]
enum DefinePropertyCallKind {
    Object,
    Reflect,
}

#[derive(Clone, Copy)]
enum DescriptorFieldReadPrecision {
    Exact,
    Unknown,
}

#[derive(Clone, Copy)]
pub(super) enum InvocationTargetProvenance<'a> {
    ProvenFunction(&'a BTreeSet<FunctionId>),
    Erased,
}

impl<'a> InvocationTargetProvenance<'a> {
    fn classify(possible_kinds: KindSet, function_targets: &'a FunctionTargetKnowledge) -> Self {
        if !possible_kinds.is_subset_of(KindSet::from_kind(ValueKind::Function)) {
            return Self::Erased;
        }
        function_targets
            .exact_targets()
            .map_or(Self::Erased, Self::ProvenFunction)
    }
}

impl<'a> From<&'a ValueInfo> for InvocationTargetProvenance<'a> {
    fn from(callee: &'a ValueInfo) -> Self {
        Self::classify(callee.possible_kinds, &callee.function_targets)
    }
}

impl<'a> From<&'a TypedExpr> for InvocationTargetProvenance<'a> {
    fn from(callee: &'a TypedExpr) -> Self {
        Self::classify(callee.possible_kinds, &callee.function_targets)
    }
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn define_property_builtin(function_id: &FunctionId) -> Option<StandardBuiltinId> {
        match StandardBuiltinId::from_function_id(function_id) {
            Some(
                builtin @ (StandardBuiltinId::ObjectDefineProperty
                | StandardBuiltinId::ReflectDefineProperty),
            ) => Some(builtin),
            _ => None,
        }
    }

    pub(super) fn invocation_target_requires_unknown_property_hook_observation(
        function_id: &FunctionId,
    ) -> bool {
        matches!(
            StandardBuiltinId::from_function_id(function_id),
            Some(
                StandardBuiltinId::ObjectDefineProperty
                    | StandardBuiltinId::ReflectDefineProperty
                    | StandardBuiltinId::FunctionPrototypeCall
                    | StandardBuiltinId::FunctionPrototypeApply
                    | StandardBuiltinId::ReflectApply
                    | StandardBuiltinId::BoundFunctionInvoker
            )
        )
    }

    pub(super) fn observe_unaccounted_invocation_effects(
        &mut self,
        target_provenance: InvocationTargetProvenance<'_>,
    ) {
        let requires_unknown_property_hook_observation = match target_provenance {
            InvocationTargetProvenance::ProvenFunction(targets) => {
                targets.is_empty()
                    || targets
                        .iter()
                        .any(Self::invocation_target_requires_unknown_property_hook_observation)
            }
            InvocationTargetProvenance::Erased => true,
        };
        if requires_unknown_property_hook_observation {
            self.observe_all_planned_source_as_unknown_property_hooks();
        }
        self.invalidate_unknown_user_code_effects();
    }

    pub(super) fn object_define_property_call_analysis(
        &mut self,
        args: &[TypedExpr],
    ) -> StandardBuiltinCallAnalysis {
        StandardBuiltinCallAnalysis::with_accounted_invocation_effects(
            self.define_property_call_info(args, DefinePropertyCallKind::Object),
        )
    }

    pub(super) fn reflect_define_property_call_analysis(
        &mut self,
        args: &[TypedExpr],
    ) -> StandardBuiltinCallAnalysis {
        StandardBuiltinCallAnalysis::with_accounted_invocation_effects(
            self.define_property_call_info(args, DefinePropertyCallKind::Reflect),
        )
    }

    pub(super) fn forwarded_define_property_call_analysis(
        &mut self,
        builtin: StandardBuiltinId,
        args: &[TypedExpr],
    ) -> StandardBuiltinCallAnalysis {
        let call_kind = match builtin {
            StandardBuiltinId::ObjectDefineProperty => DefinePropertyCallKind::Object,
            StandardBuiltinId::ReflectDefineProperty => DefinePropertyCallKind::Reflect,
            _ => unreachable!("forwarded defineProperty call must retain its builtin target"),
        };
        self.note_standard_builtin_call(builtin);
        StandardBuiltinCallAnalysis::with_accounted_invocation_effects(
            self.define_property_call_info(args, call_kind),
        )
    }

    pub(super) fn unknown_forwarded_define_property_call_analysis(
        &mut self,
        builtin: StandardBuiltinId,
    ) -> StandardBuiltinCallAnalysis {
        StandardBuiltinCallAnalysis::with_accounted_invocation_effects(
            self.unaccounted_unknown_forwarded_define_property_call_info(builtin),
        )
    }

    pub(super) fn unaccounted_unknown_forwarded_define_property_call_info(
        &mut self,
        builtin: StandardBuiltinId,
    ) -> ValueInfo {
        self.note_standard_builtin_call(builtin);
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        match builtin {
            StandardBuiltinId::ObjectDefineProperty => ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            StandardBuiltinId::ReflectDefineProperty => ValueInfo::new(ValueKind::Boolean),
            _ => unreachable!("forwarded defineProperty call must retain its builtin target"),
        }
    }

    fn define_property_call_info(
        &mut self,
        args: &[TypedExpr],
        call_kind: DefinePropertyCallKind,
    ) -> ValueInfo {
        let Some(target) = args.first() else {
            return Self::define_property_type_error_result(call_kind);
        };
        if target.possible_kinds.0 & Self::object_like_kind_set().0 == 0 {
            return Self::define_property_type_error_result(call_kind);
        }
        if self.is_builtin_property_expr(target, ARRAY_NAME, "prototype") {
            self.array_prototype_mutated = true;
        }

        let target_may_be_proxy = target.heap_shape.is_none();
        // Lowered argument shapes carry no evaluation epoch. Refresh the canonical global object
        // after later arguments. A fresh literal cannot have escaped yet, while exact function
        // targets remain distinguishable from structurally identical function shapes by identity.
        let target_for_effects = if matches!(
            &target.expr,
            ExprIr::Identifier(name) if name == GLOBAL_THIS_NAME
        ) && self.lookup_binding(GLOBAL_THIS_NAME).is_none()
            && self.lookup_global_property_info(GLOBAL_THIS_NAME).is_none()
        {
            Some(self.global_this_info())
        } else if matches!(
            &target.expr,
            ExprIr::ObjectLiteral(_) | ExprIr::ArrayLiteral(_)
        ) || (target.possible_kinds == KindSet::from_kind(ValueKind::Function)
            && target
                .function_targets
                .exact_targets()
                .is_some_and(|targets| !targets.is_empty()))
        {
            Some(target.value_info())
        } else {
            None
        };
        let key_may_call_user_code = args
            .get(1)
            .is_some_and(|key| key.possible_kinds.intersects(Self::object_like_kind_set()));
        let object_prototype_shape = self
            .is_intrinsic_global_constructor(OBJECT_NAME)
            .then(|| self.lookup_global_property(OBJECT_NAME))
            .flatten()
            .and_then(|constructor| {
                read_heap_shape_property(constructor.heap_shape.as_deref()?, "prototype")
            })
            .and_then(|prototype| match prototype {
                ObjectShapeProperty::Data(prototype) => prototype.heap_shape,
                ObjectShapeProperty::Accessor { .. } => None,
            });

        let mut descriptor_field_getters = PropertyHookTargets::default();
        let mut descriptor_field_read_precision = if key_may_call_user_code {
            DescriptorFieldReadPrecision::Unknown
        } else {
            DescriptorFieldReadPrecision::Exact
        };
        if let Some(descriptor) = args.get(2) {
            if matches!(
                descriptor_field_read_precision,
                DescriptorFieldReadPrecision::Exact
            ) {
                match (&descriptor.expr, descriptor.heap_shape.as_deref()) {
                    (ExprIr::ObjectLiteral(properties), Some(HeapShape::Object(shape)))
                        if !properties.iter().any(|property| {
                            matches!(property, ObjectPropertyIr::PrototypeSetter { .. })
                        }) =>
                    {
                        let descriptor_field_names = [
                            "enumerable",
                            "configurable",
                            "value",
                            "writable",
                            "get",
                            "set",
                        ];
                        for (field_index, name) in descriptor_field_names.iter().enumerate() {
                            let property = match shape.properties.get(*name) {
                                Some(property) => Some(property.clone()),
                                None => match object_prototype_shape.as_deref() {
                                    Some(prototype) => read_heap_shape_property(prototype, name),
                                    None => {
                                        descriptor_field_read_precision =
                                            DescriptorFieldReadPrecision::Unknown;
                                        break;
                                    }
                                },
                            };
                            let Some(ObjectShapeProperty::Accessor {
                                getter: Some(getter),
                                ..
                            }) = property
                            else {
                                continue;
                            };

                            let getter_may_run_user_code =
                                self.function_may_run_user_code_synchronously(&getter.function_id);
                            descriptor_field_getters.extend_known([getter.function_id.clone()]);
                            self.merge_function_this_info(
                                &getter.function_id,
                                descriptor.value_info(),
                            );
                            if let Some(signature) =
                                self.function_signatures.get_mut(&getter.function_id)
                            {
                                Self::merge_omitted_signature_params_as_undefined(signature, 0);
                            }

                            if getter_may_run_user_code
                                && field_index + 1 < descriptor_field_names.len()
                            {
                                descriptor_field_read_precision =
                                    DescriptorFieldReadPrecision::Unknown;
                                break;
                            }
                        }
                    }
                    _ => {
                        descriptor_field_read_precision = DescriptorFieldReadPrecision::Unknown;
                    }
                }
            }

            if matches!(
                descriptor_field_read_precision,
                DescriptorFieldReadPrecision::Unknown
            ) {
                descriptor_field_getters
                    .include_all_planned_source(self.analysis.planned_source_function_ids.clone());
            }
        }

        let descriptor_field_access_is_unknown = matches!(
            descriptor_field_read_precision,
            DescriptorFieldReadPrecision::Unknown
        );

        if target_may_be_proxy || key_may_call_user_code || descriptor_field_access_is_unknown {
            self.observe_all_planned_source_as_unknown_property_hooks();
        }
        if let Some(descriptor) = args.get(2) {
            if let Some(getter) = self.read_object_shape(descriptor, "get") {
                self.dynamically_installed_getters
                    .extend(getter.function_targets.known_targets().iter().cloned());
            }
            if let Some(setter) = self.read_object_shape(descriptor, "set") {
                self.dynamically_installed_setters
                    .extend(setter.function_targets.known_targets().iter().cloned());
            }
        }

        let descriptor_may_call_user_code =
            descriptor_field_access_is_unknown || !descriptor_field_getters.is_empty();
        if target_may_be_proxy
            || target_for_effects.is_none()
            || key_may_call_user_code
            || descriptor_may_call_user_code
        {
            self.invalidate_unknown_user_code_effects();
        } else {
            let target_for_effects = target_for_effects
                .expect("precisely accounted defineProperty target must be identified");
            let authorities = self
                .ordinary_property_mutation_authorities(std::slice::from_ref(&target_for_effects));
            let key = match args.get(1) {
                Some(TypedExpr {
                    kind: ValueKind::String,
                    expr: ExprIr::String(name),
                    ..
                }) => PropertyKeyIr::StaticString(name.clone()),
                Some(key) => PropertyKeyIr::StringExpr(Box::new(key.clone())),
                None => PropertyKeyIr::StringExpr(Box::new(TypedExpr::undefined())),
            };
            self.record_ordinary_property_mutation_authority_effects(&key, &authorities);
            self.invalidate_ordinary_property_shape_aliases(&target_for_effects);
        }

        match call_kind {
            DefinePropertyCallKind::Object => {
                let mut result = target.value_info();
                result.heap_shape = None;
                result
            }
            DefinePropertyCallKind::Reflect => ValueInfo::new(ValueKind::Boolean),
        }
    }

    fn define_property_type_error_result(call_kind: DefinePropertyCallKind) -> ValueInfo {
        match call_kind {
            DefinePropertyCallKind::Object => ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            DefinePropertyCallKind::Reflect => ValueInfo::new(ValueKind::Boolean),
        }
    }
}
