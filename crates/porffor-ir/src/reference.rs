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
}

/// Which `[[Strict]]` (if any) an IR node carries, as a total function of
/// `ExprIr`.
///
/// This match has no catch-all, and that is its entire purpose: whoever adds a
/// new reference-shaped variant to [`ExprIr`] gets `E0004` here and has to
/// decide, at that moment, whether the new node is a PutValue site. A `_` arm
/// would turn "the new write node has nowhere to record `[[Strict]]`" into a
/// silently sloppy write.
#[must_use]
pub fn carried_strictness(expr: &ExprIr) -> Option<Strictness> {
    match expr {
        // PutValue 2.a + 3.d, PutValue 3.d, and `delete` 5.e.
        ExprIr::GlobalPropertyWrite { strictness, .. }
        | ExprIr::GlobalPropertyUpdate { strictness, .. }
        | ExprIr::GlobalPropertyCompoundAssign { strictness, .. }
        | ExprIr::PropertyWrite { strictness, .. }
        | ExprIr::PropertyUpdate { strictness, .. }
        | ExprIr::PropertyCompoundAssign { strictness, .. }
        | ExprIr::SuperPropertyWrite { strictness, .. }
        | ExprIr::DeleteProperty { strictness, .. }
        | ExprIr::DeleteGlobalProperty { strictness, .. } => Some(*strictness),

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
        | ExprIr::UnaryNumber { .. }
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
        | ExprIr::BitwiseNumber { .. }
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
/// `carried_strictness`'s second arm — so a variant for it would be
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
    Super { key: PropertyKeyIr },
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
            Self::Super { key } => ExprIr::SuperPropertyRead { key: key.clone() },
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
            Self::Super { key } => ExprIr::SuperPropertyWrite {
                key,
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
            Self::Property { key, .. } | Self::Super { key } => key,
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
        ExprIr::SuperPropertyRead { key } => Ok(ReferenceBase::Super { key }),
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
            let key = match key.expr {
                ExprIr::String(name) => PropertyKeyIr::StaticString(name),
                _ => PropertyKeyIr::StringExpr(Box::new(key)),
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
        | ExprIr::PropertyUpdate { .. }
        | ExprIr::PropertyCompoundAssign { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::GlobalPropertyCompoundAssign { .. }
        | ExprIr::UnaryNumber { .. }
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
        | ExprIr::BitwiseNumber { .. }
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

    /// Only for pinning the operands the Reference must not re-evaluate; the
    /// shape of the base cannot be changed through this, only its operands.
    pub(crate) fn base_mut(&mut self) -> &mut ReferenceBase {
        &mut self.base
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

/// The temporaries this Reference's evaluation pinned, innermost last.
///
/// Not `Clone`, and the only consumer is [`Self::materialize`], so a chain
/// cannot be emitted twice (`E0382`) or dropped on the floor (`#[must_use]`).
#[derive(Debug, Default)]
#[must_use = "unmaterialised pins mean the Reference's base or key is evaluated twice"]
pub(crate) struct ReferencePins(Vec<(String, TypedExpr)>);

impl ReferencePins {
    pub(crate) fn none() -> Self {
        Self(Vec::new())
    }

    /// Records that `value` was hoisted into the temporary binding `name`.
    pub(crate) fn pin(&mut self, name: String, value: TypedExpr) {
        self.0.push((name, value));
    }

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
