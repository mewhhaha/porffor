use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_property_access(&mut self, access: &PropertyAccess) -> TypedExpr {
        match access {
            PropertyAccess::Simple(access) => {
                if let (Expression::Identifier(identifier), PropertyAccessField::Const(field)) =
                    (access.target(), access.field())
                {
                    let target_name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let member_name = self.interner.resolve_expect(field.sym()).to_string();
                    // The runtime encoding of a well-known symbol *value*: its
                    // 6.1.5.1 Table 1 [[Description]], carried as a string whose
                    // `ValueKind::Symbol` is what distinguishes it from an
                    // ordinary string of the same text.
                    if self.expression_is_builtin_symbol_intrinsic(&target_name) {
                        if let Some(symbol) =
                            WellKnownSymbol::from_member_name(SymbolMemberName::new(&member_name))
                        {
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Symbol),
                                ExprIr::String(symbol.description().to_string()),
                            );
                        }
                    }
                }
                let target = self.lower_property_target(access.target());
                let result = match target.kind {
                    ValueKind::Object | ValueKind::Function => {
                        self.lower_object_property_key(target, access.field())
                    }
                    ValueKind::Boolean => {
                        if let PropertyAccessField::Const(field) = access.field() {
                            let field_name = self.interner.resolve_expect(field.sym()).to_string();
                            let builtin = match field_name.as_str() {
                                "toString" => Some(StandardBuiltinId::BooleanPrototypeToString),
                                "valueOf" => Some(StandardBuiltinId::BooleanPrototypeValueOf),
                                _ => None,
                            };
                            if let Some(builtin) = builtin {
                                TypedExpr::from_info(
                                    Self::standard_builtin_value_info(builtin),
                                    ExprIr::PropertyRead {
                                        target: Box::new(target),
                                        key: PropertyKeyIr::StaticString(field_name),
                                    },
                                )
                            } else {
                                self.unsupported_expr("property access on boolean target")
                            }
                        } else {
                            self.unsupported_expr("dynamic property access on boolean target")
                        }
                    }
                    ValueKind::BigInt => {
                        if let PropertyAccessField::Const(field) = access.field() {
                            let field_name = self.interner.resolve_expect(field.sym()).to_string();
                            let builtin = match field_name.as_str() {
                                "toString" => Some(StandardBuiltinId::BigIntPrototypeToString),
                                "toLocaleString" => {
                                    Some(StandardBuiltinId::BigIntPrototypeToLocaleString)
                                }
                                "valueOf" => Some(StandardBuiltinId::BigIntPrototypeValueOf),
                                _ => None,
                            };
                            if let Some(builtin) = builtin {
                                TypedExpr::from_info(
                                    Self::standard_builtin_value_info(builtin),
                                    ExprIr::PropertyRead {
                                        target: Box::new(target),
                                        key: PropertyKeyIr::StaticString(field_name),
                                    },
                                )
                            } else {
                                self.unsupported_expr("property access on bigint target")
                            }
                        } else {
                            self.unsupported_expr("dynamic property access on bigint target")
                        }
                    }
                    ValueKind::Symbol => {
                        if let PropertyAccessField::Const(field) = access.field() {
                            let field_name = self.interner.resolve_expect(field.sym()).to_string();
                            if field_name == "description" {
                                TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::PropertyRead {
                                        target: Box::new(target),
                                        key: PropertyKeyIr::StaticString(field_name),
                                    },
                                )
                            } else if field_name == "constructor" {
                                TypedExpr::from_info(
                                    Self::function_value_info_with_constructable(
                                        StandardBuiltinId::SymbolConstructor.function_id(),
                                        false,
                                    ),
                                    ExprIr::PropertyRead {
                                        target: Box::new(target),
                                        key: PropertyKeyIr::StaticString(field_name),
                                    },
                                )
                            } else {
                                let builtin = match field_name.as_str() {
                                    "toString" => Some(StandardBuiltinId::SymbolPrototypeToString),
                                    "valueOf" => Some(StandardBuiltinId::SymbolPrototypeValueOf),
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(target),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    // Anything else is inherited from
                                    // `Object.prototype` via
                                    // `Symbol.prototype`'s own
                                    // `[[Prototype]]`; resolve it through the
                                    // generic runtime prototype-chain lookup.
                                    self.lower_object_property_key(target, access.field())
                                }
                            }
                        } else if let PropertyAccessField::Expr(expr) = access.field() {
                            if let Some((symbol, symbol_key)) =
                                self.lower_well_known_symbol_property_key(expr)
                            {
                                if symbol == WellKnownSymbol::ToPrimitive {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(
                                            StandardBuiltinId::SymbolPrototypeToPrimitive,
                                        ),
                                        ExprIr::PropertyRead {
                                            target: Box::new(target),
                                            key: symbol_key,
                                        },
                                    )
                                } else {
                                    // Auto-boxing: any other computed key is
                                    // resolved against `Symbol.prototype` by
                                    // the generic runtime lookup (typically
                                    // yielding `undefined`).
                                    self.lower_object_property_key(target, access.field())
                                }
                            } else {
                                self.lower_object_property_key(target, access.field())
                            }
                        } else {
                            self.unsupported_expr("dynamic property access on symbol target")
                        }
                    }
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
                    ValueKind::Number => {
                        self.unsupported_expr("property access on non-object target")
                    }
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
