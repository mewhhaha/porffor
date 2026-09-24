//! Staged evaluation of synchronous generator expressions.
//!
//! A `yield` nested inside an expression suspends the generator in the middle
//! of evaluating that expression. The lowering here evaluates, in
//! specification order, every operand that precedes the suspension into an
//! activation-owned binding, emits the `yield` as its own
//! `StatementIr::GeneratorYield`, and finishes the expression after it. The
//! operands therefore survive the suspension, and the backend's resumable
//! statement dispatcher re-enters after the `yield` without re-evaluating
//! anything.
//!
//! Most forms reuse their ordinary lowering: the operands are evaluated first
//! and *pinned* (see `pinned_async_operands`), so when the ordinary lowering
//! reaches the same AST node it reads the retained value back instead of
//! evaluating it again. Forms whose ordinary lowering would move an observable
//! step past a later suspension — a template substitution's ToString, a call's
//! callee lookup, an object spread's copy, a destructuring pattern's iterator
//! — have explicit staged lowerings instead.
//!
//! Admission lives in `generator_staging_plan.rs`; the two walks must agree on
//! every form and on the resume states each one allocates.

use super::*;

impl ScriptLowerer<'_> {
    /// Report why a generator body has no suspension plan, instead of a
    /// generic "generator suspension" that hides the shape.
    pub(super) fn unsupported_generator_body(&mut self, body: &FunctionBody) -> TypedExpr {
        match linear_generator_plan_with_reason(body) {
            Err(reason) => self.unsupported_expr(&reason.message()),
            Ok(_) => self.unsupported_expr("generator suspension"),
        }
    }

    pub(super) fn lower_staged_generator_expression(
        &mut self,
        expression: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if !contains(expression, ContainsSymbol::YieldExpression) {
            return Some((Vec::new(), self.lower_expression(expression)));
        }
        match expression {
            Expression::Parenthesized(parenthesized) => {
                self.lower_staged_generator_expression(parenthesized.expression())
            }
            Expression::Yield(yield_expression) => {
                let (mut statements, value) = match yield_expression.target() {
                    Some(target) => self.lower_staged_generator_expression(target)?,
                    None => (Vec::new(), TypedExpr::undefined()),
                };
                let received_name = self.alloc_suspension_owned_binding(
                    "generator.received.",
                    unknown_runtime_value_info(),
                );
                statements.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: received_name.clone(),
                    init: TypedExpr::undefined(),
                });
                let (yield_statement, _) = self.lower_linear_generator_yield_value(
                    value,
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::AssignIdentifier(received_name.clone()),
                );
                statements.push(yield_statement);
                Some((statements, self.lower_identifier_name(received_name, false)))
            }
            Expression::Call(call) => {
                self.lower_staged_generator_call(call.function(), call.args())
            }
            Expression::New(new_expression) => {
                if new_expression
                    .arguments()
                    .iter()
                    .any(|argument| matches!(argument, Expression::Spread(_)))
                {
                    return None;
                }
                let operands = std::iter::once(new_expression.constructor())
                    .chain(new_expression.arguments())
                    .collect::<Vec<_>>();
                self.lower_staged_generator_operands(expression, &operands)
            }
            Expression::Assign(assignment) => {
                self.lower_staged_generator_assignment(expression, assignment)
            }
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let (statements, _, value) = self.lower_staged_generator_property(access)?;
                Some((statements, value))
            }
            Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                self.lower_staged_generator_operands(expression, &[access.target()])
            }
            Expression::ArrayLiteral(array) => self.lower_staged_generator_array_literal(array),
            Expression::ObjectLiteral(object)
                if object_literal_stages_through_accumulator(object) =>
            {
                self.lower_staged_generator_object_accumulation(object)
            }
            Expression::ObjectLiteral(object) => {
                // 13.2.5.5: a computed key is converted with ToPropertyKey
                // before the property's value is evaluated.
                let operands = object
                    .properties()
                    .iter()
                    .flat_map(|property| {
                        let (key, value) = match property {
                            PropertyDefinition::Property(name, value) => (name, Some(value)),
                            PropertyDefinition::MethodDefinition(method) => (method.name(), None),
                            PropertyDefinition::SpreadObject(source) => {
                                return vec![(source, StagedOperandUse::Value)]
                            }
                            PropertyDefinition::IdentifierReference(_)
                            | PropertyDefinition::CoverInitializedName(..) => return Vec::new(),
                        };
                        let key = match key {
                            PropertyName::Computed(key) => {
                                Some((key, StagedOperandUse::PropertyKey))
                            }
                            PropertyName::Literal(_) => None,
                        };
                        key.into_iter()
                            .chain(value.map(|value| (value, StagedOperandUse::Value)))
                            .collect()
                    })
                    .collect::<Vec<_>>();
                self.lower_staged_generator_operand_uses(expression, &operands)
            }
            Expression::ClassExpression(class) => {
                let enclosing_prefix = self.async_expression_prefix.replace(Vec::new());
                let value = self.lower_class_expression(class);
                let statements =
                    std::mem::replace(&mut self.async_expression_prefix, enclosing_prefix)
                        .expect("class staging owns its evaluation prefix");
                Some((statements, value))
            }
            Expression::TemplateLiteral(template) => self.lower_staged_generator_template(template),
            Expression::Binary(binary) => match binary.op() {
                BinaryOp::Logical(_) if contains(binary.rhs(), ContainsSymbol::YieldExpression) => {
                    None
                }
                BinaryOp::Logical(_) => {
                    self.lower_staged_generator_operands(expression, &[binary.lhs()])
                }
                BinaryOp::Arithmetic(_)
                | BinaryOp::Bitwise(_)
                | BinaryOp::Relational(_)
                | BinaryOp::Comma => {
                    self.lower_staged_generator_operands(expression, &[binary.lhs(), binary.rhs()])
                }
            },
            Expression::BinaryInPrivate(binary) => {
                self.lower_staged_generator_operands(expression, &[binary.rhs()])
            }
            Expression::Unary(unary) => match unary.op() {
                UnaryOp::Delete => None,
                UnaryOp::Minus
                | UnaryOp::Plus
                | UnaryOp::Not
                | UnaryOp::Tilde
                | UnaryOp::TypeOf
                | UnaryOp::Void => {
                    self.lower_staged_generator_operands(expression, &[unary.target()])
                }
            },
            Expression::Conditional(conditional)
                if !contains(conditional.if_true(), ContainsSymbol::YieldExpression)
                    && !contains(conditional.if_false(), ContainsSymbol::YieldExpression) =>
            {
                self.lower_staged_generator_operands(expression, &[conditional.condition()])
            }
            Expression::Optional(optional) if optional_chain_is_stageable(optional) => {
                self.lower_staged_generator_operands(expression, &[optional.target()])
            }
            // `generator_staging_plan` rejects these before lowering starts.
            _ => None,
        }
    }

    /// Evaluate `operands` — `expression`'s operands in the order the
    /// specification evaluates them — up to the last one that suspends, then
    /// lower `expression` once with each evaluated operand pinned.
    ///
    /// Every staged operand is retained in an activation binding: a later
    /// operand's `yield` runs between an earlier operand's evaluation and the
    /// operation that consumes it, and retaining the last one too means the
    /// ordinary lowering only ever reads a binding, never re-evaluates a
    /// residual with effects. Operands after the last suspending one are left
    /// to the ordinary lowering, which evaluates them after the suspension, as
    /// the specification does.
    fn lower_staged_generator_operands(
        &mut self,
        expression: &Expression,
        operands: &[&Expression],
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let operands = operands
            .iter()
            .map(|operand| (*operand, StagedOperandUse::Value))
            .collect::<Vec<_>>();
        self.lower_staged_generator_operand_uses(expression, &operands)
    }

    fn lower_staged_generator_operand_uses(
        &mut self,
        expression: &Expression,
        operands: &[(&Expression, StagedOperandUse)],
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let last_suspending = operands
            .iter()
            .rposition(|(operand, _)| contains(*operand, ContainsSymbol::YieldExpression))?;
        let mut statements = Vec::new();
        let mut pins = Vec::new();
        for (operand, operand_use) in &operands[..=last_suspending] {
            let Some((prefix, value)) = self.lower_staged_generator_expression(operand) else {
                self.unpin_generator_operands(pins);
                return None;
            };
            statements.extend(prefix);
            let value = match operand_use {
                StagedOperandUse::Value => value,
                StagedOperandUse::PropertyKey => TypedExpr::spec_to_property_key(value),
            };
            let value = self.retain_generator_operand_value(&mut statements, value);
            pins.push(self.pin_generator_operand(operand, value));
        }
        let value = self.lower_expression(expression);
        self.unpin_generator_operands(pins);
        Some((statements, value))
    }

    /// Retain a value whose evaluation precedes a later suspension. Literals
    /// neither observe nor cause an effect, so they are left inline.
    fn retain_generator_operand_value(
        &mut self,
        statements: &mut Vec<StatementIr>,
        value: TypedExpr,
    ) -> TypedExpr {
        if matches!(
            value.expr,
            ExprIr::Number(_)
                | ExprIr::String(_)
                | ExprIr::Boolean(_)
                | ExprIr::BigInt(_)
                | ExprIr::Null
                | ExprIr::Undefined
        ) {
            return value;
        }
        self.retain_generator_operand(statements, value, "generator.operand.")
    }

    fn pin_generator_operand(&mut self, operand: &Expression, value: TypedExpr) -> usize {
        let key = std::ptr::from_ref(operand) as usize;
        self.pinned_async_operands.insert(key, value);
        key
    }

    fn unpin_generator_operands(&mut self, pins: Vec<usize>) {
        for key in pins {
            self.pinned_async_operands.remove(&key);
        }
    }

    fn lower_staged_generator_assignment(
        &mut self,
        expression: &Expression,
        assignment: &boa_ast::expression::operator::Assign,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let identifier_reference_is_static =
            !self.uses_runtime_identifier_environment() && self.with_environment_chain.is_empty();
        match (assignment.op(), assignment.lhs()) {
            (AssignOp::Assign, AssignTarget::Access(PropertyAccess::Simple(access))) => {
                self.lower_staged_generator_property_assignment(access, assignment.rhs())
            }
            (AssignOp::Assign, AssignTarget::Access(PropertyAccess::Private(access))) => self
                .lower_staged_generator_operands(expression, &[access.target(), assignment.rhs()]),
            (AssignOp::Assign, AssignTarget::Identifier(identifier))
                if identifier_reference_is_static =>
            {
                // 13.15.2 step 1.a resolves the binding before the RHS; a
                // static resolution is the same binding after the suspension.
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let (statements, value) =
                    self.lower_staged_generator_expression(assignment.rhs())?;
                Some((statements, self.lower_identifier_assign_value(name, value)))
            }
            (AssignOp::Assign, AssignTarget::Pattern(pattern)) => {
                self.lower_staged_generator_pattern_assignment(pattern, assignment.rhs())
            }
            (
                op @ (AssignOp::Add
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
                | AssignOp::Ushr),
                AssignTarget::Identifier(identifier),
            ) if identifier_reference_is_static => {
                // 13.15.2 step 2: GetValue(lref) precedes the RHS, so the old
                // value is retained across the RHS's suspension.
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let mut statements = Vec::new();
                let old_value = self.lower_expression(&Expression::Identifier(*identifier));
                let old_value =
                    self.retain_generator_operand(&mut statements, old_value, "generator.operand.");
                let (prefix, rhs) = self.lower_staged_generator_expression(assignment.rhs())?;
                statements.extend(prefix);
                let result = match op {
                    AssignOp::Add => self.combine_arithmetic(ArithmeticOp::Add, old_value, rhs),
                    AssignOp::Sub => self.combine_arithmetic(ArithmeticOp::Sub, old_value, rhs),
                    AssignOp::Mul => self.combine_arithmetic(ArithmeticOp::Mul, old_value, rhs),
                    AssignOp::Div => self.combine_arithmetic(ArithmeticOp::Div, old_value, rhs),
                    AssignOp::Mod => self.combine_arithmetic(ArithmeticOp::Mod, old_value, rhs),
                    AssignOp::Exp => self.combine_arithmetic(ArithmeticOp::Exp, old_value, rhs),
                    AssignOp::And => self.combine_bitwise(BitwiseBinaryOp::And, old_value, rhs),
                    AssignOp::Or => self.combine_bitwise(BitwiseBinaryOp::Or, old_value, rhs),
                    AssignOp::Xor => self.combine_bitwise(BitwiseBinaryOp::Xor, old_value, rhs),
                    AssignOp::Shl => self.combine_bitwise(BitwiseBinaryOp::Shl, old_value, rhs),
                    AssignOp::Shr => self.combine_bitwise(BitwiseBinaryOp::Shr, old_value, rhs),
                    AssignOp::Ushr => self.combine_bitwise(BitwiseBinaryOp::UShr, old_value, rhs),
                    AssignOp::Assign
                    | AssignOp::BoolAnd
                    | AssignOp::BoolOr
                    | AssignOp::Coalesce => unreachable!("guarded compound operator"),
                };
                Some((statements, self.lower_identifier_assign_value(name, result)))
            }
            _ => None,
        }
    }

    /// 13.2.8.6: each substitution is evaluated, converted with ToString and
    /// appended before the next one is evaluated, so the running string lives
    /// in an activation binding across every suspension.
    fn lower_staged_generator_template(
        &mut self,
        template: &TemplateLiteral,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let accumulator_name = self.alloc_suspension_owned_binding(
            "generator.template.",
            ValueInfo::new(ValueKind::String),
        );
        let mut statements = vec![StatementIr::Lexical {
            mode: BindingMode::Let,
            name: accumulator_name.clone(),
            init: TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String(String::new()),
            ),
        }];
        for element in template.elements() {
            let part = match element {
                TemplateElement::String(sym) => {
                    let value = self.interner.resolve_expect(*sym).join(
                        |string| string.to_string(),
                        Self::utf16_units_to_runtime_string,
                        true,
                    );
                    TypedExpr::from_info(ValueInfo::new(ValueKind::String), ExprIr::String(value))
                }
                TemplateElement::Expr(expression) => {
                    let (prefix, value) = self.lower_staged_generator_expression(expression)?;
                    statements.extend(prefix);
                    value
                }
            };
            let accumulator = self.lower_identifier_name(accumulator_name.clone(), false);
            let concatenated = TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::StringConcat {
                    lhs: Box::new(accumulator),
                    rhs: Box::new(part),
                },
            );
            statements.push(StatementIr::Expression(
                self.lower_identifier_assign_value(accumulator_name.clone(), concatenated),
            ));
        }
        Some((
            statements,
            self.lower_identifier_name(accumulator_name, false),
        ))
    }

    /// An object literal of literal-keyed data properties and spreads, built
    /// in a suspension-owned accumulator so each spread copies its source at
    /// its own position (13.2.5.5 PropertyDefinitionEvaluation).
    fn lower_staged_generator_object_accumulation(
        &mut self,
        object: &ObjectLiteral,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let object_info = ValueInfo {
            kind: ValueKind::Object,
            possible_kinds: KindSet::from_kind(ValueKind::Object),
            heap_shape: Some(Box::new(HeapShape::Object(ObjectShape::default()))),
            function_targets: FunctionTargetKnowledge::none(),
        };
        let accumulator_name =
            self.alloc_suspension_owned_binding("generator.object.spread.", object_info.clone());
        let mut statements = vec![StatementIr::Lexical {
            mode: BindingMode::Let,
            name: accumulator_name.clone(),
            init: TypedExpr::from_info(object_info, ExprIr::ObjectLiteral(Vec::new())),
        }];
        for property in object.properties() {
            match property {
                PropertyDefinition::SpreadObject(source) => {
                    let (source_statements, source) =
                        self.lower_staged_generator_expression(source)?;
                    statements.extend(source_statements);
                    self.invalidate_unknown_user_code_effects();
                    let accumulator = self.lower_identifier_name(accumulator_name.clone(), false);
                    statements.push(StatementIr::Expression(
                        TypedExpr::spec_copy_data_properties(accumulator, source),
                    ));
                }
                PropertyDefinition::Property(PropertyName::Literal(name), value) => {
                    let key = self.interner.resolve_expect(name.sym()).to_string();
                    let (value_statements, value) =
                        self.lower_staged_generator_expression(value)?;
                    statements.extend(value_statements);
                    let accumulator = self.lower_identifier_name(accumulator_name.clone(), false);
                    statements.push(StatementIr::Expression(
                        TypedExpr::spec_create_data_property_or_throw(
                            accumulator,
                            TypedExpr::from_info(
                                ValueInfo::new(ValueKind::String),
                                ExprIr::String(key),
                            ),
                            value,
                        ),
                    ));
                }
                _ => return None,
            }
        }
        Some((
            statements,
            self.lower_identifier_name(accumulator_name, false),
        ))
    }

    pub(super) fn lower_discarded_generator_expression(
        &mut self,
        expression: &Expression,
    ) -> Option<Vec<StatementIr>> {
        match expression {
            Expression::Parenthesized(parenthesized) => {
                self.lower_discarded_generator_expression(parenthesized.expression())
            }
            Expression::Yield(yield_expression)
                if !yield_expression
                    .target()
                    .is_some_and(|target| contains(target, ContainsSymbol::YieldExpression)) =>
            {
                let (statement, _) = self.lower_linear_generator_yield(
                    yield_expression.target(),
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                );
                Some(vec![statement])
            }
            Expression::ArrayLiteral(array)
                if !array
                    .as_ref()
                    .iter()
                    .flatten()
                    .any(|element| matches!(element, Expression::Spread(_))) =>
            {
                let mut statements = Vec::new();
                for element in array.as_ref().iter().flatten() {
                    statements.extend(self.lower_discarded_generator_expression(element)?);
                }
                Some(statements)
            }
            Expression::Binary(binary) if binary.op() == BinaryOp::Comma => {
                let mut statements = self.lower_discarded_generator_expression(binary.lhs())?;
                statements.extend(self.lower_discarded_generator_expression(binary.rhs())?);
                Some(statements)
            }
            Expression::Binary(binary)
                if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Add) =>
            {
                let (mut statements, lhs) = self.lower_staged_generator_expression(binary.lhs())?;
                let lhs_name =
                    self.alloc_suspension_owned_binding("generator.binary.lhs.", lhs.value_info());
                statements.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: lhs_name.clone(),
                    init: lhs,
                });
                let (rhs_statements, rhs) = self.lower_staged_generator_expression(binary.rhs())?;
                statements.extend(rhs_statements);
                let possible_kinds = KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Number))
                    .union(KindSet::from_kind(ValueKind::BigInt));
                statements.push(StatementIr::Expression(TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::CoerciveAdd {
                        lhs: Box::new(self.lower_identifier_name(lhs_name, false)),
                        rhs: Box::new(rhs),
                    },
                )));
                Some(statements)
            }
            Expression::Conditional(conditional)
                if discarded_conditional_has_direct_yield_branches(conditional) =>
            {
                self.lower_discarded_generator_conditional(conditional)
            }
            expression if contains(expression, ContainsSymbol::YieldExpression) => {
                let (mut statements, value) = self.lower_staged_generator_expression(expression)?;
                statements.push(StatementIr::Expression(value));
                Some(statements)
            }
            _ => Some(vec![StatementIr::Expression(
                self.lower_expression(expression),
            )]),
        }
    }

    /// `(yield a) ? yield b : yield c`: the received condition selects one of
    /// two direct yields through a two-branch `GeneratorIf`.
    fn lower_discarded_generator_conditional(
        &mut self,
        conditional: &boa_ast::expression::operator::Conditional,
    ) -> Option<Vec<StatementIr>> {
        let Expression::Yield(condition_yield) =
            Self::unwrap_parenthesized_expr(conditional.condition())
        else {
            return None;
        };
        let received_name = self.alloc_temp_binding_name("yield.condition.");
        self.declare_binding(
            received_name.clone(),
            BindingInfo {
                mode: BindingMode::Let,
                storage_name: received_name.clone(),
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
                initialization: Initialization::Initialized,
            },
        );
        let (condition_yield_statement, _) = self.lower_linear_generator_yield(
            condition_yield.target(),
            condition_yield.delegate(),
            GeneratorResumeModeIr::AssignIdentifier(received_name.clone()),
        );
        let entry_state = self.current_generator_resume_state?;

        let mut then_statements =
            self.lower_discarded_generator_expression(conditional.if_true())?;
        let then_yield_statement = then_statements.pop()?;
        let StatementIr::GeneratorYield {
            resume_state: then_resume_state,
            ..
        } = &then_yield_statement
        else {
            return None;
        };
        if !then_statements.is_empty() {
            return None;
        }
        let then_resume_state = *then_resume_state;

        let mut else_statements =
            self.lower_discarded_generator_expression(conditional.if_false())?;
        let else_yield_statement = else_statements.pop()?;
        let StatementIr::GeneratorYield {
            resume_state: else_resume_state,
            ..
        } = &else_yield_statement
        else {
            return None;
        };
        if !else_statements.is_empty() {
            return None;
        }
        let else_resume_state = *else_resume_state;
        let exit_state = self.current_generator_resume_state? + 1;
        self.current_generator_resume_state = Some(exit_state);
        let condition = self.lower_identifier_name(received_name.clone(), false);

        Some(vec![
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: received_name,
                init: TypedExpr::undefined(),
            },
            condition_yield_statement,
            StatementIr::GeneratorIf {
                condition,
                then_before_yield: Vec::new(),
                then_yield_statement: Some(Box::new(then_yield_statement)),
                then_after_yield: Vec::new(),
                else_before_yield: Vec::new(),
                else_yield_statement: Some(Box::new(else_yield_statement)),
                else_after_yield: Vec::new(),
                entry_state,
                then_resume_state: Some(then_resume_state),
                else_resume_state: Some(else_resume_state),
                exit_state,
            },
        ])
    }

    /// `pattern = rhs` (13.15.2 step 2): the RHS is evaluated first, then
    /// 13.15.5.2 DestructuringAssignmentEvaluation runs against its value, and
    /// the expression's value is the RHS value.
    fn lower_staged_generator_pattern_assignment(
        &mut self,
        pattern: &Pattern,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let (mut statements, value) = self.lower_staged_generator_expression(rhs)?;
        if !contains(pattern, ContainsSymbol::YieldExpression) {
            let value = self.lower_pattern_assign_value(pattern, value)?;
            return Some((statements, value));
        }
        // An async generator's resume states are preplanned in visit order;
        // the structured shapes below allocate states as they lower.
        if self.current_resumable_plan.is_some() {
            self.unsupported(
                "async generator destructuring assignment whose pattern suspends has no \
                 preplanned structured resume point",
            );
            return None;
        }
        let value =
            self.retain_generator_operand(&mut statements, value, "generator.destructuring.value.");
        self.stage_generator_pattern(pattern, value.clone(), &mut statements)?;
        Some((statements, value))
    }

    fn stage_generator_pattern(
        &mut self,
        pattern: &Pattern,
        value: TypedExpr,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        if !contains(pattern, ContainsSymbol::YieldExpression) {
            let assigned = self.lower_pattern_assign_value(pattern, value)?;
            statements.push(StatementIr::Expression(assigned));
            return Some(());
        }
        match pattern {
            Pattern::Array(array) => {
                self.stage_generator_array_pattern(array.bindings(), value, statements)
            }
            Pattern::Object(object) => {
                self.stage_generator_object_pattern(object.bindings(), value, statements)
            }
        }
    }

    /// 13.15.5.2 ArrayAssignmentPattern whose elements suspend.
    ///
    /// The Iterator Record lives in activation slots. Consecutive elements
    /// that do not suspend run as one `Elements` step; a suspending element
    /// first evaluates its target Reference into activation bindings, so its
    /// iterator step and PutValue read only retained values. Step 3's
    /// IteratorClose runs from the catch block (a throw completion) and the
    /// finally block (a normal completion, or a `return()` resumption) of a
    /// synthesized try statement around the elements, both under the
    /// `[[Done]]` guard.
    fn stage_generator_array_pattern(
        &mut self,
        elements: &[ArrayPatternElement],
        value: TypedExpr,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let record = self.alloc_destructuring_iterator_record();
        for slot in [
            record.iterator().as_str(),
            record.next_method().as_str(),
            record.done().as_str(),
        ] {
            statements.push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: slot.to_string(),
                init: TypedExpr::undefined(),
            });
        }
        statements.push(StatementIr::Expression(resumable_array_destructuring_step(
            &record,
            ResumableArrayDestructuringStepIr::Open {
                value: Box::new(value),
                protocol: ResumableArrayPatternProtocol::RESUMABLE_ARRAY_DESTRUCTURING,
            },
        )));
        self.invalidate_unknown_user_code_effects();

        let entry_state = self.current_generator_resume_state?;
        let mut body = Vec::new();
        let mut run = Vec::new();
        for element in elements {
            self.stage_generator_array_pattern_element(&record, element, &mut run, &mut body)?;
        }
        flush_resumable_array_elements(&record, &mut run, &mut body);

        let try_exit_state = self.advance_generator_resume_state()?;
        let catch_name = self.alloc_temp_binding_name("generator.destructuring.thrown.");
        let catch_block = BlockIr {
            statements: vec![
                StatementIr::Expression(resumable_array_destructuring_step(
                    &record,
                    ResumableArrayDestructuringStepIr::Close(ResumableIteratorCloseIr::Throw),
                )),
                StatementIr::Throw(TypedExpr::from_info(
                    unknown_runtime_value_info(),
                    ExprIr::Identifier(catch_name.clone()),
                )),
            ],
            result_kind: ValueKind::Undefined,
            lexical_environment: None,
        };
        let catch_exit_state = self.advance_generator_resume_state()?;
        let finally_block = BlockIr {
            statements: vec![StatementIr::Expression(resumable_array_destructuring_step(
                &record,
                ResumableArrayDestructuringStepIr::Close(ResumableIteratorCloseIr::NormalOrReturn),
            ))],
            result_kind: ValueKind::Undefined,
            lexical_environment: None,
        };
        let exit_state = self.advance_generator_resume_state()?;
        self.invalidate_unknown_user_code_effects();
        statements.push(StatementIr::TryCatchFinally {
            try_block: BlockIr {
                statements: body,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            catch_name: catch_name.clone(),
            catch_source_name: catch_name,
            catch_parameter_environment: None,
            catch_block,
            finally_block,
            generator_plan: Some(GeneratorTryPlanIr {
                entry_state,
                try_exit_state,
                catch_entry_state: Some(try_exit_state),
                catch_exit_state: Some(catch_exit_state),
                finally_entry_state: Some(catch_exit_state),
                finally_exit_state: Some(exit_state),
                exit_state,
            }),
            async_plan: None,
        });
        Some(())
    }

    fn advance_generator_resume_state(&mut self) -> Option<u32> {
        let state = self.current_generator_resume_state?.checked_add(1)?;
        self.current_generator_resume_state = Some(state);
        Some(state)
    }

    fn stage_generator_array_pattern_element(
        &mut self,
        record: &IteratorRecordIr,
        element: &ArrayPatternElement,
        run: &mut Vec<ArrayDestructuringElementIr>,
        body: &mut Vec<StatementIr>,
    ) -> Option<()> {
        if !contains(element, ContainsSymbol::YieldExpression) {
            run.push(self.lower_array_assignment_element(element)?);
            return Some(());
        }
        flush_resumable_array_elements(record, run, body);
        match element {
            // Only a target Reference suspends: evaluate its operands now and
            // let the ordinary element read them back.
            ArrayPatternElement::PropertyAccess {
                access,
                default_init: None,
            }
            | ArrayPatternElement::PropertyAccessRest { access } => {
                self.stage_reference_suspending_array_element(access, element, run, body)?;
            }
            ArrayPatternElement::PropertyAccess {
                access,
                default_init: Some(default),
            } if !contains(default, ContainsSymbol::YieldExpression) => {
                self.stage_reference_suspending_array_element(access, element, run, body)?;
            }
            // 13.15.5.5 AssignmentElement: lref, IteratorStepValue, the
            // Initializer when the value is undefined, then PutValue.
            ArrayPatternElement::SingleName {
                ident,
                default_init: Some(default),
            } => {
                let value_name = self.stage_resumable_array_value(record, false, body);
                self.stage_generator_destructuring_default(&value_name, default, body)?;
                let value = self.lower_identifier_name(value_name, false);
                body.push(StatementIr::Expression(
                    self.put_staged_identifier(*ident, value)?,
                ));
            }
            ArrayPatternElement::PropertyAccess {
                access,
                default_init: Some(default),
            } => {
                let reference = self.stage_generator_destructuring_reference(access, body)?;
                let value_name = self.stage_resumable_array_value(record, false, body);
                self.stage_generator_destructuring_default(&value_name, default, body)?;
                let value = self.lower_identifier_name(value_name, false);
                body.push(StatementIr::Expression(
                    self.put_staged_reference(reference, value),
                ));
            }
            ArrayPatternElement::Pattern {
                pattern,
                default_init,
            } => {
                let value_name = self.stage_resumable_array_value(record, false, body);
                if let Some(default) = default_init {
                    self.stage_generator_destructuring_default(&value_name, default, body)?;
                }
                let value = self.lower_identifier_name(value_name, false);
                self.stage_generator_pattern(pattern, value, body)?;
            }
            ArrayPatternElement::PatternRest { pattern } => {
                let value_name = self.stage_resumable_array_value(record, true, body);
                let value = self.lower_identifier_name(value_name, false);
                self.stage_generator_pattern(pattern, value, body)?;
            }
            // These contain no expression that could suspend.
            ArrayPatternElement::Elision
            | ArrayPatternElement::SingleName {
                default_init: None, ..
            }
            | ArrayPatternElement::SingleNameRest { .. } => return None,
        }
        Some(())
    }

    /// An element whose only suspension is in its target Reference: the
    /// Reference's operands are evaluated and pinned, and the ordinary
    /// element then reads them back inside the next `Elements` step.
    fn stage_reference_suspending_array_element(
        &mut self,
        access: &PropertyAccess,
        element: &ArrayPatternElement,
        run: &mut Vec<ArrayDestructuringElementIr>,
        body: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let pins = self.stage_generator_reference_operands(access, body)?;
        let lowered = self.lower_array_assignment_element(element);
        self.unpin_generator_operands(pins);
        run.push(lowered?);
        Some(())
    }

    /// Store the element's value — the next iterator value, or for a rest
    /// element the remaining values in a new Array — in an activation binding
    /// through a binding target, so the steps after it can suspend.
    fn stage_resumable_array_value(
        &mut self,
        record: &IteratorRecordIr,
        rest: bool,
        body: &mut Vec<StatementIr>,
    ) -> String {
        let info = if rest {
            ValueInfo {
                kind: ValueKind::Array,
                possible_kinds: KindSet::from_kind(ValueKind::Array),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            }
        } else {
            unknown_runtime_value_info()
        };
        let name = self.alloc_suspension_owned_binding("generator.destructuring.element.", info);
        body.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: name.clone(),
            init: TypedExpr::undefined(),
        });
        let target = DestructuringTargetIr::Binding {
            mode: BindingMode::Let,
            name: name.clone(),
        };
        let element = if rest {
            ArrayDestructuringElementIr::Rest { target }
        } else {
            ArrayDestructuringElementIr::Target {
                target,
                default: None,
            }
        };
        body.push(StatementIr::Expression(resumable_array_destructuring_step(
            record,
            ResumableArrayDestructuringStepIr::Elements(vec![element]),
        )));
        self.invalidate_unknown_user_code_effects();
        name
    }

    /// 13.15.5.3 ObjectAssignmentPattern whose properties suspend. There is no
    /// iterator to close, so each suspending property evaluates its key and
    /// target Reference into activation bindings and reads the property
    /// afterwards.
    fn stage_generator_object_pattern(
        &mut self,
        elements: &[ObjectPatternElement],
        value: TypedExpr,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        // Step 1: RequireObjectCoercible(value) precedes every property.
        statements.push(StatementIr::Expression(object_destructure(
            value.clone(),
            Vec::new(),
        )));
        let mut run = Vec::new();
        for element in elements {
            if !contains(element, ContainsSymbol::YieldExpression) {
                run.push(self.lower_object_assignment_property(element)?);
                continue;
            }
            if !run.is_empty() {
                statements.push(StatementIr::Expression(object_destructure(
                    value.clone(),
                    std::mem::take(&mut run),
                )));
            }
            let (name, target, default) = match element {
                ObjectPatternElement::SingleName {
                    name, default_init, ..
                } => (name, None, default_init.as_ref()),
                ObjectPatternElement::AssignmentPropertyAccess {
                    name,
                    access,
                    default_init,
                } => (name, Some(access), default_init.as_ref()),
                ObjectPatternElement::Pattern {
                    name, default_init, ..
                } => (name, None, default_init.as_ref()),
                ObjectPatternElement::RestProperty { .. }
                | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => return None,
            };
            let mut pins = Vec::new();
            // 13.15.5.3 step 1 evaluates the PropertyName, including
            // ToPropertyKey, before the target Reference.
            let key = match name {
                PropertyName::Literal(literal) => {
                    let key = self.interner.resolve_expect(literal.sym()).to_string();
                    TypedExpr::from_info(ValueInfo::new(ValueKind::String), ExprIr::String(key))
                }
                PropertyName::Computed(expression) => {
                    let (prefix, key) = self.lower_staged_generator_expression(expression)?;
                    statements.extend(prefix);
                    let key = self.retain_generator_operand(
                        statements,
                        TypedExpr::spec_to_property_key(key),
                        "generator.destructuring.key.",
                    );
                    pins.push(self.pin_generator_operand(expression, key.clone()));
                    key
                }
            };
            let suspends_after_read = default
                .is_some_and(|default| contains(default, ContainsSymbol::YieldExpression))
                || matches!(element, ObjectPatternElement::Pattern { pattern, .. }
                    if contains(pattern, ContainsSymbol::YieldExpression));
            if !suspends_after_read {
                if let Some(access) = target {
                    let Some(reference_pins) =
                        self.stage_generator_reference_operands(access, statements)
                    else {
                        self.unpin_generator_operands(pins);
                        return None;
                    };
                    pins.extend(reference_pins);
                }
                let lowered = self.lower_object_assignment_property(element);
                self.unpin_generator_operands(pins);
                statements.push(StatementIr::Expression(object_destructure(
                    value.clone(),
                    vec![lowered?],
                )));
                continue;
            }
            self.unpin_generator_operands(pins);
            let reference = match target {
                Some(access) => {
                    Some(self.stage_generator_destructuring_reference(access, statements)?)
                }
                None => None,
            };
            let value_name = self.alloc_suspension_owned_binding(
                "generator.destructuring.element.",
                unknown_runtime_value_info(),
            );
            self.invalidate_unknown_user_code_effects();
            statements.push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: value_name.clone(),
                init: TypedExpr::spec_get_v(value.clone(), key),
            });
            if let Some(default) = default {
                self.stage_generator_destructuring_default(&value_name, default, statements)?;
            }
            let property_value = self.lower_identifier_name(value_name, false);
            match element {
                ObjectPatternElement::SingleName { ident, .. } => statements.push(
                    StatementIr::Expression(self.put_staged_identifier(*ident, property_value)?),
                ),
                ObjectPatternElement::AssignmentPropertyAccess { .. } => {
                    let reference = reference?;
                    statements.push(StatementIr::Expression(
                        self.put_staged_reference(reference, property_value),
                    ));
                }
                ObjectPatternElement::Pattern { pattern, .. } => {
                    self.stage_generator_pattern(pattern, property_value, statements)?;
                }
                ObjectPatternElement::RestProperty { .. }
                | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => return None,
            }
        }
        if !run.is_empty() {
            statements.push(StatementIr::Expression(object_destructure(value, run)));
        }
        Some(())
    }

    /// Evaluate a property Reference's base (and raw computed key) now and pin
    /// them, so an ordinary destructuring target reads them back.
    fn stage_generator_reference_operands(
        &mut self,
        access: &PropertyAccess,
        statements: &mut Vec<StatementIr>,
    ) -> Option<Vec<usize>> {
        let mut pins = Vec::new();
        let operands: Vec<&Expression> = match access {
            PropertyAccess::Simple(access) => match access.field() {
                PropertyAccessField::Const(_) => vec![access.target()],
                PropertyAccessField::Expr(key) => vec![access.target(), key],
            },
            PropertyAccess::Private(access) => vec![access.target()],
            PropertyAccess::Super(_) => return None,
        };
        for operand in operands {
            let Some((prefix, value)) = self.lower_staged_generator_expression(operand) else {
                self.unpin_generator_operands(pins);
                return None;
            };
            statements.extend(prefix);
            let value = self.retain_generator_operand_value(statements, value);
            pins.push(self.pin_generator_operand(operand, value));
        }
        Some(pins)
    }

    /// Evaluate a property Reference whose PutValue follows a suspending
    /// Initializer: the base and the raw key are retained, and ToPropertyKey
    /// stays with PutValue.
    fn stage_generator_destructuring_reference(
        &mut self,
        access: &PropertyAccess,
        statements: &mut Vec<StatementIr>,
    ) -> Option<StagedDestructuringReference> {
        match access {
            PropertyAccess::Simple(access) => {
                let (prefix, base) = self.lower_staged_generator_expression(access.target())?;
                statements.extend(prefix);
                let base = self.retain_generator_operand(statements, base, "generator.receiver.");
                let key = match access.field() {
                    PropertyAccessField::Const(field) => PropertyKeyIr::StaticString(
                        self.interner.resolve_expect(field.sym()).to_string(),
                    ),
                    PropertyAccessField::Expr(expression) => {
                        let (prefix, key) = self.lower_staged_generator_expression(expression)?;
                        statements.extend(prefix);
                        let key = self.retain_generator_operand(statements, key, "generator.key.");
                        PropertyKeyIr::StringExpr(Box::new(key))
                    }
                };
                Some(StagedDestructuringReference::Property {
                    base,
                    key,
                    strictness: self.reference_strictness(),
                })
            }
            PropertyAccess::Private(access) => {
                let private_name_id = self.current_private_name_id(access.field())?;
                let (prefix, base) = self.lower_staged_generator_expression(access.target())?;
                statements.extend(prefix);
                let base = self.retain_generator_operand(statements, base, "generator.receiver.");
                Some(StagedDestructuringReference::Private {
                    base,
                    private_name_id,
                })
            }
            PropertyAccess::Super(_) => None,
        }
    }

    fn put_staged_reference(
        &mut self,
        reference: StagedDestructuringReference,
        value: TypedExpr,
    ) -> TypedExpr {
        match reference {
            StagedDestructuringReference::Property {
                base,
                key,
                strictness,
            } => {
                let (_, mut possible_setters) = self.possible_unknown_accessor_functions();
                possible_setters.extend_known(self.dynamically_installed_setters.iter().cloned());
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                OrdinaryPropertyReferencePlan::new(Box::new(base), key, strictness)
                    .plain_assignment(value, possible_setters)
            }
            StagedDestructuringReference::Private {
                base,
                private_name_id,
            } => TypedExpr::from_info(
                value.value_info(),
                ExprIr::PrivateWrite {
                    target: Box::new(base),
                    private_name_id,
                    value: Box::new(value),
                },
            ),
        }
    }

    /// PutValue through a statically resolved identifier Reference. A runtime
    /// environment (`with`, sloppy direct `eval`) would have to retain the
    /// resolved Reference across the suspension instead.
    fn put_staged_identifier(
        &mut self,
        identifier: boa_ast::expression::Identifier,
        value: TypedExpr,
    ) -> Option<TypedExpr> {
        if self.uses_runtime_identifier_environment() || !self.with_environment_chain.is_empty() {
            return None;
        }
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        Some(self.lower_identifier_assign_value(name, value))
    }

    /// 13.15.5.5 step 4 / 13.15.5.6 step 3: the Initializer runs only when the
    /// value is undefined. A suspending Initializer is a one-branch
    /// `GeneratorIf` around its single `yield`.
    fn stage_generator_destructuring_default(
        &mut self,
        value_name: &str,
        default: &Expression,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let value = self.lower_identifier_name(value_name.to_string(), false);
        let condition = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::StrictEquality {
                op: EqualityBinaryOp::StrictEqual,
                lhs: Box::new(value),
                rhs: Box::new(TypedExpr::undefined()),
            },
        );
        // The branch's own temporaries live in a scope of their own, so the
        // facts after the branch join with the facts of the skipped path.
        let before = self.capture_conditional_flow_facts();
        if !contains(default, ContainsSymbol::YieldExpression) {
            self.push_scope();
            let default_value = self.lower_expression(default);
            let assignment =
                self.lower_identifier_assign_value(value_name.to_string(), default_value);
            self.pop_scope();
            let then_facts = self.capture_conditional_flow_facts();
            self.merge_conditional_flow_facts(then_facts, before);
            statements.push(StatementIr::If {
                condition,
                then_branch: Box::new(StatementIr::Expression(assignment)),
                else_branch: None,
            });
            return Some(());
        }
        let entry_state = self.current_generator_resume_state?;
        self.push_scope();
        let staged = self.lower_staged_generator_expression(default);
        let Some((mut branch, default_value)) = staged else {
            self.pop_scope();
            return None;
        };
        branch.push(StatementIr::Expression(self.lower_identifier_assign_value(
            value_name.to_string(),
            default_value,
        )));
        self.pop_scope();
        let then_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(then_facts, before);
        let yield_index = branch.iter().position(|statement| {
            matches!(
                statement,
                StatementIr::GeneratorYield {
                    form: YieldForm::Plain,
                    ..
                }
            )
        })?;
        if branch.iter().enumerate().any(|(index, statement)| {
            index != yield_index && crate::ir::statement_contains_suspension(statement)
        }) {
            return None;
        }
        let then_after_yield = branch.split_off(yield_index + 1);
        let then_yield_statement = branch.pop()?;
        let StatementIr::GeneratorYield {
            suspend_state,
            resume_state,
            ..
        } = &then_yield_statement
        else {
            return None;
        };
        if *suspend_state != entry_state {
            return None;
        }
        let then_resume_state = *resume_state;
        let exit_state = self.advance_generator_resume_state()?;
        statements.push(StatementIr::GeneratorIf {
            condition,
            then_before_yield: branch,
            then_yield_statement: Some(Box::new(then_yield_statement)),
            then_after_yield,
            else_before_yield: Vec::new(),
            else_yield_statement: None,
            else_after_yield: Vec::new(),
            entry_state,
            then_resume_state: Some(then_resume_state),
            else_resume_state: None,
            exit_state,
        });
        Some(())
    }
}

/// How the operation consumes a staged operand retained across a later
/// suspension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StagedOperandUse {
    /// The operand's value, exactly as evaluated.
    Value,
    /// A computed property name, whose ToPropertyKey belongs to its own
    /// evaluation step and so must not wait for the later suspension.
    PropertyKey,
}

/// A destructuring target Reference evaluated before the element's value
/// (13.15.5.5 step 1, 13.15.5.6 step 1), for a PutValue that follows a
/// suspending Initializer.
enum StagedDestructuringReference {
    Property {
        base: TypedExpr,
        key: PropertyKeyIr,
        strictness: Strictness,
    },
    Private {
        base: TypedExpr,
        private_name_id: PrivateNameId,
    },
}

fn resumable_array_destructuring_step(
    record: &IteratorRecordIr,
    step: ResumableArrayDestructuringStepIr,
) -> TypedExpr {
    TypedExpr::from_info(
        ValueInfo::undefined(),
        ExprIr::ResumableArrayDestructuring(Box::new(ResumableArrayDestructuringIr::new(
            record.clone(),
            step,
        ))),
    )
}

fn flush_resumable_array_elements(
    record: &IteratorRecordIr,
    run: &mut Vec<ArrayDestructuringElementIr>,
    body: &mut Vec<StatementIr>,
) {
    if run.is_empty() {
        return;
    }
    body.push(StatementIr::Expression(resumable_array_destructuring_step(
        record,
        ResumableArrayDestructuringStepIr::Elements(std::mem::take(run)),
    )));
}

fn object_destructure(
    value: TypedExpr,
    properties: Vec<ObjectDestructuringPropertyIr>,
) -> TypedExpr {
    TypedExpr::from_info(
        value.value_info(),
        ExprIr::ObjectDestructure {
            value: Box::new(value),
            pattern: Box::new(ObjectDestructuringPatternIr {
                properties,
                rest: None,
            }),
        },
    )
}
