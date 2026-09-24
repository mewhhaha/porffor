//! The suspension plan of a synchronous generator expression evaluated one
//! operand at a time.
//!
//! `ScriptLowerer::lower_staged_generator_expression` evaluates every operand
//! the specification orders before a `yield` into an activation-owned binding,
//! then emits the `yield` as its own statement, so any expression position can
//! suspend and resume with its partially evaluated operands intact. This module
//! is the admission side of that lowering: it walks the same forms in the same
//! order and allocates the resume states the lowering consumes, or names the
//! form that has no staged evaluation order. The two walks are kept in lockstep
//! by the `staged_generator_*` tests in `lib.rs`, which compare the planned
//! state count with the states the lowered IR uses.
//!
//! Most suspensions are linear: state `s` suspends and `s + 1` resumes. Two
//! structured shapes need more:
//!
//! - a destructuring Initializer that suspends runs only when the value is
//!   `undefined`, so it becomes a one-branch `StatementIr::GeneratorIf`
//!   (suspend `e`, resume `e + 1`, exit `e + 2`), and
//! - an array destructuring pattern that suspends keeps its Iterator Record
//!   across the suspension and must close it on an abrupt resumption, so its
//!   elements run inside a synthesized try/catch/finally that reserves one
//!   state after each of its three blocks.

use super::*;

#[cfg(test)]
#[path = "generator_staging_plan_tests.rs"]
mod tests;

/// Why a `yield` nested in an expression has no staged suspension plan.
///
/// Closed and matched exhaustively by [`StagedYieldRejection::message`]; every
/// rejection names the construct a sweep should group it under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StagedYieldRejection {
    /// `&&`, `||`, `??`, a `?:` branch, a logical assignment, or an optional
    /// chain link: the `yield` runs on only some paths, which needs a
    /// branching resume point rather than a staged operand.
    ConditionalOperand,
    /// `delete`, `++`/`--`, a compound assignment through a property, private
    /// or `super` Reference, or a `super` property: the Reference itself would
    /// have to survive the suspension.
    UnstagedReference,
    /// A call or `new` whose argument list has a spread element, or a tagged
    /// template.
    UnstagedArgumentList,
    /// `import()`, `super(...)`, an optional call, or another form with no
    /// staged evaluation order.
    UnstagedForm,
    /// A destructuring Initializer that suspends more than once, delegates
    /// with `yield*`, or suspends inside a structured statement of its own: a
    /// one-branch resume point holds exactly one plain `yield`.
    DestructuringInitializer,
    /// An object rest element in a destructuring pattern that suspends: the
    /// excluded keys would have to be collected across the suspension.
    DestructuringObjectRest,
    /// An object literal spread or shorthand property evaluated before a later
    /// `yield` in the same literal.
    ObjectLiteralOperand,
    /// A class heritage or computed element name whose suspension needs a
    /// branching or finalizing resume point; class evaluation prefixes are
    /// linear.
    ClassOperand,
}

impl StagedYieldRejection {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::ConditionalOperand => {
                "a yield in a conditionally evaluated operand (`&&`, `||`, `??`, a `?:` branch, a \
                 logical assignment or an optional chain) needs a branching resume point"
            }
            Self::UnstagedReference => {
                "a yield inside a `delete`, `++`/`--`, compound assignment or `super` Reference \
                 has no staged Reference"
            }
            Self::UnstagedArgumentList => {
                "a yield in an argument list with a spread element or in a tagged template has \
                 no staged argument list"
            }
            Self::UnstagedForm => {
                "a yield inside `import()`, `super()`, an optional call or another form has no \
                 staged evaluation order"
            }
            Self::DestructuringInitializer => {
                "a destructuring initializer that suspends more than once or delegates with \
                 `yield*` needs more than one conditional resume point"
            }
            Self::DestructuringObjectRest => {
                "an object rest element in a destructuring pattern that suspends needs its \
                 excluded keys across the suspension"
            }
            Self::ObjectLiteralOperand => {
                "an object literal spread or shorthand property evaluated before a later yield \
                 has no retained operand"
            }
            Self::ClassOperand => {
                "a class heritage or computed element name whose yield needs a branching or \
                 finalizing resume point"
            }
        }
    }
}

/// The resume states one staged evaluation allocates, in allocation order.
pub(crate) struct StagedSuspensionPlan<'a> {
    current_state: &'a mut u32,
    suspension_points: &'a mut Vec<GeneratorSuspensionPointIr>,
}

impl<'a> StagedSuspensionPlan<'a> {
    pub(crate) fn new(
        current_state: &'a mut u32,
        suspension_points: &'a mut Vec<GeneratorSuspensionPointIr>,
    ) -> Self {
        Self {
            current_state,
            suspension_points,
        }
    }

    /// One `yield` statement: suspend in the current state, resume in the next.
    fn suspend(&mut self) {
        let suspend_state = *self.current_state;
        *self.current_state += 1;
        self.suspension_points.push(GeneratorSuspensionPointIr {
            suspend_state,
            resume_state: *self.current_state,
        });
    }

    /// A one-branch `GeneratorIf` around exactly one `yield`: the branch
    /// suspends in the entry state and resumes in the next, and the statement
    /// exits one state later (see `lower_if_statement`).
    fn conditional_suspend(&mut self) {
        self.suspend();
        *self.current_state += 1;
    }

    /// A generator try statement reserves one state after each block.
    fn reserve(&mut self) {
        *self.current_state += 1;
    }
}

/// Plan `expression` evaluated for its value, one operand at a time.
pub(crate) fn plan_staged_generator_expression(
    expression: &Expression,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    if !contains(expression, ContainsSymbol::YieldExpression) {
        return Ok(());
    }
    match expression {
        Expression::Parenthesized(parenthesized) => {
            plan_staged_generator_expression(parenthesized.expression(), plan)
        }
        Expression::Yield(yield_expression) => {
            if let Some(target) = yield_expression.target() {
                plan_staged_generator_expression(target, plan)?;
            }
            plan.suspend();
            Ok(())
        }
        Expression::Call(call) => {
            if call
                .args()
                .iter()
                .any(|argument| matches!(argument, Expression::Spread(_)))
            {
                return Err(StagedYieldRejection::UnstagedArgumentList);
            }
            match unwrap_parenthesized(call.function()) {
                Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                    plan_staged_simple_access(access, plan)?;
                }
                Expression::PropertyAccess(
                    PropertyAccess::Private(_) | PropertyAccess::Super(_),
                )
                | Expression::Optional(_)
                | Expression::SuperCall(_)
                    if contains(call, ContainsSymbol::YieldExpression) =>
                {
                    return Err(StagedYieldRejection::UnstagedForm);
                }
                callee => plan_staged_generator_expression(callee, plan)?,
            }
            plan_staged_generator_operands(call.args().iter(), plan)
        }
        Expression::New(new_expression) => {
            if new_expression
                .arguments()
                .iter()
                .any(|argument| matches!(argument, Expression::Spread(_)))
            {
                return Err(StagedYieldRejection::UnstagedArgumentList);
            }
            plan_staged_generator_expression(new_expression.constructor(), plan)?;
            plan_staged_generator_operands(new_expression.arguments().iter(), plan)
        }
        Expression::Assign(assignment) => plan_staged_generator_assignment(assignment, plan),
        Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
            plan_staged_simple_access(access, plan)
        }
        Expression::PropertyAccess(PropertyAccess::Private(access)) => {
            plan_staged_generator_expression(access.target(), plan)
        }
        Expression::PropertyAccess(PropertyAccess::Super(_)) => {
            Err(StagedYieldRejection::UnstagedReference)
        }
        Expression::ArrayLiteral(array) => plan_staged_generator_operands(
            array
                .as_ref()
                .iter()
                .flatten()
                .map(|element| match element {
                    Expression::Spread(spread) => spread.target(),
                    element => element,
                }),
            plan,
        ),
        Expression::ObjectLiteral(object) => plan_staged_generator_object_literal(object, plan),
        Expression::ClassExpression(class) => {
            plan_linear_class_operands(class.super_ref(), class.elements(), plan)
        }
        Expression::TemplateLiteral(template) => plan_staged_generator_operands(
            template
                .elements()
                .iter()
                .filter_map(|element| match element {
                    TemplateElement::Expr(expression) => Some(expression),
                    TemplateElement::String(_) => None,
                }),
            plan,
        ),
        Expression::Binary(binary) => match binary.op() {
            BinaryOp::Logical(_) if contains(binary.rhs(), ContainsSymbol::YieldExpression) => {
                Err(StagedYieldRejection::ConditionalOperand)
            }
            BinaryOp::Logical(_) => plan_staged_generator_expression(binary.lhs(), plan),
            BinaryOp::Arithmetic(_)
            | BinaryOp::Bitwise(_)
            | BinaryOp::Relational(_)
            | BinaryOp::Comma => {
                plan_staged_generator_operands([binary.lhs(), binary.rhs()].into_iter(), plan)
            }
        },
        Expression::BinaryInPrivate(binary) => plan_staged_generator_expression(binary.rhs(), plan),
        Expression::Unary(unary) => match unary.op() {
            UnaryOp::Delete => Err(StagedYieldRejection::UnstagedReference),
            UnaryOp::Minus
            | UnaryOp::Plus
            | UnaryOp::Not
            | UnaryOp::Tilde
            | UnaryOp::TypeOf
            | UnaryOp::Void => plan_staged_generator_expression(unary.target(), plan),
        },
        Expression::Update(_) => Err(StagedYieldRejection::UnstagedReference),
        Expression::Conditional(conditional) => {
            if contains(conditional.if_true(), ContainsSymbol::YieldExpression)
                || contains(conditional.if_false(), ContainsSymbol::YieldExpression)
            {
                return Err(StagedYieldRejection::ConditionalOperand);
            }
            plan_staged_generator_expression(conditional.condition(), plan)
        }
        Expression::Optional(optional) => {
            if !optional_chain_is_stageable(optional) {
                return Err(StagedYieldRejection::ConditionalOperand);
            }
            plan_staged_generator_expression(optional.target(), plan)
        }
        Expression::TaggedTemplate(_) => Err(StagedYieldRejection::UnstagedArgumentList),
        Expression::SuperCall(_)
        | Expression::ImportCall(_)
        | Expression::Spread(_)
        | Expression::Await(_)
        | Expression::FormalParameterList(_) => Err(StagedYieldRejection::UnstagedForm),
        // These forms cannot contain a `yield` of this generator: literals and
        // references have no operands, and a nested function's `yield`
        // belongs to that function.
        Expression::This(_)
        | Expression::Identifier(_)
        | Expression::Literal(_)
        | Expression::RegExpLiteral(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunction(_)
        | Expression::AsyncArrowFunction(_)
        | Expression::GeneratorExpression(_)
        | Expression::AsyncFunctionExpression(_)
        | Expression::AsyncGeneratorExpression(_)
        | Expression::NewTarget(_)
        | Expression::ImportMeta(_)
        | Expression::Debugger => Ok(()),
    }
}

/// An optional chain whose only `yield` is in its always-evaluated target and
/// whose links do not call: a call link would need the Reference of the
/// target, not its value.
pub(crate) fn optional_chain_is_stageable(optional: &Optional) -> bool {
    optional.chain().iter().all(|operation| {
        !contains(operation, ContainsSymbol::YieldExpression)
            && !matches!(operation.kind(), OptionalOperationKind::Call { .. })
    })
}

fn plan_staged_generator_operands<'e>(
    operands: impl Iterator<Item = &'e Expression>,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    for operand in operands {
        plan_staged_generator_expression(operand, plan)?;
    }
    Ok(())
}

fn plan_staged_simple_access(
    access: &boa_ast::expression::access::SimplePropertyAccess,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    plan_staged_generator_expression(access.target(), plan)?;
    match access.field() {
        PropertyAccessField::Const(_) => Ok(()),
        PropertyAccessField::Expr(key) => plan_staged_generator_expression(key, plan),
    }
}

fn plan_staged_generator_assignment(
    assignment: &boa_ast::expression::operator::Assign,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    match assignment.op() {
        AssignOp::Assign => match assignment.lhs() {
            AssignTarget::Identifier(_) => plan_staged_generator_expression(assignment.rhs(), plan),
            AssignTarget::Access(PropertyAccess::Simple(access)) => {
                plan_staged_simple_access(access, plan)?;
                plan_staged_generator_expression(assignment.rhs(), plan)
            }
            AssignTarget::Access(PropertyAccess::Private(access)) => {
                plan_staged_generator_expression(access.target(), plan)?;
                plan_staged_generator_expression(assignment.rhs(), plan)
            }
            AssignTarget::Access(PropertyAccess::Super(_)) | AssignTarget::WebCompatCall(_) => {
                Err(StagedYieldRejection::UnstagedReference)
            }
            AssignTarget::Pattern(pattern) => {
                plan_staged_generator_expression(assignment.rhs(), plan)?;
                plan_generator_destructuring_pattern(pattern, plan)
            }
        },
        AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
            Err(StagedYieldRejection::ConditionalOperand)
        }
        AssignOp::Add
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
        | AssignOp::Ushr => match assignment.lhs() {
            AssignTarget::Identifier(_) => plan_staged_generator_expression(assignment.rhs(), plan),
            AssignTarget::Access(_) | AssignTarget::Pattern(_) | AssignTarget::WebCompatCall(_) => {
                Err(StagedYieldRejection::UnstagedReference)
            }
        },
    }
}

/// Object literals stage either through a suspension-owned accumulator (only
/// literal-keyed data properties and spreads) or by retaining each operand and
/// lowering the literal once. The second form cannot retain a spread, whose
/// copy observes the source, or a shorthand property, which has no operand
/// node, that precedes a later `yield`.
fn plan_staged_generator_object_literal(
    object: &ObjectLiteral,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    if !object_literal_stages_through_accumulator(object) {
        let last_suspending = object
            .properties()
            .iter()
            .rposition(|property| contains(property, ContainsSymbol::YieldExpression))
            .expect("a staged object literal contains a yield");
        if object.properties()[..last_suspending]
            .iter()
            .any(|property| {
                matches!(
                    property,
                    PropertyDefinition::SpreadObject(_)
                        | PropertyDefinition::IdentifierReference(_)
                )
            })
        {
            return Err(StagedYieldRejection::ObjectLiteralOperand);
        }
    }
    for property in object.properties() {
        match property {
            PropertyDefinition::Property(name, value) => {
                if let PropertyName::Computed(key) = name {
                    plan_staged_generator_expression(key, plan)?;
                }
                plan_staged_generator_expression(value, plan)?;
            }
            PropertyDefinition::MethodDefinition(method) => {
                if let PropertyName::Computed(key) = method.name() {
                    plan_staged_generator_expression(key, plan)?;
                }
            }
            PropertyDefinition::SpreadObject(source) => {
                plan_staged_generator_expression(source, plan)?;
            }
            PropertyDefinition::IdentifierReference(_) => {}
            PropertyDefinition::CoverInitializedName(..) => {
                return Err(StagedYieldRejection::UnstagedForm);
            }
        }
    }
    Ok(())
}

/// True when every property is a literal-keyed data property other than
/// `__proto__`, or a spread: the accumulator form copies each in order.
pub(crate) fn object_literal_stages_through_accumulator(object: &ObjectLiteral) -> bool {
    object.properties().iter().all(|property| match property {
        PropertyDefinition::SpreadObject(_) => true,
        PropertyDefinition::Property(PropertyName::Literal(name), _) => {
            name.sym() != boa_interner::Sym::__PROTO__
        }
        PropertyDefinition::Property(PropertyName::Computed(_), _)
        | PropertyDefinition::IdentifierReference(_)
        | PropertyDefinition::MethodDefinition(_)
        | PropertyDefinition::CoverInitializedName(..) => false,
    })
}

/// Class heritage and computed element names stage into a class evaluation
/// prefix, which resumes linearly.
pub(crate) fn plan_linear_class_operands(
    heritage: Option<&Expression>,
    elements: &[ClassElement],
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    for operand in class_evaluation_expressions(heritage, elements) {
        let entry_state = *plan.current_state;
        let first_point = plan.suspension_points.len();
        plan_staged_generator_expression(operand, plan)?;
        let points = &plan.suspension_points[first_point..];
        let linear = points.iter().enumerate().all(|(offset, point)| {
            point.suspend_state == entry_state + offset as u32
                && point.resume_state == point.suspend_state + 1
        });
        if !linear || *plan.current_state != entry_state + points.len() as u32 {
            return Err(StagedYieldRejection::ClassOperand);
        }
    }
    Ok(())
}

/// Plan the DestructuringAssignmentEvaluation of `pattern` (13.15.5.2).
///
/// A pattern without a `yield` runs as one `ArrayDestructure` or
/// `ObjectDestructure` and allocates nothing.
fn plan_generator_destructuring_pattern(
    pattern: &Pattern,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    if !contains(pattern, ContainsSymbol::YieldExpression) {
        return Ok(());
    }
    match pattern {
        Pattern::Array(array) => {
            for element in array.bindings() {
                plan_generator_array_pattern_element(element, plan)?;
            }
            // The try, catch and finally blocks each reserve one state.
            plan.reserve();
            plan.reserve();
            plan.reserve();
            Ok(())
        }
        Pattern::Object(object) => {
            for element in object.bindings() {
                plan_generator_object_pattern_element(element, plan)?;
            }
            Ok(())
        }
    }
}

fn plan_generator_array_pattern_element(
    element: &ArrayPatternElement,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    match element {
        ArrayPatternElement::Elision | ArrayPatternElement::SingleNameRest { .. } => Ok(()),
        ArrayPatternElement::SingleName { default_init, .. } => {
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)
        }
        ArrayPatternElement::PropertyAccess {
            access,
            default_init,
        } => {
            plan_generator_destructuring_reference(access, plan)?;
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)
        }
        ArrayPatternElement::PropertyAccessRest { access } => {
            plan_generator_destructuring_reference(access, plan)
        }
        ArrayPatternElement::Pattern {
            pattern,
            default_init,
        } => {
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)?;
            plan_generator_destructuring_pattern(pattern, plan)
        }
        ArrayPatternElement::PatternRest { pattern } => {
            plan_generator_destructuring_pattern(pattern, plan)
        }
    }
}

fn plan_generator_object_pattern_element(
    element: &ObjectPatternElement,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    match element {
        ObjectPatternElement::SingleName {
            name, default_init, ..
        } => {
            plan_generator_destructuring_property_name(name, plan)?;
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)
        }
        ObjectPatternElement::AssignmentPropertyAccess {
            name,
            access,
            default_init,
        } => {
            plan_generator_destructuring_property_name(name, plan)?;
            plan_generator_destructuring_reference(access, plan)?;
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)
        }
        ObjectPatternElement::Pattern {
            name,
            pattern,
            default_init,
        } => {
            plan_generator_destructuring_property_name(name, plan)?;
            plan_generator_destructuring_initializer(default_init.as_ref(), plan)?;
            plan_generator_destructuring_pattern(pattern, plan)
        }
        ObjectPatternElement::RestProperty { .. }
        | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => {
            Err(StagedYieldRejection::DestructuringObjectRest)
        }
    }
}

fn plan_generator_destructuring_property_name(
    name: &PropertyName,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    match name {
        PropertyName::Literal(_) => Ok(()),
        PropertyName::Computed(key) => plan_staged_generator_expression(key, plan),
    }
}

fn plan_generator_destructuring_reference(
    access: &PropertyAccess,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    match access {
        PropertyAccess::Simple(access) => plan_staged_simple_access(access, plan),
        PropertyAccess::Private(access) => plan_staged_generator_expression(access.target(), plan),
        PropertyAccess::Super(_) => Err(StagedYieldRejection::UnstagedReference),
    }
}

/// A suspending Initializer runs only when the value is undefined, so it is a
/// one-branch `GeneratorIf` around its single plain `yield`.
fn plan_generator_destructuring_initializer(
    initializer: Option<&Expression>,
    plan: &mut StagedSuspensionPlan<'_>,
) -> Result<(), StagedYieldRejection> {
    let Some(initializer) = initializer else {
        return Ok(());
    };
    if !contains(initializer, ContainsSymbol::YieldExpression) {
        return Ok(());
    }
    if !destructuring_initializer_has_one_plain_suspension(initializer) {
        return Err(StagedYieldRejection::DestructuringInitializer);
    }
    let mut scratch_state = 0;
    let mut scratch_points = Vec::new();
    plan_staged_generator_expression(
        initializer,
        &mut StagedSuspensionPlan::new(&mut scratch_state, &mut scratch_points),
    )?;
    if scratch_points.len() != 1 || scratch_state != 1 {
        return Err(StagedYieldRejection::DestructuringInitializer);
    }
    plan.conditional_suspend();
    Ok(())
}

/// The Initializer contains exactly one `yield` of this generator, it is not
/// `yield*`, and it is not inside a class operand (which suspends through a
/// class evaluation prefix rather than a plain yield statement).
fn destructuring_initializer_has_one_plain_suspension(initializer: &Expression) -> bool {
    #[derive(Default)]
    struct YieldCensus {
        plain: usize,
        structured: usize,
    }
    impl<'ast> Visitor<'ast> for YieldCensus {
        type BreakTy = ();

        fn visit_yield(
            &mut self,
            yield_expression: &'ast boa_ast::expression::Yield,
        ) -> ControlFlow<Self::BreakTy> {
            if yield_expression.delegate() {
                self.structured += 1;
            } else {
                self.plain += 1;
            }
            match yield_expression.target() {
                Some(target) => self.visit_expression(target),
                None => ControlFlow::Continue(()),
            }
        }

        // A nested function's `yield` belongs to that function.
        fn visit_function_expression(
            &mut self,
            _: &'ast FunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_generator_expression(
            &mut self,
            _: &'ast GeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_function_expression(
            &mut self,
            _: &'ast AsyncFunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_generator_expression(
            &mut self,
            _: &'ast AsyncGeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_arrow_function(&mut self, _: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_async_arrow_function(
            &mut self,
            _: &'ast AsyncArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }

        fn visit_object_method_definition(
            &mut self,
            method: &'ast ObjectMethodDefinition,
        ) -> ControlFlow<Self::BreakTy> {
            method.name().visit_with(self)
        }

        fn visit_class_expression(
            &mut self,
            class: &'ast ClassExpression,
        ) -> ControlFlow<Self::BreakTy> {
            if contains(class, ContainsSymbol::YieldExpression) {
                self.structured += 1;
            }
            ControlFlow::Continue(())
        }
    }
    let mut census = YieldCensus::default();
    let _ = initializer.visit_with(&mut census);
    census.plain == 1 && census.structured == 0
}

fn unwrap_parenthesized(mut expression: &Expression) -> &Expression {
    while let Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression();
    }
    expression
}
