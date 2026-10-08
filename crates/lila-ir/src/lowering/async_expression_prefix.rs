use super::async_generator_assignment::RetainedAssignmentReference;
use super::*;
use boa_ast::expression::operator::Conditional;

/// Only an admitted current-activation source may arm an ordinary await prefix.
/// Nested function bodies remain owned by their own lowering. Conditional and
/// logical/property branches need the plain async If dispatcher; linear loop prefixes and
/// generator protocols retain their existing admission boundary.
pub(super) struct CheckedAsyncPrefixSource<'ast> {
    source: &'ast Expression,
}

impl<'ast> CheckedAsyncPrefixSource<'ast> {
    pub(super) fn new(source: &'ast Expression, lowerer: &ScriptLowerer<'_>) -> Option<Self> {
        (lowerer.current_async_resume_state.is_some()
            && synchronous_resource_loop::source_expression_suspends(source)
            && contains(source, ContainsSymbol::AwaitExpression)
            && !contains(source, ContainsSymbol::YieldExpression)
            && !lowerer.has_branch_sensitive_await(source))
        .then_some(Self { source })
    }

    pub(super) fn lower(self, lowerer: &mut ScriptLowerer<'_>) -> (Vec<StatementIr>, TypedExpr) {
        let entry = lowerer
            .current_async_resume_state
            .expect("admitted async source");
        let saved = lowerer.async_expression_prefix.replace(Vec::new());
        let value = lowerer.lower_expression(self.source);
        let prefix = std::mem::replace(&mut lowerer.async_expression_prefix, saved)
            .expect("admitted async source retains its prefix");
        if lowerer.plain_async_entry_state().is_some()
            && crate::async_switch::sequence_exit(&prefix, entry).ok()
                != lowerer.plain_async_entry_state()
        {
            lowerer.unsupported("async expression prefix continuation ownership");
        }
        (prefix, value)
    }

    pub(super) fn lower_head<'lower>(
        self,
        lowerer: &mut ScriptLowerer<'lower>,
        lower: impl FnOnce(&mut ScriptLowerer<'lower>) -> (StatementIr, ValueKind),
    ) -> (StatementIr, ValueKind) {
        let saved = lowerer.async_expression_prefix.replace(Vec::new());
        let (statement, kind) = lower(lowerer);
        let mut prefix = std::mem::replace(&mut lowerer.async_expression_prefix, saved)
            .expect("admitted async head retains its prefix");
        if prefix.is_empty() {
            return (statement, kind);
        }
        prefix.push(statement);
        (StatementIr::LexicalBlock(prefix), kind)
    }
}

/// The closed source domain retains the original Reference after its complete
/// left operands, before only the selected RHS is reached.
pub(super) struct AwaitedLogicalAssignmentSource<'ast> {
    target: &'ast AssignTarget,
    op: LogicalOp,
    rhs: &'ast Expression,
}

impl<'ast> AwaitedLogicalAssignmentSource<'ast> {
    pub(super) fn new(
        lowerer: &ScriptLowerer<'_>,
        op: AssignOp,
        lhs: &'ast AssignTarget,
        rhs: &'ast Expression,
    ) -> Option<Self> {
        if lowerer.plain_async_entry_state().is_none()
            || (lowerer.loop_depth != 0 && !lowerer.has_checked_async_source_value_branch_owner())
            || !(contains(lhs, ContainsSymbol::AwaitExpression)
                || contains(rhs, ContainsSymbol::AwaitExpression))
            || contains(lhs, ContainsSymbol::YieldExpression)
            || contains(rhs, ContainsSymbol::YieldExpression)
        {
            return None;
        }
        let op = match op {
            AssignOp::BoolAnd => LogicalOp::And,
            AssignOp::BoolOr => LogicalOp::Or,
            AssignOp::Coalesce => LogicalOp::Coalesce,
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
            | AssignOp::Ushr => return None,
        };
        match lhs {
            AssignTarget::Identifier(_) | AssignTarget::Access(_) => Some(Self {
                target: lhs,
                op,
                rhs,
            }),
            AssignTarget::Pattern(_) | AssignTarget::WebCompatCall(_) => None,
        }
    }
}

/// Context admission belongs to the consumed tail. Property-only chains keep
/// their checked value contexts; a Call also requires an actual source owner
/// when it appears inside a loop or its restartable condition.
#[must_use]
pub(super) struct CheckedAwaitedOptionalChainSource<'ast> {
    chain: AwaitedOptionalChainSource<'ast>,
}

impl<'ast> CheckedAwaitedOptionalChainSource<'ast> {
    pub(super) fn new(lowerer: &ScriptLowerer<'_>, source: &'ast Optional) -> Option<Self> {
        if contains(source, ContainsSymbol::YieldExpression) {
            return None;
        }
        Self::admit(lowerer, AwaitedOptionalChainSource::new(source)?)
    }

    pub(super) fn new_delete(lowerer: &ScriptLowerer<'_>, source: &'ast Optional) -> Option<Self> {
        if contains(source, ContainsSymbol::YieldExpression) {
            return None;
        }
        Self::admit(lowerer, AwaitedOptionalChainSource::for_delete(source)?)
    }

    fn admit(lowerer: &ScriptLowerer<'_>, chain: AwaitedOptionalChainSource<'ast>) -> Option<Self> {
        (lowerer.has_plain_async_value_branch_owner()
            && (!chain.has_calls()
                || lowerer.loop_depth == 0
                || lowerer.has_checked_async_source_value_branch_owner()))
        .then_some(Self { chain })
    }

    fn into_parts(self) -> (&'ast Expression, AwaitedOptionalChainTail<'ast>) {
        self.chain.into_parts()
    }
}

/// A grouped property has a surviving Reference; a terminal Call has consumed
/// it and supplies only a Value. Private fields couple each admitted tail to
/// its actual source disposition before any continuation state is allocated.
#[must_use]
pub(super) enum AwaitedGroupedOptionalInvocationSource<'ast> {
    PropertyReference(AwaitedGroupedOptionalReferenceSource<'ast>),
    CallValue(AwaitedGroupedOptionalCallValueSource<'ast>),
}

pub(super) struct AwaitedGroupedOptionalReferenceSource<'ast> {
    chain: CheckedAwaitedOptionalChainSource<'ast>,
}

pub(super) struct AwaitedGroupedOptionalCallValueSource<'ast> {
    chain: CheckedAwaitedOptionalChainSource<'ast>,
}

impl<'ast> AwaitedGroupedOptionalInvocationSource<'ast> {
    pub(super) fn new(lowerer: &ScriptLowerer<'_>, source: &'ast Expression) -> Option<Self> {
        if lowerer.plain_async_entry_state().is_none()
            || (lowerer.loop_depth != 0 && !lowerer.has_checked_async_source_value_branch_owner())
            || contains(source, ContainsSymbol::YieldExpression)
        {
            return None;
        }
        let (chain, terminal) =
            AwaitedOptionalChainSource::from_grouped_invocation(source)?.into_parts();
        let chain = CheckedAwaitedOptionalChainSource::admit(lowerer, chain)?;
        Some(match terminal {
            AwaitedOptionalChainTerminal::PropertyReference => {
                Self::PropertyReference(AwaitedGroupedOptionalReferenceSource { chain })
            }
            AwaitedOptionalChainTerminal::CallValue => {
                Self::CallValue(AwaitedGroupedOptionalCallValueSource { chain })
            }
        })
    }
}

impl AwaitedGroupedOptionalCallValueSource<'_> {
    pub(super) fn lower(self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        // The full-tail consumer also retains a first-Call target Reference
        // when only the target suspends; a generic value fallback cannot do so.
        lowerer.lower_awaited_optional_chain_value(self.chain)
    }
}

/// This narrow walk is required by the shared prefix admission before it can
/// allocate any state. Nested activation bodies do not inherit the enclosing
/// binding eligibility; current class keys, heritage and static evaluation do.
pub(super) fn awaited_reference_operands_are_owned(
    source: &Expression,
    lowerer: &ScriptLowerer<'_>,
) -> bool {
    struct References<'borrow, 'lower> {
        lowerer: &'borrow ScriptLowerer<'lower>,
    }
    impl<'ast> Visitor<'ast> for References<'_, '_> {
        type BreakTy = ();
        fn visit_expression(&mut self, source: &'ast Expression) -> ControlFlow<()> {
            if matches!(source, Expression::SuperCall(_))
                && contains(source, ContainsSymbol::AwaitExpression)
                && !self.lowerer.owns_derived_super_call()
            {
                return ControlFlow::Break(());
            }
            if let Expression::Optional(optional) = source {
                let suspended_call_tail = optional
                    .chain()
                    .iter()
                    .any(|link| contains(link, ContainsSymbol::AwaitExpression))
                    && optional
                        .chain()
                        .iter()
                        .any(|link| matches!(link.kind(), OptionalOperationKind::Call { .. }));
                if suspended_call_tail
                    && CheckedAwaitedOptionalChainSource::new(self.lowerer, optional).is_none()
                {
                    return ControlFlow::Break(());
                }
            }
            let reference = match source {
                Expression::Call(call) => Some(call.function()),
                Expression::TaggedTemplate(template) => Some(template.tag()),
                _ => None,
            };
            if let Some(reference) = reference {
                if awaited_optional_reference(reference)
                    && AwaitedGroupedOptionalInvocationSource::new(self.lowerer, reference)
                        .is_none()
                {
                    return ControlFlow::Break(());
                }
            }
            source.visit_with(self)
        }
        fn visit_assign(
            &mut self,
            source: &'ast boa_ast::expression::operator::assign::Assign,
        ) -> ControlFlow<()> {
            if matches!(
                source.op(),
                AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce
            ) && (contains(source.lhs(), ContainsSymbol::AwaitExpression)
                || contains(source.rhs(), ContainsSymbol::AwaitExpression))
                && AwaitedLogicalAssignmentSource::new(
                    self.lowerer,
                    source.op(),
                    source.lhs(),
                    source.rhs(),
                )
                .is_none()
            {
                return ControlFlow::Break(());
            }
            source.visit_with(self)
        }
        fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
        fn visit_formal_parameter_list(&mut self, _: &'ast FormalParameterList) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
        fn visit_class_element(&mut self, element: &'ast ClassElement) -> ControlFlow<()> {
            match element {
                ClassElement::FieldDefinition(field)
                | ClassElement::AccessorFieldDefinition(field) => {
                    for decorator in field.decorators() {
                        self.visit_expression(decorator)?;
                    }
                    self.visit_property_name(field.name())
                }
                ClassElement::PrivateFieldDefinition(field) => {
                    for decorator in field.decorators() {
                        self.visit_expression(decorator)?;
                    }
                    ControlFlow::Continue(())
                }
                ClassElement::StaticBlock(block) => {
                    self.visit_statement_list(block.statements().statement_list())
                }
                ClassElement::MethodDefinition(_)
                | ClassElement::StaticFieldDefinition(_)
                | ClassElement::StaticAccessorFieldDefinition(_)
                | ClassElement::PrivateStaticFieldDefinition(_) => element.visit_with(self),
            }
        }
    }
    source.visit_with(&mut References { lowerer }).is_continue()
}

/// A write owns exactly the Reference used by the completed Get. No selected
/// arm can resolve the identifier again or rebuild a raw property Reference.
struct AwaitedLogicalAssignmentWrite(RetainedAssignmentReference);

impl AwaitedLogicalAssignmentWrite {
    fn lower(self, lowerer: &mut ScriptLowerer<'_>, source: &Expression) -> TypedExpr {
        let accounting = lowerer.prepare_potentially_effectful_expression(source);
        let epoch = lowerer.intervening_effect_epoch;
        let rhs = lowerer.lower_expression(source);
        // A suspension admits unrelated jobs; retained Reference identity does
        // not retain pre-suspension object contents or accessor facts.
        let intervening = contains(source, ContainsSymbol::AwaitExpression)
            || accounting.intervening_effects_observed(epoch, lowerer.intervening_effect_epoch);
        if intervening {
            lowerer.observe_all_planned_source_as_unknown_property_hooks();
            lowerer.invalidate_unknown_user_code_effects();
        }
        self.0.put_value(lowerer, rhs)
    }
}

/// A branch's residual value is still unevaluated. Its prefix and state range
/// are inseparable after construction, and only the result owner can consume it.
struct CompletedConditionalArm {
    prefix: Vec<StatementIr>,
    value: TypedExpr,
    entry: u32,
    ready: u32,
}

impl CompletedConditionalArm {
    fn lower(
        lowerer: &mut ScriptLowerer<'_>,
        source: ConditionalValueArmSource<'_>,
        entry: u32,
    ) -> Result<Self, &'static str> {
        lowerer.current_async_resume_state = Some(entry);
        let saved = lowerer.async_expression_prefix.replace(Vec::new());
        // Arm temporaries are activation-owned, but are not source bindings in
        // the outer fact domain. Pop this private lowering scope before the
        // outer join: arms may allocate different numbers and names of slots.
        lowerer.push_scope();
        let value = source.lower_value(lowerer);
        lowerer.pop_scope();
        let prefix = std::mem::replace(&mut lowerer.async_expression_prefix, saved)
            .expect("conditional arm owns its prefix");
        let ready = lowerer
            .plain_async_entry_state()
            .ok_or("conditional arm activation")?;
        if crate::async_switch::sequence_exit(&prefix, entry).ok() != Some(ready) {
            return Err("conditional arm continuation ownership");
        }
        Ok(Self {
            prefix,
            value,
            entry,
            ready,
        })
    }
}

/// Constructing this owner evaluates the actual left source and retains its
/// GetValue once. Only this owner can create its condition and skipped-arm read.
struct EvaluatedLogicalLeft {
    name: String,
}

impl EvaluatedLogicalLeft {
    fn lower(lowerer: &mut ScriptLowerer<'_>, source: &Expression) -> Self {
        let value = lowerer.lower_expression(source);
        Self::retain(lowerer, value)
    }

    fn retain(lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> Self {
        let name =
            lowerer.alloc_suspension_owned_binding("async.logical.left.", value.value_info());
        lowerer
            .async_expression_prefix
            .as_mut()
            .expect("logical prefix")
            .push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: name.clone(),
                init: value,
            });
        Self { name }
    }

    fn condition(&self, lowerer: &mut ScriptLowerer<'_>, op: LogicalOp) -> TypedExpr {
        let left = lowerer.lower_identifier_name(self.name.clone(), false);
        match op {
            LogicalOp::And | LogicalOp::Or => left,
            LogicalOp::Coalesce => strict_nullish_condition(left),
        }
    }

    fn into_value(self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        lowerer.lower_identifier_name(self.name, false)
    }
}

/// A branch either evaluates its source inside a checked private arm scope, or
/// publishes the left value whose original GetValue has already completed.
enum ConditionalValueArmSource<'ast> {
    Expression(&'ast Expression),
    RetainedLeft(EvaluatedLogicalLeft),
    ReleasedIdentifierLeft {
        left: EvaluatedLogicalLeft,
        release: (String, EnvironmentIdentifierOperationIr),
    },
    Undefined,
    BooleanTrue,
    LogicalAssignment {
        write: AwaitedLogicalAssignmentWrite,
        rhs: &'ast Expression,
    },
    OptionalChain {
        base: EvaluatedOptionalChainBase,
        first: AwaitedOptionalChainLink<'ast>,
        rest: AwaitedOptionalChainTail<'ast>,
        result: OptionalChainTailResult,
    },
}

impl ConditionalValueArmSource<'_> {
    fn lower(
        self,
        lowerer: &mut ScriptLowerer<'_>,
        entry: u32,
    ) -> Result<CompletedConditionalArm, &'static str> {
        CompletedConditionalArm::lower(lowerer, self, entry)
    }

    fn lower_value(self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        match self {
            Self::Expression(source) => lowerer.lower_expression(source),
            Self::RetainedLeft(left) => left.into_value(lowerer),
            Self::ReleasedIdentifierLeft { left, release } => {
                let release = lowerer.environment_identifier(release.0, release.1);
                let value = left.into_value(lowerer);
                TypedExpr::from_info(
                    value.value_info(),
                    ExprIr::Comma {
                        lhs: Box::new(release),
                        rhs: Box::new(value),
                    },
                )
            }
            Self::Undefined => TypedExpr::undefined(),
            Self::BooleanTrue => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true))
            }
            Self::LogicalAssignment { write, rhs } => write.lower(lowerer, rhs),
            Self::OptionalChain {
                base,
                first,
                rest,
                result,
            } => lowerer.lower_selected_optional_chain_tail(base, first, rest, result),
        }
    }
}

/// Only the terminal read consumes the Reference destination. Earlier reads
/// supply retained values, so their receivers cannot replace the final one.
enum OptionalChainTailResult {
    Value,
    DeleteProperty,
    CallReference(CapturedCallReceiverIr),
}

impl OptionalChainTailResult {
    fn complete(self, chain_expression: TypedExpr) -> TypedExpr {
        match self {
            Self::Value => chain_expression,
            Self::DeleteProperty => {
                unreachable!("Delete consumes the terminal Reference before GetValue")
            }
            Self::CallReference(receiver) => {
                let info = chain_expression.value_info();
                let TypedExpr {
                    expr: ExprIr::OptionalPropertyChain { target, chain },
                    ..
                } = chain_expression
                else {
                    unreachable!("checked terminal property owner emits one actual chain")
                };
                let capture =
                    OptionalCallReferenceCaptureIr::new(info.clone(), target, chain, receiver);
                TypedExpr::from_info(info, ExprIr::CaptureOptionalCallReference(capture))
            }
        }
    }
}

fn strict_nullish_condition(saved: TypedExpr) -> TypedExpr {
    let equal = |rhs| {
        TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::StrictEquality {
                op: EqualityBinaryOp::StrictEqual,
                lhs: Box::new(saved.clone()),
                rhs: Box::new(rhs),
            },
        )
    };
    TypedExpr::from_info(
        ValueInfo::new(ValueKind::Boolean),
        ExprIr::LogicalShortCircuit {
            op: LogicalBinaryOp::Or,
            lhs: Box::new(equal(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Null),
                ExprIr::Null,
            ))),
            rhs: Box::new(equal(TypedExpr::undefined())),
        },
    )
}

/// This owner retains the actual base's completed GetValue. A later key can
/// mutate its source binding without changing the base consumed by the read.
struct EvaluatedOptionalBase {
    name: String,
}

impl EvaluatedOptionalBase {
    fn from_source(lowerer: &mut ScriptLowerer<'_>, source: &Expression) -> Self {
        let value = lowerer.lower_property_target(source);
        Self::retain(lowerer, value)
    }

    fn retain(lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> Self {
        let name =
            lowerer.alloc_suspension_owned_binding("async.optional.base.", value.value_info());
        lowerer
            .async_expression_prefix
            .as_mut()
            .expect("optional prefix")
            .push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: name.clone(),
                init: value,
            });
        Self { name }
    }

    fn nullish_condition(&self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        strict_nullish_condition(lowerer.lower_identifier_name(self.name.clone(), false))
    }

    fn into_value(self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        lowerer.lower_identifier_name(self.name, false)
    }
}

/// A selected property consumes the retained base and the checked source link.
/// Its GetValue must complete before a later suspending link can retain it.
pub(super) struct CompletedOptionalPropertyRead {
    value: TypedExpr,
}

impl CompletedOptionalPropertyRead {
    pub(super) fn from_evaluated_private(
        lowerer: &mut ScriptLowerer<'_>,
        target: TypedExpr,
        private_name_id: PrivateNameId,
    ) -> Self {
        let mut analysis = OptionalChainAnalysisState::from_target(&target);
        lowerer.analyze_optional_chain_private_property(&mut analysis, private_name_id, false);
        let (result, effects) = lowerer.finish_optional_chain_analysis(analysis);
        let value = TypedExpr::from_info(
            result,
            ExprIr::OptionalPropertyChain {
                target: Box::new(target),
                chain: vec![OptionalChainOperationIr::PrivateProperty {
                    private_name_id,
                    shorted: false,
                }],
            },
        );
        Self {
            value: effects.attach_to_emitted_call(value),
        }
    }

    fn lower(
        lowerer: &mut ScriptLowerer<'_>,
        base: EvaluatedOptionalBase,
        link: AwaitedOptionalPropertyLink<'_>,
    ) -> Option<Self> {
        let mut target = base.into_value(lowerer);
        let (field, _) = link.into_parts();
        let epoch = lowerer.intervening_effect_epoch;
        let key = lowerer.lower_optional_chain_property_key(field)?;
        if lowerer.intervening_effect_epoch != epoch {
            target.heap_shape = None;
        }
        Some(Self::from_evaluated(lowerer, target, key))
    }

    pub(super) fn from_evaluated(
        lowerer: &mut ScriptLowerer<'_>,
        target: TypedExpr,
        key: PropertyKeyIr,
    ) -> Self {
        let mut analysis = OptionalChainAnalysisState::from_target(&target);
        lowerer.analyze_optional_chain_property(&mut analysis, &key, false);
        let (result, effects) = lowerer.finish_optional_chain_analysis(analysis);
        let value = TypedExpr::from_info(
            result,
            ExprIr::OptionalPropertyChain {
                target: Box::new(target),
                chain: vec![OptionalChainOperationIr::Property {
                    key,
                    shorted: false,
                }],
            },
        );
        Self {
            value: effects.attach_to_emitted_call(value),
        }
    }

    pub(super) fn into_value(self) -> TypedExpr {
        self.value
    }
}

/// The next checked link chooses whether the retained operand is a Value or
/// an evaluated Call Reference. Receivers remain private to the invocation owner.
enum EvaluatedOptionalChainBase {
    Value(EvaluatedOptionalBase),
    Call(super::suspended_call::EvaluatedCallReference),
}

impl EvaluatedOptionalChainBase {
    fn from_source(
        lowerer: &mut ScriptLowerer<'_>,
        source: &Expression,
        tail: &AwaitedOptionalChainTail<'_>,
    ) -> Option<Self> {
        if tail.starts_with_call() {
            lowerer
                .capture_optional_chain_target_reference(
                    source,
                    super::suspended_call::InvocationSuspension::Await,
                )
                .map(Self::Call)
        } else {
            Some(Self::Value(EvaluatedOptionalBase::from_source(
                lowerer, source,
            )))
        }
    }

    fn from_read(
        lowerer: &mut ScriptLowerer<'_>,
        read: CompletedOptionalPropertyRead,
        tail: &AwaitedOptionalChainTail<'_>,
    ) -> Self {
        if tail.starts_with_call() {
            Self::Call(lowerer.capture_optional_chain_property_reference(read))
        } else {
            Self::Value(EvaluatedOptionalBase::retain(lowerer, read.into_value()))
        }
    }

    fn from_call_result(
        lowerer: &mut ScriptLowerer<'_>,
        value: TypedExpr,
        tail: &AwaitedOptionalChainTail<'_>,
    ) -> Self {
        if tail.starts_with_call() {
            Self::Call(lowerer.capture_optional_chain_call_result(value))
        } else {
            Self::Value(EvaluatedOptionalBase::retain(lowerer, value))
        }
    }

    fn nullish_condition(&self, lowerer: &mut ScriptLowerer<'_>) -> TypedExpr {
        match self {
            Self::Value(base) => base.nullish_condition(lowerer),
            Self::Call(reference) => strict_nullish_condition(reference.callee_value()),
        }
    }
}

/// Allocating the result here owns its declaration, both final writes and the
/// resumed read. Callers cannot supply independent binding names for those uses.
struct ConditionalAwaitResult {
    name: String,
}

impl ConditionalAwaitResult {
    fn allocate(lowerer: &mut ScriptLowerer<'_>) -> Self {
        let name = lowerer.alloc_suspension_owned_binding(
            "async.conditional.result.",
            unknown_runtime_value_info(),
        );
        lowerer
            .async_expression_prefix
            .as_mut()
            .expect("conditional prefix")
            .push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: name.clone(),
                init: TypedExpr::undefined(),
            });
        Self { name }
    }

    fn finish(
        self,
        lowerer: &mut ScriptLowerer<'_>,
        condition: TypedExpr,
        entry: u32,
        then_arm: CompletedConditionalArm,
        else_arm: CompletedConditionalArm,
    ) -> Result<TypedExpr, &'static str> {
        let plan = AsyncFunctionIfPlanIr::new(entry, then_arm.ready, else_arm.ready)
            .map_err(|_| "conditional value state order/overflow")?;
        let then_entry = entry
            .checked_add(1)
            .ok_or("conditional value state overflow")?;
        let else_entry = then_arm
            .ready
            .checked_add(1)
            .ok_or("conditional value state overflow")?;
        if then_arm.entry != then_entry || else_arm.entry != else_entry {
            return Err("conditional value branch association");
        }
        let info =
            lowerer.merge_value_infos(then_arm.value.value_info(), else_arm.value.value_info());
        let mut write_arm = |arm: CompletedConditionalArm| {
            let mut prefix = arm.prefix;
            prefix.push(StatementIr::Expression(
                lowerer.lower_identifier_assign_value(self.name.clone(), arm.value),
            ));
            Box::new(StatementIr::LexicalBlock(prefix))
        };
        let then_branch = write_arm(then_arm);
        let else_branch = Some(write_arm(else_arm));
        lowerer.set_binding_value_info(&self.name, info);
        let (statement, exit) = match plan {
            Some(plan) => (
                StatementIr::AsyncFunctionIf {
                    condition,
                    then_branch,
                    else_branch,
                    plan,
                },
                plan.exit_state(),
            ),
            None => (
                StatementIr::If {
                    condition,
                    then_branch,
                    else_branch,
                },
                entry,
            ),
        };
        lowerer
            .async_expression_prefix
            .as_mut()
            .expect("conditional prefix")
            .push(statement);
        lowerer.current_async_resume_state = Some(exit);
        Ok(lowerer.lower_identifier_name(self.name, false))
    }
}

impl ScriptLowerer<'_> {
    /// These References complete their original acquisition before a later
    /// operand suspends; generic operand pinning cannot replace it.
    pub(super) fn lower_async_reference_expression(
        &mut self,
        source: &Expression,
    ) -> Option<TypedExpr> {
        self.plain_async_entry_state()?;
        if self.async_expression_prefix.is_none()
            || !contains(source, ContainsSymbol::AwaitExpression)
        {
            return None;
        }
        let result = match source {
            Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                self.lower_resumable_private_read(access)
            }
            Expression::PropertyAccess(PropertyAccess::Super(_))
            | Expression::BinaryInPrivate(_) => self.lower_staged_generator_special_read(source),
            Expression::Update(source) => self.lower_resumable_update(source),
            Expression::ImportCall(source) => self.lower_resumable_import_call(source),
            Expression::Unary(source) if source.op() == UnaryOp::Delete => {
                self.lower_generator_delete_reference(source.target())
            }
            _ => return None,
        };
        let Some((prefix, value)) = result else {
            return Some(self.unsupported_expr("async Reference operand continuation ownership"));
        };
        self.async_expression_prefix
            .as_mut()
            .expect("async Reference owns its prefix")
            .extend(prefix);
        Some(value)
    }

    pub(super) fn lower_conditional_await_value(&mut self, source: &Conditional) -> TypedExpr {
        let result = ConditionalAwaitResult::allocate(self);
        // Condition lowering appends its own prefix before branch entry. The
        // existing If owner evaluates the residual test exactly once.
        let condition = self.lower_expression(source.condition());
        self.lower_await_value_branches(
            result,
            condition,
            ConditionalValueArmSource::Expression(source.if_true()),
            ConditionalValueArmSource::Expression(source.if_false()),
        )
    }

    pub(super) fn lower_logical_await_value(
        &mut self,
        op: LogicalOp,
        lhs: &Expression,
        rhs: &Expression,
    ) -> TypedExpr {
        let result = ConditionalAwaitResult::allocate(self);
        let left = EvaluatedLogicalLeft::lower(self, lhs);
        let condition = left.condition(self, op);
        let (then_source, else_source) = match op {
            LogicalOp::And | LogicalOp::Coalesce => (
                ConditionalValueArmSource::Expression(rhs),
                ConditionalValueArmSource::RetainedLeft(left),
            ),
            LogicalOp::Or => (
                ConditionalValueArmSource::RetainedLeft(left),
                ConditionalValueArmSource::Expression(rhs),
            ),
        };
        self.lower_await_value_branches(result, condition, then_source, else_source)
    }

    pub(super) fn lower_logical_assignment_await_value(
        &mut self,
        source: AwaitedLogicalAssignmentSource<'_>,
    ) -> TypedExpr {
        let AwaitedLogicalAssignmentSource { target, op, rhs } = source;
        let mut capture = Vec::new();
        let Some((reference, read)) =
            self.capture_resumable_assignment_reference(target, &mut capture)
        else {
            return self.unsupported_expr("awaited logical assignment Reference capture");
        };
        self.async_expression_prefix
            .as_mut()
            .expect("logical assignment prefix")
            .extend(capture);
        let left = EvaluatedLogicalLeft::retain(self, read);
        let result = ConditionalAwaitResult::allocate(self);
        let condition = left.condition(self, op);
        let skipped = match reference.release_operation() {
            Some(release) => ConditionalValueArmSource::ReleasedIdentifierLeft { left, release },
            None => ConditionalValueArmSource::RetainedLeft(left),
        };
        let selected = ConditionalValueArmSource::LogicalAssignment {
            write: AwaitedLogicalAssignmentWrite(reference),
            rhs,
        };
        let (then_source, else_source) = match op {
            LogicalOp::And | LogicalOp::Coalesce => (selected, skipped),
            LogicalOp::Or => (skipped, selected),
        };
        self.lower_await_value_branches(result, condition, then_source, else_source)
    }

    pub(super) fn lower_awaited_optional_chain_value(
        &mut self,
        source: CheckedAwaitedOptionalChainSource<'_>,
    ) -> TypedExpr {
        let (target, tail) = source.into_parts();
        let Some(base) = EvaluatedOptionalChainBase::from_source(self, target, &tail) else {
            return self.unsupported_expr("awaited optional target Reference");
        };
        self.lower_optional_chain_tail(base, tail, OptionalChainTailResult::Value)
    }

    pub(super) fn lower_awaited_grouped_optional_reference(
        &mut self,
        source: AwaitedGroupedOptionalReferenceSource<'_>,
        receiver: CapturedCallReceiverIr,
    ) -> TypedExpr {
        let (target, tail) = source.chain.into_parts();
        let Some(base) = EvaluatedOptionalChainBase::from_source(self, target, &tail) else {
            return self.unsupported_expr("awaited grouped optional target Reference");
        };
        self.lower_optional_chain_tail(base, tail, OptionalChainTailResult::CallReference(receiver))
    }

    pub(super) fn lower_awaited_optional_chain_delete(
        &mut self,
        source: CheckedAwaitedOptionalChainSource<'_>,
    ) -> TypedExpr {
        let (target, tail) = source.into_parts();
        let Some(base) = EvaluatedOptionalChainBase::from_source(self, target, &tail) else {
            return self.unsupported_expr("awaited optional target Reference");
        };
        self.lower_optional_chain_tail(base, tail, OptionalChainTailResult::DeleteProperty)
    }

    fn lower_optional_chain_tail(
        &mut self,
        base: EvaluatedOptionalChainBase,
        tail: AwaitedOptionalChainTail<'_>,
        result: OptionalChainTailResult,
    ) -> TypedExpr {
        if !tail.suspends() && !tail.has_calls() {
            let EvaluatedOptionalChainBase::Value(base) = base else {
                unreachable!("a property tail owns a Value base");
            };
            let target = base.into_value(self);
            let mut analysis = OptionalChainAnalysisState::from_target(&target);
            let mut chain = Vec::new();
            let mut tail = tail;
            while let Some((link, rest)) = tail.next() {
                match link {
                    AwaitedOptionalChainLink::Property(link) => {
                        let (field, shorted) = link.into_parts();
                        let epoch = self.intervening_effect_epoch;
                        let Some(key) = self.lower_optional_chain_property_key(field) else {
                            return self
                                .unsupported_expr("unsupported optional computed property key");
                        };
                        if self.intervening_effect_epoch != epoch {
                            analysis.current.heap_shape = None;
                        }
                        if rest.is_empty()
                            && matches!(result, OptionalChainTailResult::DeleteProperty)
                        {
                            self.invalidate_unknown_user_code_effects();
                            analysis.current = ValueInfo::new(ValueKind::Boolean);
                            analysis.property_receiver = None;
                        } else {
                            self.analyze_optional_chain_property(&mut analysis, &key, shorted);
                        }
                        chain.push(OptionalChainOperationIr::Property { key, shorted });
                    }
                    AwaitedOptionalChainLink::Private { field, shorted } => {
                        let Some(private_name_id) = self.current_private_name_id(field) else {
                            return self.unsupported_expr("private class element");
                        };
                        self.analyze_optional_chain_private_property(
                            &mut analysis,
                            private_name_id,
                            shorted,
                        );
                        chain.push(OptionalChainOperationIr::PrivateProperty {
                            private_name_id,
                            shorted,
                        });
                    }
                    AwaitedOptionalChainLink::Call(_) => {
                        unreachable!("checked property-only residual tail")
                    }
                }
                tail = rest;
            }
            if chain.is_empty() {
                if matches!(result, OptionalChainTailResult::CallReference(_)) {
                    unreachable!("checked Reference tail retains its terminal property");
                }
                return target;
            }
            let (value_info, effects) = self.finish_optional_chain_analysis(analysis);
            if matches!(result, OptionalChainTailResult::DeleteProperty) {
                let Ok(delete) =
                    DeleteOptionalPropertyChainIr::new(target, chain, self.reference_strictness())
                else {
                    unreachable!("checked Delete ends in ordinary property");
                };
                return effects.attach_to_emitted_call(TypedExpr::from_info(
                    value_info,
                    ExprIr::DeleteOptionalPropertyChain(Box::new(delete)),
                ));
            }
            let value = effects.attach_to_emitted_call(TypedExpr::from_info(
                value_info,
                ExprIr::OptionalPropertyChain {
                    target: Box::new(target),
                    chain,
                },
            ));
            return result.complete(value);
        }
        let Some((first, rest)) = tail.next() else {
            unreachable!("checked optional tail owns a link");
        };
        if !first.shorted() {
            return self.lower_selected_optional_chain_tail(base, first, rest, result);
        }
        let value_result = ConditionalAwaitResult::allocate(self);
        let condition = base.nullish_condition(self);
        let skipped = if matches!(result, OptionalChainTailResult::DeleteProperty) {
            ConditionalValueArmSource::BooleanTrue
        } else {
            ConditionalValueArmSource::Undefined
        };
        self.lower_await_value_branches(
            value_result,
            condition,
            skipped,
            ConditionalValueArmSource::OptionalChain {
                base,
                first,
                rest,
                result,
            },
        )
    }

    fn lower_selected_optional_chain_tail(
        &mut self,
        base: EvaluatedOptionalChainBase,
        first: AwaitedOptionalChainLink<'_>,
        rest: AwaitedOptionalChainTail<'_>,
        result: OptionalChainTailResult,
    ) -> TypedExpr {
        match (base, first) {
            (EvaluatedOptionalChainBase::Value(base), AwaitedOptionalChainLink::Property(link)) => {
                if rest.is_empty() && matches!(result, OptionalChainTailResult::DeleteProperty) {
                    let (field, _) = link.into_parts();
                    let Some(key) = self.lower_optional_chain_property_key(field) else {
                        return self.unsupported_expr("unsupported optional computed property key");
                    };
                    let target = base.into_value(self);
                    return self.lower_delete_property_from_evaluated(target, key);
                }
                let Some(read) = CompletedOptionalPropertyRead::lower(self, base, link) else {
                    return self.unsupported_expr("unsupported optional computed property key");
                };
                if rest.is_empty() && matches!(&result, OptionalChainTailResult::CallReference(_)) {
                    return result.complete(read.into_value());
                }
                let base = EvaluatedOptionalChainBase::from_read(self, read, &rest);
                self.lower_optional_chain_tail(base, rest, result)
            }
            (
                EvaluatedOptionalChainBase::Value(base),
                AwaitedOptionalChainLink::Private { field, .. },
            ) => {
                let Some(private_name_id) = self.current_private_name_id(field) else {
                    return self.unsupported_expr("private class element");
                };
                let target = base.into_value(self);
                let read = CompletedOptionalPropertyRead::from_evaluated_private(
                    self,
                    target,
                    private_name_id,
                );
                if rest.is_empty() && matches!(&result, OptionalChainTailResult::CallReference(_)) {
                    return result.complete(read.into_value());
                }
                let base = EvaluatedOptionalChainBase::from_read(self, read, &rest);
                self.lower_optional_chain_tail(base, rest, result)
            }
            (EvaluatedOptionalChainBase::Call(reference), AwaitedOptionalChainLink::Call(link)) => {
                let value = self.lower_selected_optional_call(reference, link);
                if rest.is_empty() {
                    if matches!(result, OptionalChainTailResult::CallReference(_)) {
                        unreachable!("a checked grouped Reference ends in a property");
                    }
                    return value;
                }
                let base = EvaluatedOptionalChainBase::from_call_result(self, value, &rest);
                self.lower_optional_chain_tail(base, rest, result)
            }
            (EvaluatedOptionalChainBase::Value(_), AwaitedOptionalChainLink::Call(_))
            | (EvaluatedOptionalChainBase::Call(_), AwaitedOptionalChainLink::Property(_))
            | (EvaluatedOptionalChainBase::Call(_), AwaitedOptionalChainLink::Private { .. }) => {
                unreachable!("the next checked link owns its selected operand kind")
            }
        }
    }

    fn lower_await_value_branches(
        &mut self,
        result: ConditionalAwaitResult,
        condition: TypedExpr,
        then_source: ConditionalValueArmSource<'_>,
        else_source: ConditionalValueArmSource<'_>,
    ) -> TypedExpr {
        let entry = self
            .plain_async_entry_state()
            .expect("plain async conditional owner");
        let Some(then_entry) = entry.checked_add(1) else {
            return self.unsupported_expr("conditional value state overflow");
        };
        let before = self.capture_conditional_flow_facts();
        let then_arm = match then_source.lower(self, then_entry) {
            Ok(arm) => arm,
            Err(reason) => return self.unsupported_expr(reason),
        };
        let Some(else_entry) = then_arm.ready.checked_add(1) else {
            return self.unsupported_expr("conditional value state overflow");
        };
        let then_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        let else_arm = match else_source.lower(self, else_entry) {
            Ok(arm) => arm,
            Err(reason) => return self.unsupported_expr(reason),
        };
        let else_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(then_facts, else_facts);
        match result.finish(self, condition, entry, then_arm, else_arm) {
            Ok(value) => value,
            Err(reason) => self.unsupported_expr(reason),
        }
    }
}
