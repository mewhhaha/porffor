use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn standard_builtin_call_info(
        &mut self,
        builtin: StandardBuiltinId,
        args: &[TypedExpr],
        context: BuiltinCallContext,
    ) -> Option<ValueInfo> {
        self.record_boxed_builtin_invocation(builtin, context);
        match builtin {
            StandardBuiltinId::EvalFunction | StandardBuiltinId::FunctionConstructor => {
                unreachable!(
                    "dynamic-source builtins must consume their resolved disposition before builtin result analysis"
                )
            }
            StandardBuiltinId::FunctionPrototype => Some(ValueInfo::undefined()),
            StandardBuiltinId::FunctionPrototypeSymbolHasInstance => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::PromiseConstructor => {
                if let Some(executor_id) = args
                    .first()
                    .and_then(|executor| self.resolve_single_function_target(executor))
                {
                    self.merge_function_param_infos(
                        &executor_id,
                        &[
                            Self::standard_builtin_value_info(
                                StandardBuiltinId::PromiseResolveFunction,
                            ),
                            Self::standard_builtin_value_info(
                                StandardBuiltinId::PromiseRejectFunction,
                            ),
                        ],
                    );
                    self.merge_function_this_info(&executor_id, ValueInfo::undefined());
                }
                Some(Self::value_info_from_shape(Some(
                    Self::promise_instance_shape(),
                )))
            }
            StandardBuiltinId::PromiseResolve
            | StandardBuiltinId::PromiseTry
            | StandardBuiltinId::PromiseReject
            | StandardBuiltinId::PromiseAll
            | StandardBuiltinId::PromiseAllSettled
            | StandardBuiltinId::PromiseAllKeyed
            | StandardBuiltinId::PromiseAllSettledKeyed
            | StandardBuiltinId::PromiseAny
            | StandardBuiltinId::PromiseRace
            | StandardBuiltinId::ArrayFromAsync => Some(Self::value_info_from_shape(Some(
                Self::promise_instance_shape(),
            ))),
            StandardBuiltinId::PromiseWithResolvers => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::PromisePrototypeThen
            | StandardBuiltinId::PromisePrototypeCatch
            | StandardBuiltinId::PromisePrototypeFinally
            | StandardBuiltinId::PromiseThenFinally
            | StandardBuiltinId::PromiseCatchFinally
            | StandardBuiltinId::PromiseValueThunk
            | StandardBuiltinId::PromiseThrower
            | StandardBuiltinId::PromiseSpeciesGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::PromiseCapabilityExecutor
            | StandardBuiltinId::PromiseAllResolveElement
            | StandardBuiltinId::PromiseAllSettledResolveElement
            | StandardBuiltinId::PromiseAllSettledRejectElement
            | StandardBuiltinId::PromiseAnyRejectElement
            | StandardBuiltinId::PromiseAllKeyedResolveElement
            | StandardBuiltinId::PromiseAllSettledKeyedResolveElement
            | StandardBuiltinId::PromiseAllSettledKeyedRejectElement
            | StandardBuiltinId::PromiseResolveFunction
            | StandardBuiltinId::PromiseRejectFunction
            | StandardBuiltinId::ArrayFromAsyncFulfilled
            | StandardBuiltinId::ArrayFromAsyncRejected => Some(ValueInfo::undefined()),
            StandardBuiltinId::MapConstructor => Some(Self::value_info_from_shape(Some(
                Self::map_instance_shape(),
            ))),
            StandardBuiltinId::MapSpeciesGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::MapGroupBy => Some(Self::value_info_from_shape(Some(
                Self::map_instance_shape(),
            ))),
            StandardBuiltinId::ObjectGroupBy | StandardBuiltinId::ObjectFromEntries => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::MapPrototypeSet => Some(Self::value_info_from_shape(Some(
                Self::map_instance_shape(),
            ))),
            StandardBuiltinId::MapPrototypeClear | StandardBuiltinId::MapPrototypeForEach => {
                Some(ValueInfo::undefined())
            }
            StandardBuiltinId::MapPrototypeKeys
            | StandardBuiltinId::MapPrototypeValues
            | StandardBuiltinId::MapPrototypeEntries => Some(Self::value_info_from_shape(Some(
                Self::map_iterator_instance_shape(),
            ))),
            StandardBuiltinId::MapIteratorNext => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::MapPrototypeDelete | StandardBuiltinId::MapPrototypeHas => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::MapPrototypeGet
            | StandardBuiltinId::MapPrototypeGetOrInsert
            | StandardBuiltinId::MapPrototypeGetOrInsertComputed => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::MapPrototypeSizeGetter => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::WeakMapConstructor => Some(Self::value_info_from_shape(Some(
                Self::weak_map_instance_shape(),
            ))),
            StandardBuiltinId::WeakMapPrototypeSet => Some(Self::value_info_from_shape(Some(
                Self::weak_map_instance_shape(),
            ))),
            StandardBuiltinId::WeakMapPrototypeDelete | StandardBuiltinId::WeakMapPrototypeHas => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::WeakMapPrototypeGet
            | StandardBuiltinId::WeakMapPrototypeGetOrInsert
            | StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::WeakSetConstructor => Some(Self::value_info_from_shape(Some(
                Self::weak_set_instance_shape(),
            ))),
            StandardBuiltinId::WeakSetPrototypeAdd => Some(Self::value_info_from_shape(Some(
                Self::weak_set_instance_shape(),
            ))),
            StandardBuiltinId::WeakSetPrototypeDelete | StandardBuiltinId::WeakSetPrototypeHas => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::WeakRefConstructor => Some(Self::value_info_from_shape(Some(
                Self::weak_ref_instance_shape(),
            ))),
            StandardBuiltinId::WeakRefPrototypeDeref => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::FinalizationRegistryConstructor => Some(
                Self::value_info_from_shape(Some(Self::finalization_registry_instance_shape())),
            ),
            StandardBuiltinId::FinalizationRegistryPrototypeRegister => {
                Some(ValueInfo::undefined())
            }
            StandardBuiltinId::FinalizationRegistryPrototypeUnregister => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            // Explicit-resource-management stack builtins. Every arm is
            // `Some`: `None` here is not "no static information", it is a
            // refusal — the three consumers read it as "this call does not
            // happen" and return `TypedExpr::undefined()` with the argument
            // vector replaced by `Vec::new()` (see the `let Some(...) else`
            // bindings at the `construct` and `call` sites). A `None` arm for a
            // builtin that must actually run silently drops the call.
            //
            // The kinds match `standard_builtin_signature` above; the
            // `IntlDateTimeFormatConstructor` arm is again the precedent for a
            // constructor with no instance shape.
            StandardBuiltinId::AsyncDisposableStackConstructor
            | StandardBuiltinId::DisposableStackConstructor
            | StandardBuiltinId::AsyncDisposableStackPrototypeMove
            | StandardBuiltinId::DisposableStackPrototypeMove
            | StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync => {
                Some(ValueInfo::new(ValueKind::Object))
            }
            StandardBuiltinId::AsyncDisposableStackPrototypeUse
            | StandardBuiltinId::AsyncDisposableStackPrototypeAdopt
            | StandardBuiltinId::DisposableStackPrototypeUse
            | StandardBuiltinId::DisposableStackPrototypeAdopt => {
                Some(ValueInfo::new(ValueKind::Dynamic))
            }
            StandardBuiltinId::AsyncDisposableStackPrototypeDefer
            | StandardBuiltinId::DisposableStackPrototypeDefer
            | StandardBuiltinId::DisposableStackPrototypeDispose
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected => {
                Some(ValueInfo::undefined())
            }
            StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter
            | StandardBuiltinId::DisposableStackPrototypeDisposedGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::SetConstructor => Some(Self::value_info_from_shape(Some(
                Self::set_instance_shape(),
            ))),
            StandardBuiltinId::SetSpeciesGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::SetPrototypeAdd => Some(Self::value_info_from_shape(Some(
                Self::set_instance_shape(),
            ))),
            StandardBuiltinId::SetPrototypeDifference
            | StandardBuiltinId::SetPrototypeIntersection
            | StandardBuiltinId::SetPrototypeSymmetricDifference
            | StandardBuiltinId::SetPrototypeUnion => Some(Self::value_info_from_shape(Some(
                Self::set_instance_shape(),
            ))),
            StandardBuiltinId::SetPrototypeIsDisjointFrom
            | StandardBuiltinId::SetPrototypeIsSubsetOf
            | StandardBuiltinId::SetPrototypeIsSupersetOf => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::SetPrototypeClear | StandardBuiltinId::SetPrototypeForEach => {
                Some(ValueInfo::undefined())
            }
            StandardBuiltinId::SetPrototypeValues | StandardBuiltinId::SetPrototypeEntries => Some(
                Self::value_info_from_shape(Some(Self::set_iterator_instance_shape())),
            ),
            StandardBuiltinId::SetIteratorNext => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::SetPrototypeDelete | StandardBuiltinId::SetPrototypeHas => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::SetPrototypeSizeGetter => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::FunctionPrototypeCall => {
                if let Some(this_arg) = args.first() {
                    if this_arg
                        .possible_kinds
                        .is_subset_of(Self::boxed_primitive_kind_set())
                    {
                        self.boxed_receiver_adaptations += 1;
                    }
                }
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::FunctionPrototypeApply => {
                if let Some(this_arg) = args.first() {
                    if this_arg
                        .possible_kinds
                        .is_subset_of(Self::boxed_primitive_kind_set())
                    {
                        self.boxed_receiver_adaptations += 1;
                    }
                }
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::FunctionPrototypeBind => {
                Some(Self::function_value_info_with_constructable(
                    StandardBuiltinId::BoundFunctionInvoker.function_id(),
                    true,
                ))
            }
            StandardBuiltinId::ObjectConstructor => {
                if let Some(arg) = args.first() {
                    let nullish = KindSet::from_kind(ValueKind::Undefined)
                        .union(KindSet::from_kind(ValueKind::Null));
                    if arg
                        .possible_kinds
                        .is_subset_of(Self::object_like_kind_set())
                    {
                        return Some(arg.value_info());
                    }
                    if arg.possible_kinds.is_subset_of(nullish) {
                        return Some(Self::fresh_constructed_instance_info());
                    }
                    if arg
                        .possible_kinds
                        .is_subset_of(Self::boxed_primitive_kind_set())
                    {
                        return Some(Self::boxed_primitive_instance_info(arg.value_info()));
                    }
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: Object primitive boxing"
                    ));
                    None
                } else {
                    Some(Self::fresh_constructed_instance_info())
                }
            }
            StandardBuiltinId::ObjectCreate => {
                if args.get(1).is_some_and(|properties| {
                    properties.possible_kinds != KindSet::from_kind(ValueKind::Undefined)
                }) {
                    self.invalidate_unknown_user_code_effects();
                    return Some(ValueInfo::new(ValueKind::Object));
                }
                let Some(proto) = args.first() else {
                    return Some(ValueInfo::new(ValueKind::Object));
                };
                let null_kind = KindSet::from_kind(ValueKind::Null);
                let allowed = Self::object_like_kind_set().union(null_kind);
                if !proto.possible_kinds.is_subset_of(allowed) {
                    return Some(ValueInfo::new(ValueKind::Object));
                }
                if proto.possible_kinds == null_kind {
                    return Some(Self::value_info_from_shape(Some(Box::new(
                        HeapShape::Object(ObjectShape {
                            prototype: None,
                            properties: BTreeMap::new(),
                            private_brands: BTreeSet::new(),
                            boxed_primitive: None,
                        }),
                    ))));
                }
                if proto.possible_kinds.contains(ValueKind::Null) {
                    return Some(ValueInfo::new(ValueKind::Object));
                }
                let Some(prototype) = proto.heap_shape.clone() else {
                    return Some(ValueInfo::new(ValueKind::Object));
                };
                Some(Self::value_info_from_shape(Some(Box::new(
                    HeapShape::Object(ObjectShape {
                        prototype: Some(prototype),
                        properties: BTreeMap::new(),
                        private_brands: BTreeSet::new(),
                        boxed_primitive: None,
                    }),
                ))))
            }
            StandardBuiltinId::ObjectGetPrototypeOf => {
                let Some(target) = args.first() else {
                    return Some(ValueInfo {
                        kind: ValueKind::Object,
                        possible_kinds: KindSet::from_kind(ValueKind::Object)
                            .union(KindSet::from_kind(ValueKind::Null)),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    });
                };
                if !target.function_targets.is_empty()
                    && target.function_targets.iter().all(|function_id| {
                        StandardBuiltinId::from_function_id(function_id)
                            .is_some_and(Self::is_typed_array_constructor)
                    })
                {
                    return Some(ValueInfo {
                        kind: ValueKind::Function,
                        possible_kinds: KindSet::from_kind(ValueKind::Function),
                        heap_shape: Some(Self::function_heap_shape(false)),
                        function_targets: BTreeSet::new(),
                    });
                }
                let prototype = target.heap_shape.as_deref().and_then(|shape| match shape {
                    HeapShape::Object(object) => object.prototype.clone(),
                    HeapShape::Array(array) => array
                        .prototype
                        .clone()
                        .or_else(|| Some(Self::array_prototype_shape())),
                });
                Some(if prototype.is_some() {
                    Self::value_info_from_shape(prototype)
                } else {
                    ValueInfo {
                        kind: ValueKind::Object,
                        possible_kinds: KindSet::from_kind(ValueKind::Object)
                            .union(KindSet::from_kind(ValueKind::Null)),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    }
                })
            }
            StandardBuiltinId::ObjectSetPrototypeOf => {
                let mut result = args
                    .first()
                    .map(TypedExpr::value_info)
                    .unwrap_or_else(ValueInfo::undefined);
                result.heap_shape = None;
                self.invalidate_unknown_user_code_effects();
                Some(result)
            }
            StandardBuiltinId::ObjectDefineProperty => {
                let Some(target) = args.first() else {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: Object.defineProperty requires object"
                    ));
                    return None;
                };
                if target.possible_kinds.0 & Self::object_like_kind_set().0 == 0 {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: Object.defineProperty requires object"
                    ));
                    return None;
                }
                if self.is_builtin_property_expr(target, ARRAY_NAME, "prototype") {
                    self.array_prototype_mutated = true;
                }
                if let Some(descriptor) = args.get(2) {
                    if let Some(getter) = self.read_object_shape(descriptor, "get") {
                        self.dynamically_installed_getters
                            .extend(getter.function_targets);
                    }
                    if let Some(setter) = self.read_object_shape(descriptor, "set") {
                        self.dynamically_installed_setters
                            .extend(setter.function_targets);
                    }
                }
                let mut result = target.value_info();
                result.heap_shape = None;
                self.invalidate_unknown_user_code_effects();
                Some(result)
            }
            StandardBuiltinId::ObjectDefineProperties => {
                if args.first().is_some_and(|target| {
                    self.is_builtin_property_expr(target, ARRAY_NAME, "prototype")
                }) {
                    self.array_prototype_mutated = true;
                }
                let mut result = args
                    .first()
                    .map(TypedExpr::value_info)
                    .unwrap_or_else(|| ValueInfo::new(ValueKind::Object));
                result.heap_shape = None;
                self.invalidate_unknown_user_code_effects();
                Some(result)
            }
            StandardBuiltinId::ObjectGetOwnPropertyDescriptor => {
                let Some(target) = args.first() else {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: Object.getOwnPropertyDescriptor requires object"
                    ));
                    return None;
                };
                if !target
                    .possible_kinds
                    .is_subset_of(Self::object_like_kind_set().union(KindSet::all_runtime_tags()))
                {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: Object.getOwnPropertyDescriptor requires object"
                    ));
                    return None;
                }
                if let Some(key_arg) = args.get(1) {
                    if let ExprIr::String(key) = &key_arg.expr {
                        let is_species_symbol = key_arg.kind == ValueKind::Symbol
                            && WellKnownSymbol::from_description(SymbolDescription::new(key))
                                == Some(WellKnownSymbol::Species);
                        let species_getter = if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::ArrayConstructor.function_id())
                        {
                            Some(StandardBuiltinId::ArraySpeciesGetter)
                        } else if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::ArrayBufferConstructor.function_id())
                        {
                            Some(StandardBuiltinId::ArrayBufferSpeciesGetter)
                        } else if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::RegExpConstructor.function_id())
                        {
                            Some(StandardBuiltinId::RegExpSpeciesGetter)
                        } else if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::PromiseConstructor.function_id())
                        {
                            Some(StandardBuiltinId::PromiseSpeciesGetter)
                        } else if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::MapConstructor.function_id())
                        {
                            Some(StandardBuiltinId::MapSpeciesGetter)
                        } else if is_species_symbol
                            && target
                                .function_targets
                                .contains(&StandardBuiltinId::SetConstructor.function_id())
                        {
                            Some(StandardBuiltinId::SetSpeciesGetter)
                        } else {
                            None
                        };
                        if let Some(species_getter) = species_getter {
                            return Some(ValueInfo {
                                kind: ValueKind::Object,
                                possible_kinds: KindSet::from_kind(ValueKind::Object),
                                heap_shape: Some(Self::property_descriptor_shape(vec![
                                    ("get", Self::standard_builtin_value_info(species_getter)),
                                    ("set", ValueInfo::undefined()),
                                    ("enumerable", Self::boolean_value_info()),
                                    ("configurable", Self::boolean_value_info()),
                                ])),
                                function_targets: BTreeSet::new(),
                            });
                        }
                        if let Some(property) = self.read_own_object_shape_property(target, key) {
                            let fields = match property {
                                ObjectShapeProperty::Data(value) => vec![
                                    ("value", value),
                                    ("writable", Self::boolean_value_info()),
                                    ("enumerable", Self::boolean_value_info()),
                                    ("configurable", Self::boolean_value_info()),
                                ],
                                ObjectShapeProperty::Accessor { getter, setter } => vec![
                                    (
                                        "get",
                                        getter
                                            .map(|accessor| {
                                                Self::function_value_info_with_constructable(
                                                    accessor.function_id,
                                                    false,
                                                )
                                            })
                                            .unwrap_or_else(ValueInfo::undefined),
                                    ),
                                    (
                                        "set",
                                        setter
                                            .map(|accessor| {
                                                Self::function_value_info_with_constructable(
                                                    accessor.function_id,
                                                    false,
                                                )
                                            })
                                            .unwrap_or_else(ValueInfo::undefined),
                                    ),
                                    ("enumerable", Self::boolean_value_info()),
                                    ("configurable", Self::boolean_value_info()),
                                ],
                            };
                            return Some(ValueInfo {
                                kind: ValueKind::Object,
                                possible_kinds: KindSet::from_kind(ValueKind::Object),
                                heap_shape: Some(Self::property_descriptor_shape(fields)),
                                function_targets: BTreeSet::new(),
                            });
                        }
                    }
                }
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object)
                        .union(KindSet::from_kind(ValueKind::Undefined)),
                    heap_shape: Some(Self::generic_property_descriptor_shape()),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ObjectAssign => {
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: Self::object_like_kind_set(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ObjectGetOwnPropertyDescriptors => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ObjectKeys => {
                let mut elements = Vec::new();
                if let Some(arg) = args.first() {
                    match arg.heap_shape.as_deref() {
                        Some(HeapShape::Array(shape)) => {
                            elements.extend((0..shape.elements.len()).map(|_| ValueInfo {
                                kind: ValueKind::String,
                                possible_kinds: KindSet::from_kind(ValueKind::String),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            }));
                            elements.extend(shape.properties.keys().map(|_| ValueInfo {
                                kind: ValueKind::String,
                                possible_kinds: KindSet::from_kind(ValueKind::String),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            }));
                        }
                        Some(HeapShape::Object(shape)) => {
                            elements.extend(shape.properties.keys().map(|_| ValueInfo {
                                kind: ValueKind::String,
                                possible_kinds: KindSet::from_kind(ValueKind::String),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            }));
                        }
                        _ => {}
                    }
                }
                Some(Self::array_value_info_from_elements(elements))
            }
            StandardBuiltinId::ObjectValues => {
                let mut elements = Vec::new();
                if let Some(arg) = args.first() {
                    match arg.heap_shape.as_deref() {
                        Some(HeapShape::Array(shape)) => {
                            elements.extend(shape.elements.iter().cloned());
                            elements.extend(shape.properties.values().filter_map(|property| {
                                match property {
                                    ObjectShapeProperty::Data(info) => Some(info.clone()),
                                    ObjectShapeProperty::Accessor { .. } => None,
                                }
                            }));
                        }
                        Some(HeapShape::Object(shape)) => {
                            elements.extend(shape.properties.values().filter_map(|property| {
                                match property {
                                    ObjectShapeProperty::Data(info) => Some(info.clone()),
                                    ObjectShapeProperty::Accessor { .. } => None,
                                }
                            }));
                        }
                        _ => {}
                    }
                }
                Some(Self::array_value_info_from_elements(elements))
            }
            StandardBuiltinId::ObjectEntries => {
                let shape = ArrayShape::default();
                Some(Self::value_info_from_shape(Some(Box::new(
                    HeapShape::Array(shape),
                ))))
            }
            StandardBuiltinId::ObjectIs
            | StandardBuiltinId::ObjectIsSealed
            | StandardBuiltinId::ObjectIsFrozen
            | StandardBuiltinId::ObjectIsExtensible => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::ObjectSeal
            | StandardBuiltinId::ObjectFreeze
            | StandardBuiltinId::ObjectPreventExtensions => {
                let mut result = args
                    .first()
                    .map(TypedExpr::value_info)
                    .unwrap_or_else(ValueInfo::undefined);
                result.heap_shape = None;
                self.invalidate_unknown_user_code_effects();
                Some(result)
            }
            StandardBuiltinId::ObjectPrototypeHasOwnProperty => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ObjectPrototypeLookupGetter
            | StandardBuiltinId::ObjectPrototypeLookupSetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Function)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ObjectPrototypeProtoGetter => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object)
                    .union(KindSet::from_kind(ValueKind::Null)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ObjectPrototypeProtoSetter => Some(ValueInfo::undefined()),
            StandardBuiltinId::ObjectPrototypePropertyIsEnumerable => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ObjectPrototypeIsPrototypeOf => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ObjectPrototypeValueOf => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ErrorIsError => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::ObjectPrototypeToString => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::ObjectPrototypeToLocaleString => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ProxyConstructor => {
                if let (Some(target), Some(handler)) = (args.first(), args.get(1)) {
                    self.observe_proxy_handler_traps(target, handler);
                }
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    // A Proxy is not an ordinary empty object: every property
                    // operation may dispatch a source trap. `None` preserves
                    // that exotic uncertainty through aliases and joins.
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ProxyRevocable => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ProxyRevoke => Some(ValueInfo::undefined()),
            StandardBuiltinId::ReflectConstruct => {
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ReflectApply => {
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ReflectGet => {
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ReflectGetPrototypeOf => {
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::from_kind(ValueKind::Object)
                        .union(KindSet::from_kind(ValueKind::Null)),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ReflectGetOwnPropertyDescriptor => {
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object)
                        .union(KindSet::from_kind(ValueKind::Undefined)),
                    heap_shape: Some(Self::generic_property_descriptor_shape()),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ReflectDefineProperty | StandardBuiltinId::ReflectDeleteProperty => {
                if args.first().is_some_and(|target| {
                    self.is_builtin_property_expr(target, ARRAY_NAME, "prototype")
                }) {
                    self.array_prototype_mutated = true;
                }
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ReflectSet => {
                if args.first().is_some_and(|target| {
                    self.is_builtin_property_expr(target, ARRAY_NAME, "prototype")
                }) {
                    self.array_prototype_mutated = true;
                }
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ReflectHas => {
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ReflectIsExtensible => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::ReflectPreventExtensions
            | StandardBuiltinId::ReflectSetPrototypeOf => {
                self.invalidate_unknown_user_code_effects();
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ReflectOwnKeys => {
                let shape = ArrayShape::default();
                Some(Self::value_info_from_shape(Some(Box::new(
                    HeapShape::Array(shape),
                ))))
            }
            StandardBuiltinId::ArrayConstructor => {
                let mut shape = ArrayShape::default();
                if !(args.len() == 1
                    && args[0].possible_kinds == KindSet::from_kind(ValueKind::Number))
                {
                    shape
                        .elements
                        .extend(args.iter().map(TypedExpr::value_info));
                }
                Some(ValueInfo {
                    kind: ValueKind::Array,
                    possible_kinds: KindSet::from_kind(ValueKind::Array),
                    heap_shape: Some(Box::new(HeapShape::Array(shape))),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayFrom => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayOf => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorFrom => {
                let base = args
                    .first()
                    .map(TypedExpr::value_info)
                    .unwrap_or(ValueInfo {
                        kind: ValueKind::Object,
                        possible_kinds: Self::object_like_kind_set(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    });
                Some(self.iterator_from_wrapper_value_info(base))
            }
            StandardBuiltinId::IteratorConcat => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_concat_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorZip | StandardBuiltinId::IteratorZipKeyed => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Self::iterator_zip_helper_shape()),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayIsArray => Some(ValueInfo {
                kind: ValueKind::Boolean,
                possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::NumberIsInteger | StandardBuiltinId::NumberIsSafeInteger => {
                Some(ValueInfo {
                    kind: ValueKind::Boolean,
                    possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::NumberIsNaN
            | StandardBuiltinId::NumberIsFinite
            | StandardBuiltinId::GlobalIsFinite
            | StandardBuiltinId::GlobalIsNaN => Some(ValueInfo {
                kind: ValueKind::Boolean,
                possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::NumberPrototypeToExponential
            | StandardBuiltinId::NumberPrototypeToFixed
            | StandardBuiltinId::NumberPrototypeToPrecision
            | StandardBuiltinId::NumberPrototypeToString
            | StandardBuiltinId::NumberPrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::NumberPrototypeValueOf => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::BooleanPrototypeToString => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::BooleanPrototypeValueOf => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::MathAbs
            | StandardBuiltinId::MathAcos
            | StandardBuiltinId::MathAcosh
            | StandardBuiltinId::MathAsin
            | StandardBuiltinId::MathAsinh
            | StandardBuiltinId::MathAtan
            | StandardBuiltinId::MathAtan2
            | StandardBuiltinId::MathAtanh
            | StandardBuiltinId::MathCbrt
            | StandardBuiltinId::MathCeil
            | StandardBuiltinId::MathClz32
            | StandardBuiltinId::MathCos
            | StandardBuiltinId::MathCosh
            | StandardBuiltinId::MathExp
            | StandardBuiltinId::MathExpm1
            | StandardBuiltinId::MathF16Round
            | StandardBuiltinId::MathFloor
            | StandardBuiltinId::MathFround
            | StandardBuiltinId::MathHypot
            | StandardBuiltinId::MathImul
            | StandardBuiltinId::MathLog
            | StandardBuiltinId::MathLog10
            | StandardBuiltinId::MathLog1p
            | StandardBuiltinId::MathLog2
            | StandardBuiltinId::MathPow
            | StandardBuiltinId::MathRandom
            | StandardBuiltinId::MathRound
            | StandardBuiltinId::MathSign
            | StandardBuiltinId::MathSin
            | StandardBuiltinId::MathSinh
            | StandardBuiltinId::MathSqrt
            | StandardBuiltinId::MathSumPrecise
            | StandardBuiltinId::MathTan
            | StandardBuiltinId::MathTanh
            | StandardBuiltinId::MathTrunc
            | StandardBuiltinId::MathMin
            | StandardBuiltinId::MathMax => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeConcat
            | StandardBuiltinId::ArrayPrototypeSlice
            | StandardBuiltinId::ArrayPrototypeSplice => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::TypedArrayPrototypeReverse
            | StandardBuiltinId::TypedArrayPrototypeSort
            | StandardBuiltinId::TypedArrayPrototypeSubarray
            | StandardBuiltinId::TypedArrayPrototypeSlice
            | StandardBuiltinId::TypedArrayPrototypeToReversed
            | StandardBuiltinId::TypedArrayPrototypeToSorted
            | StandardBuiltinId::TypedArrayPrototypeWith
            | StandardBuiltinId::TypedArrayPrototypeCopyWithin => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TypedArrayPrototypeSet => Some(ValueInfo::undefined()),
            StandardBuiltinId::ArrayPrototypeJoin
            | StandardBuiltinId::ArrayPrototypeToLocaleString
            | StandardBuiltinId::TypedArrayPrototypeToString
            | StandardBuiltinId::TypedArrayPrototypeJoin
            | StandardBuiltinId::TypedArrayPrototypeToLocaleString => Some(ValueInfo {
                kind: ValueKind::String,
                possible_kinds: KindSet::from_kind(ValueKind::String),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeFlat => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeFlatMap => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeAt | StandardBuiltinId::TypedArrayPrototypeAt => {
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayPrototypeToReversed => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeWith => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeToSpliced => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeToSorted => Some(Self::unshaped_array_result_info()),
            StandardBuiltinId::ArrayPrototypeReverse => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeCopyWithin => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: Self::object_like_kind_set(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeIncludes
            | StandardBuiltinId::TypedArrayPrototypeIncludes => Some(ValueInfo {
                kind: ValueKind::Boolean,
                possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeIndexOf
            | StandardBuiltinId::TypedArrayPrototypeIndexOf => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeLastIndexOf
            | StandardBuiltinId::TypedArrayPrototypeLastIndexOf => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeFind | StandardBuiltinId::TypedArrayPrototypeFind => {
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayPrototypeFindIndex
            | StandardBuiltinId::TypedArrayPrototypeFindIndex => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeFindLast
            | StandardBuiltinId::TypedArrayPrototypeFindLast => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeFindLastIndex
            | StandardBuiltinId::TypedArrayPrototypeFindLastIndex => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeEvery
            | StandardBuiltinId::TypedArrayPrototypeEvery => Some(ValueInfo {
                kind: ValueKind::Boolean,
                possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeSome | StandardBuiltinId::TypedArrayPrototypeSome => {
                Some(ValueInfo {
                    kind: ValueKind::Boolean,
                    possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayPrototypeForEach
            | StandardBuiltinId::TypedArrayPrototypeForEach => Some(ValueInfo::undefined()),
            StandardBuiltinId::ArrayPrototypeFilter => Some(ValueInfo {
                kind: ValueKind::Array,
                possible_kinds: KindSet::from_kind(ValueKind::Array),
                heap_shape: Some(Box::new(HeapShape::Array(ArrayShape::default()))),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeMap => Some(ValueInfo {
                kind: ValueKind::Array,
                possible_kinds: KindSet::from_kind(ValueKind::Array),
                heap_shape: Some(Box::new(HeapShape::Array(ArrayShape::default()))),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TypedArrayPrototypeMap
            | StandardBuiltinId::TypedArrayPrototypeFilter => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeReduce
            | StandardBuiltinId::ArrayPrototypeReduceRight
            | StandardBuiltinId::TypedArrayPrototypeReduce
            | StandardBuiltinId::TypedArrayPrototypeReduceRight => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypePop => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeShift => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeFill | StandardBuiltinId::ArrayPrototypeSort => {
                Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::ArrayPrototypePush => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeUnshift => Some(ValueInfo {
                kind: ValueKind::Number,
                possible_kinds: KindSet::from_kind(ValueKind::Number),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayPrototypeKeys
            | StandardBuiltinId::ArrayPrototypeEntries
            | StandardBuiltinId::ArrayPrototypeValues
            | StandardBuiltinId::TypedArrayPrototypeKeys
            | StandardBuiltinId::TypedArrayPrototypeEntries
            | StandardBuiltinId::TypedArrayPrototypeValues
            | StandardBuiltinId::StringPrototypeIterator => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::array_iterator_instance_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorConstructor => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::IteratorConstructor,
            )),
            StandardBuiltinId::IteratorPrototypeToArray => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::IteratorPrototypeToArray,
            )),
            StandardBuiltinId::IteratorPrototypeForEach => Some(ValueInfo::undefined()),
            StandardBuiltinId::IteratorPrototypeEvery => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::IteratorPrototypeSome => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::IteratorPrototypeFind => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorPrototypeReduce => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorPrototypeMap => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_map_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorConcatNext
            | StandardBuiltinId::IteratorConcatReturn
            | StandardBuiltinId::IteratorZipNext
            | StandardBuiltinId::IteratorZipReturn => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorHelperNext | StandardBuiltinId::IteratorHelperReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorPrototypeFilter => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_filter_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorPrototypeFlatMap => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_flat_map_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorPrototypeTake => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_take_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorPrototypeDrop => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::iterator_drop_helper_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::IteratorTakeNext | StandardBuiltinId::IteratorTakeReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorDropNext | StandardBuiltinId::IteratorDropReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorMapNext | StandardBuiltinId::IteratorMapReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorFilterNext | StandardBuiltinId::IteratorFilterReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorFlatMapNext | StandardBuiltinId::IteratorFlatMapReturn => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::IteratorPrototypeConstructorGetter => {
                Some(Self::standard_builtin_value_info(
                    StandardBuiltinId::IteratorPrototypeConstructorGetter,
                ))
            }
            StandardBuiltinId::IteratorPrototypeConstructorSetter => {
                Some(Self::standard_builtin_value_info(
                    StandardBuiltinId::IteratorPrototypeConstructorSetter,
                ))
            }
            StandardBuiltinId::IteratorPrototypeSymbolDispose => {
                Some(Self::standard_builtin_value_info(
                    StandardBuiltinId::IteratorPrototypeSymbolDispose,
                ))
            }
            StandardBuiltinId::IteratorPrototypeToStringTagGetter => {
                Some(Self::standard_builtin_value_info(
                    StandardBuiltinId::IteratorPrototypeToStringTagGetter,
                ))
            }
            StandardBuiltinId::IteratorPrototypeToStringTagSetter => {
                Some(Self::standard_builtin_value_info(
                    StandardBuiltinId::IteratorPrototypeToStringTagSetter,
                ))
            }
            StandardBuiltinId::IteratorFromWrapperNext => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::IteratorFromWrapperNext,
            )),
            StandardBuiltinId::IteratorFromWrapperReturn => Some(
                Self::standard_builtin_value_info(StandardBuiltinId::IteratorFromWrapperReturn),
            ),
            StandardBuiltinId::ArrayIteratorNext
            | StandardBuiltinId::StringIteratorNext
            | StandardBuiltinId::GeneratorPrototypeNext
            | StandardBuiltinId::GeneratorPrototypeReturn
            | StandardBuiltinId::GeneratorPrototypeThrow => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::AsyncGeneratorPrototypeNext
            | StandardBuiltinId::AsyncGeneratorPrototypeReturn
            | StandardBuiltinId::AsyncGeneratorPrototypeThrow
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose => Some(
                Self::value_info_from_shape(Some(Self::promise_instance_shape())),
            ),
            StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected => {
                Some(ValueInfo::undefined())
            }
            StandardBuiltinId::ArrayIteratorIdentity => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Box::new(Self::empty_object_shape())),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ArrayBufferConstructor => Some(Self::value_info_from_shape(Some(
                Self::array_buffer_instance_shape(),
            ))),
            StandardBuiltinId::SharedArrayBufferConstructor => Some(Self::value_info_from_shape(
                Some(Self::shared_array_buffer_instance_shape()),
            )),
            StandardBuiltinId::ArraySpeciesGetter => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::ArrayConstructor,
            )),
            StandardBuiltinId::TypedArraySpeciesGetter => Some(ValueInfo::new(ValueKind::Function)),
            StandardBuiltinId::ArrayBufferSpeciesGetter => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::ArrayBufferConstructor,
            )),
            StandardBuiltinId::RegExpSpeciesGetter => Some(Self::standard_builtin_value_info(
                StandardBuiltinId::RegExpConstructor,
            )),
            StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::SharedArrayBufferPrototypeGrow => Some(ValueInfo::undefined()),
            StandardBuiltinId::ArrayBufferPrototypeDetachedGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::ArrayBufferPrototypeResizableGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::ArrayBufferPrototypeResize => Some(ValueInfo::undefined()),
            StandardBuiltinId::ArrayBufferPrototypeSlice => Some(Self::value_info_from_shape(
                Some(Self::array_buffer_instance_shape()),
            )),
            StandardBuiltinId::SharedArrayBufferPrototypeSlice => Some(
                Self::value_info_from_shape(Some(Self::shared_array_buffer_instance_shape())),
            ),
            StandardBuiltinId::ArrayBufferPrototypeTransfer
            | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
            | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
            | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable => Some(
                Self::value_info_from_shape(Some(Self::array_buffer_instance_shape())),
            ),
            StandardBuiltinId::ArrayBufferIsView => Some(ValueInfo {
                kind: ValueKind::Boolean,
                possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::DataViewConstructor => {
                let Some(buffer) = args.first() else {
                    self.unsupported_with_message(
                        "unsupported in lila wasm-aot first slice: DataView requires ArrayBuffer"
                            .to_string(),
                    );
                    return None;
                };
                let _ = buffer;
                Some(Self::value_info_from_shape(Some(
                    Self::data_view_instance_shape(),
                )))
            }
            StandardBuiltinId::DataViewPrototypeBufferGetter => Some(Self::value_info_from_shape(
                Some(Self::array_buffer_instance_shape()),
            )),
            StandardBuiltinId::TypedArrayPrototypeBufferGetter => Some(
                Self::value_info_from_shape(Some(Self::array_buffer_instance_shape())),
            ),
            StandardBuiltinId::DataViewPrototypeByteLengthGetter
            | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeLengthGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TypedArrayPrototypeToStringTagGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Undefined)
                    .union(KindSet::from_kind(ValueKind::String)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TypedArrayFrom | StandardBuiltinId::TypedArrayOf => {
                Some(ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                    function_targets: BTreeSet::new(),
                })
            }
            StandardBuiltinId::Float64ArrayConstructor
            | StandardBuiltinId::Float32ArrayConstructor
            | StandardBuiltinId::Int32ArrayConstructor
            | StandardBuiltinId::Int16ArrayConstructor
            | StandardBuiltinId::Int8ArrayConstructor
            | StandardBuiltinId::Uint32ArrayConstructor
            | StandardBuiltinId::Uint16ArrayConstructor
            | StandardBuiltinId::Uint8ArrayConstructor
            | StandardBuiltinId::Uint8ClampedArrayConstructor
            | StandardBuiltinId::BigInt64ArrayConstructor
            | StandardBuiltinId::BigUint64ArrayConstructor => Some(Self::value_info_from_shape(
                Some(Self::typed_array_instance_shape_for_constructor(builtin)),
            )),
            StandardBuiltinId::DataViewPrototypeGetUint8
            | StandardBuiltinId::DataViewPrototypeGetInt8
            | StandardBuiltinId::DataViewPrototypeGetUint16
            | StandardBuiltinId::DataViewPrototypeGetInt16
            | StandardBuiltinId::DataViewPrototypeGetUint32
            | StandardBuiltinId::DataViewPrototypeGetInt32
            | StandardBuiltinId::DataViewPrototypeGetFloat16
            | StandardBuiltinId::DataViewPrototypeGetFloat32
            | StandardBuiltinId::DataViewPrototypeGetFloat64 => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::DataViewPrototypeGetBigInt64
            | StandardBuiltinId::DataViewPrototypeGetBigUint64 => {
                Some(ValueInfo::new(ValueKind::BigInt))
            }
            StandardBuiltinId::DataViewPrototypeSetUint8
            | StandardBuiltinId::DataViewPrototypeSetInt8
            | StandardBuiltinId::DataViewPrototypeSetUint16
            | StandardBuiltinId::DataViewPrototypeSetInt16
            | StandardBuiltinId::DataViewPrototypeSetUint32
            | StandardBuiltinId::DataViewPrototypeSetInt32
            | StandardBuiltinId::DataViewPrototypeSetFloat16
            | StandardBuiltinId::DataViewPrototypeSetFloat32
            | StandardBuiltinId::DataViewPrototypeSetFloat64
            | StandardBuiltinId::DataViewPrototypeSetBigInt64
            | StandardBuiltinId::DataViewPrototypeSetBigUint64 => Some(ValueInfo::undefined()),
            StandardBuiltinId::BigIntConstructor => {
                if context == BuiltinCallContext::Construct {
                    Some(ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    })
                } else {
                    Some(ValueInfo::new(ValueKind::BigInt))
                }
            }
            StandardBuiltinId::BigIntAsIntN
            | StandardBuiltinId::BigIntAsUintN
            | StandardBuiltinId::BigIntPrototypeValueOf => Some(ValueInfo::new(ValueKind::BigInt)),
            StandardBuiltinId::BigIntPrototypeToString
            | StandardBuiltinId::BigIntPrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::NumberConstructor => {
                if context == BuiltinCallContext::Construct {
                    Some(Self::boxed_primitive_instance_info(ValueInfo::new(
                        ValueKind::Number,
                    )))
                } else {
                    Some(ValueInfo::new(ValueKind::Number))
                }
            }
            StandardBuiltinId::StringConstructor => {
                if context == BuiltinCallContext::Construct {
                    Some(Self::boxed_primitive_instance_info(ValueInfo::new(
                        ValueKind::String,
                    )))
                } else {
                    Some(ValueInfo::new(ValueKind::String))
                }
            }
            StandardBuiltinId::StringFromCharCode
            | StandardBuiltinId::StringFromCodePoint
            | StandardBuiltinId::StringRaw => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::StringPrototypeToString
            | StandardBuiltinId::StringPrototypeValueOf => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::StringPrototypeCharAt | StandardBuiltinId::StringPrototypeConcat => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::StringPrototypeCharCodeAt => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::StringPrototypeIndexOf
            | StandardBuiltinId::StringPrototypeLastIndexOf
            | StandardBuiltinId::StringPrototypeLocaleCompare => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::StringPrototypeCodePointAt => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::StringPrototypeEndsWith
            | StandardBuiltinId::StringPrototypeIncludes
            | StandardBuiltinId::StringPrototypeStartsWith => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::StringPrototypeMatchAll
            | StandardBuiltinId::RegExpPrototypeSymbolMatchAll => Some(ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: Some(Self::array_iterator_instance_shape()),
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::BooleanConstructor => {
                if context == BuiltinCallContext::Construct {
                    Some(Self::boxed_primitive_instance_info(ValueInfo::new(
                        ValueKind::Boolean,
                    )))
                } else {
                    Some(ValueInfo::new(ValueKind::Boolean))
                }
            }
            StandardBuiltinId::SymbolConstructor => Some(ValueInfo::new(ValueKind::Symbol)),
            StandardBuiltinId::SymbolFor => Some(ValueInfo::new(ValueKind::Symbol)),
            StandardBuiltinId::SymbolKeyFor => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::SymbolPrototypeDescriptionGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::SymbolPrototypeToString => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::SymbolPrototypeValueOf
            | StandardBuiltinId::SymbolPrototypeToPrimitive => {
                Some(ValueInfo::new(ValueKind::Symbol))
            }
            StandardBuiltinId::ErrorConstructor
            | StandardBuiltinId::EvalErrorConstructor
            | StandardBuiltinId::AggregateErrorConstructor
            | StandardBuiltinId::SuppressedErrorConstructor
            | StandardBuiltinId::RangeErrorConstructor
            | StandardBuiltinId::SyntaxErrorConstructor
            | StandardBuiltinId::TypeErrorConstructor
            | StandardBuiltinId::URIErrorConstructor
            | StandardBuiltinId::ReferenceErrorConstructor => {
                Some(Self::standard_error_instance_info(builtin))
            }
            StandardBuiltinId::FunctionPrototypeToString
            | StandardBuiltinId::ErrorPrototypeToString
            | StandardBuiltinId::StringPrototypeAnchor
            | StandardBuiltinId::StringPrototypeBig
            | StandardBuiltinId::StringPrototypeBlink
            | StandardBuiltinId::StringPrototypeBold
            | StandardBuiltinId::StringPrototypeFixed
            | StandardBuiltinId::StringPrototypeFontcolor
            | StandardBuiltinId::StringPrototypeFontsize
            | StandardBuiltinId::StringPrototypeItalics
            | StandardBuiltinId::StringPrototypeLink
            | StandardBuiltinId::StringPrototypeSmall
            | StandardBuiltinId::StringPrototypeStrike
            | StandardBuiltinId::StringPrototypeSub
            | StandardBuiltinId::StringPrototypeSubstr
            | StandardBuiltinId::StringPrototypeSubstring
            | StandardBuiltinId::StringPrototypeSup
            | StandardBuiltinId::StringPrototypeSlice
            | StandardBuiltinId::StringPrototypePadStart
            | StandardBuiltinId::StringPrototypePadEnd
            | StandardBuiltinId::StringPrototypeRepeat
            | StandardBuiltinId::StringPrototypeNormalize
            | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            | StandardBuiltinId::StringPrototypeToLocaleUpperCase
            | StandardBuiltinId::StringPrototypeToLowerCase
            | StandardBuiltinId::StringPrototypeToUpperCase
            | StandardBuiltinId::StringPrototypeTrim
            | StandardBuiltinId::StringPrototypeTrimStart
            | StandardBuiltinId::StringPrototypeTrimEnd
            | StandardBuiltinId::StringPrototypeToWellFormed
            | StandardBuiltinId::RegExpEscape
            | StandardBuiltinId::Escape
            | StandardBuiltinId::Unescape
            | StandardBuiltinId::EncodeUri
            | StandardBuiltinId::EncodeUriComponent
            | StandardBuiltinId::DecodeUri
            | StandardBuiltinId::DecodeUriComponent => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::StringPrototypeAt => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::StringPrototypeIsWellFormed => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::RegExpPrototypeTest => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::StringPrototypeMatch
            | StandardBuiltinId::StringPrototypeReplace
            | StandardBuiltinId::StringPrototypeReplaceAll
            | StandardBuiltinId::StringPrototypeSearch
            | StandardBuiltinId::StringPrototypeSplit
            | StandardBuiltinId::RegExpLegacyStaticGetter
            | StandardBuiltinId::RegExpPrototypeCompile
            | StandardBuiltinId::RegExpPrototypeExec
            | StandardBuiltinId::RegExpPrototypeToString
            | StandardBuiltinId::RegExpPrototypeSymbolMatch
            | StandardBuiltinId::RegExpPrototypeSymbolReplace
            | StandardBuiltinId::RegExpPrototypeSymbolSearch
            | StandardBuiltinId::RegExpPrototypeSymbolSplit => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::RegExpLegacyStaticSetter => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::DateConstructor => Some(Self::value_info_from_shape(Some(
                Self::date_instance_shape(),
            ))),
            StandardBuiltinId::TemporalNowTimeZoneId => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::TemporalPlainDateConstructor
            | StandardBuiltinId::TemporalPlainDateFrom
            | StandardBuiltinId::TemporalPlainDatePrototypeWith
            | StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDatePrototypeAdd
            | StandardBuiltinId::TemporalPlainDatePrototypeSubtract => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDatePrototypeUntil
            | StandardBuiltinId::TemporalPlainDatePrototypeSince => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_year_month_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_month_day_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainYearMonthConstructor
            | StandardBuiltinId::TemporalPlainYearMonthFrom
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeWith
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_year_month_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainMonthDayConstructor
            | StandardBuiltinId::TemporalPlainMonthDayFrom
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeWith => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_month_day_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSince => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainYearMonthCompare
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalDurationConstructor
            | StandardBuiltinId::TemporalDurationFrom
            | StandardBuiltinId::TemporalDurationPrototypeWith
            | StandardBuiltinId::TemporalDurationPrototypeNegated
            | StandardBuiltinId::TemporalDurationPrototypeAbs
            | StandardBuiltinId::TemporalDurationPrototypeAdd
            | StandardBuiltinId::TemporalDurationPrototypeSubtract
            | StandardBuiltinId::TemporalDurationPrototypeRound => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainTimeConstructor
            | StandardBuiltinId::TemporalPlainTimeFrom
            | StandardBuiltinId::TemporalPlainTimePrototypeWith
            | StandardBuiltinId::TemporalPlainTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainTimePrototypeRound => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainTimePrototypeSince => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainTimeCompare
            | StandardBuiltinId::TemporalPlainTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalPlainTimePrototypeToString
            | StandardBuiltinId::TemporalPlainTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalPlainTimePrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalPlainTimePrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalDurationCompare
            | StandardBuiltinId::TemporalDurationPrototypeYearsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMonthsGetter
            | StandardBuiltinId::TemporalDurationPrototypeWeeksGetter
            | StandardBuiltinId::TemporalDurationPrototypeDaysGetter
            | StandardBuiltinId::TemporalDurationPrototypeHoursGetter
            | StandardBuiltinId::TemporalDurationPrototypeMinutesGetter
            | StandardBuiltinId::TemporalDurationPrototypeSecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeSignGetter
            | StandardBuiltinId::TemporalDurationPrototypeTotal => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalDurationPrototypeToString
            | StandardBuiltinId::TemporalDurationPrototypeToJson
            | StandardBuiltinId::TemporalDurationPrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalDurationPrototypeBlankGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalDurationPrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalPlainDateCompare
            | StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeToString
            | StandardBuiltinId::TemporalPlainDatePrototypeToJson
            | StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalPlainDateTimeConstructor
            | StandardBuiltinId::TemporalPlainDateTimeFrom
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWith
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDateTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainDateTimePrototypeRound => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDateTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSince => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime => Some(
                Self::value_info_from_shape(Some(Self::temporal_zoned_date_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalPlainDateTimeCompare
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToString
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TemporalNowInstant => Some(Self::value_info_from_shape(Some(
                Self::temporal_instant_instance_shape(),
            ))),
            StandardBuiltinId::TemporalNowZonedDateTimeIso => Some(Self::value_info_from_shape(
                Some(Self::temporal_zoned_date_time_instance_shape()),
            )),
            StandardBuiltinId::TemporalInstantConstructor => Some(Self::value_info_from_shape(
                Some(Self::temporal_instant_instance_shape()),
            )),
            StandardBuiltinId::TemporalInstantFrom
            | StandardBuiltinId::TemporalInstantFromEpochMilliseconds
            | StandardBuiltinId::TemporalInstantFromEpochNanoseconds => Some(
                Self::value_info_from_shape(Some(Self::temporal_instant_instance_shape())),
            ),
            StandardBuiltinId::TemporalInstantCompare => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter => {
                Some(ValueInfo::new(ValueKind::BigInt))
            }
            StandardBuiltinId::TemporalInstantPrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::TemporalInstantPrototypeToString
            | StandardBuiltinId::TemporalInstantPrototypeToJson => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalInstantPrototypeValueOf => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::IntlGetCanonicalLocales => Some(ValueInfo::new(ValueKind::Array)),
            StandardBuiltinId::IntlLocaleConstructor => Some(Self::value_info_from_shape(Some(
                Self::intl_locale_instance_shape(),
            ))),
            StandardBuiltinId::IntlLocalePrototypeLanguageGetter
            | StandardBuiltinId::IntlLocalePrototypeBaseNameGetter
            | StandardBuiltinId::IntlLocalePrototypeToString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::IntlLocalePrototypeScriptGetter
            | StandardBuiltinId::IntlLocalePrototypeRegionGetter => None,
            StandardBuiltinId::IntlDateTimeFormatConstructor
            | StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions => {
                Some(ValueInfo::new(ValueKind::Object))
            }
            StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts => {
                Some(ValueInfo::new(ValueKind::Array))
            }
            StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter => {
                Some(ValueInfo::new(ValueKind::Function))
            }
            StandardBuiltinId::IntlDateTimeFormatBoundFormat
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalZonedDateTimeConstructor => Some(
                Self::value_info_from_shape(Some(Self::temporal_zoned_date_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalZonedDateTimeFrom => Some(Self::value_info_from_shape(
                Some(Self::temporal_zoned_date_time_instance_shape()),
            )),
            StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter => {
                Some(ValueInfo::new(ValueKind::BigInt))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeEquals => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            // Same declaration as the PlainDate / PlainDateTime / PlainYearMonth
            // era pair; see the sibling comment on the kind table above.
            StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter => {
                Some(ValueInfo::new(ValueKind::Undefined))
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant => Some(
                Self::value_info_from_shape(Some(Self::temporal_instant_instance_shape())),
            ),
            // Same grouping as the kind table above; the two tables must not be
            // able to disagree about which ZonedDateTime data methods return a
            // `Temporal.Duration` and which return a ZonedDateTime.
            StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalZonedDateTimePrototypeAdd
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract => Some(
                Self::value_info_from_shape(Some(Self::temporal_zoned_date_time_instance_shape())),
            ),
            StandardBuiltinId::TemporalZonedDateTimePrototypeUntil
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSince => Some(
                Self::value_info_from_shape(Some(Self::temporal_duration_instance_shape())),
            ),
            StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime => Some(
                Self::value_info_from_shape(Some(Self::temporal_plain_date_time_instance_shape())),
            ),
            StandardBuiltinId::RegExpConstructor => Some(Self::value_info_from_shape(Some(
                Self::regexp_prototype_shape(),
            ))),
            StandardBuiltinId::DateNow
            | StandardBuiltinId::DateParse
            | StandardBuiltinId::DateUtc
            | StandardBuiltinId::DatePrototypeGetTime
            | StandardBuiltinId::DatePrototypeSetTime
            | StandardBuiltinId::DatePrototypeValueOf
            | StandardBuiltinId::DatePrototypeGetFullYear
            | StandardBuiltinId::DatePrototypeGetUtcFullYear
            | StandardBuiltinId::DatePrototypeGetMonth
            | StandardBuiltinId::DatePrototypeGetUtcMonth
            | StandardBuiltinId::DatePrototypeGetDate
            | StandardBuiltinId::DatePrototypeGetUtcDate
            | StandardBuiltinId::DatePrototypeGetDay
            | StandardBuiltinId::DatePrototypeGetUtcDay
            | StandardBuiltinId::DatePrototypeGetHours
            | StandardBuiltinId::DatePrototypeGetUtcHours
            | StandardBuiltinId::DatePrototypeGetMinutes
            | StandardBuiltinId::DatePrototypeGetUtcMinutes
            | StandardBuiltinId::DatePrototypeGetSeconds
            | StandardBuiltinId::DatePrototypeGetUtcSeconds
            | StandardBuiltinId::DatePrototypeGetMilliseconds
            | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeGetTimezoneOffset
            | StandardBuiltinId::DatePrototypeGetYear
            | StandardBuiltinId::DatePrototypeSetYear
            | StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear
            | StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth
            | StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate
            | StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours
            | StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes
            | StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds
            | StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds => {
                Some(ValueInfo::new(ValueKind::Number))
            }
            StandardBuiltinId::DatePrototypeToJson => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::DatePrototypeToPrimitive => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::DatePrototypeToTemporalInstant => {
                Some(ValueInfo::new(ValueKind::Object))
            }
            StandardBuiltinId::DatePrototypeToIsoString
            | StandardBuiltinId::DatePrototypeToDateString
            | StandardBuiltinId::DatePrototypeToLocaleDateString
            | StandardBuiltinId::DatePrototypeToLocaleString
            | StandardBuiltinId::DatePrototypeToLocaleTimeString
            | StandardBuiltinId::DatePrototypeToTimeString
            | StandardBuiltinId::DatePrototypeToString
            | StandardBuiltinId::DatePrototypeToUtcString => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::AtomicsNotify => Some(ValueInfo::new(ValueKind::Number)),
            StandardBuiltinId::AtomicsWait => Some(ValueInfo::new(ValueKind::String)),
            StandardBuiltinId::AtomicsPause => Some(ValueInfo::new(ValueKind::Undefined)),
            StandardBuiltinId::AtomicsWaitAsync => Some(Self::value_info_from_shape(Some(
                Box::new(HeapShape::Object(ObjectShape {
                    prototype: Some(Box::new(Self::empty_object_shape())),
                    properties: BTreeMap::from([
                        (
                            "async".to_string(),
                            ObjectShapeProperty::Data(ValueInfo::new(ValueKind::Boolean)),
                        ),
                        (
                            "value".to_string(),
                            ObjectShapeProperty::Data(ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::from_kind(ValueKind::String)
                                    .union(KindSet::from_kind(ValueKind::Object)),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            }),
                        ),
                    ]),
                    private_brands: BTreeSet::new(),
                    boxed_primitive: None,
                })),
            ))),
            StandardBuiltinId::ObjectHasOwn
            | StandardBuiltinId::JsonIsRawJson
            | StandardBuiltinId::AtomicsIsLockFree => Some(ValueInfo::new(ValueKind::Boolean)),
            StandardBuiltinId::AtomicsAdd
            | StandardBuiltinId::AtomicsAnd
            | StandardBuiltinId::AtomicsCompareExchange
            | StandardBuiltinId::AtomicsExchange
            | StandardBuiltinId::AtomicsLoad
            | StandardBuiltinId::AtomicsOr
            | StandardBuiltinId::AtomicsSub
            | StandardBuiltinId::AtomicsStore
            | StandardBuiltinId::AtomicsXor => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::BigInt)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ObjectGetOwnPropertyNames
            | StandardBuiltinId::ObjectGetOwnPropertySymbols => Some(Self::value_info_from_shape(
                Some(Box::new(HeapShape::Array(ArrayShape::default()))),
            )),
            StandardBuiltinId::JsonParse => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::JsonStringify => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Undefined)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::JsonRawJson => Some(Self::value_info_from_shape(Some(Box::new(
                Self::raw_json_object_shape(),
            )))),
            StandardBuiltinId::RegExpPrototypeFlagsGetter => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::RegExpPrototypeSourceGetter => {
                Some(ValueInfo::new(ValueKind::String))
            }
            StandardBuiltinId::RegExpPrototypeHasIndicesGetter
            | StandardBuiltinId::RegExpPrototypeGlobalGetter
            | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
            | StandardBuiltinId::RegExpPrototypeMultilineGetter
            | StandardBuiltinId::RegExpPrototypeDotAllGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
            | StandardBuiltinId::RegExpPrototypeStickyGetter => {
                Some(ValueInfo::new(ValueKind::Boolean))
            }
            StandardBuiltinId::BoundFunctionInvoker if context == BuiltinCallContext::Construct => {
                Some(Self::fresh_constructed_instance_info())
            }
            StandardBuiltinId::BoundFunctionInvoker => Some(ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            }),
            StandardBuiltinId::ThrowTypeError => Some(ValueInfo::undefined()),
        }
    }
}
