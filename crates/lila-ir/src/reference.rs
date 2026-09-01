//! Reference Records (ECMA-262 6.2.5), reified for the duration of one
//! lowering step.
//!
//! See `docs/rust-rewrite/contracts/reference-records.md`. The short version:
//!
//! - `[[Strict]]` is a field of the *record*, populated when the Reference is
//!   created, not a property of the code that happens to be emitting the
//!   write. [`Strictness`] exists so the two cannot be confused, and so that
//!   the value cannot be produced by accident from a `bool` sitting next to
//!   `implicit` or `configurable` in the same argument list.
//! - `[[Base]]`'s partition (6.2.5.1) is closed, so [`ReferenceBase`] is an
//!   enum matched exhaustively with no `_` arm anywhere in this file.
//! - 13.15.2 evaluates the LeftHandSideExpression **once** and consumes that
//!   one record at both GetValue and PutValue. [`ReferenceRecord::read`]
//!   therefore borrows and [`ReferenceRecord::write`] consumes; the record is
//!   neither `Clone` nor `Copy`, so a second write is `E0382`.
//! - The temporaries an effectful base or computed key was pinned into are
//!   returned as a separate [`ReferencePins`], and the only way to turn the
//!   written reference back into a [`TypedExpr`] is to spend them
//!   ([`ReferencePins::materialize`]). Forgetting is `E0308`, doing it twice
//!   is `E0382`.

use super::*;
use crate::{WellKnownSymbol, WithObjectBindingName};
use boa_ast::expression::operator::binary::{ArithmeticOp, BitwiseOp};
use std::sync::Arc;

const DELETE_SUPER_THIS_BINDING: &str = "$delete.super.this";
const DELETE_SUPER_KEY_BINDING: &str = "$delete.super.key";
const DELETE_SUPER_REFERENCE_ERROR: &str = "Cannot delete a super property";
const OBJECT_ENVIRONMENT_VALUE_BINDING: &str = "$object.environment.set.value";
const OBJECT_ENVIRONMENT_RECHECK_BINDING: &str = "$object.environment.set.exists";
const OBJECT_ENVIRONMENT_REFERENCE_ERROR: &str = "object environment binding no longer exists";

fn dynamic_value_info() -> ValueInfo {
    ValueInfo {
        kind: ValueKind::Dynamic,
        possible_kinds: KindSet::all_runtime_tags(),
        heap_shape: None,
        function_targets: FunctionTargetKnowledge::unknown(),
    }
}

/// The closed set of functions a deferred property operation may dispatch.
///
/// Exact shape and descriptor discoveries stay in `known`. When lost shape or
/// arbitrary source effects make every planned source function possible, one
/// shared immutable universe is retained instead of cloning every ID into
/// every Reference carrier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyHookTargets {
    known: BTreeSet<FunctionId>,
    all_planned_source: Option<Arc<BTreeSet<FunctionId>>>,
}

impl PropertyHookTargets {
    pub(crate) fn from_known(known: BTreeSet<FunctionId>) -> Self {
        Self {
            known,
            all_planned_source: None,
        }
    }

    pub(crate) fn extend_known(&mut self, targets: impl IntoIterator<Item = FunctionId>) {
        self.known.extend(targets);
    }

    pub(crate) fn include_all_planned_source(&mut self, targets: Arc<BTreeSet<FunctionId>>) {
        if let Some(existing) = &self.all_planned_source {
            debug_assert_eq!(existing.as_ref(), targets.as_ref());
            return;
        }
        self.all_planned_source = Some(targets);
    }

    pub(crate) fn extend_targets(&mut self, targets: Self) {
        self.known.extend(targets.known);
        if let Some(all_planned_source) = targets.all_planned_source {
            self.include_all_planned_source(all_planned_source);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FunctionId> {
        self.known.iter().chain(
            self.all_planned_source
                .iter()
                .flat_map(|targets| targets.iter())
                .filter(|target| !self.known.contains(*target)),
        )
    }

    pub(crate) fn includes_all_planned_source(&self) -> bool {
        self.all_planned_source.is_some()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.known.is_empty()
            && self
                .all_planned_source
                .as_ref()
                .is_none_or(|targets| targets.is_empty())
    }

    #[must_use]
    pub fn contains(&self, target: &FunctionId) -> bool {
        self.known.contains(target)
            || self
                .all_planned_source
                .as_ref()
                .is_some_and(|targets| targets.contains(target))
    }
}

/// The closed eager compound-assignment domain shared by every consuming
/// Reference plan. Logical assignment has a separate short-circuit lifecycle
/// and cannot enter this operation by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EagerCompoundAssignmentOp {
    Arithmetic(ArithmeticOp),
    Bitwise(BitwiseOp),
}

impl EagerCompoundAssignmentOp {
    /// Apply the selected eager operation to the old Reference value and RHS.
    ///
    /// This is a method on the closed operation so a consuming Reference plan
    /// can mint the old-value operand and apply it itself. Callers cannot pass
    /// an arbitrary closure which ignores or substitutes that operand.
    pub(crate) fn apply(self, lhs: TypedExpr, rhs: TypedExpr) -> TypedExpr {
        match self {
            EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add) => {
                let possible_kinds = KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Number))
                    .union(KindSet::from_kind(ValueKind::BigInt));
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::CoerciveAdd {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            EagerCompoundAssignmentOp::Arithmetic(arithmetic) => {
                let op = match arithmetic {
                    ArithmeticOp::Sub => ArithmeticBinaryOp::Sub,
                    ArithmeticOp::Mul => ArithmeticBinaryOp::Mul,
                    ArithmeticOp::Div => ArithmeticBinaryOp::Div,
                    ArithmeticOp::Mod => ArithmeticBinaryOp::Mod,
                    ArithmeticOp::Exp => ArithmeticBinaryOp::Exp,
                    ArithmeticOp::Add => unreachable!("addition has string-or-numeric semantics"),
                };
                let possible_kinds = KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::BigInt));
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::CoerciveBinaryNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            EagerCompoundAssignmentOp::Bitwise(bitwise) => {
                let op = match bitwise {
                    BitwiseOp::And => BitwiseBinaryOp::And,
                    BitwiseOp::Or => BitwiseBinaryOp::Or,
                    BitwiseOp::Xor => BitwiseBinaryOp::Xor,
                    BitwiseOp::Shl => BitwiseBinaryOp::Shl,
                    BitwiseOp::Shr => BitwiseBinaryOp::Shr,
                    BitwiseOp::UShr => BitwiseBinaryOp::UShr,
                };
                let possible_kinds = if matches!(bitwise, BitwiseOp::UShr) {
                    KindSet::from_kind(ValueKind::Number)
                } else {
                    KindSet::from_kind(ValueKind::Number)
                        .union(KindSet::from_kind(ValueKind::BigInt))
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::BitwiseNumeric {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
        }
    }
}

/// One fused eager mutation of an ordinary property Reference.
///
/// Fields are private so the backend cannot receive a write which has lost the
/// Reference's base/receiver identity, raw referenced name, or `[[Strict]]`.
/// The lowerer can construct this value only by consuming an
/// [`OrdinaryPropertyReferencePlan`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "an ordinary property Reference mutation must be consumed by the backend"]
pub struct OrdinaryPropertyEagerCompoundAssignmentIr {
    base_and_receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    strictness: Strictness,
    old_value_binding: String,
    result: Box<TypedExpr>,
    possible_getters: PropertyHookTargets,
    possible_setters: PropertyHookTargets,
}

impl OrdinaryPropertyEagerCompoundAssignmentIr {
    fn new(
        base_and_receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        strictness: Strictness,
        old_value_binding: String,
        result: Box<TypedExpr>,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> Self {
        Self {
            base_and_receiver,
            referenced_name,
            strictness,
            old_value_binding,
            result,
            possible_getters,
            possible_setters,
        }
    }

    #[must_use]
    pub fn base_and_receiver(&self) -> &TypedExpr {
        &self.base_and_receiver
    }

    #[must_use]
    pub fn referenced_name(&self) -> &PropertyKeyIr {
        &self.referenced_name
    }

    #[must_use]
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }

    #[must_use]
    pub fn old_value_binding(&self) -> &str {
        &self.old_value_binding
    }

    #[must_use]
    pub fn result(&self) -> &TypedExpr {
        &self.result
    }

    #[must_use]
    pub fn possible_getters(&self) -> &PropertyHookTargets {
        &self.possible_getters
    }

    #[must_use]
    pub fn possible_setters(&self) -> &PropertyHookTargets {
        &self.possible_setters
    }
}

/// One plain assignment through an ordinary property Reference.
///
/// The base, raw referenced name, RHS, and `[[Strict]]` remain one fused
/// obligation. In particular, the backend cannot validate or coerce the
/// Reference before it has evaluated the carried RHS.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "an ordinary property assignment must be consumed by the backend"]
pub struct OrdinaryPropertyAssignmentIr {
    base_and_receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    rhs: Box<TypedExpr>,
    strictness: Strictness,
    possible_setters: PropertyHookTargets,
}

impl OrdinaryPropertyAssignmentIr {
    fn new(
        base_and_receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        rhs: Box<TypedExpr>,
        strictness: Strictness,
        possible_setters: PropertyHookTargets,
    ) -> Self {
        Self {
            base_and_receiver,
            referenced_name,
            rhs,
            strictness,
            possible_setters,
        }
    }

    #[must_use]
    pub fn base_and_receiver(&self) -> &TypedExpr {
        &self.base_and_receiver
    }

    #[must_use]
    pub fn referenced_name(&self) -> &PropertyKeyIr {
        &self.referenced_name
    }

    #[must_use]
    pub fn rhs(&self) -> &TypedExpr {
        &self.rhs
    }

    #[must_use]
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }

    #[must_use]
    pub fn possible_setters(&self) -> &PropertyHookTargets {
        &self.possible_setters
    }
}

/// One logical assignment through an ordinary property Reference.
///
/// The carrier keeps the evaluated base/receiver, raw referenced name,
/// branch-local RHS, logical operation, and `[[Strict]]` together. Its backend
/// consumer therefore owns the single `ToPropertyKey`/GetValue pair and can
/// place both RHS evaluation and PutValue wholly inside the selected branch.
/// Possible accessor targets are carried with the Reference so later planning
/// cannot lose a getter or setter merely because control-flow joining erased
/// the base's single heap shape.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "an ordinary property logical assignment must be consumed by the backend"]
pub struct OrdinaryPropertyLogicalAssignmentIr {
    base_and_receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    rhs: Box<TypedExpr>,
    op: LogicalBinaryOp,
    strictness: Strictness,
    possible_getters: PropertyHookTargets,
    possible_setters: PropertyHookTargets,
}

impl OrdinaryPropertyLogicalAssignmentIr {
    fn new(
        base_and_receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        rhs: Box<TypedExpr>,
        op: LogicalBinaryOp,
        strictness: Strictness,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> Self {
        Self {
            base_and_receiver,
            referenced_name,
            rhs,
            op,
            strictness,
            possible_getters,
            possible_setters,
        }
    }

    #[must_use]
    pub fn base_and_receiver(&self) -> &TypedExpr {
        &self.base_and_receiver
    }

    #[must_use]
    pub fn referenced_name(&self) -> &PropertyKeyIr {
        &self.referenced_name
    }

    #[must_use]
    pub fn rhs(&self) -> &TypedExpr {
        &self.rhs
    }

    #[must_use]
    pub fn op(&self) -> LogicalBinaryOp {
        self.op
    }

    #[must_use]
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }

    #[must_use]
    pub fn possible_getters(&self) -> &PropertyHookTargets {
        &self.possible_getters
    }

    #[must_use]
    pub fn possible_setters(&self) -> &PropertyHookTargets {
        &self.possible_setters
    }
}

/// One fused numeric update of an ordinary property Reference.
///
/// The operation and return mode are separate closed domains: the backend must
/// choose the numeric delta and the old/new publication role independently,
/// after consuming the same base/key/strictness tuple for GetValue and
/// PutValue.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "an ordinary property numeric update must be consumed by the backend"]
pub struct OrdinaryPropertyNumericUpdateIr {
    base_and_receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    strictness: Strictness,
    op: NumericUpdateOp,
    return_mode: UpdateReturnMode,
    value_kind: NumericUpdateValueKind,
    possible_getters: PropertyHookTargets,
    possible_setters: PropertyHookTargets,
}

impl OrdinaryPropertyNumericUpdateIr {
    fn new(
        base_and_receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        strictness: Strictness,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> Self {
        Self {
            base_and_receiver,
            referenced_name,
            strictness,
            op,
            return_mode,
            value_kind: NumericUpdateValueKind::Dynamic,
            possible_getters,
            possible_setters,
        }
    }

    #[must_use]
    pub fn base_and_receiver(&self) -> &TypedExpr {
        &self.base_and_receiver
    }

    #[must_use]
    pub fn referenced_name(&self) -> &PropertyKeyIr {
        &self.referenced_name
    }

    #[must_use]
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }

    #[must_use]
    pub fn op(&self) -> NumericUpdateOp {
        self.op
    }

    #[must_use]
    pub fn return_mode(&self) -> UpdateReturnMode {
        self.return_mode
    }

    #[must_use]
    pub fn value_kind(&self) -> NumericUpdateValueKind {
        self.value_kind
    }

    #[must_use]
    pub fn possible_getters(&self) -> &PropertyHookTargets {
        &self.possible_getters
    }

    #[must_use]
    pub fn possible_setters(&self) -> &PropertyHookTargets {
        &self.possible_setters
    }
}

/// A lowerer-owned ordinary property Reference which must be consumed as one
/// mutation rather than decomposed into independent read and write nodes.
///
/// Neither `Clone` nor `Copy`: the same base/raw-key/strictness tuple cannot be
/// spent twice or rebuilt between GetValue and PutValue.
#[derive(Debug)]
#[must_use = "an ordinary property Reference plan must be consumed by one mutation"]
pub(crate) struct OrdinaryPropertyReferencePlan {
    base_and_receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    strictness: Strictness,
}

impl OrdinaryPropertyReferencePlan {
    pub(crate) fn new(
        base_and_receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        strictness: Strictness,
    ) -> Self {
        Self {
            base_and_receiver,
            referenced_name,
            strictness,
        }
    }

    /// Consume the retained Reference together with an already-lowered RHS.
    /// Runtime validation and key coercion remain backend-owned so both occur
    /// after RHS evaluation.
    #[must_use]
    pub(crate) fn plain_assignment(
        self,
        rhs: TypedExpr,
        possible_setters: PropertyHookTargets,
    ) -> TypedExpr {
        TypedExpr::from_info(
            rhs.value_info(),
            ExprIr::OrdinaryPropertyAssignment(OrdinaryPropertyAssignmentIr::new(
                self.base_and_receiver,
                self.referenced_name,
                Box::new(rhs),
                self.strictness,
                possible_setters,
            )),
        )
    }

    /// Consume one Reference into the closed logical-assignment lifecycle.
    ///
    /// The result remains dynamic because the expression can publish either
    /// the runtime property value or the RHS. The backend, rather than a
    /// decomposed `LogicalShortCircuit`, owns which branch performs PutValue.
    #[must_use]
    pub(crate) fn logical_assignment(
        self,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> TypedExpr {
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::OrdinaryPropertyLogicalAssignment(OrdinaryPropertyLogicalAssignmentIr::new(
                self.base_and_receiver,
                self.referenced_name,
                Box::new(rhs),
                op,
                self.strictness,
                possible_getters,
                possible_setters,
            )),
        )
    }

    #[must_use]
    pub(crate) fn eager_compound_assignment(
        self,
        old_value_binding: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> TypedExpr {
        let old_value = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Identifier(old_value_binding.clone()),
        );
        let result = op.apply(old_value, rhs);
        TypedExpr::from_info(
            result.value_info(),
            ExprIr::OrdinaryPropertyEagerCompoundAssignment(
                OrdinaryPropertyEagerCompoundAssignmentIr::new(
                    self.base_and_receiver,
                    self.referenced_name,
                    self.strictness,
                    old_value_binding,
                    Box::new(result),
                    possible_getters,
                    possible_setters,
                ),
            ),
        )
    }

    #[must_use]
    pub(crate) fn numeric_update(
        self,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        possible_getters: PropertyHookTargets,
        possible_setters: PropertyHookTargets,
    ) -> TypedExpr {
        let value_kind = ValueKind::Dynamic;
        let info = ValueInfo {
            kind: value_kind,
            possible_kinds: KindSet::from_kind(ValueKind::Number)
                .union(KindSet::from_kind(ValueKind::BigInt)),
            heap_shape: None,
            function_targets: FunctionTargetKnowledge::none(),
        };
        TypedExpr::from_info(
            info,
            ExprIr::OrdinaryPropertyNumericUpdate(OrdinaryPropertyNumericUpdateIr::new(
                self.base_and_receiver,
                self.referenced_name,
                self.strictness,
                op,
                return_mode,
                possible_getters,
                possible_setters,
            )),
        )
    }
}

/// One fused mutation of a Super Property Reference.
///
/// The fields are private so the backend cannot receive a mutation which has
/// lost the Reference's receiver, raw referenced name, or `[[Strict]]`. The
/// lowerer can construct this value only by consuming a
/// [`SuperPropertyReferencePlan`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a Super Property Reference mutation must be consumed by the backend"]
pub struct SuperPropertyMutationIr {
    receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    strictness: Strictness,
    operation: SuperPropertyMutationOperationIr,
}

/// The exhaustive operation which consumes a fused Super Property Reference.
///
/// Logical assignment is absent deliberately: its conditional RHS and
/// PutValue lifecycle cannot be represented as an eager operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuperPropertyMutationOperationIr {
    NumericUpdate {
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: NumericUpdateValueKind,
    },
    EagerCompound {
        old_value_binding: String,
        result: Box<TypedExpr>,
    },
}

impl SuperPropertyMutationIr {
    fn new(
        receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        strictness: Strictness,
        operation: SuperPropertyMutationOperationIr,
    ) -> Self {
        Self {
            receiver,
            referenced_name,
            strictness,
            operation,
        }
    }

    #[must_use]
    pub fn receiver(&self) -> &TypedExpr {
        &self.receiver
    }

    #[must_use]
    pub fn referenced_name(&self) -> &PropertyKeyIr {
        &self.referenced_name
    }

    #[must_use]
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }

    #[must_use]
    pub fn operation(&self) -> &SuperPropertyMutationOperationIr {
        &self.operation
    }
}

/// A lowerer-owned Super Property Reference which must be consumed as one
/// mutation rather than decomposed into independent read and write nodes.
///
/// Neither `Clone` nor `Copy`: the same captured receiver/key/strictness tuple
/// cannot be spent twice or rebuilt between GetValue and PutValue.
#[derive(Debug)]
#[must_use = "a Super Property Reference plan must be consumed by one mutation"]
pub(crate) struct SuperPropertyReferencePlan {
    receiver: Box<TypedExpr>,
    referenced_name: PropertyKeyIr,
    strictness: Strictness,
}

impl SuperPropertyReferencePlan {
    pub(crate) fn new(
        receiver: Box<TypedExpr>,
        referenced_name: PropertyKeyIr,
        strictness: Strictness,
    ) -> Self {
        Self {
            receiver,
            referenced_name,
            strictness,
        }
    }

    #[must_use]
    pub(crate) fn numeric_update(
        self,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: NumericUpdateValueKind,
    ) -> TypedExpr {
        let info = match value_kind {
            NumericUpdateValueKind::Number | NumericUpdateValueKind::BigInt => {
                ValueInfo::new(value_kind.value_kind())
            }
            NumericUpdateValueKind::Dynamic => ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::BigInt)),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
        };
        TypedExpr::from_info(
            info,
            ExprIr::SuperPropertyMutation(SuperPropertyMutationIr::new(
                self.receiver,
                self.referenced_name,
                self.strictness,
                SuperPropertyMutationOperationIr::NumericUpdate {
                    op,
                    return_mode,
                    value_kind,
                },
            )),
        )
    }

    #[must_use]
    pub(crate) fn eager_compound_assignment(
        self,
        old_value_binding: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
    ) -> TypedExpr {
        let old_value = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Identifier(old_value_binding.clone()),
        );
        let result = op.apply(old_value, rhs);
        TypedExpr::from_info(
            result.value_info(),
            ExprIr::SuperPropertyMutation(SuperPropertyMutationIr::new(
                self.receiver,
                self.referenced_name,
                self.strictness,
                SuperPropertyMutationOperationIr::EagerCompound {
                    old_value_binding,
                    result: Box::new(result),
                },
            )),
        )
    }
}

/// A Super Reference whose only consumer is the `delete` operator.
///
/// `delete super[key]` has a deliberately unusual lifecycle. SuperProperty
/// evaluation obtains `actualThis`, then evaluates and gets the computed key's
/// value without applying `ToPropertyKey`. Delete subsequently observes that
/// the Reference is a Super Reference and throws before inspecting `[[Base]]`,
/// coercing the key, or calling `[[Delete]]`.
///
/// This private plan fuses those operations without turning their ordering
/// back into a lowering convention. Its constructor always creates the
/// `actualThis` operand as [`ExprIr::This`]; callers cannot substitute the
/// super base. Its key conversion and consuming match are both exhaustive, so
/// a new [`PropertyKeyIr`] representation or a new lifecycle state is a compile
/// error rather than a path which accidentally performs `ToPropertyKey`.
///
/// The plan lowers to nested [`ExprIr::MaterializeBinding`] nodes rather than a
/// comma expression because MaterializeBinding propagates an abrupt completion
/// from each evaluated operand before entering its body. The bound values are
/// intentionally never read. Their fixed names cannot collide with source
/// bindings (they contain `.`), and each materialization owns a lexical scope.
#[derive(Debug)]
#[must_use = "a delete-super Reference must be consumed into its ReferenceError"]
pub(crate) struct DeleteSuperReferencePlan {
    actual_this: Box<TypedExpr>,
    referenced_name: DeleteSuperReferencedName,
}

#[derive(Debug)]
enum DeleteSuperReferencedName {
    Static,
    Uncoerced(Box<TypedExpr>),
}

impl DeleteSuperReferencePlan {
    pub(crate) fn new(actual_this: ValueInfo, referenced_name: PropertyKeyIr) -> Self {
        let referenced_name = match referenced_name {
            PropertyKeyIr::StaticString(name) => {
                drop(name);
                DeleteSuperReferencedName::Static
            }
            PropertyKeyIr::ArrayLength => DeleteSuperReferencedName::Static,
            PropertyKeyIr::StringExpr(value) | PropertyKeyIr::ArrayIndex(value) => {
                DeleteSuperReferencedName::Uncoerced(value)
            }
        };
        Self {
            actual_this: Box::new(TypedExpr::from_info(actual_this, ExprIr::This)),
            referenced_name,
        }
    }

    #[must_use]
    pub(crate) fn into_reference_error(self) -> TypedExpr {
        let Self {
            actual_this,
            referenced_name,
        } = self;
        let throw = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: DELETE_SUPER_REFERENCE_ERROR,
            },
        );
        let body = match referenced_name {
            DeleteSuperReferencedName::Static => throw,
            DeleteSuperReferencedName::Uncoerced(value) => {
                let info = throw.value_info();
                TypedExpr::from_info(
                    info,
                    ExprIr::MaterializeBinding {
                        name: DELETE_SUPER_KEY_BINDING.to_string(),
                        value,
                        body: Box::new(throw),
                    },
                )
            }
        };
        let info = body.value_info();
        TypedExpr::from_info(
            info,
            ExprIr::MaterializeBinding {
                name: DELETE_SUPER_THIS_BINDING.to_string(),
                value: actual_this,
                body: Box::new(body),
            },
        )
    }
}

/// The exact binding object of one Object Environment Record.
///
/// The private source domain distinguishes an already-materialized `with`
/// object from the compiler-owned global object. Callers cannot provide an
/// arbitrary [`TypedExpr`], so cloning this value never re-evaluates a source
/// expression and every operation in one Reference reads the same identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectEnvironmentBindingObject {
    source: ObjectEnvironmentBindingObjectSource,
    info: ValueInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObjectEnvironmentBindingObjectSource {
    Materialized(String),
    GlobalObject,
}

impl ObjectEnvironmentBindingObject {
    pub(crate) fn materialized(binding_name: &WithObjectBindingName, info: ValueInfo) -> Self {
        Self {
            source: ObjectEnvironmentBindingObjectSource::Materialized(
                binding_name.as_str().to_string(),
            ),
            info,
        }
    }

    fn global_object(info: ValueInfo) -> Self {
        Self {
            source: ObjectEnvironmentBindingObjectSource::GlobalObject,
            info,
        }
    }

    fn read(&self) -> TypedExpr {
        let expr = match &self.source {
            ObjectEnvironmentBindingObjectSource::Materialized(storage_name) => {
                ExprIr::Identifier(storage_name.clone())
            }
            ObjectEnvironmentBindingObjectSource::GlobalObject => {
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string())
            }
        };
        TypedExpr::from_info(self.info.clone(), expr)
    }

    fn has_property(&self, referenced_name: &str) -> TypedExpr {
        TypedExpr::spec_has_property(
            self.read(),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String(referenced_name.to_string()),
            ),
        )
    }

    /// Object Environment Record HasBinding, including `Symbol.unscopables`.
    ///
    /// The temporary name is allocated by the lowerer because it owns lexical
    /// name allocation. The object operand is not supplied separately: both
    /// HasProperty and the unscopables read come from this validated binding
    /// object, so they cannot silently disagree about the environment queried.
    fn binding_visible(&self, referenced_name: &str, unscopables_binding: String) -> TypedExpr {
        let has_property = self.has_property(referenced_name);
        let unscopables = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::PropertyRead {
                target: Box::new(self.read()),
                key: PropertyKeyIr::StringExpr(Box::new(TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Symbol),
                    ExprIr::String(WellKnownSymbol::Unscopables.description().to_string()),
                ))),
            },
        );
        let read_unscopables = || {
            TypedExpr::from_info(
                dynamic_value_info(),
                ExprIr::Identifier(unscopables_binding.clone()),
            )
        };
        let unscopables_type = || {
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::TypeOf {
                    expr: Box::new(read_unscopables()),
                },
            )
        };
        let type_is_object = TypedExpr::spec_strict_equality_comparison(
            unscopables_type(),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("object".to_string()),
            ),
        );
        let is_not_null = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalNot {
                expr: Box::new(TypedExpr::spec_strict_equality_comparison(
                    read_unscopables(),
                    TypedExpr::from_info(ValueInfo::new(ValueKind::Null), ExprIr::Null),
                )),
            },
        );
        let is_non_null_object = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::And,
                lhs: Box::new(type_is_object),
                rhs: Box::new(is_not_null),
            },
        );
        let type_is_function = TypedExpr::spec_strict_equality_comparison(
            unscopables_type(),
            TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String("function".to_string()),
            ),
        );
        let is_object = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::Or,
                lhs: Box::new(is_non_null_object),
                rhs: Box::new(type_is_function),
            },
        );
        let blocked = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::Conditional {
                condition: Box::new(is_object),
                then_expr: Box::new(TypedExpr::spec_to_boolean(TypedExpr::from_info(
                    dynamic_value_info(),
                    ExprIr::PropertyRead {
                        target: Box::new(read_unscopables()),
                        key: PropertyKeyIr::StaticString(referenced_name.to_string()),
                    },
                ))),
                else_expr: Box::new(TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Boolean),
                    ExprIr::Boolean(false),
                )),
            },
        );
        let binding_unblocked = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalNot {
                expr: Box::new(blocked),
            },
        );
        let binding_unblocked = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::MaterializeBinding {
                name: unscopables_binding,
                value: Box::new(unscopables),
                body: Box::new(binding_unblocked),
            },
        );
        TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::And,
                lhs: Box::new(has_property),
                rhs: Box::new(binding_unblocked),
            },
        )
    }

    /// GetBindingValue on the Object Environment Record selected by
    /// HasBinding. The second HasProperty is independently observable: the
    /// unscopables getter can delete the binding, and a Proxy can complete
    /// abruptly here even after the initial query succeeded.
    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {
        let recheck = self.has_property(referenced_name);
        let read = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::PropertyRead {
                target: Box::new(self.read()),
                key: PropertyKeyIr::StaticString(referenced_name.to_string()),
            },
        );
        let missing = match strictness {
            Strictness::Sloppy => TypedExpr::undefined(),
            Strictness::Strict => TypedExpr::from_info(
                dynamic_value_info(),
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
                },
            ),
        };
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Conditional {
                condition: Box::new(recheck),
                then_expr: Box::new(read),
                else_expr: Box::new(missing),
            },
        )
    }

    /// SetMutableBinding on the Object Environment Record selected before RHS.
    fn put_value(
        self,
        referenced_name: &str,
        strictness: Strictness,
        value: TypedExpr,
    ) -> TypedExpr {
        let value_info = value.value_info();
        let read_value = TypedExpr::from_info(
            value_info.clone(),
            ExprIr::Identifier(OBJECT_ENVIRONMENT_VALUE_BINDING.to_string()),
        );
        let recheck = self.has_property(referenced_name);
        let write = TypedExpr::from_info(
            value_info.clone(),
            ExprIr::PropertyWrite {
                target: Box::new(self.read()),
                key: PropertyKeyIr::StaticString(referenced_name.to_string()),
                value: Box::new(read_value),
                strictness,
            },
        );
        let after_recheck = match strictness {
            Strictness::Sloppy => TypedExpr::from_info(
                value_info.clone(),
                ExprIr::MaterializeBinding {
                    name: OBJECT_ENVIRONMENT_RECHECK_BINDING.to_string(),
                    value: Box::new(recheck),
                    body: Box::new(write),
                },
            ),
            Strictness::Strict => TypedExpr::from_info(
                value_info.clone(),
                ExprIr::Conditional {
                    condition: Box::new(recheck),
                    then_expr: Box::new(write),
                    else_expr: Box::new(TypedExpr::from_info(
                        value_info.clone(),
                        ExprIr::RuntimeThrow {
                            name: NativeErrorKind::ReferenceError,
                            message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
                        },
                    )),
                },
            ),
        };
        TypedExpr::from_info(
            value_info,
            ExprIr::MaterializeBinding {
                name: OBJECT_ENVIRONMENT_VALUE_BINDING.to_string(),
                value: Box::new(value),
                body: Box::new(after_recheck),
            },
        )
    }

    /// GetValue, ToNumeric/delta, same-base PutValue, then prefix/postfix
    /// result. Initial ResolveBinding selection belongs to the
    /// environment-specific plan.
    fn numeric_update(
        self,
        referenced_name: &str,
        strictness: Strictness,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        bindings: &NumericUpdateBindings,
    ) -> TypedExpr {
        let NumericUpdateBindings {
            old_value: old_value_name,
            result: result_name,
            write: write_name,
        } = bindings;
        let old_value = self.clone().get_value(referenced_name, strictness);
        let numeric_info = ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::from_kind(ValueKind::Number)
                .union(KindSet::from_kind(ValueKind::BigInt)),
            heap_shape: None,
            function_targets: FunctionTargetKnowledge::none(),
        };
        let update = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::UpdateIdentifier {
                name: old_value_name.clone(),
                op,
                return_mode,
                value_kind: NumericUpdateValueKind::Dynamic,
            },
        );
        let updated_value = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::Identifier(old_value_name.clone()),
        );
        let write = self.put_value(referenced_name, strictness, updated_value);
        let result = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::Identifier(result_name.clone()),
        );
        let after_write = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::MaterializeBinding {
                name: write_name.clone(),
                value: Box::new(write),
                body: Box::new(result),
            },
        );
        let after_update = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::MaterializeBinding {
                name: result_name.clone(),
                value: Box::new(update),
                body: Box::new(after_write),
            },
        );
        TypedExpr::from_info(
            numeric_info,
            ExprIr::MaterializeBinding {
                name: old_value_name.clone(),
                value: Box::new(old_value),
                body: Box::new(after_update),
            },
        )
    }

    /// GetValue, logical selection, then same-base PutValue only in the taken
    /// branch. Initial ResolveBinding selection belongs to the
    /// environment-specific plan.
    fn logical_assignment(
        self,
        referenced_name: &str,
        strictness: Strictness,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
    ) -> TypedExpr {
        let lhs = self.clone().get_value(referenced_name, strictness);
        let write = self.put_value(referenced_name, strictness, rhs);
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::LogicalShortCircuit {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(write),
            },
        )
    }

    /// GetValue, eager operation, same-base PutValue, then result. Initial
    /// ResolveBinding selection belongs to the environment-specific plan.
    fn eager_compound_assignment(
        self,
        referenced_name: &str,
        strictness: Strictness,
        assignment: &EagerCompoundAssignment,
    ) -> TypedExpr {
        let EagerCompoundAssignment {
            bindings:
                EagerCompoundAssignmentBindings {
                    old_value: old_value_name,
                    result: result_name,
                    write: write_name,
                },
            result: applied,
        } = assignment;
        let old_value = self.clone().get_value(referenced_name, strictness);
        let result_info = applied.value_info();
        let result =
            TypedExpr::from_info(result_info.clone(), ExprIr::Identifier(result_name.clone()));
        let write = self.put_value(referenced_name, strictness, result.clone());
        let after_write = TypedExpr::from_info(
            result_info.clone(),
            ExprIr::MaterializeBinding {
                name: write_name.clone(),
                value: Box::new(write),
                body: Box::new(result),
            },
        );
        let after_apply = TypedExpr::from_info(
            result_info.clone(),
            ExprIr::MaterializeBinding {
                name: result_name.clone(),
                value: Box::new(applied.clone()),
                body: Box::new(after_write),
            },
        );
        TypedExpr::from_info(
            result_info,
            ExprIr::MaterializeBinding {
                name: old_value_name.clone(),
                value: Box::new(old_value),
                body: Box::new(after_apply),
            },
        )
    }
}

/// Declarative-frame depth in the function currently being lowered. This is a
/// different domain from a captured definition cursor and cannot be passed to
/// a captured position by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurrentScopeDepth(usize);

impl CurrentScopeDepth {
    pub(crate) fn at_with_entry(scope_count: usize) -> Self {
        assert!(scope_count > 0, "a lowerer must have an activation scope");
        Self(scope_count)
    }

    pub(crate) fn of_binding_scope(scope_index: usize) -> Self {
        Self(scope_index + 1)
    }

    pub(crate) fn activation() -> Self {
        Self(1)
    }

    fn index(self) -> usize {
        self.0
    }
}

/// Index in a function's definition cursor chain, counted inner-to-outer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapturedCursorDepth(usize);

impl CapturedCursorDepth {
    pub(crate) fn at(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurrentObjectPosition(CurrentScopeDepth);

impl CurrentObjectPosition {
    fn depth(self) -> usize {
        self.0.index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurrentBindingPosition(CurrentScopeDepth);

impl CurrentBindingPosition {
    fn depth(self) -> usize {
        self.0.index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapturedObjectPosition(CapturedCursorDepth);

impl CapturedObjectPosition {
    pub(crate) fn at(depth: CapturedCursorDepth) -> Self {
        Self(depth)
    }

    fn depth(self) -> usize {
        self.0.index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapturedBindingPosition(CapturedCursorDepth);

impl CapturedBindingPosition {
    pub(crate) fn at(depth: CapturedCursorDepth) -> Self {
        Self(depth)
    }

    fn depth(self) -> usize {
        self.0.index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectEnvironmentPosition {
    Current(CurrentObjectPosition),
    Captured(CapturedObjectPosition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclarativeEnvironmentPosition {
    Current(CurrentBindingPosition),
    Captured(CapturedBindingPosition),
}

impl DeclarativeEnvironmentPosition {
    pub(crate) fn current(depth: CurrentScopeDepth) -> Self {
        Self::Current(CurrentBindingPosition(depth))
    }

    pub(crate) fn captured(position: CapturedBindingPosition) -> Self {
        Self::Captured(position)
    }
}

impl ObjectEnvironmentPosition {
    /// Whether this Object Environment Record is encountered before the
    /// already-located declarative fallback during ResolveBinding. No `_` arm:
    /// adding either current/captured position class is E0004 here.
    fn precedes(self, binding: DeclarativeEnvironmentPosition) -> bool {
        match (self, binding) {
            (Self::Current(object), DeclarativeEnvironmentPosition::Current(binding)) => {
                object.depth() >= binding.depth()
            }
            (Self::Current(_), DeclarativeEnvironmentPosition::Captured(_)) => true,
            (Self::Captured(_), DeclarativeEnvironmentPosition::Current(_)) => false,
            (Self::Captured(object), DeclarativeEnvironmentPosition::Captured(binding)) => {
                object.depth() < binding.depth()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PositionedWithEnvironment {
    binding_object: ObjectEnvironmentBindingObject,
    position: ObjectEnvironmentPosition,
}

impl PositionedWithEnvironment {
    pub(crate) fn captured(
        binding_object: ObjectEnvironmentBindingObject,
        position: CapturedObjectPosition,
    ) -> Self {
        Self {
            binding_object,
            position: ObjectEnvironmentPosition::Captured(position),
        }
    }
}

/// Current and captured Object Environment Records in ResolveBinding order.
#[derive(Debug, Default)]
pub(crate) struct OrderedWithEnvironmentChain {
    current: Vec<PositionedWithEnvironment>,
    captured: Vec<PositionedWithEnvironment>,
}

impl OrderedWithEnvironmentChain {
    pub(crate) fn enter_current(
        &mut self,
        binding_object: ObjectEnvironmentBindingObject,
        depth: CurrentScopeDepth,
    ) {
        self.current.push(PositionedWithEnvironment {
            binding_object,
            position: ObjectEnvironmentPosition::Current(CurrentObjectPosition(depth)),
        });
    }

    pub(crate) fn leave_current(&mut self) {
        self.current
            .pop()
            .expect("with statement must restore its ordered environment chain");
    }

    pub(crate) fn seed_captured(&mut self, captured: Vec<PositionedWithEnvironment>) {
        assert!(
            self.captured.is_empty(),
            "captured with environments may be seeded only once"
        );
        self.captured = captured;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.current.is_empty() && self.captured.is_empty()
    }

    /// Select every Object Environment Record encountered before an already
    /// located declarative fallback. The returned type is structurally
    /// non-empty; callers cannot request only the innermost object and thereby
    /// skip outer resolution or declarative cutoff.
    pub(crate) fn select_preceding(
        &self,
        fallback: Option<DeclarativeEnvironmentPosition>,
    ) -> Option<SelectedWithEnvironmentObjects> {
        let mut selected = self
            .current
            .iter()
            .rev()
            .chain(self.captured.iter())
            .filter(|environment| {
                fallback.is_none_or(|binding| environment.position.precedes(binding))
            })
            .map(|environment| environment.binding_object.clone());
        let innermost = selected.next()?;
        Some(SelectedWithEnvironmentObjects {
            innermost,
            outer: selected.collect(),
        })
    }
}

/// A non-empty Object Environment Record chain which ResolveBinding encounters
/// before its declarative/global fallback, in inner-to-outer order.
///
/// Deliberately neither `Clone` nor `Copy`: consuming this selection is the
/// only external way to obtain a [`WithEnvironmentReferencePlan`].
#[derive(Debug)]
pub(crate) struct SelectedWithEnvironmentObjects {
    innermost: ObjectEnvironmentBindingObject,
    outer: Vec<ObjectEnvironmentBindingObject>,
}

/// One dynamically queried Object Environment Record in ResolveBinding.
#[derive(Debug)]
struct WithEnvironmentResolution {
    binding_object: ObjectEnvironmentBindingObject,
    unscopables_binding: String,
}

impl WithEnvironmentResolution {
    fn create(binding_object: ObjectEnvironmentBindingObject, unscopables_binding: String) -> Self {
        Self {
            binding_object,
            unscopables_binding,
        }
    }

    fn get_value_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let with_value = binding_object.get_value(referenced_name, strictness);
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(with_value),
                else_expr: Box::new(fallback),
            },
        )
    }

    /// Resolve one identifier-call Reference through this Object Environment
    /// candidate. The same validated binding object produces both the
    /// GetBindingValue callee and WithBaseObject receiver; callers cannot
    /// provide or transpose those roles independently.
    fn call_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        args: &[TypedExpr],
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let callee = binding_object
            .clone()
            .get_value(referenced_name, strictness);
        let receiver = binding_object.read();
        let selected = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::CallIndirect {
                callee: Box::new(callee),
                this_arg: Some(Box::new(receiver)),
                args: args.to_vec(),
                static_regexp_compilation: None,
            },
        );
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(selected),
                else_expr: Box::new(fallback),
            },
        )
    }

    fn put_value_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        value: TypedExpr,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let with_write = binding_object.put_value(referenced_name, strictness, value);
        TypedExpr::from_info(
            with_write.value_info(),
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(with_write),
                else_expr: Box::new(fallback),
            },
        )
    }

    /// Compose one selected Object Environment Record's independently
    /// observable GetBindingValue and SetMutableBinding around a numeric
    /// update. Resolution is not restarted after the getter runs.
    fn numeric_update_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        bindings: &NumericUpdateBindings,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let selected_update =
            binding_object.numeric_update(referenced_name, strictness, op, return_mode, bindings);
        let numeric_info = selected_update.value_info();
        TypedExpr::from_info(
            numeric_info,
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(selected_update),
                else_expr: Box::new(fallback),
            },
        )
    }

    /// Compose one selected Object Environment Record's GetBindingValue and
    /// branch-local SetMutableBinding around a logical assignment. Resolution
    /// is not restarted after either GetValue or the RHS.
    fn logical_assignment_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let selected = binding_object.logical_assignment(referenced_name, strictness, op, rhs);
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(selected),
                else_expr: Box::new(fallback),
            },
        )
    }

    /// Compose one selected Object Environment Record's GetBindingValue,
    /// eager operator application and SetMutableBinding. The sealed assignment
    /// carries the only old/result/write roles that can reach this operation.
    fn compound_assignment_or_else(
        self,
        referenced_name: &str,
        strictness: Strictness,
        assignment: &EagerCompoundAssignment,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            binding_object,
            unscopables_binding,
        } = self;
        let binding_visible = binding_object.binding_visible(referenced_name, unscopables_binding);
        let selected_assignment =
            binding_object.eager_compound_assignment(referenced_name, strictness, assignment);
        let result_info = selected_assignment.value_info();
        TypedExpr::from_info(
            result_info,
            ExprIr::Conditional {
                condition: Box::new(binding_visible),
                then_expr: Box::new(selected_assignment),
                else_expr: Box::new(fallback),
            },
        )
    }
}

impl SelectedWithEnvironmentObjects {
    /// Consume the selected inner-to-outer objects into the one Reference plan
    /// used by both GetValue and PutValue. Nested conditionals are assembled
    /// outermost-first, hence the single explicit reversal here.
    pub(crate) fn into_reference_plan(
        self,
        referenced_name: String,
        strictness: Strictness,
        mut allocate_unscopables_binding: impl FnMut() -> String,
    ) -> WithEnvironmentReferencePlan {
        let Self { innermost, outer } = self;
        let innermost =
            WithEnvironmentResolution::create(innermost, allocate_unscopables_binding());
        let mut outer = outer
            .into_iter()
            .map(|object| WithEnvironmentResolution::create(object, allocate_unscopables_binding()))
            .collect::<Vec<_>>();
        outer.reverse();
        WithEnvironmentReferencePlan::create(innermost, outer, referenced_name, strictness)
    }

    /// Consume this non-empty selection into the identifier-call-only
    /// Reference capability. There is no constructor from a value-only
    /// identifier, so WithBaseObject cannot be recovered after GetValue.
    pub(crate) fn into_identifier_call_plan(
        self,
        referenced_name: String,
        strictness: Strictness,
        allocate_unscopables_binding: impl FnMut() -> String,
    ) -> WithEnvironmentIdentifierCallReferencePlan {
        WithEnvironmentIdentifierCallReferencePlan {
            reference: self.into_reference_plan(
                referenced_name,
                strictness,
                allocate_unscopables_binding,
            ),
        }
    }
}

/// A non-empty ResolveBinding chain whose only result is an identifier call.
///
/// The wrapped Reference is deliberately inaccessible and neither type is
/// `Clone` or `Copy`. [`Self::call`] is therefore the only way to obtain the
/// callee/WithBaseObject receiver product, and consuming it twice is E0382.
#[derive(Debug)]
#[must_use = "a with-environment identifier-call Reference must be consumed by Call"]
pub(crate) struct WithEnvironmentIdentifierCallReferencePlan {
    reference: WithEnvironmentReferencePlan,
}

impl WithEnvironmentIdentifierCallReferencePlan {
    /// Consume the Reference into mutually exclusive selected-object calls and
    /// one ordinary undefined-this fallback call. Argument IR is cloned only
    /// across runtime-exclusive branches; the source arguments were lowered
    /// once before entering this transition.
    #[must_use]
    pub(crate) fn call(self, args: Vec<TypedExpr>, fallback: TypedExpr) -> TypedExpr {
        let WithEnvironmentReferencePlan {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self.reference;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.call_or_else(&referenced_name, strictness, &args, resolved);
        }
        innermost.call_or_else(&referenced_name, strictness, &args, resolved)
    }
}

/// A non-empty ResolveBinding chain for an identifier Reference inside `with`.
///
/// The innermost resolution is a required field instead of the first element
/// of a `Vec`, so an empty Object Environment chain is not representable. The
/// plan is deliberately neither `Clone` nor `Copy`; [`Self::get_value`],
/// [`Self::put_value`], [`Self::logical_assignment`], [`Self::numeric_update`]
/// and [`Self::compound_assignment`] consume it, making a second use E0382.
#[derive(Debug)]
#[must_use = "a with-environment Reference must be consumed by GetValue, PutValue, logical assignment, numeric update, or compound assignment"]
pub(crate) struct WithEnvironmentReferencePlan {
    innermost: WithEnvironmentResolution,
    outer: Vec<WithEnvironmentResolution>,
    referenced_name: String,
    strictness: Strictness,
}

/// One identifier Reference selected by the Global Environment Record's
/// Object Record.
///
/// This plan is deliberately neither `Clone` nor `Copy`. Its constructor owns
/// the compiler-known global object identity, while its only consumer performs
/// the initial HasBinding/HasProperty before the shared GetValue/apply/PutValue
/// lifecycle. Unlike [`WithEnvironmentReferencePlan`], this type has no
/// unscopables state or fallback chain.
#[derive(Debug)]
#[must_use = "a global Object Environment Reference must be consumed by logical assignment, numeric update, or eager compound assignment"]
pub(crate) struct GlobalObjectEnvironmentReferencePlan {
    binding_object: ObjectEnvironmentBindingObject,
    referenced_name: String,
    strictness: Strictness,
}

impl GlobalObjectEnvironmentReferencePlan {
    pub(crate) fn new(
        global_object_info: ValueInfo,
        referenced_name: String,
        strictness: Strictness,
    ) -> Self {
        Self {
            binding_object: ObjectEnvironmentBindingObject::global_object(global_object_info),
            referenced_name,
            strictness,
        }
    }

    /// ResolveBinding's Object Record HasBinding is a plain HasProperty: the
    /// global record has `[[IsWithEnvironment]] = false` and never observes
    /// `Symbol.unscopables`. A miss is an unresolvable Reference whose GetValue
    /// throws before the sealed operation can evaluate its RHS.
    #[must_use]
    pub(crate) fn compound_assignment(self, assignment: EagerCompoundAssignment) -> TypedExpr {
        let Self {
            binding_object,
            referenced_name,
            strictness,
        } = self;
        let present = binding_object.has_property(&referenced_name);
        let selected =
            binding_object.eager_compound_assignment(&referenced_name, strictness, &assignment);
        let result_info = selected.value_info();
        let missing = TypedExpr::from_info(
            result_info.clone(),
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
            },
        );
        TypedExpr::from_info(
            result_info,
            ExprIr::Conditional {
                condition: Box::new(present),
                then_expr: Box::new(selected),
                else_expr: Box::new(missing),
            },
        )
    }

    /// Consume one global Object Record Reference for `++`/`--`. The initial
    /// plain HasProperty happens before the shared GetValue/ToNumeric/delta/
    /// PutValue lifecycle, and a miss throws before ToNumeric in both modes.
    #[must_use]
    pub(crate) fn numeric_update(
        self,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        bindings: NumericUpdateBindings,
    ) -> TypedExpr {
        let Self {
            binding_object,
            referenced_name,
            strictness,
        } = self;
        let present = binding_object.has_property(&referenced_name);
        let selected =
            binding_object.numeric_update(&referenced_name, strictness, op, return_mode, &bindings);
        let result_info = selected.value_info();
        let missing = TypedExpr::from_info(
            result_info.clone(),
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
            },
        );
        TypedExpr::from_info(
            result_info,
            ExprIr::Conditional {
                condition: Box::new(present),
                then_expr: Box::new(selected),
                else_expr: Box::new(missing),
            },
        )
    }

    /// Consume one global Object Record Reference for `&&=`/`||=`/`??=`. An
    /// initial miss throws before the RHS; a short circuit never enters the
    /// shared PutValue branch.
    #[must_use]
    pub(crate) fn logical_assignment(self, op: LogicalBinaryOp, rhs: TypedExpr) -> TypedExpr {
        let Self {
            binding_object,
            referenced_name,
            strictness,
        } = self;
        let present = binding_object.has_property(&referenced_name);
        let selected = binding_object.logical_assignment(&referenced_name, strictness, op, rhs);
        let missing = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
            },
        );
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Conditional {
                condition: Box::new(present),
                then_expr: Box::new(selected),
                else_expr: Box::new(missing),
            },
        )
    }
}

/// Compiler-private bindings used by one Object Environment numeric update.
///
/// All three roles are `String`, so accepting them positionally would allow a
/// result/write transposition to compile. The constructor allocates the roles
/// in one fixed order and the fields remain private to this module.
#[derive(Debug)]
pub(crate) struct NumericUpdateBindings {
    old_value: String,
    result: String,
    write: String,
}

/// Compiler-private bindings for one eager Object Environment compound
/// assignment.
///
/// The old-value, result and write-completion roles intentionally cannot be
/// supplied as three positional `String`s. The sole allocator fixes their
/// meanings, [`Self::old_value`] is the only old-value operand exposed to the
/// lowerer, and [`Self::seal`] consumes the carrier before the Reference plan
/// accepts the operation.
#[derive(Debug)]
#[must_use = "eager compound-assignment bindings must be sealed into an operation"]
pub(crate) struct EagerCompoundAssignmentBindings {
    old_value: String,
    result: String,
    write: String,
}

impl EagerCompoundAssignmentBindings {
    pub(crate) fn allocate(mut allocate: impl FnMut(&str) -> String) -> Self {
        Self {
            old_value: allocate("object.environment.compound.old."),
            result: allocate("object.environment.compound.result."),
            write: allocate("object.environment.compound.write."),
        }
    }

    /// The dynamically obtained GetBindingValue result. Returning the operand
    /// from the role carrier prevents a caller from spelling the old binding
    /// name independently of the names the consuming plan will materialize.
    pub(crate) fn old_value(&self) -> TypedExpr {
        TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::Identifier(self.old_value.clone()),
        )
    }

    pub(crate) fn seal(self, result: TypedExpr) -> EagerCompoundAssignment {
        EagerCompoundAssignment {
            bindings: self,
            result,
        }
    }
}

/// One eager operation result sealed to the private bindings which establish
/// its GetValue/apply/PutValue/result lifecycle.
#[derive(Debug)]
#[must_use = "a sealed eager compound assignment must consume its Reference plan"]
pub(crate) struct EagerCompoundAssignment {
    bindings: EagerCompoundAssignmentBindings,
    result: TypedExpr,
}

impl NumericUpdateBindings {
    pub(crate) fn allocate(mut allocate: impl FnMut(&str) -> String) -> Self {
        Self {
            old_value: allocate("object.environment.update.old."),
            result: allocate("object.environment.update.result."),
            write: allocate("object.environment.update.write."),
        }
    }
}

impl WithEnvironmentReferencePlan {
    fn create(
        innermost: WithEnvironmentResolution,
        outer: Vec<WithEnvironmentResolution>,
        referenced_name: String,
        strictness: Strictness,
    ) -> Self {
        Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        }
    }

    #[must_use]
    pub(crate) fn get_value(self, fallback: TypedExpr) -> TypedExpr {
        let Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.get_value_or_else(&referenced_name, strictness, resolved);
        }
        innermost.get_value_or_else(&referenced_name, strictness, resolved)
    }

    #[must_use]
    pub(crate) fn put_value(self, value: TypedExpr, fallback: TypedExpr) -> TypedExpr {
        let Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.put_value_or_else(
                &referenced_name,
                strictness,
                value.clone(),
                resolved,
            );
        }
        innermost.put_value_or_else(&referenced_name, strictness, value, resolved)
    }

    /// Consume one ResolveBinding result for `++`/`--`. The required private
    /// materializations keep the selected GetValue, numeric delta, same-base
    /// PutValue and returned prefix/postfix result in their specified order.
    #[must_use]
    pub(crate) fn numeric_update(
        self,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        bindings: NumericUpdateBindings,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.numeric_update_or_else(
                &referenced_name,
                strictness,
                op,
                return_mode,
                &bindings,
                resolved,
            );
        }
        innermost.numeric_update_or_else(
            &referenced_name,
            strictness,
            op,
            return_mode,
            &bindings,
            resolved,
        )
    }

    /// Consume one ResolveBinding result for logical assignment. Each Object
    /// Environment candidate owns an independent selection condition, while
    /// the RHS and PutValue remain inside only its taken short-circuit branch.
    #[must_use]
    pub(crate) fn logical_assignment(
        self,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.logical_assignment_or_else(
                &referenced_name,
                strictness,
                op,
                rhs.clone(),
                resolved,
            );
        }
        innermost.logical_assignment_or_else(&referenced_name, strictness, op, rhs, resolved)
    }

    /// Consume one ResolveBinding result for an eager compound assignment.
    /// Every selected branch performs GetValue, evaluates/applies the RHS,
    /// completes same-base PutValue, and only then exposes the applied result.
    #[must_use]
    pub(crate) fn compound_assignment(
        self,
        assignment: EagerCompoundAssignment,
        fallback: TypedExpr,
    ) -> TypedExpr {
        let Self {
            innermost,
            outer,
            referenced_name,
            strictness,
        } = self;
        let mut resolved = fallback;
        for environment in outer {
            resolved = environment.compound_assignment_or_else(
                &referenced_name,
                strictness,
                &assignment,
                resolved,
            );
        }
        innermost.compound_assignment_or_else(&referenced_name, strictness, &assignment, resolved)
    }
}

/// `[[Strict]]` of a Reference Record (6.2.5).
///
/// Deliberately not a `bool`. The consumers of this field — PutValue 2.a
/// (unresolvable write in strict code is a ReferenceError), PutValue 3.d (a
/// `[[Set]]` that answered `false` is a TypeError in strict code) and
/// `delete` 5.e (a `[[Delete]]` that answered `false` is a TypeError in
/// strict code) — all sit next to other boolean parameters (`implicit`,
/// `configurable`, `succeeded`), and a transposition between them is silent
/// under `bool`.
///
/// Prohibited on purpose, each for a defect this lane is closing:
///
/// - no `Default`: there is no defensible default; the decision is forced at
///   [`crate::lowering`]'s single producer instead;
/// - no `From<bool>`/`Into<bool>`: that reintroduces the transposition with
///   one extra keystroke;
/// - no `PartialOrd`/`Ord`: `strictness > other` means nothing;
/// - no `#[repr(u8)]`/`as` casts: `strictness as i64` at an `I64Const` site is
///   how the wrong flag word reached an outlined helper once already.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strictness {
    Sloppy,
    Strict,
}

impl Strictness {
    /// PutValue 2.a / 3.d and `delete` 5.e: does an observed failure throw?
    ///
    /// This is the only exit from the type, and it is named for the spec
    /// obligation rather than for the mode, so a call site reads as spec text
    /// and a `bool` parameter it is handed to is visibly a *result*, never the
    /// mode itself.
    #[must_use]
    pub fn throws_on_failed_set(self) -> bool {
        matches!(self, Self::Strict)
    }

    /// The runtime flag word the backend's object-write guards read for
    /// PutValue 3.d, when the guard has to be selected at run time because the
    /// write body is shared between callers of both modes.
    ///
    /// This is the single named conversion to a machine word. It exists so
    /// that an `I64Const` at a write site cannot be fed anything else: there
    /// is no `as` cast and no `repr`, so `strictness as i64` does not compile
    /// and `i64::from(some_other_bool)` is visibly not this function.
    #[must_use]
    pub fn helper_flag_word(self) -> i64 {
        i64::from(self.throws_on_failed_set())
    }
}

/// An ordinary property Reference evaluated before a generator suspension and
/// consumed by PutValue only after a normal resume.
///
/// Fields are private so a resume mode cannot carry a base/key pair while
/// dropping the Reference's `[[Strict]]`. For an ordinary property Reference,
/// 6.2.5.3 makes `[[Base]]` the receiver too; storing one value makes a
/// disagreement between them unrepresentable. A future Super Reference needs
/// a distinct use-view variant with its separate `[[ThisValue]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a suspended property Reference must be consumed on normal resume"]
pub struct SuspendedPropertyReferenceIr {
    base_and_receiver: Box<TypedExpr>,
    key: PropertyKeyIr,
    strictness: Strictness,
}

/// The exhaustive operation a backend performs with a suspended property
/// Reference. This borrowed view exposes no constructor for invalid records.
#[derive(Debug, Clone, Copy)]
pub enum SuspendedPropertyReferenceUse<'a> {
    Ordinary {
        base_and_receiver: &'a TypedExpr,
        key: &'a PropertyKeyIr,
        strictness: Strictness,
    },
}

impl SuspendedPropertyReferenceIr {
    pub(crate) fn ordinary(
        base_and_receiver: Box<TypedExpr>,
        key: PropertyKeyIr,
        strictness: Strictness,
    ) -> Self {
        Self {
            base_and_receiver,
            key,
            strictness,
        }
    }

    #[must_use]
    pub fn use_view(&self) -> SuspendedPropertyReferenceUse<'_> {
        SuspendedPropertyReferenceUse::Ordinary {
            base_and_receiver: &self.base_and_receiver,
            key: &self.key,
            strictness: self.strictness,
        }
    }
}

/// A validated identifier Reference whose PutValue is deferred until a
/// destructuring assignment has produced the value to write.
///
/// The representation is private because the old IR encoded the same domain
/// as `name: String` plus three independent booleans (`global`, `implicit`,
/// `immutable`). That admitted combinations which are not References at all —
/// an implicit local, an immutable global — and let the backend silently drop
/// both `implicit` and the Reference's `[[Strict]]`. The only constructors are
/// the outcomes of `ResolveBinding` in the lowerer.
///
/// This is deliberately narrower than [`ReferenceRecord`]. Ordinary property
/// assignment can be read and written in one lowering step, while a
/// destructuring identifier target must survive until the iterator/property
/// value and any default initializer have been evaluated. Keeping this type
/// write-only avoids inventing a second general Reference hierarchy.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a validated identifier Reference must be consumed by its deferred write"]
pub struct IdentifierWriteReferenceIr {
    base: IdentifierWriteReferenceBaseIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IdentifierWriteReferenceBaseIr {
    MutableBinding {
        storage_name: String,
    },
    IgnoredImmutableBinding {
        referenced_name: String,
    },
    Abrupt {
        referenced_name: String,
        error: IdentifierWriteErrorIr,
    },
    Global {
        referenced_name: String,
        strictness: Strictness,
    },
}

/// The exhaustive action a backend performs when PutValue consumes an
/// [`IdentifierWriteReferenceIr`].
///
/// This is a borrowed view rather than public fields on the stored IR type, so
/// another crate can consume a validated Reference but cannot manufacture an
/// impossible combination of flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierWriteDisposition<'a> {
    MutableBinding {
        storage_name: &'a str,
    },
    /// SetMutableBinding step 6.b on the non-strict binding of a named function
    /// expression: the write is ignored after its operand has been evaluated.
    IgnoreImmutableBinding,
    Throw {
        error: IdentifierWriteErrorIr,
    },
    /// The global object or an unresolvable Reference. Which one exists is a
    /// runtime fact; `strictness` selects PutValue step 2.a and 3.d.
    Global {
        referenced_name: &'a str,
        strictness: Strictness,
    },
}

/// The closed set of abrupt outcomes for a deferred identifier PutValue.
///
/// Keeping the error and message here gives ordinary and destructuring writes
/// one semantic spelling instead of a backend-only `immutable: bool` branch
/// with its own diagnostic text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierWriteErrorIr {
    UninitializedBinding,
    ImmutableBinding,
    ImmutableClassName,
}

impl IdentifierWriteErrorIr {
    #[must_use]
    pub fn kind(self) -> NativeErrorKind {
        match self {
            Self::UninitializedBinding => NativeErrorKind::ReferenceError,
            Self::ImmutableBinding | Self::ImmutableClassName => NativeErrorKind::TypeError,
        }
    }

    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::UninitializedBinding => "lexical binding accessed before initialization",
            Self::ImmutableBinding => "assignment to immutable binding",
            Self::ImmutableClassName => "assignment to immutable class name",
        }
    }
}

impl IdentifierWriteReferenceIr {
    pub(crate) fn mutable_binding(storage_name: String) -> Self {
        Self {
            base: IdentifierWriteReferenceBaseIr::MutableBinding { storage_name },
        }
    }

    pub(crate) fn ignored_immutable_binding(referenced_name: String) -> Self {
        Self {
            base: IdentifierWriteReferenceBaseIr::IgnoredImmutableBinding { referenced_name },
        }
    }

    pub(crate) fn immutable_binding(referenced_name: String) -> Self {
        Self::abrupt(referenced_name, IdentifierWriteErrorIr::ImmutableBinding)
    }

    pub(crate) fn immutable_class_name(referenced_name: String) -> Self {
        Self::abrupt(referenced_name, IdentifierWriteErrorIr::ImmutableClassName)
    }

    /// Defers 9.1.1.1.5 step 3 until destructuring reaches PutValue.
    ///
    /// Requiring and consuming the unforgeable [`crate::TdzViolation`] is the
    /// link to `ResolveBinding`: the lowerer cannot spell this outcome after
    /// skipping the binding lifecycle check, and it cannot discard the witness
    /// in the one destructuring arm which previously did exactly that.
    pub(crate) fn uninitialized_binding(
        referenced_name: String,
        _violation: crate::TdzViolation,
    ) -> Self {
        Self::abrupt(
            referenced_name,
            IdentifierWriteErrorIr::UninitializedBinding,
        )
    }

    pub(crate) fn global(referenced_name: String, strictness: Strictness) -> Self {
        Self {
            base: IdentifierWriteReferenceBaseIr::Global {
                referenced_name,
                strictness,
            },
        }
    }

    fn abrupt(referenced_name: String, error: IdentifierWriteErrorIr) -> Self {
        Self {
            base: IdentifierWriteReferenceBaseIr::Abrupt {
                referenced_name,
                error,
            },
        }
    }

    /// The source/storage name, for analyses such as string-pool collection
    /// which must not infer the Reference kind from the spelling.
    #[must_use]
    pub fn name(&self) -> &str {
        match &self.base {
            IdentifierWriteReferenceBaseIr::MutableBinding { storage_name } => storage_name,
            IdentifierWriteReferenceBaseIr::IgnoredImmutableBinding { referenced_name }
            | IdentifierWriteReferenceBaseIr::Abrupt {
                referenced_name, ..
            }
            | IdentifierWriteReferenceBaseIr::Global {
                referenced_name, ..
            } => referenced_name,
        }
    }

    /// PutValue's exhaustive, already-validated action.
    #[must_use]
    pub fn write_disposition(&self) -> IdentifierWriteDisposition<'_> {
        match &self.base {
            IdentifierWriteReferenceBaseIr::MutableBinding { storage_name } => {
                IdentifierWriteDisposition::MutableBinding { storage_name }
            }
            IdentifierWriteReferenceBaseIr::IgnoredImmutableBinding { .. } => {
                IdentifierWriteDisposition::IgnoreImmutableBinding
            }
            IdentifierWriteReferenceBaseIr::Abrupt { error, .. } => {
                IdentifierWriteDisposition::Throw { error: *error }
            }
            IdentifierWriteReferenceBaseIr::Global {
                referenced_name,
                strictness,
            } => IdentifierWriteDisposition::Global {
                referenced_name,
                strictness: *strictness,
            },
        }
    }
}

/// Which spec error a *failed* PutValue on this node raises, when the
/// Reference's `[[Strict]]` is `Strict`.
///
/// PutValue has two strict throws and they are **different error types**:
/// step 2.a raises a **ReferenceError** when the Reference is unresolvable,
/// and step 3.d a **TypeError** when `[[Set]]` answered `false`. Attributing
/// one where the other can occur is not a rounding error — it decides the
/// inferred shape of a `catch` binding, and with it every static answer to
/// `e.name`, `e.constructor` and a prototype lookup on `e`.
///
/// Closed, and matched exhaustively at the consumer, so a third failure mode
/// has to be given a merge rule rather than defaulting to one of these two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutValueFailure {
    /// PutValue 3.d, or `delete` 5.e. The base is resolved, so 2.a is
    /// unreachable and a TypeError is the only outcome.
    TypeErrorOnly,
    /// PutValue 2.a **or** 3.d. [`ReferenceBase::Global`] covers both "the
    /// global object" and "unresolvable" because the two are not separable at
    /// compile time — `globalThis.x = 1` can create the binding at run time —
    /// so a strict failure here is a ReferenceError *or* a TypeError and the
    /// consumer must admit both.
    TypeErrorOrReferenceError,
}

/// Which `[[Strict]]` (if any) an IR node carries, and what its strict failure
/// raises — as a total function of `ExprIr`.
///
/// This match has no catch-all, and that is its entire purpose: whoever adds a
/// new reference-shaped variant to [`ExprIr`] gets `E0004` here and has to
/// decide, at that moment, whether the new node is a PutValue site **and which
/// of PutValue's two strict throws it can reach**. A `_` arm would turn "the
/// new write node has nowhere to record `[[Strict]]`" into a silently sloppy
/// write; a bare `Option<Strictness>` return turned "the new global-write node
/// forgot that 2.a is a ReferenceError" into a wrong catch-binding shape.
#[must_use]
pub fn carried_put_value_failure(expr: &ExprIr) -> Option<(Strictness, PutValueFailure)> {
    match expr {
        // PutValue 2.a **and** 3.d: the base is the global object or
        // unresolvable, and which one is a runtime fact.
        ExprIr::GlobalPropertyWrite { strictness, .. }
        | ExprIr::GlobalPropertyUpdate { strictness, .. }
        | ExprIr::GlobalPropertyCompoundAssign { strictness, .. } => {
            Some((*strictness, PutValueFailure::TypeErrorOrReferenceError))
        }

        // PutValue 3.d, and `delete` 5.e. `DeleteGlobalProperty` is here and
        // not above on purpose: `delete` 4.a is an *assertion* that
        // `[[Strict]]` is false for an unresolvable Reference (13.5.1.1 makes
        // `delete <identifier>` an early SyntaxError in strict code), so the
        // ReferenceError branch cannot arise for a delete.
        ExprIr::PropertyWrite { strictness, .. }
        | ExprIr::SuperPropertyWrite { strictness, .. }
        | ExprIr::DeleteProperty { strictness, .. }
        | ExprIr::DeleteGlobalProperty { strictness, .. } => {
            Some((*strictness, PutValueFailure::TypeErrorOnly))
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            Some((mutation.strictness(), PutValueFailure::TypeErrorOnly))
        }
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            Some((assignment.strictness(), PutValueFailure::TypeErrorOnly))
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            Some((assignment.strictness(), PutValueFailure::TypeErrorOnly))
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) => {
            Some((assignment.strictness(), PutValueFailure::TypeErrorOnly))
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            Some((update.strictness(), PutValueFailure::TypeErrorOnly))
        }

        // Everything else. `AssignIdentifier`, `CompoundAssignIdentifier` and
        // `UpdateIdentifier` are here on purpose and not by omission: their
        // PutValue branch is 4.c, SetMutableBinding, whose only
        // mode-dependent step is 6.b (immutable binding). Whether a resolved
        // binding is immutable is decided at lowering time, so the strictness
        // never has to survive into the IR. `PrivateWrite` is here because
        // PrivateSet throws unconditionally on failure, and
        // `OptionalPropertyChain` because 13.3.9.1 makes an optional chain an
        // early error as an assignment target.
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::NewTarget
        | ExprIr::Boolean(..)
        | ExprIr::Number(..)
        | ExprIr::BigInt(..)
        | ExprIr::String(..)
        | ExprIr::FunctionValue(..)
        | ExprIr::ObjectLiteral(..)
        | ExprIr::ArrayLiteral(..)
        | ExprIr::ArrayAccumulation(..)
        | ExprIr::Identifier(..)
        | ExprIr::TemplateObject(..)
        | ExprIr::SpreadArgument(..)
        | ExprIr::ClassDefinition(..)
        | ExprIr::Symbol { .. }
        | ExprIr::RegExpLiteral { .. }
        | ExprIr::DynamicImport { .. }
        | ExprIr::ImportMeta { .. }
        | ExprIr::ModuleNamespace { .. }
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
        | ExprIr::AssignIdentifier { .. }
        | ExprIr::PropertyRead { .. }
        | ExprIr::OptionalPropertyChain { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::UnaryPlus { .. }
        | ExprIr::UnaryMinusNumeric { .. }
        | ExprIr::UnaryBitwiseNumeric { .. }
        | ExprIr::Void { .. }
        | ExprIr::DeleteValue { .. }
        | ExprIr::DeleteIdentifier { .. }
        | ExprIr::TypeOf { .. }
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::LogicalNot { .. }
        | ExprIr::SpecOperation { .. }
        | ExprIr::BinaryNumber { .. }
        | ExprIr::CoerciveAdd { .. }
        | ExprIr::CoerciveBinaryNumber { .. }
        | ExprIr::BitwiseNumeric { .. }
        | ExprIr::StringFromCharCode { .. }
        | ExprIr::StringCharCodeAt { .. }
        | ExprIr::StringConcat { .. }
        | ExprIr::CompareNumber { .. }
        | ExprIr::CompareValue { .. }
        | ExprIr::StrictEquality { .. }
        | ExprIr::LooseEquality { .. }
        | ExprIr::LogicalShortCircuit { .. }
        | ExprIr::Conditional { .. }
        | ExprIr::Comma { .. }
        | ExprIr::MaterializeBinding { .. }
        | ExprIr::ArrayDestructure { .. }
        | ExprIr::ObjectDestructure { .. }
        | ExprIr::CallNamed { .. }
        | ExprIr::AssertSameValue { .. }
        | ExprIr::RuntimeThrow { .. }
        | ExprIr::CallIndirect { .. }
        | ExprIr::JsonParseStaticReviver { .. }
        | ExprIr::Construct { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. }
        | ExprIr::InstanceOf { .. }
        | ExprIr::In { .. } => None,
    }
}

/// The closed partition of `[[Base]]` from 6.2.5.1, refined by what this
/// compiler can prove at lowering time.
///
/// There is no `Binding` variant. PutValue branch 4 (an Environment Record
/// base) is discharged before any Reference is reified — see
/// `carried_put_value_failure`'s `None` arm — so a variant for it would be
/// constructed nowhere and matched everywhere.
#[derive(Debug)]
pub(crate) enum ReferenceBase {
    /// `IsPropertyReference` true, `IsPrivateReference` false,
    /// `IsSuperReference` false.
    Property {
        target: TypedExpr,
        key: PropertyKeyIr,
    },
    /// `IsPrivateReference` true. PutValue 3.b: PrivateSet throws
    /// unconditionally on failure, so no `[[Strict]]` reaches the IR.
    Private {
        target: TypedExpr,
        private_name_id: PrivateNameId,
    },
    /// `IsSuperReference` true.
    Super {
        key: PropertyKeyIr,
        receiver: TypedExpr,
    },
    /// `[[Base]]` is unresolvable, or is the global object. The two are not
    /// separable at compile time, so they share a variant and the backend
    /// performs the presence test that PutValue 2.a needs.
    Global { name: String },
}

impl ReferenceBase {
    /// GetValue (6.2.5.5), as an IR node. Borrows, because 13.15.2 needs
    /// GetValue *and then* PutValue on the same record.
    pub(crate) fn read_ir(&self) -> ExprIr {
        match self {
            Self::Property { target, key } => ExprIr::PropertyRead {
                target: Box::new(target.clone()),
                key: key.clone(),
            },
            Self::Private {
                target,
                private_name_id,
            } => ExprIr::PrivateRead {
                target: Box::new(target.clone()),
                private_name_id: *private_name_id,
            },
            Self::Super { key, receiver } => ExprIr::SuperPropertyRead {
                key: key.clone(),
                receiver: Box::new(receiver.clone()),
            },
            Self::Global { name } => ExprIr::GlobalPropertyRead { name: name.clone() },
        }
    }

    /// PutValue (6.2.5.6), as an IR node. Consumes the base, so the same base
    /// cannot be written twice.
    fn write_ir(self, value: TypedExpr, strictness: Strictness) -> ExprIr {
        match self {
            Self::Property { target, key } => ExprIr::PropertyWrite {
                target: Box::new(target),
                key,
                value: Box::new(value),
                strictness,
            },
            // PutValue 3.b. PrivateSet has no `[[Strict]]` parameter.
            Self::Private {
                target,
                private_name_id,
            } => ExprIr::PrivateWrite {
                target: Box::new(target),
                private_name_id,
                value: Box::new(value),
            },
            Self::Super { key, receiver } => ExprIr::SuperPropertyWrite {
                key,
                receiver: Box::new(receiver),
                value: Box::new(value),
                strictness,
            },
            Self::Global { name } => ExprIr::GlobalPropertyWrite {
                name,
                value: Box::new(value),
                implicit: false,
                strictness,
            },
        }
    }

    /// The `[[Base]]` operand that must not be evaluated twice, when there is
    /// one. A super base is loaded from the home object rather than evaluated,
    /// and a global base is a name, so neither can be re-evaluated at all.
    pub(crate) fn evaluated_base_mut(&mut self) -> Option<&mut TypedExpr> {
        match self {
            Self::Property { target, .. } | Self::Private { target, .. } => Some(target),
            Self::Super { .. } | Self::Global { .. } => None,
        }
    }

    /// The computed-key operand that must not be evaluated twice, when there
    /// is one.
    pub(crate) fn computed_key_mut(&mut self) -> Option<&mut TypedExpr> {
        let key = match self {
            Self::Property { key, .. } | Self::Super { key, .. } => key,
            Self::Private { .. } | Self::Global { .. } => return None,
        };
        match key {
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => Some(&mut **expr),
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => None,
        }
    }
}

/// Why a lowered read is not usable as a Reference. Closed on purpose: there
/// is no `Other`, so each way of failing has to be named and given a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsupportedTarget {
    /// The lowering specialised this read into a shape that is not a writable
    /// Reference (a folded constant, an intrinsic value, ...).
    NotAReferenceRead,
    /// A `Get`/`GetV` spec operation with an operand count other than the
    /// (target, key) pair the accessor form requires.
    MalformedPropertyAccessor,
}

impl UnsupportedTarget {
    /// The feature string for `unsupported_expr`. Exhaustive, so a new way of
    /// failing cannot silently reuse another's message.
    pub(crate) fn feature(self) -> &'static str {
        match self {
            Self::NotAReferenceRead => "unsupported property assignment operator",
            Self::MalformedPropertyAccessor => "property accessor reference operands",
        }
    }
}

/// Recovers the Reference a lowered read denotes.
///
/// This match is exhaustive over all 77 `ExprIr` variants with no catch-all.
/// That is what stops the historical failure mode: a new read specialisation
/// added anywhere in the lowering used to fall into a `_` arm here and
/// silently remove compound assignment and `++` for that shape, with nothing
/// to compile-error about it. Now the author of the new variant is asked.
pub(crate) fn reference_base_of_lowered_read(
    expr: ExprIr,
) -> Result<ReferenceBase, UnsupportedTarget> {
    match expr {
        ExprIr::PropertyRead { target, key } => Ok(ReferenceBase::Property {
            target: *target,
            key,
        }),
        ExprIr::PrivateRead {
            target,
            private_name_id,
        } => Ok(ReferenceBase::Private {
            target: *target,
            private_name_id,
        }),
        ExprIr::SuperPropertyRead { key, receiver } => Ok(ReferenceBase::Super {
            key,
            receiver: *receiver,
        }),
        // `globalThis.x` on a known global resolves to the global binding
        // itself rather than to a property of an object.
        ExprIr::GlobalPropertyRead { name } | ExprIr::GlobalIdentifierRead { name } => {
            Ok(ReferenceBase::Global { name })
        }
        // A dynamic key lowers the read to Get/GetV; the same base and key
        // written back is an ordinary property write. 13.3.3.1 evaluates the
        // key *expression* here and defers ToPropertyKey to the consumer, so
        // the operand is pinned as it stands and not coerced.
        ExprIr::SpecOperation {
            operation: SpecOperationIr::Get | SpecOperationIr::GetV,
            operands,
        } => {
            let mut operands = operands.into_iter();
            let (Some(target), Some(key), None) =
                (operands.next(), operands.next(), operands.next())
            else {
                return Err(UnsupportedTarget::MalformedPropertyAccessor);
            };
            let key = match key {
                TypedExpr {
                    kind: ValueKind::String,
                    expr: ExprIr::String(name),
                    ..
                } => PropertyKeyIr::StaticString(name),
                // Well-known Symbol values deliberately use the same
                // `ExprIr::String(description)` payload shape. Their typed
                // `ValueKind::Symbol` is the identity boundary, so preserve
                // the whole operand for the backend to add the symbol marker.
                key => PropertyKeyIr::StringExpr(Box::new(key)),
            };
            Ok(ReferenceBase::Property { target, key })
        }

        ExprIr::SpecOperation { .. }
        | ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::NewTarget
        | ExprIr::Boolean(..)
        | ExprIr::Number(..)
        | ExprIr::BigInt(..)
        | ExprIr::String(..)
        | ExprIr::FunctionValue(..)
        | ExprIr::ObjectLiteral(..)
        | ExprIr::ArrayLiteral(..)
        | ExprIr::ArrayAccumulation(..)
        | ExprIr::Identifier(..)
        | ExprIr::TemplateObject(..)
        | ExprIr::SpreadArgument(..)
        | ExprIr::ClassDefinition(..)
        | ExprIr::Symbol { .. }
        | ExprIr::RegExpLiteral { .. }
        | ExprIr::DynamicImport { .. }
        | ExprIr::ImportMeta { .. }
        | ExprIr::ModuleNamespace { .. }
        | ExprIr::AssignIdentifier { .. }
        | ExprIr::GlobalPropertyWrite { .. }
        | ExprIr::OptionalPropertyChain { .. }
        | ExprIr::PropertyWrite { .. }
        | ExprIr::OrdinaryPropertyAssignment(_)
        | ExprIr::OrdinaryPropertyLogicalAssignment(_)
        | ExprIr::OrdinaryPropertyNumericUpdate(_)
        | ExprIr::OrdinaryPropertyEagerCompoundAssignment(_)
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::GlobalPropertyCompoundAssign { .. }
        | ExprIr::UnaryPlus { .. }
        | ExprIr::UnaryMinusNumeric { .. }
        | ExprIr::UnaryBitwiseNumeric { .. }
        | ExprIr::Void { .. }
        | ExprIr::DeleteValue { .. }
        | ExprIr::DeleteIdentifier { .. }
        | ExprIr::DeleteGlobalProperty { .. }
        | ExprIr::DeleteProperty { .. }
        | ExprIr::TypeOf { .. }
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::LogicalNot { .. }
        | ExprIr::BinaryNumber { .. }
        | ExprIr::CoerciveAdd { .. }
        | ExprIr::CoerciveBinaryNumber { .. }
        | ExprIr::BitwiseNumeric { .. }
        | ExprIr::StringFromCharCode { .. }
        | ExprIr::StringCharCodeAt { .. }
        | ExprIr::StringConcat { .. }
        | ExprIr::CompareNumber { .. }
        | ExprIr::CompareValue { .. }
        | ExprIr::StrictEquality { .. }
        | ExprIr::LooseEquality { .. }
        | ExprIr::LogicalShortCircuit { .. }
        | ExprIr::Conditional { .. }
        | ExprIr::Comma { .. }
        | ExprIr::MaterializeBinding { .. }
        | ExprIr::ArrayDestructure { .. }
        | ExprIr::ObjectDestructure { .. }
        | ExprIr::CallNamed { .. }
        | ExprIr::AssertSameValue { .. }
        | ExprIr::RuntimeThrow { .. }
        | ExprIr::CallIndirect { .. }
        | ExprIr::JsonParseStaticReviver { .. }
        | ExprIr::Construct { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::SuperPropertyMutation(_)
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. }
        | ExprIr::InstanceOf { .. }
        | ExprIr::In { .. } => Err(UnsupportedTarget::NotAReferenceRead),
    }
}

/// How the write sits inside the surrounding expression.
///
/// Closed and matched exhaustively: a third assignment shape has to be spelled
/// out here rather than fall into a `_` arm that drops the read operand.
#[derive(Debug)]
pub(crate) enum Composition {
    /// The write is the whole expression: `ref = v`, `ref op= v`, `++ref`.
    Value,
    /// 13.15.1 `&&=`, `||=`, `??=`: the write is the short-circuit branch, and
    /// the value of the whole expression is the merge of both branches.
    ShortCircuit {
        op: LogicalBinaryOp,
        read: TypedExpr,
        merged: ValueInfo,
    },
}

/// A Reference Record (6.2.5), reified for the duration of one lowering step.
///
/// Deliberately neither `Clone` nor `Copy`: 13.15.2 evaluates the
/// LeftHandSideExpression once and the record it produced is the record both
/// GetValue and PutValue consume. [`Self::write`] takes `self` by value, so a
/// second write of the same Reference is `E0382 use of moved value` rather
/// than a duplicated evaluation of an effectful base or computed key.
#[derive(Debug)]
#[must_use = "a Reference Record that is neither read nor written has evaluated \
              its base and key for nothing"]
pub(crate) struct ReferenceRecord {
    base: ReferenceBase,
    strictness: Strictness,
}

impl ReferenceRecord {
    /// The only constructor. Taking [`Strictness`] rather than `bool` is what
    /// makes "forgot to ask which mode created this Reference" impossible to
    /// express: there is no other value of that type in scope at a call site.
    pub(crate) fn create(base: ReferenceBase, strictness: Strictness) -> Self {
        Self { base, strictness }
    }

    pub(crate) fn base(&self) -> &ReferenceBase {
        &self.base
    }

    /// Pins the operands PutValue must not re-evaluate (13.15.2 obligation O1),
    /// and returns the only [`ReferencePins`] that exists for this Reference.
    ///
    /// This is the **sole producer** of `ReferencePins`, and it needs a record
    /// to be called on, so no code path can hold a bare pin chain — which is
    /// what made `ReferencePins::none().materialize(record.write(..))`
    /// type-check while silently discarding the real chain. It replaces a
    /// `base_mut() -> &mut ReferenceBase` accessor whose doc comment claimed
    /// "the shape of the base cannot be changed through this, only its
    /// operands" while `*record.base_mut() = ReferenceBase::Global { name }`
    /// compiled and swapped a property Reference for a global one *after* its
    /// `[[Strict]]` had been chosen.
    ///
    /// `pin` returns `Some((name, value))` when it hoisted the operand into a
    /// temporary binding, having replaced the operand in place with a reference
    /// to it. Base before key: 13.3.3.1 evaluates the base, then the key
    /// expression.
    pub(crate) fn pin_operands(
        &mut self,
        mut pin: impl FnMut(ReferenceOperand, &mut TypedExpr) -> Option<(String, TypedExpr)>,
    ) -> ReferencePins {
        let mut pins = ReferencePins(Vec::new());
        if let Some(target) = self.base.evaluated_base_mut() {
            if let Some(pinned) = pin(ReferenceOperand::Base, target) {
                pins.0.push(pinned);
            }
        }
        if let Some(key) = self.base.computed_key_mut() {
            if let Some(pinned) = pin(ReferenceOperand::ComputedKey, key) {
                pins.0.push(pinned);
            }
        }
        pins
    }

    /// GetValue (6.2.5.5). Borrows: 13.15.2 step 2 reads and step 6 writes the
    /// *same* record.
    pub(crate) fn read(&self, info: ValueInfo) -> TypedExpr {
        TypedExpr::from_info(info, self.base.read_ir())
    }

    /// PutValue (6.2.5.6). Consumes the record.
    ///
    /// The result is a [`PendingReferenceWrite`], not a [`TypedExpr`], because
    /// the pins this Reference created still have to be materialised around
    /// it — see [`ReferencePins::materialize`].
    pub(crate) fn write(self, value: TypedExpr, compose: Composition) -> PendingReferenceWrite {
        let Self { base, strictness } = self;
        let info = value.value_info();
        let write = TypedExpr::from_info(info, base.write_ir(value, strictness));
        match compose {
            Composition::Value => PendingReferenceWrite(write),
            Composition::ShortCircuit { op, read, merged } => {
                PendingReferenceWrite(TypedExpr::from_info(
                    merged,
                    ExprIr::LogicalShortCircuit {
                        op,
                        lhs: Box::new(read),
                        rhs: Box::new(write),
                    },
                ))
            }
        }
    }
}

/// A written Reference whose pins have not been materialised around it yet.
///
/// No public field, no `Deref`, no `From`/`Into<TypedExpr>`: the only exit is
/// [`ReferencePins::materialize`], so "built the write, forgot to wrap it in
/// its `MaterializeBinding` chain" is `E0308 mismatched types` at the point
/// the `TypedExpr` was wanted.
#[derive(Debug)]
#[must_use = "a pending Reference write must be materialised through its ReferencePins"]
pub(crate) struct PendingReferenceWrite(TypedExpr);

/// Which operand of a Reference is being pinned.
///
/// The record knows *which operands exist* (a super base is loaded from the
/// home object, a global base is a name, a static key is not an expression);
/// the caller knows what to name the temporary. Closed, so a third pinnable
/// operand has to be given a name at the call site rather than reusing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReferenceOperand {
    /// `[[Base]]`, when it is an evaluated expression.
    Base,
    /// The computed key expression, before `ToPropertyKey`.
    ComputedKey,
}

/// The temporaries this Reference's evaluation pinned, innermost last.
///
/// Not `Clone`, and the only consumer is [`Self::materialize`], so a chain
/// cannot be emitted twice (`E0382`) or dropped on the floor (`#[must_use]`).
///
/// No `Default`, no `none()`, and a private field: the only way to obtain one
/// is [`ReferenceRecord::pin_operands`], which needs the record it belongs to.
/// `#[derive(Default)]` on a `pub(crate)` type and a public `none()` were both
/// constructors for an *empty* chain that type-checks in
/// `ReferencePins::none().materialize(record.write(v, compose))` — leaving the
/// real chain bound to an unused local, which is a warning at most and not even
/// that once it is passed anywhere. Ledger L3 shrinks accordingly.
#[derive(Debug)]
#[must_use = "unmaterialised pins mean the Reference's base or key is evaluated twice"]
pub(crate) struct ReferencePins(Vec<(String, TypedExpr)>);

impl ReferencePins {
    /// The single exit: wraps the write in this Reference's pin chain,
    /// innermost pin last, so the outermost `MaterializeBinding` is the first
    /// operand that was pinned and evaluation order matches 13.3.3.1 (base,
    /// then key expression).
    pub(crate) fn materialize(self, write: PendingReferenceWrite) -> TypedExpr {
        let mut result = write.0;
        for (name, value) in self.0.into_iter().rev() {
            let info = result.value_info();
            result = TypedExpr::from_info(
                info,
                ExprIr::MaterializeBinding {
                    name,
                    value: Box::new(value),
                    body: Box::new(result),
                },
            );
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_hook_targets_share_the_source_universe_and_keep_exact_builtins() {
        let source = Arc::new(BTreeSet::from(["f0".to_string(), "f1".to_string()]));
        let builtin = StandardBuiltinId::ObjectPrototypeProtoGetter.function_id();
        let mut targets = PropertyHookTargets::from_known(BTreeSet::from([builtin.clone()]));
        targets.include_all_planned_source(source.clone());
        let cloned = targets.clone();

        assert!(Arc::ptr_eq(
            targets.all_planned_source.as_ref().unwrap(),
            cloned.all_planned_source.as_ref().unwrap(),
        ));
        assert!(cloned.contains(&"f1".to_string()));
        assert!(cloned.contains(&builtin));
        assert!(!cloned.contains(&StandardBuiltinId::ArrayPrototypePush.function_id()));
    }

    fn with_environment_resolution(
        storage_name: &str,
        unscopables_binding: &str,
    ) -> WithEnvironmentResolution {
        WithEnvironmentResolution::create(
            ObjectEnvironmentBindingObject {
                source: ObjectEnvironmentBindingObjectSource::Materialized(
                    storage_name.to_string(),
                ),
                info: ValueInfo::new(ValueKind::Object),
            },
            unscopables_binding.to_string(),
        )
    }

    #[test]
    fn ordered_with_positions_stop_at_current_and_captured_declarative_bindings() {
        let current_outer =
            ObjectEnvironmentPosition::Current(CurrentObjectPosition(CurrentScopeDepth(1)));
        let current_inner =
            ObjectEnvironmentPosition::Current(CurrentObjectPosition(CurrentScopeDepth(2)));
        let current_binding =
            DeclarativeEnvironmentPosition::Current(CurrentBindingPosition(CurrentScopeDepth(2)));
        assert!(!current_outer.precedes(current_binding));
        assert!(current_inner.precedes(current_binding));

        let captured_inner =
            ObjectEnvironmentPosition::Captured(CapturedObjectPosition(CapturedCursorDepth(0)));
        let captured_outer =
            ObjectEnvironmentPosition::Captured(CapturedObjectPosition(CapturedCursorDepth(2)));
        let captured_binding = DeclarativeEnvironmentPosition::Captured(CapturedBindingPosition(
            CapturedCursorDepth(1),
        ));
        assert!(captured_inner.precedes(captured_binding));
        assert!(!captured_outer.precedes(captured_binding));
        assert!(current_inner.precedes(captured_binding));
        assert!(!captured_inner.precedes(current_binding));
    }

    fn identifier(name: &str, kind: ValueKind) -> TypedExpr {
        TypedExpr::from_info(ValueInfo::new(kind), ExprIr::Identifier(name.to_string()))
    }

    fn identifier_name(expr: &TypedExpr) -> &str {
        let ExprIr::Identifier(name) = &expr.expr else {
            panic!("expected an identifier read, got {expr:?}");
        };
        name
    }

    fn has_property_target(expr: &TypedExpr) -> &str {
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::HasProperty,
            operands,
        } = &expr.expr
        else {
            panic!("expected HasProperty, got {expr:?}");
        };
        assert_eq!(operands.len(), 2);
        identifier_name(&operands[0])
    }

    fn initial_resolution_target(expr: &TypedExpr) -> &str {
        let ExprIr::LogicalShortCircuit {
            op: LogicalBinaryOp::And,
            lhs,
            rhs: _,
        } = &expr.expr
        else {
            panic!("expected Object Environment HasBinding, got {expr:?}");
        };
        has_property_target(lhs)
    }

    fn assert_strict_selected_write(
        expr: &TypedExpr,
        object_name: &str,
        expected_value_name: &str,
    ) {
        let ExprIr::MaterializeBinding {
            name: value_name,
            value,
            body,
        } = &expr.expr
        else {
            panic!("RHS must be materialized in the selected branch");
        };
        assert_eq!(value_name, OBJECT_ENVIRONMENT_VALUE_BINDING);
        assert_eq!(identifier_name(value), expected_value_name);

        let ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = &body.expr
        else {
            panic!("strict SetMutableBinding must branch on its post-RHS recheck");
        };
        assert_eq!(has_property_target(condition), object_name);
        let ExprIr::PropertyWrite {
            target,
            key,
            value,
            strictness,
        } = &then_expr.expr
        else {
            panic!("a present binding must reach checked Set");
        };
        assert_eq!(identifier_name(target), object_name);
        assert!(matches!(
            key,
            PropertyKeyIr::StaticString(name) if name == "x"
        ));
        assert_eq!(identifier_name(value), OBJECT_ENVIRONMENT_VALUE_BINDING);
        assert_eq!(*strictness, Strictness::Strict);
        assert!(matches!(
            &else_expr.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
            }
        ));
    }

    fn assert_selected_get_value(expr: &TypedExpr, object_name: &str, strictness: Strictness) {
        let ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = &expr.expr
        else {
            panic!("GetBindingValue must branch on its second HasProperty");
        };
        assert_eq!(has_property_target(condition), object_name);
        let ExprIr::PropertyRead { target, key } = &then_expr.expr else {
            panic!("a present binding must reach Get");
        };
        assert_eq!(identifier_name(target), object_name);
        assert!(matches!(
            key,
            PropertyKeyIr::StaticString(name) if name == "x"
        ));
        match strictness {
            Strictness::Sloppy => assert!(matches!(&else_expr.expr, ExprIr::Undefined)),
            Strictness::Strict => assert!(matches!(
                &else_expr.expr,
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
                }
            )),
        }
    }

    #[test]
    fn with_environment_strict_put_value_resolves_inner_to_outer_then_rechecks_same_object() {
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.inner", "$with.unscopables.inner"),
            vec![with_environment_resolution(
                "$with.outer",
                "$with.unscopables.outer",
            )],
            "x".to_string(),
            Strictness::Strict,
        )
        .put_value(
            identifier("rhs", ValueKind::Number),
            identifier("fallback", ValueKind::Number),
        );

        let ExprIr::Conditional {
            condition: inner_condition,
            then_expr: inner_write,
            else_expr: outer_branch,
        } = &lowered.expr
        else {
            panic!("innermost Object Environment must be queried first");
        };
        assert_eq!(initial_resolution_target(inner_condition), "$with.inner");
        assert_strict_selected_write(inner_write, "$with.inner", "rhs");

        let ExprIr::Conditional {
            condition: outer_condition,
            then_expr: outer_write,
            else_expr: fallback,
        } = &outer_branch.expr
        else {
            panic!("an inner miss must continue through the outer environment");
        };
        assert_eq!(initial_resolution_target(outer_condition), "$with.outer");
        assert_strict_selected_write(outer_write, "$with.outer", "rhs");
        assert_eq!(identifier_name(fallback), "fallback");
    }

    #[test]
    fn with_environment_sloppy_put_value_observes_recheck_before_checked_set() {
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.object", "$with.unscopables.object"),
            Vec::new(),
            "x".to_string(),
            Strictness::Sloppy,
        )
        .put_value(
            identifier("rhs", ValueKind::Number),
            identifier("fallback", ValueKind::Number),
        );

        let ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = &lowered.expr
        else {
            panic!("the Object Environment resolution must guard PutValue");
        };
        assert_eq!(initial_resolution_target(condition), "$with.object");
        assert_eq!(identifier_name(else_expr), "fallback");

        let ExprIr::MaterializeBinding {
            name: value_name,
            value,
            body: after_rhs,
        } = &then_expr.expr
        else {
            panic!("RHS must be materialized before SetMutableBinding");
        };
        assert_eq!(value_name, OBJECT_ENVIRONMENT_VALUE_BINDING);
        assert_eq!(identifier_name(value), "rhs");
        let ExprIr::MaterializeBinding {
            name: recheck_name,
            value: recheck,
            body: write,
        } = &after_rhs.expr
        else {
            panic!("sloppy SetMutableBinding must still observe HasProperty");
        };
        assert_eq!(recheck_name, OBJECT_ENVIRONMENT_RECHECK_BINDING);
        assert_eq!(has_property_target(recheck), "$with.object");
        let ExprIr::PropertyWrite {
            target,
            key,
            value,
            strictness,
        } = &write.expr
        else {
            panic!("the recheck must be followed by checked Set");
        };
        assert_eq!(identifier_name(target), "$with.object");
        assert!(matches!(
            key,
            PropertyKeyIr::StaticString(name) if name == "x"
        ));
        assert_eq!(identifier_name(value), OBJECT_ENVIRONMENT_VALUE_BINDING);
        assert_eq!(*strictness, Strictness::Sloppy);
    }

    #[test]
    fn with_environment_get_value_resolves_inner_to_outer_then_rechecks_selected_object() {
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.inner", "$with.unscopables.inner"),
            vec![with_environment_resolution(
                "$with.outer",
                "$with.unscopables.outer",
            )],
            "x".to_string(),
            Strictness::Strict,
        )
        .get_value(identifier("fallback", ValueKind::Number));

        let ExprIr::Conditional {
            condition: inner_condition,
            then_expr: inner_read,
            else_expr: outer_branch,
        } = &lowered.expr
        else {
            panic!("innermost Object Environment must be queried first");
        };
        assert_eq!(initial_resolution_target(inner_condition), "$with.inner");
        assert_selected_get_value(inner_read, "$with.inner", Strictness::Strict);

        let ExprIr::Conditional {
            condition: outer_condition,
            then_expr: outer_read,
            else_expr: fallback,
        } = &outer_branch.expr
        else {
            panic!("an inner miss must continue through the outer environment");
        };
        assert_eq!(initial_resolution_target(outer_condition), "$with.outer");
        assert_selected_get_value(outer_read, "$with.outer", Strictness::Strict);
        assert_eq!(identifier_name(fallback), "fallback");

        let sloppy = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.object", "$with.unscopables.object"),
            Vec::new(),
            "x".to_string(),
            Strictness::Sloppy,
        )
        .get_value(identifier("fallback", ValueKind::Number));
        let ExprIr::Conditional { then_expr, .. } = &sloppy.expr else {
            panic!("the Object Environment resolution must guard GetValue");
        };
        assert_selected_get_value(then_expr, "$with.object", Strictness::Sloppy);
    }

    #[test]
    fn with_environment_numeric_update_sequences_same_object_get_delta_put_then_result() {
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.object", "$with.unscopables.object"),
            Vec::new(),
            "x".to_string(),
            Strictness::Strict,
        )
        .numeric_update(
            NumericUpdateOp::Increment,
            UpdateReturnMode::Prefix,
            NumericUpdateBindings::allocate(|prefix| format!("${}", prefix.trim_end_matches('.'))),
            identifier("fallback", ValueKind::Number),
        );

        let ExprIr::Conditional {
            condition,
            then_expr: selected,
            else_expr: fallback,
        } = &lowered.expr
        else {
            panic!("ResolveBinding must select the Object Environment once");
        };
        assert_eq!(initial_resolution_target(condition), "$with.object");
        assert_eq!(identifier_name(fallback), "fallback");

        let ExprIr::MaterializeBinding {
            name: old_name,
            value: old_value,
            body: after_get,
        } = &selected.expr
        else {
            panic!("GetBindingValue must be materialized before ToNumeric");
        };
        assert_eq!(old_name, "$object.environment.update.old");
        assert_selected_get_value(old_value, "$with.object", Strictness::Strict);

        let ExprIr::MaterializeBinding {
            name: result_name,
            value: update,
            body: after_update,
        } = &after_get.expr
        else {
            panic!("the numeric result must be retained across PutValue");
        };
        assert_eq!(result_name, "$object.environment.update.result");
        assert!(matches!(
            &update.expr,
            ExprIr::UpdateIdentifier {
                name,
                op: NumericUpdateOp::Increment,
                return_mode: UpdateReturnMode::Prefix,
                value_kind: NumericUpdateValueKind::Dynamic,
            } if name == "$object.environment.update.old"
        ));

        let ExprIr::MaterializeBinding {
            name: write_name,
            value: write,
            body: result,
        } = &after_update.expr
        else {
            panic!("PutValue must complete before the update result is returned");
        };
        assert_eq!(write_name, "$object.environment.update.write");
        assert_strict_selected_write(write, "$with.object", "$object.environment.update.old");
        assert_eq!(identifier_name(result), "$object.environment.update.result");
    }

    #[test]
    fn with_environment_compound_assignment_sequences_same_object_get_apply_put_then_result() {
        let bindings = EagerCompoundAssignmentBindings::allocate(|prefix| {
            format!("${}", prefix.trim_end_matches('.'))
        });
        let old_value = bindings.old_value();
        let applied = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::CoerciveAdd {
                lhs: Box::new(old_value),
                rhs: Box::new(identifier("rhs", ValueKind::Number)),
            },
        );
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.object", "$with.unscopables.object"),
            Vec::new(),
            "x".to_string(),
            Strictness::Strict,
        )
        .compound_assignment(
            bindings.seal(applied),
            identifier("fallback", ValueKind::Number),
        );

        let ExprIr::Conditional {
            condition,
            then_expr: selected,
            else_expr: fallback,
        } = &lowered.expr
        else {
            panic!("ResolveBinding must select the Object Environment once");
        };
        assert_eq!(initial_resolution_target(condition), "$with.object");
        assert_eq!(identifier_name(fallback), "fallback");

        let ExprIr::MaterializeBinding {
            name: old_name,
            value: old_value,
            body: after_get,
        } = &selected.expr
        else {
            panic!("GetBindingValue must be materialized before RHS/application");
        };
        assert_eq!(old_name, "$object.environment.compound.old");
        assert_selected_get_value(old_value, "$with.object", Strictness::Strict);

        let ExprIr::MaterializeBinding {
            name: result_name,
            value: applied,
            body: after_apply,
        } = &after_get.expr
        else {
            panic!("the applied result must be retained across PutValue");
        };
        assert_eq!(result_name, "$object.environment.compound.result");
        let ExprIr::CoerciveAdd { lhs, rhs } = &applied.expr else {
            panic!("the sealed eager operation must remain between GetValue and PutValue");
        };
        assert_eq!(identifier_name(lhs), "$object.environment.compound.old");
        assert_eq!(identifier_name(rhs), "rhs");

        let ExprIr::MaterializeBinding {
            name: write_name,
            value: write,
            body: result,
        } = &after_apply.expr
        else {
            panic!("PutValue must complete before the compound result is returned");
        };
        assert_eq!(write_name, "$object.environment.compound.write");
        assert_strict_selected_write(write, "$with.object", "$object.environment.compound.result");
        assert_eq!(
            identifier_name(result),
            "$object.environment.compound.result"
        );
    }

    #[test]
    fn global_object_environment_compound_assignment_has_plain_resolution_then_get_apply_put() {
        let bindings = EagerCompoundAssignmentBindings::allocate(|prefix| {
            format!("${}", prefix.trim_end_matches('.'))
        });
        let applied = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::CoerciveAdd {
                lhs: Box::new(bindings.old_value()),
                rhs: Box::new(identifier("rhs", ValueKind::Number)),
            },
        );
        let lowered = GlobalObjectEnvironmentReferencePlan::new(
            ValueInfo::new(ValueKind::Object),
            "x".to_string(),
            Strictness::Strict,
        )
        .compound_assignment(bindings.seal(applied));

        let ExprIr::Conditional {
            condition,
            then_expr: selected,
            else_expr: missing,
        } = &lowered.expr
        else {
            panic!("global ResolveBinding must branch on Object Record HasBinding");
        };
        assert_eq!(has_property_target(condition), GLOBAL_THIS_NAME);
        assert!(matches!(
            &missing.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
            }
        ));

        let ExprIr::MaterializeBinding {
            name: old_name,
            value: old_value,
            body: after_get,
        } = &selected.expr
        else {
            panic!("GetBindingValue must precede RHS/application");
        };
        assert_eq!(old_name, "$object.environment.compound.old");
        assert_selected_get_value(old_value, GLOBAL_THIS_NAME, Strictness::Strict);

        let ExprIr::MaterializeBinding {
            name: result_name,
            value: applied,
            body: after_apply,
        } = &after_get.expr
        else {
            panic!("the eager result must be retained across PutValue");
        };
        assert_eq!(result_name, "$object.environment.compound.result");
        let ExprIr::CoerciveAdd { lhs, rhs } = &applied.expr else {
            panic!("the sealed operation must remain after GetBindingValue");
        };
        assert_eq!(identifier_name(lhs), "$object.environment.compound.old");
        assert_eq!(identifier_name(rhs), "rhs");

        let ExprIr::MaterializeBinding {
            name: write_name,
            value: write,
            body: result,
        } = &after_apply.expr
        else {
            panic!("PutValue must complete before exposing the result");
        };
        assert_eq!(write_name, "$object.environment.compound.write");
        assert_strict_selected_write(
            write,
            GLOBAL_THIS_NAME,
            "$object.environment.compound.result",
        );
        assert_eq!(
            identifier_name(result),
            "$object.environment.compound.result"
        );
    }

    #[test]
    fn with_environment_logical_assignment_selects_once_and_puts_only_in_rhs_branch() {
        let lowered = WithEnvironmentReferencePlan::create(
            with_environment_resolution("$with.object", "$with.unscopables.object"),
            Vec::new(),
            "x".to_string(),
            Strictness::Strict,
        )
        .logical_assignment(
            LogicalBinaryOp::Or,
            identifier("rhs", ValueKind::Number),
            identifier("fallback", ValueKind::Number),
        );

        let ExprIr::Conditional {
            condition,
            then_expr: selected,
            else_expr: fallback,
        } = &lowered.expr
        else {
            panic!("ResolveBinding must select the with Object Record once");
        };
        assert_eq!(initial_resolution_target(condition), "$with.object");
        assert_eq!(identifier_name(fallback), "fallback");

        let ExprIr::LogicalShortCircuit { op, lhs, rhs } = &selected.expr else {
            panic!("the selected GetValue must control the only PutValue branch");
        };
        assert_eq!(*op, LogicalBinaryOp::Or);
        assert_selected_get_value(lhs, "$with.object", Strictness::Strict);
        assert_strict_selected_write(rhs, "$with.object", "rhs");
    }

    #[test]
    fn with_environment_identifier_call_keeps_callee_and_base_object_together() {
        let fallback = TypedExpr::from_info(
            dynamic_value_info(),
            ExprIr::CallIndirect {
                callee: Box::new(identifier("fallback", ValueKind::Dynamic)),
                this_arg: None,
                args: vec![identifier("arg", ValueKind::Number)],
                static_regexp_compilation: None,
            },
        );
        let lowered = WithEnvironmentIdentifierCallReferencePlan {
            reference: WithEnvironmentReferencePlan::create(
                with_environment_resolution("$with.object", "$with.unscopables.object"),
                Vec::new(),
                "x".to_string(),
                Strictness::Sloppy,
            ),
        }
        .call(vec![identifier("arg", ValueKind::Number)], fallback);

        let ExprIr::Conditional {
            condition,
            then_expr: selected,
            else_expr: fallback,
        } = &lowered.expr
        else {
            panic!("ResolveBinding must select the with Object Record once");
        };
        assert_eq!(initial_resolution_target(condition), "$with.object");

        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(receiver),
            args,
            ..
        } = &selected.expr
        else {
            panic!("a selected identifier Reference must carry WithBaseObject");
        };
        assert_selected_get_value(callee, "$with.object", Strictness::Sloppy);
        assert_eq!(identifier_name(receiver), "$with.object");
        assert_eq!(identifier_name(&args[0]), "arg");

        let ExprIr::CallIndirect {
            callee,
            this_arg: None,
            args,
            ..
        } = &fallback.expr
        else {
            panic!("ordinary fallback must retain the undefined-this path");
        };
        assert_eq!(identifier_name(callee), "fallback");
        assert_eq!(identifier_name(&args[0]), "arg");
    }

    #[test]
    fn global_object_environment_logical_assignment_has_plain_resolution_and_branch_local_put() {
        for op in [
            LogicalBinaryOp::And,
            LogicalBinaryOp::Or,
            LogicalBinaryOp::Coalesce,
        ] {
            let lowered = GlobalObjectEnvironmentReferencePlan::new(
                ValueInfo::new(ValueKind::Object),
                "x".to_string(),
                Strictness::Strict,
            )
            .logical_assignment(op, identifier("rhs", ValueKind::Number));

            let ExprIr::Conditional {
                condition,
                then_expr: selected,
                else_expr: missing,
            } = &lowered.expr
            else {
                panic!("global ResolveBinding must precede logical selection");
            };
            assert_eq!(has_property_target(condition), GLOBAL_THIS_NAME);
            assert!(matches!(
                &missing.expr,
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
                }
            ));

            let ExprIr::LogicalShortCircuit {
                op: actual_op,
                lhs,
                rhs,
            } = &selected.expr
            else {
                panic!("the selected GetValue must control the only PutValue branch");
            };
            assert_eq!(*actual_op, op);
            assert_selected_get_value(lhs, GLOBAL_THIS_NAME, Strictness::Strict);
            assert_strict_selected_write(rhs, GLOBAL_THIS_NAME, "rhs");
        }
    }

    #[test]
    fn global_object_environment_numeric_update_has_plain_resolution_then_get_delta_put() {
        let modes = [
            (NumericUpdateOp::Increment, UpdateReturnMode::Postfix),
            (NumericUpdateOp::Increment, UpdateReturnMode::Prefix),
            (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix),
            (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix),
        ];

        for (op, return_mode) in modes {
            let bindings = NumericUpdateBindings::allocate(|prefix| {
                format!("${}", prefix.trim_end_matches('.'))
            });
            let lowered = GlobalObjectEnvironmentReferencePlan::new(
                ValueInfo::new(ValueKind::Object),
                "x".to_string(),
                Strictness::Strict,
            )
            .numeric_update(op, return_mode, bindings);

            let ExprIr::Conditional {
                condition,
                then_expr: selected,
                else_expr: missing,
            } = &lowered.expr
            else {
                panic!("global ResolveBinding must branch on Object Record HasBinding");
            };
            assert_eq!(has_property_target(condition), GLOBAL_THIS_NAME);
            assert!(matches!(
                &missing.expr,
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    message: OBJECT_ENVIRONMENT_REFERENCE_ERROR,
                }
            ));

            let ExprIr::MaterializeBinding {
                name: old_name,
                value: old_value,
                body: after_get,
            } = &selected.expr
            else {
                panic!("GetBindingValue must precede ToNumeric");
            };
            assert_eq!(old_name, "$object.environment.update.old");
            assert_selected_get_value(old_value, GLOBAL_THIS_NAME, Strictness::Strict);

            let ExprIr::MaterializeBinding {
                name: result_name,
                value: update,
                body: after_update,
            } = &after_get.expr
            else {
                panic!("the numeric result must be retained across PutValue");
            };
            assert_eq!(result_name, "$object.environment.update.result");
            assert!(matches!(
                &update.expr,
                ExprIr::UpdateIdentifier {
                    name,
                    op: actual_op,
                    return_mode: actual_return_mode,
                    value_kind: NumericUpdateValueKind::Dynamic,
                } if name == "$object.environment.update.old"
                    && *actual_op == op
                    && *actual_return_mode == return_mode
            ));

            let ExprIr::MaterializeBinding {
                name: write_name,
                value: write,
                body: result,
            } = &after_update.expr
            else {
                panic!("PutValue must complete before exposing the update result");
            };
            assert_eq!(write_name, "$object.environment.update.write");
            assert_strict_selected_write(write, GLOBAL_THIS_NAME, "$object.environment.update.old");
            assert_eq!(identifier_name(result), "$object.environment.update.result");
        }
    }

    #[test]
    fn selected_with_objects_are_non_empty_and_stop_at_declarative_fallback() {
        let object = |storage_name: &str| ObjectEnvironmentBindingObject {
            source: ObjectEnvironmentBindingObjectSource::Materialized(storage_name.to_string()),
            info: ValueInfo::new(ValueKind::Object),
        };
        let mut chain = OrderedWithEnvironmentChain::default();
        chain.enter_current(object("$with.outer"), CurrentScopeDepth(1));
        chain.enter_current(object("$with.inner"), CurrentScopeDepth(3));

        let selected = chain
            .select_preceding(Some(DeclarativeEnvironmentPosition::current(
                CurrentScopeDepth(2),
            )))
            .expect("the inner Object Environment must precede the binding");
        assert!(matches!(
            &selected.innermost.source,
            ObjectEnvironmentBindingObjectSource::Materialized(storage_name)
                if storage_name == "$with.inner"
        ));
        assert!(selected.outer.is_empty());

        assert!(
            chain
                .select_preceding(Some(DeclarativeEnvironmentPosition::current(
                    CurrentScopeDepth(4),
                )))
                .is_none(),
            "a nearer declarative binding must cut off every Object Environment"
        );
    }

    #[test]
    fn delete_super_reference_plan_sequences_this_raw_key_then_reference_error() {
        let raw_key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier("key".to_string()),
        );
        let lowered = DeleteSuperReferencePlan::new(
            ValueInfo::new(ValueKind::Object),
            PropertyKeyIr::StringExpr(Box::new(raw_key)),
        )
        .into_reference_error();

        assert_eq!(lowered.kind, ValueKind::Boolean);
        let ExprIr::MaterializeBinding {
            name: this_name,
            value: actual_this,
            body: after_this,
        } = lowered.expr
        else {
            panic!("actualThis must be materialized before the computed key");
        };
        assert_eq!(this_name, DELETE_SUPER_THIS_BINDING);
        assert!(matches!(actual_this.expr, ExprIr::This));

        let ExprIr::MaterializeBinding {
            name: key_name,
            value: key_value,
            body: after_key,
        } = after_this.expr
        else {
            panic!("the raw computed key must be materialized after actualThis");
        };
        assert_eq!(key_name, DELETE_SUPER_KEY_BINDING);
        assert!(matches!(
            &key_value.expr,
            ExprIr::Identifier(name) if name == "key"
        ));
        assert!(matches!(
            after_key.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: DELETE_SUPER_REFERENCE_ERROR,
            }
        ));

        let static_lowered = DeleteSuperReferencePlan::new(
            ValueInfo::new(ValueKind::Object),
            PropertyKeyIr::StaticString("field".to_string()),
        )
        .into_reference_error();
        let ExprIr::MaterializeBinding { body, .. } = static_lowered.expr else {
            panic!("actualThis must be materialized for a static super property");
        };
        assert!(matches!(
            body.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: DELETE_SUPER_REFERENCE_ERROR,
            }
        ));
    }

    #[test]
    fn recovered_reference_preserves_symbol_tag_on_string_shaped_key() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier("object".to_string()),
        );
        let symbol_key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Symbol),
            ExprIr::String("Symbol.iterator".to_string()),
        );
        let read = ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands: vec![target, symbol_key],
        };

        let ReferenceBase::Property { key, .. } =
            reference_base_of_lowered_read(read).expect("GetV should reconstruct a Reference")
        else {
            panic!("GetV should reconstruct a property Reference");
        };
        let PropertyKeyIr::StringExpr(key) = key else {
            panic!("a Symbol key must remain a typed computed key");
        };
        assert_eq!(key.kind, ValueKind::Symbol);
        assert!(matches!(
            &key.expr,
            ExprIr::String(name) if name == "Symbol.iterator"
        ));
    }
}
