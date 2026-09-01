use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_assign(
        &mut self,
        op: AssignOp,
        lhs: &AssignTarget,
        rhs: &Expression,
    ) -> TypedExpr {
        if let AssignTarget::WebCompatCall(call) = lhs {
            return self.lower_web_compat_call_assignment_target(call);
        }
        match op {
            AssignOp::Assign => match lhs {
                AssignTarget::Identifier(identifier) => {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    // ResolveBinding is the LHS evaluation. Locate its
                    // declarative/global fallback before lowering the RHS, and
                    // carry that same value through Object-ER selection and
                    // the eventual write.
                    let with_fallback = (!self.with_environment_chain.is_empty())
                        .then(|| self.locate_identifier_reference(&name));
                    let static_to_string_regexp_object =
                        self.static_to_string_returns_regexp_object_expr(rhs);
                    let static_iterator_values = self.static_object_iterator_literal_values(rhs);
                    let value = self.lower_expression(rhs);
                    if let Some(fallback) = with_fallback {
                        let objects = self
                            .with_environment_chain
                            .select_preceding(fallback.declarative_position());
                        if let Some(objects) = objects {
                            return self.lower_with_scoped_identifier_write(
                                name, value, objects, fallback,
                            );
                        }
                        return self.lower_located_identifier_assign_value(name, value, fallback);
                    }
                    let result = self.lower_identifier_assign_value(name.clone(), value);
                    if let Some(values) = static_iterator_values {
                        self.static_iterator_binding_values
                            .insert(name.clone(), values);
                    } else {
                        self.static_iterator_binding_values.remove(&name);
                    }
                    if static_to_string_regexp_object {
                        self.static_to_string_regexp_object_bindings.insert(name);
                    } else {
                        self.static_to_string_regexp_object_bindings.remove(&name);
                    }
                    result
                }
                AssignTarget::Access(access) => match access {
                    PropertyAccess::Simple(access) => {
                        self.lower_ordinary_property_plain_assignment(access, rhs)
                    }
                    PropertyAccess::Private(_) | PropertyAccess::Super(_) => {
                        self.lower_property_assign(access, rhs)
                    }
                },
                AssignTarget::Pattern(pattern) => self.lower_pattern_assign(pattern, rhs),
                // Spelled out rather than `_`. `AssignTarget` is a closed
                // 4-variant boa enum (`boa_ast-0.21.1/src/expression/operator/
                // assign/mod.rs:126`) and this is invariant I7's AST half: a
                // fifth production that yields a Reference must be decided
                // here, as `error[E0004]`, not swallowed as an unsupported
                // expression with nothing to compile-error about.
                //
                // `WebCompatCall` is handled by the early return at the top of
                // this function — Annex B `f() = v` is a runtime
                // ReferenceError, not a compiler gap — so it is unreachable
                // here; it is still named, because `unreachable!()` in its
                // place would reintroduce a catch-all by another spelling.
                AssignTarget::WebCompatCall(call) => {
                    self.lower_web_compat_call_assignment_target(call)
                }
            },
            AssignOp::Add
            | AssignOp::Sub
            | AssignOp::Mul
            | AssignOp::Div
            | AssignOp::Mod
            | AssignOp::Exp => {
                let arithmetic = match op {
                    AssignOp::Add => ArithmeticOp::Add,
                    AssignOp::Sub => ArithmeticOp::Sub,
                    AssignOp::Mul => ArithmeticOp::Mul,
                    AssignOp::Div => ArithmeticOp::Div,
                    AssignOp::Mod => ArithmeticOp::Mod,
                    AssignOp::Exp => ArithmeticOp::Exp,
                    AssignOp::Assign
                    | AssignOp::BoolAnd
                    | AssignOp::BoolOr
                    | AssignOp::Coalesce
                    | AssignOp::And
                    | AssignOp::Or
                    | AssignOp::Xor
                    | AssignOp::Shl
                    | AssignOp::Shr
                    | AssignOp::Ushr => {
                        unreachable!("this match arm covers only the arithmetic operators")
                    }
                };
                if let AssignTarget::Access(access) = lhs {
                    return match access {
                        PropertyAccess::Simple(access) => self
                            .lower_ordinary_property_eager_compound_assignment(
                                access,
                                EagerCompoundAssignmentOp::Arithmetic(arithmetic),
                                rhs,
                            ),
                        PropertyAccess::Super(access) => self
                            .lower_super_property_eager_compound_assignment(
                                access,
                                EagerCompoundAssignmentOp::Arithmetic(arithmetic),
                                rhs,
                            ),
                        PropertyAccess::Private(_) => self.lower_property_reference_update(
                            access,
                            PropertyUpdateOp::Arithmetic(arithmetic),
                            rhs,
                        ),
                    };
                }
                let AssignTarget::Identifier(identifier) = lhs else {
                    return self.unsupported_expr("unsupported property assignment operator");
                };

                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let reference = self.locate_identifier_reference(&name);
                let selected = self
                    .with_environment_chain
                    .select_preceding(reference.declarative_position());
                if let Some(objects) = selected {
                    let value = self.lower_expression(rhs);
                    return self.lower_with_scoped_identifier_eager_compound_assignment(
                        name,
                        EagerCompoundAssignmentOp::Arithmetic(arithmetic),
                        value,
                        objects,
                        reference,
                    );
                }
                if matches!(&reference, LocatedIdentifierReference::Unresolvable)
                    && !self.global_property_is_proven_present(&name)
                {
                    let value = self.lower_expression(rhs);
                    return self.lower_global_object_environment_eager_compound_assignment(
                        name,
                        EagerCompoundAssignmentOp::Arithmetic(arithmetic),
                        value,
                    );
                }
                let value = self.lower_expression(rhs);
                // 13.15.4 ApplyStringOrNumericAssignment does GetValue then
                // PutValue, so both 9.1.1.1.6 step 2 and 9.1.1.1.5 step 3 apply
                // and neither was checked here before. Ledger **L4**: the RHS is
                // lowered above, so the throw follows its side effects where
                // 13.15.4 steps 1-2 put it before them; the resolution is placed
                // at the existing `lookup_binding` line rather than hoisted over
                // ~300 lines of arm without a runtime oracle.
                match self.resolve_binding_reference(&name) {
                    BindingResolution::Uninitialized(violation) => {
                        let error = violation.into_throw();
                        return TypedExpr::from_info(
                            error.value_info(),
                            ExprIr::Comma {
                                lhs: Box::new(value),
                                rhs: Box::new(error),
                            },
                        );
                    }
                    BindingResolution::Initialized(_) | BindingResolution::Unresolvable => {}
                }
                let binding = self.lookup_binding(&name);
                // 13.15.2 `AssignmentExpression : LeftHandSideExpression op=
                // AssignmentExpression` evaluates the LHS Reference, GetValues
                // it, evaluates the RHS (done above), applies
                // ApplyStringOrNumericBinaryOperator, and *then* PutValues. A
                // `const` target fails only at that last step, so everything
                // before it still runs and can still throw first — `const s =
                // 'a'; s += { toString() { throw new RangeError(); } }` is a
                // RangeError, not a TypeError.
                //
                // This replaces three separate `unsupported_expr("assignment to
                // const binding")` sites further down (the coercive-add, the
                // general-form and the specialised number/string paths). They
                // sat *after* the specialisation analysis, so which of the three
                // fired depended on inference — three ways to spell one refusal
                // of a program the spec says must run. Deciding it once here, on
                // the binding's mode alone, is both the fix and the reason the
                // three sites can go: the analysis below is now only ever
                // reached for a mutable target.
                let const_target = binding.as_ref().and_then(|binding| {
                    (binding.mode == BindingMode::Const).then(|| {
                        (
                            binding.storage_name.clone(),
                            ValueInfo {
                                kind: binding.kind,
                                possible_kinds: binding.possible_kinds,
                                heap_shape: binding.heap_shape.clone(),
                                function_targets: binding.function_targets.clone(),
                            },
                        )
                    })
                });
                if let Some((storage_name, lhs_info)) = const_target {
                    let arithmetic = match op {
                        AssignOp::Add => ArithmeticOp::Add,
                        AssignOp::Sub => ArithmeticOp::Sub,
                        AssignOp::Mul => ArithmeticOp::Mul,
                        AssignOp::Div => ArithmeticOp::Div,
                        AssignOp::Mod => ArithmeticOp::Mod,
                        AssignOp::Exp => ArithmeticOp::Exp,
                        _ => unreachable!("this match arm covers only the arithmetic operators"),
                    };
                    let lhs_read =
                        TypedExpr::from_info(lhs_info, ExprIr::Identifier(storage_name.clone()));
                    // `combine_arithmetic` has refusals of its own: an operand
                    // that is neither statically coercible nor PRIMITIVE_ONLY
                    // (`const x = {}; x -= 1`) still reaches
                    // `unsupported_expr("string or coercive `+`")` or the
                    // non-primitive `Sub | Mul | Div | Mod` fallback. So this arm
                    // does not make *every* const compound assignment compile —
                    // it makes the ones whose arithmetic is representable
                    // compile, and moves the rest to a message that no longer
                    // names `const`. Do not read the const refusal's
                    // disappearance from the grep as those programs working.
                    let applied = self.combine_arithmetic(arithmetic, lhs_read, value);
                    return self.immutable_binding_write(&storage_name, applied);
                }
                let script_global_reference =
                    self.is_script_global_var_name(&name) && !self.has_scope_binding(&name);
                let binding_storage_name = binding.as_ref().and_then(|binding| {
                    (!script_global_reference).then(|| binding.storage_name.clone())
                });
                let global_info = self.lookup_global_property_info(&name).cloned();
                let rhs_may_string = value.possible_kinds.contains(ValueKind::String);
                let binding_known_string = binding
                    .as_ref()
                    .is_some_and(|binding| binding.kind == ValueKind::String);
                let global_known_string = global_info.as_ref().is_some_and(|info| {
                    info.proven_present && info.value_info.kind == ValueKind::String
                });
                let binding_allows_string_add = binding.as_ref().is_some_and(|binding| {
                    binding.kind == ValueKind::String
                        || binding.kind == ValueKind::Dynamic
                        || binding.kind == ValueKind::Undefined
                        || binding.possible_kinds.contains(ValueKind::String)
                });
                let global_allows_string_add = global_info.as_ref().is_some_and(|info| {
                    info.proven_present
                        && (info.value_info.kind == ValueKind::String
                            || info.value_info.kind == ValueKind::Dynamic
                            || info.value_info.kind == ValueKind::Undefined
                            || info.value_info.possible_kinds.contains(ValueKind::String))
                });
                let string_add = matches!(op, AssignOp::Add)
                    && ((binding_known_string || global_known_string)
                        || (value.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                            && (rhs_may_string
                                || binding_allows_string_add
                                || global_allows_string_add)));
                let lhs_info = binding
                    .as_ref()
                    .map(|binding| ValueInfo {
                        kind: binding.kind,
                        possible_kinds: binding.possible_kinds,
                        heap_shape: binding.heap_shape.clone(),
                        function_targets: binding.function_targets.clone(),
                    })
                    .or_else(|| {
                        global_info
                            .as_ref()
                            .filter(|info| info.proven_present)
                            .map(|info| info.value_info.clone())
                    });
                let coercive_add = matches!(op, AssignOp::Add)
                    && !string_add
                    && lhs_info.as_ref().is_some_and(|lhs| {
                        lhs.kind != ValueKind::Number || value.kind != ValueKind::Number
                    });
                if coercive_add {
                    // A `const` target returned above; only mutable bindings
                    // and global properties reach here.
                    let Some(lhs_info) = lhs_info else {
                        self.unsupported_with_message(format!(
                            "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                        ));
                        return TypedExpr::undefined();
                    };
                    let lhs = TypedExpr::from_info(
                        lhs_info,
                        if let Some(storage_name) = binding_storage_name.clone() {
                            ExprIr::Identifier(storage_name)
                        } else {
                            ExprIr::GlobalPropertyRead { name: name.clone() }
                        },
                    );
                    let possible_kinds = KindSet::from_kind(ValueKind::String)
                        .union(KindSet::from_kind(ValueKind::Number))
                        .union(KindSet::from_kind(ValueKind::BigInt));
                    let result_info = ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    };
                    let result = TypedExpr::from_info(
                        result_info.clone(),
                        ExprIr::CoerciveAdd {
                            lhs: Box::new(lhs),
                            rhs: Box::new(value),
                        },
                    );
                    if let Some(storage_name) = binding_storage_name {
                        self.set_binding_value_info(&name, result_info.clone());
                        return TypedExpr::from_info(
                            result_info,
                            ExprIr::AssignIdentifier {
                                name: storage_name,
                                value: Box::new(result),
                            },
                        );
                    }
                    if script_global_reference {
                        self.set_binding_value_info(&name, result_info.clone());
                    }
                    self.set_global_property_value_info(name.clone(), result_info.clone());
                    let strictness = self.reference_strictness();
                    return TypedExpr::from_info(
                        result_info,
                        ExprIr::GlobalPropertyWrite {
                            name,
                            value: Box::new(result),
                            implicit: false,
                            strictness,
                        },
                    );
                }
                // Anything the specialised number/string forms below cannot
                // represent still has a meaning: read the binding, apply
                // ApplyStringOrNumericBinaryOperator, assign the result back.
                let needs_general_form = (!string_add && value.kind != ValueKind::Number)
                    || match &binding {
                        // No `binding.mode != BindingMode::Const` conjunct: a
                        // `const` target returned at `const_target` above, so it
                        // was trivially true here and read as though the const
                        // case were still live.
                        Some(binding) => {
                            if string_add {
                                !binding_known_string
                                    && !rhs_may_string
                                    && !binding_allows_string_add
                            } else {
                                binding.kind != ValueKind::Number
                            }
                        }
                        None => {
                            self.global_property_is_proven_present(&name)
                                && !global_info.as_ref().is_some_and(|info| {
                                    info.proven_present
                                        && ((string_add
                                            && (global_known_string
                                                || rhs_may_string
                                                || global_allows_string_add))
                                            || (!string_add
                                                && info.value_info.kind == ValueKind::Number))
                                })
                        }
                    };
                if needs_general_form {
                    // A `const` target returned above.
                    if let Some(lhs_info) = lhs_info {
                        let arithmetic = match op {
                            AssignOp::Add => ArithmeticOp::Add,
                            AssignOp::Sub => ArithmeticOp::Sub,
                            AssignOp::Mul => ArithmeticOp::Mul,
                            AssignOp::Div => ArithmeticOp::Div,
                            AssignOp::Mod => ArithmeticOp::Mod,
                            AssignOp::Exp => ArithmeticOp::Exp,
                            _ => unreachable!(),
                        };
                        return self.lower_identifier_arithmetic_general(
                            &name,
                            binding_storage_name,
                            lhs_info,
                            arithmetic,
                            value,
                        );
                    }
                    return self.unsupported_expr("coercive compound assignment");
                }
                let result_info = if string_add {
                    ValueInfo::new(ValueKind::String)
                } else {
                    ValueInfo {
                        kind: ValueKind::Number,
                        possible_kinds: KindSet::from_kind(ValueKind::Number),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    }
                };
                if let Some(binding) = binding {
                    // A `const` target returned above.
                    if string_add {
                        if !binding_known_string && !rhs_may_string && !binding_allows_string_add {
                            return self
                                .unsupported_expr("compound assignment on non-string binding");
                        }
                    } else if binding.kind != ValueKind::Number {
                        return self.unsupported_expr("compound assignment on non-number binding");
                    }
                    self.set_binding_value_info(&name, result_info.clone());
                    if script_global_reference {
                        self.set_global_property_value_info(name.clone(), result_info.clone());
                    }
                } else if global_info.as_ref().is_some_and(|info| {
                    info.proven_present
                        && ((string_add
                            && (global_known_string || rhs_may_string || global_allows_string_add))
                            || (!string_add && info.value_info.kind == ValueKind::Number))
                }) {
                    self.set_global_property_value_info(name.clone(), result_info.clone());
                } else if self.global_property_is_proven_present(&name) {
                    if string_add {
                        return self.unsupported_expr("compound assignment on non-string binding");
                    }
                    return self.unsupported_expr("compound assignment on non-number binding");
                } else {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ));
                    return TypedExpr::undefined();
                }
                let op = match op {
                    AssignOp::Add => ArithmeticBinaryOp::Add,
                    AssignOp::Sub => ArithmeticBinaryOp::Sub,
                    AssignOp::Mul => ArithmeticBinaryOp::Mul,
                    AssignOp::Div => ArithmeticBinaryOp::Div,
                    AssignOp::Mod => ArithmeticBinaryOp::Mod,
                    AssignOp::Exp => ArithmeticBinaryOp::Exp,
                    _ => unreachable!(),
                };
                let strictness = self.reference_strictness();
                let expr = if let Some(storage_name) = binding_storage_name {
                    ExprIr::CompoundAssignIdentifier {
                        name: storage_name,
                        op,
                        value: Box::new(value),
                    }
                } else {
                    ExprIr::GlobalPropertyCompoundAssign {
                        name,
                        op,
                        value: Box::new(value),
                        strictness,
                    }
                };
                TypedExpr::from_info(result_info, expr)
            }
            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                let logical_op = match op {
                    AssignOp::BoolAnd => LogicalBinaryOp::And,
                    AssignOp::BoolOr => LogicalBinaryOp::Or,
                    AssignOp::Coalesce => LogicalBinaryOp::Coalesce,
                    AssignOp::Assign
                    | AssignOp::Add
                    | AssignOp::Sub
                    | AssignOp::Mul
                    | AssignOp::Div
                    | AssignOp::Mod
                    | AssignOp::Exp
                    | AssignOp::And
                    | AssignOp::Or
                    | AssignOp::Xor
                    | AssignOp::Shl
                    | AssignOp::Shr
                    | AssignOp::Ushr => {
                        unreachable!("this match arm covers only the logical operators")
                    }
                };
                let AssignTarget::Identifier(identifier) = lhs else {
                    if let AssignTarget::Access(access) = lhs {
                        return match access {
                            PropertyAccess::Simple(access) => self
                                .lower_ordinary_property_logical_assignment(
                                    access, logical_op, rhs,
                                ),
                            PropertyAccess::Super(_) | PropertyAccess::Private(_) => self
                                .lower_property_reference_update(
                                    access,
                                    PropertyUpdateOp::Logical(logical_op),
                                    rhs,
                                ),
                        };
                    }
                    return self.unsupported_expr("logical assignment");
                };
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let reference = self.locate_identifier_logical_assignment(&name);
                let selected = self
                    .with_environment_chain
                    .select_preceding(reference.declarative_position());
                let reference = match selected.is_none() {
                    true => match reference.reject_definite_tdz() {
                        Ok(reference) => reference,
                        Err(error) => return error,
                    },
                    false => reference,
                };
                let rhs_value = self.lower_conditionally_reached_expression(rhs);
                if let Some(objects) = selected {
                    let plan = self.with_environment_reference_plan(name.clone(), objects);
                    let fallback = self.lower_located_identifier_logical_assignment(
                        name,
                        logical_op,
                        rhs_value.clone(),
                        reference,
                        LogicalAssignmentReachability::WithEnvironmentFallback,
                    );
                    return plan.logical_assignment(logical_op, rhs_value, fallback);
                }
                if reference.is_unproven_global() {
                    return self.lower_global_object_environment_logical_assignment(
                        name, logical_op, rhs_value,
                    );
                }
                self.lower_located_identifier_logical_assignment(
                    name,
                    logical_op,
                    rhs_value,
                    reference,
                    LogicalAssignmentReachability::Definite,
                )
            }
            AssignOp::And
            | AssignOp::Or
            | AssignOp::Xor
            | AssignOp::Shl
            | AssignOp::Shr
            | AssignOp::Ushr => {
                let AssignTarget::Identifier(identifier) = lhs else {
                    if let AssignTarget::Access(access) = lhs {
                        let bitwise = match op {
                            AssignOp::And => BitwiseOp::And,
                            AssignOp::Or => BitwiseOp::Or,
                            AssignOp::Xor => BitwiseOp::Xor,
                            AssignOp::Shl => BitwiseOp::Shl,
                            AssignOp::Shr => BitwiseOp::Shr,
                            AssignOp::Ushr => BitwiseOp::UShr,
                            AssignOp::Assign
                            | AssignOp::Add
                            | AssignOp::Sub
                            | AssignOp::Mul
                            | AssignOp::Div
                            | AssignOp::Mod
                            | AssignOp::Exp
                            | AssignOp::BoolAnd
                            | AssignOp::BoolOr
                            | AssignOp::Coalesce => {
                                unreachable!("this match arm covers only the bitwise operators")
                            }
                        };
                        return match access {
                            PropertyAccess::Simple(access) => self
                                .lower_ordinary_property_eager_compound_assignment(
                                    access,
                                    EagerCompoundAssignmentOp::Bitwise(bitwise),
                                    rhs,
                                ),
                            PropertyAccess::Super(access) => self
                                .lower_super_property_eager_compound_assignment(
                                    access,
                                    EagerCompoundAssignmentOp::Bitwise(bitwise),
                                    rhs,
                                ),
                            PropertyAccess::Private(_) => self.lower_property_reference_update(
                                access,
                                PropertyUpdateOp::Bitwise(bitwise),
                                rhs,
                            ),
                        };
                    }
                    return self.unsupported_expr("unsupported property assignment operator");
                };
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let bitwise = match op {
                    AssignOp::And => BitwiseOp::And,
                    AssignOp::Or => BitwiseOp::Or,
                    AssignOp::Xor => BitwiseOp::Xor,
                    AssignOp::Shl => BitwiseOp::Shl,
                    AssignOp::Shr => BitwiseOp::Shr,
                    AssignOp::Ushr => BitwiseOp::UShr,
                    AssignOp::Assign
                    | AssignOp::Add
                    | AssignOp::Sub
                    | AssignOp::Mul
                    | AssignOp::Div
                    | AssignOp::Mod
                    | AssignOp::Exp
                    | AssignOp::BoolAnd
                    | AssignOp::BoolOr
                    | AssignOp::Coalesce => {
                        unreachable!("this match arm covers only the bitwise operators")
                    }
                };
                let reference = self.locate_identifier_reference(&name);
                let selected = self
                    .with_environment_chain
                    .select_preceding(reference.declarative_position());
                if let Some(objects) = selected {
                    let value = self.lower_expression(rhs);
                    return self.lower_with_scoped_identifier_eager_compound_assignment(
                        name,
                        EagerCompoundAssignmentOp::Bitwise(bitwise),
                        value,
                        objects,
                        reference,
                    );
                }
                if matches!(&reference, LocatedIdentifierReference::Unresolvable)
                    && !self.global_property_is_proven_present(&name)
                {
                    let value = self.lower_expression(rhs);
                    return self.lower_global_object_environment_eager_compound_assignment(
                        name,
                        EagerCompoundAssignmentOp::Bitwise(bitwise),
                        value,
                    );
                }
                // 13.15.3 / 13.15.4: GetValue then PutValue, so 9.1.1.1.6 step 2
                // and 9.1.1.1.5 step 3 both apply — and step 3 precedes the
                // immutability test below, which is the only test this arm used
                // to make. The RHS is not lowered yet here, so the throw is in
                // the right place.
                match self.resolve_binding_reference(&name) {
                    BindingResolution::Uninitialized(violation) => return violation.into_throw(),
                    BindingResolution::Initialized(_) | BindingResolution::Unresolvable => {}
                }
                let binding = self.lookup_binding(&name);
                let binding_storage_name =
                    binding.as_ref().map(|binding| binding.storage_name.clone());
                let global_info = self.lookup_global_property_info(&name).cloned();
                // As for the arithmetic forms: 13.15.2 applies the operator
                // before PutValue, so a `const` target still coerces both
                // operands and only then throws.
                let const_storage_name = binding.as_ref().and_then(|binding| {
                    (binding.mode == BindingMode::Const).then(|| binding.storage_name.clone())
                });
                if binding.is_none()
                    && !global_info.as_ref().is_some_and(|info| info.proven_present)
                {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ));
                    return TypedExpr::undefined();
                }
                let op = match op {
                    AssignOp::And => BitwiseBinaryOp::And,
                    AssignOp::Or => BitwiseBinaryOp::Or,
                    AssignOp::Xor => BitwiseBinaryOp::Xor,
                    AssignOp::Shl => BitwiseBinaryOp::Shl,
                    AssignOp::Shr => BitwiseBinaryOp::Shr,
                    AssignOp::Ushr => BitwiseBinaryOp::UShr,
                    _ => unreachable!(),
                };
                let lhs_value = TypedExpr::from_info(
                    binding
                        .as_ref()
                        .map(|binding| ValueInfo {
                            kind: binding.kind,
                            possible_kinds: binding.possible_kinds,
                            heap_shape: binding.heap_shape.clone(),
                            function_targets: binding.function_targets.clone(),
                        })
                        .or_else(|| global_info.as_ref().map(|info| info.value_info.clone()))
                        .unwrap_or_else(|| ValueInfo::new(ValueKind::Dynamic)),
                    if binding.is_some() {
                        ExprIr::Identifier(binding_storage_name.clone().expect("binding storage"))
                    } else {
                        ExprIr::GlobalPropertyRead { name: name.clone() }
                    },
                );
                let rhs = self.lower_expression(rhs);
                let value = self.combine_bitwise(op, lhs_value, rhs);
                if let Some(storage_name) = const_storage_name {
                    return self.immutable_binding_write(&storage_name, value);
                }
                if let Some(storage_name) = binding_storage_name {
                    self.set_binding_value_info(&name, value.value_info());
                    TypedExpr::from_info(
                        value.value_info(),
                        ExprIr::AssignIdentifier {
                            name: storage_name,
                            value: Box::new(value),
                        },
                    )
                } else {
                    self.set_global_property_value_info(name.clone(), value.value_info());
                    let strictness = self.reference_strictness();
                    TypedExpr::from_info(
                        value.value_info(),
                        ExprIr::GlobalPropertyWrite {
                            name,
                            value: Box::new(value),
                            implicit: false,
                            strictness,
                        },
                    )
                }
            }
        }
    }
}
