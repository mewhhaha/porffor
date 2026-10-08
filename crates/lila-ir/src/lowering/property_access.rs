use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Primitive Gets share live intrinsic facts and the ordinary GetV effect
    /// boundary. A mutable prototype getter must invalidate caller facts even
    /// when the acquired property is never called.
    pub(super) fn lower_primitive_property_key(
        &mut self,
        prototype: IntrinsicPrototype,
        target: TypedExpr,
        field: &PropertyAccessField,
    ) -> TypedExpr {
        match field {
            PropertyAccessField::Const(field) => {
                let name = self.interner.resolve_expect(field.sym()).to_string();
                if prototype == IntrinsicPrototype::Symbol && name == "description" {
                    if let Some(read) = self.intrinsic_symbol_description_read(&target) {
                        return read;
                    }
                }
                if let Some(read) = self.intrinsic_method_read(prototype, &target, &name) {
                    return read;
                }
            }
            PropertyAccessField::Expr(expression) => {
                if let Some((symbol, key)) = self.lower_well_known_symbol_property_key(expression) {
                    if let Some(read) =
                        self.intrinsic_symbol_method_read(prototype, &target, symbol, key)
                    {
                        return read;
                    }
                }
            }
        }
        self.lower_object_property_key(target, field)
    }

    /// The actual private Get owns accessor effects before a following argument
    /// or optional choice can consume caller facts. The retained target carries
    /// the post-Get shape without evaluating its source expression again.
    pub(super) fn lower_private_get_value(
        &mut self,
        target: &mut TypedExpr,
        private_name_id: PrivateNameId,
    ) -> TypedExpr {
        let property = self.read_object_shape_property(target, &private_data_key(private_name_id));
        let may_run_user_code = match &property {
            Some(ObjectShapeProperty::Data(_))
            | Some(ObjectShapeProperty::Accessor { getter: None, .. }) => false,
            Some(ObjectShapeProperty::Accessor {
                getter: Some(_), ..
            })
            | None => true,
        };
        if may_run_user_code {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
            target.heap_shape = None;
        }
        let mut info = match property {
            Some(ObjectShapeProperty::Data(info)) => info,
            Some(ObjectShapeProperty::Accessor {
                getter: Some(getter),
                ..
            }) => self.accessor_return_info(&getter.function_id),
            Some(ObjectShapeProperty::Accessor { getter: None, .. }) => ValueInfo::undefined(),
            None => unknown_runtime_value_info(),
        };
        if may_run_user_code {
            info.heap_shape = None;
        }
        TypedExpr::from_info(
            info,
            ExprIr::PrivateRead {
                target: Box::new(target.clone()),
                private_name_id,
            },
        )
    }

    pub(super) fn lower_private_property_access(
        &mut self,
        access: &PrivatePropertyAccess,
    ) -> TypedExpr {
        let Some(private_name_id) = self.current_private_name_id(access.field()) else {
            return self.unsupported_expr("private class element");
        };
        let mut target = self.lower_property_target(access.target());
        self.lower_private_get_value(&mut target, private_name_id)
    }

    pub(super) fn lower_property_access(&mut self, access: &PropertyAccess) -> TypedExpr {
        match access {
            PropertyAccess::Simple(access) => {
                if let (Expression::Identifier(identifier), PropertyAccessField::Const(field)) =
                    (access.target(), access.field())
                {
                    let target_name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let member_name = self.interner.resolve_expect(field.sym()).to_string();
                    // Retain the actual closed Symbol identity. Its description
                    // is an ordinary string and cannot represent this value.
                    if self.expression_is_builtin_symbol_intrinsic(&target_name) {
                        if let Some(symbol) =
                            WellKnownSymbol::from_member_name(SymbolMemberName::new(&member_name))
                        {
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Symbol),
                                ExprIr::WellKnownSymbol(symbol),
                            );
                        }
                    }
                }
                let target = self.lower_property_target(access.target());
                let result = match target.kind {
                    ValueKind::Object | ValueKind::Function => {
                        self.lower_object_property_key(target, access.field())
                    }
                    ValueKind::Boolean => self.lower_primitive_property_key(
                        IntrinsicPrototype::Boolean,
                        target,
                        access.field(),
                    ),
                    ValueKind::BigInt => self.lower_primitive_property_key(
                        IntrinsicPrototype::BigInt,
                        target,
                        access.field(),
                    ),
                    ValueKind::Symbol => self.lower_primitive_property_key(
                        IntrinsicPrototype::Symbol,
                        target,
                        access.field(),
                    ),
                    ValueKind::String => self.lower_string_index_key(target, access.field()),
                    ValueKind::Array
                        if self.array_prototype_mutated
                            || Self::array_shape_has_custom_prototype(&target) =>
                    {
                        if self.property_access_field_is_array_length(access.field()) {
                            TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Number),
                                ExprIr::PropertyRead {
                                    target: Box::new(target),
                                    key: PropertyKeyIr::ArrayLength,
                                },
                            )
                        } else {
                            self.lower_object_property_key(target, access.field())
                        }
                    }
                    ValueKind::Array => self.lower_array_index_key(target, access.field()),
                    ValueKind::Arguments => self.lower_arguments_index_key(target, access.field()),
                    ValueKind::Undefined if matches!(target.expr, ExprIr::Arguments) => self
                        .lower_arguments_index_key(
                            TypedExpr::from_info(
                                ValueInfo {
                                    kind: ValueKind::Arguments,
                                    possible_kinds: KindSet::from_kind(ValueKind::Arguments),
                                    heap_shape: None,
                                    function_targets: FunctionTargetKnowledge::none(),
                                },
                                target.expr,
                            ),
                            access.field(),
                        ),
                    ValueKind::Undefined | ValueKind::Null => self.lower_object_property_key(
                        TypedExpr::from_info(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::from_kind(target.kind),
                                heap_shape: None,
                                function_targets: FunctionTargetKnowledge::none(),
                            },
                            target.expr,
                        ),
                        access.field(),
                    ),
                    ValueKind::Dynamic
                        if target.possible_kinds.contains(ValueKind::Array)
                            && self.property_access_field_is_proven_numeric(access.field()) =>
                    {
                        self.lower_array_index_key(target, access.field())
                    }
                    ValueKind::Dynamic => self.lower_object_property_key(target, access.field()),
                    ValueKind::Number => self.lower_primitive_property_key(
                        IntrinsicPrototype::Number,
                        target,
                        access.field(),
                    ),
                };
                if matches!(
                    &result.expr,
                    ExprIr::PropertyRead { key, .. }
                        if Self::property_key_may_call_user_code(key)
                ) {
                    self.observe_all_planned_source_as_unknown_property_hooks();
                    self.invalidate_unknown_user_code_effects();
                }
                result
            }
            PropertyAccess::Private(access) => self.lower_private_property_access(access),
            PropertyAccess::Super(access) => self.lower_super_property_access(access),
        }
    }
}
