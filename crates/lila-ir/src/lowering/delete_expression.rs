use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_delete(&mut self, target: &Expression) -> TypedExpr {
        match target {
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                if self.is_constructor_prototype_expr(access.target(), ARRAY_NAME) {
                    self.array_prototype_mutated = true;
                }
                if self.is_number_prototype_property_expr(target, "toString") {
                    self.number_prototype_to_string_state = PrototypeToStringState::ObjectPrototype;
                }
                if self.is_number_prototype_property_expr(target, "match") {
                    self.number_prototype_match_is_string_match = false;
                }
                if self.is_number_prototype_property_expr(target, "split") {
                    self.number_prototype_split_is_string_split = false;
                }
                if self.is_boolean_prototype_property_expr(target, "toString") {
                    self.boolean_prototype_to_string_state =
                        PrototypeToStringState::ObjectPrototype;
                }
                let mut target = self.lower_property_target(access.target());
                if target.kind == ValueKind::Undefined
                    && self.is_current_param_expr(access.target())
                {
                    target.kind = ValueKind::Dynamic;
                    target.possible_kinds = KindSet::all_runtime_tags();
                    target.heap_shape = None;
                    target.function_targets.clear();
                }
                let key = match target.kind {
                    ValueKind::Object | ValueKind::Function | ValueKind::Dynamic => {
                        match access.field() {
                            PropertyAccessField::Const(name) => PropertyKeyIr::StaticString(
                                self.interner.resolve_expect(name.sym()).to_string(),
                            ),
                            PropertyAccessField::Expr(expr) => {
                                if let Some(key) = self.lower_static_property_key(expr) {
                                    key
                                } else {
                                    let mut lowered = self.lower_expression(expr);
                                    if lowered.kind == ValueKind::Undefined
                                        && self.is_current_param_expr(expr)
                                    {
                                        lowered.kind = ValueKind::Dynamic;
                                        lowered.possible_kinds = KindSet::all_runtime_tags();
                                        lowered.heap_shape = None;
                                        lowered.function_targets.clear();
                                    }
                                    if lowered.kind == ValueKind::Number {
                                        PropertyKeyIr::ArrayIndex(Box::new(lowered))
                                    } else if lowered.kind == ValueKind::String
                                        || lowered.kind == ValueKind::Symbol
                                        || lowered.possible_kinds.contains(ValueKind::String)
                                        || lowered.possible_kinds.contains(ValueKind::Symbol)
                                        || lowered
                                            .possible_kinds
                                            .is_subset_of(KindSet::PROPERTY_KEY_COERCIBLE)
                                        || lowered.possible_kinds == KindSet::all_runtime_tags()
                                    {
                                        PropertyKeyIr::StringExpr(Box::new(lowered))
                                    } else {
                                        return self.unsupported_expr("unsupported unary operator");
                                    }
                                }
                            }
                        }
                    }
                    ValueKind::Array | ValueKind::Arguments => match access.field() {
                        PropertyAccessField::Const(name) => PropertyKeyIr::StaticString(
                            self.interner.resolve_expect(name.sym()).to_string(),
                        ),
                        PropertyAccessField::Expr(expr) => {
                            if let Some(key) = self.lower_static_property_key(expr) {
                                key
                            } else {
                                let mut lowered = self.lower_expression(expr);
                                if lowered.kind == ValueKind::Undefined
                                    && self.is_current_param_expr(expr)
                                {
                                    lowered.kind = ValueKind::Dynamic;
                                    lowered.possible_kinds = KindSet::all_runtime_tags();
                                    lowered.heap_shape = None;
                                    lowered.function_targets.clear();
                                }
                                if lowered.kind == ValueKind::Number {
                                    PropertyKeyIr::ArrayIndex(Box::new(lowered))
                                } else if lowered.kind == ValueKind::String
                                    || lowered.kind == ValueKind::Symbol
                                    || lowered.possible_kinds.contains(ValueKind::String)
                                    || lowered.possible_kinds.contains(ValueKind::Symbol)
                                    || lowered
                                        .possible_kinds
                                        .is_subset_of(KindSet::PROPERTY_KEY_COERCIBLE)
                                    || lowered.possible_kinds == KindSet::all_runtime_tags()
                                {
                                    PropertyKeyIr::StringExpr(Box::new(lowered))
                                } else {
                                    return self.unsupported_expr("unsupported unary operator");
                                }
                            }
                        }
                    },
                    _ => return self.unsupported_expr("unsupported unary operator"),
                };
                if self.is_global_this_expr(access.target()) {
                    if let PropertyKeyIr::StaticString(name) = &key {
                        // The direct-global IR has a dedicated early return,
                        // but aliases still carry the same object shape. A
                        // successful delete can expose an inherited accessor;
                        // a failed delete cannot be distinguished yet.
                        self.invalidate_ordinary_property_shape_aliases(&target.value_info());
                        let info = self.lookup_global_property_info(name).cloned();
                        if info.as_ref().is_none_or(|info| info.configurable) {
                            self.mark_global_property_deleted(name);
                        }
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            ExprIr::DeleteGlobalProperty {
                                name: name.clone(),
                                strictness: self.reference_strictness(),
                            },
                        );
                    }
                }
                self.update_well_known_symbol_prototype_property(access.target(), &key, None);
                if target.heap_shape.is_none() || Self::property_key_may_call_user_code(&key) {
                    // An unknown object can be a Proxy, and an object-valued
                    // key can run arbitrary source code during ToPropertyKey.
                    self.invalidate_unknown_user_code_effects();
                } else {
                    // Delete may succeed and expose an inherited accessor, or
                    // fail and preserve the own property. Until descriptor
                    // attributes are tracked, neither outcome may retain an
                    // exact target or enclosing-alias shape.
                    self.invalidate_ordinary_property_shape_aliases(&target.value_info());
                }
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Boolean),
                    ExprIr::DeleteProperty {
                        target: Box::new(target),
                        key,
                        strictness: self.reference_strictness(),
                    },
                )
            }
            Expression::PropertyAccess(PropertyAccess::Private(_)) => {
                self.unsupported_expr("unsupported unary operator")
            }
            Expression::PropertyAccess(PropertyAccess::Super(access)) => {
                if self.class_context.is_none() {
                    return self.unsupported_expr("object literal method");
                }
                let Some(key) = self.lower_super_property_key(access.field()) else {
                    return TypedExpr::undefined();
                };
                DeleteSuperReferencePlan::new(self.current_this_info(), key).into_reference_error()
            }
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                if name == GLOBAL_THIS_NAME
                    || name == "undefined"
                    || self.lookup_binding(&name).is_some()
                    || self.visible_function_names.contains_key(&name)
                    || (name == "arguments"
                        && self.lookup_binding(LEXICAL_ARGUMENTS_NAME).is_some())
                {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::DeleteIdentifier {
                            name,
                            kind: DeleteIdentifierKindIr::NonDeletable,
                        },
                    );
                }
                if let Some(info) = self.lookup_global_property_info(&name).cloned() {
                    if info.proven_present && info.configurable {
                        self.mark_global_property_deleted(&name);
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            // `delete <identifier>` is an early SyntaxError in
                            // strict code, and this arm only fires for a
                            // configurable property, so [[Delete]] cannot fail.
                            ExprIr::DeleteGlobalProperty {
                                name,
                                strictness: Strictness::Sloppy,
                            },
                        );
                    }
                    if info.proven_present {
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            ExprIr::DeleteIdentifier {
                                name,
                                kind: DeleteIdentifierKindIr::NonDeletable,
                            },
                        );
                    }
                }
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Boolean),
                    ExprIr::DeleteIdentifier {
                        name,
                        kind: DeleteIdentifierKindIr::Missing,
                    },
                )
            }
            _ => TypedExpr::from_info(
                ValueInfo::new(ValueKind::Boolean),
                ExprIr::DeleteValue {
                    expr: Box::new(self.lower_expression(target)),
                },
            ),
        }
    }
}
