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

const DELETE_SUPER_THIS_BINDING: &str = "$delete.super.this";
const DELETE_SUPER_KEY_BINDING: &str = "$delete.super.key";
const DELETE_SUPER_REFERENCE_ERROR: &str = "Cannot delete a super property";

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
        | ExprIr::PropertyUpdate { strictness, .. }
        | ExprIr::PropertyCompoundAssign { strictness, .. }
        | ExprIr::SuperPropertyWrite { strictness, .. }
        | ExprIr::DeleteProperty { strictness, .. }
        | ExprIr::DeleteGlobalProperty { strictness, .. } => {
            Some((*strictness, PutValueFailure::TypeErrorOnly))
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
        | ExprIr::UnaryNumber { .. }
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
        | ExprIr::PropertyUpdate { .. }
        | ExprIr::PropertyCompoundAssign { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::GlobalPropertyCompoundAssign { .. }
        | ExprIr::UnaryNumber { .. }
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
