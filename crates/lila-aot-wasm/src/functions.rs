use super::*;
use lila_ir::{ClassMethodKindIr, StaticRegExpCompilation};

/// A Wasm local proven to contain an allocated realm record.
///
/// The constructor stays in this module so realm-aware materializers cannot
/// be called with an arbitrary payload, tag, or scratch local.
#[derive(Clone, Copy)]
pub(crate) struct RealmRecordLocal(u32);

/// Storage reserved for a created realm's `%Array.prototype%`, before an
/// Array-layout object has been emitted into it.
///
/// This type is deliberately neither `Copy` nor constructible outside this
/// module. Initialization consumes it, so bootstrap cannot publish the local
/// while it still contains an arbitrary payload.
#[must_use]
pub(crate) struct ReservedRealmArrayPrototypeLocal(u32);

/// A Wasm local proven to contain an initialized created-realm
/// `%Array.prototype%` Array exotic object.
///
/// The raw local is private. Created-realm publication and property/link
/// installation accept only this state, and final release consumes it.
#[must_use]
pub(crate) struct RealmArrayPrototypeLocal(u32);

/// A realm-intrinsic slot whose representation is not constrained by the
/// `%Array.prototype%` typestate.
///
/// The Array slot is intentionally absent. Callers cannot manufacture this
/// fieldless enum from a raw offset, so adding Array to the generic writer
/// requires an explicit change to this closed domain and its exhaustive map.
#[derive(Clone, Copy)]
pub(crate) enum NonArrayRealmIntrinsicSlot {
    ThrowTypeError,
    TypeErrorPrototype,
    GeneratorFunctionConstructor,
    AsyncFunctionConstructor,
    AsyncGeneratorFunctionConstructor,
    ObjectPrototype,
    ArrayIteratorPrototype,
    StringIteratorPrototype,
    MapIteratorPrototype,
    SetIteratorPrototype,
    IteratorHelperPrototype,
    IteratorPrototype,
    IteratorFromWrapperPrototype,
    GeneratorPrototype,
    GeneratorFunctionPrototype,
    AsyncIteratorPrototype,
    AsyncFunctionPrototype,
    AsyncGeneratorPrototype,
    AsyncGeneratorFunctionPrototype,
    NumberPrototype,
    StringPrototype,
    BooleanPrototype,
    SymbolPrototype,
    BigIntPrototype,
    MapPrototype,
    SetPrototype,
    WeakMapPrototype,
    WeakSetPrototype,
    WeakRefPrototype,
    FinalizationRegistryPrototype,
    RegExpPrototype,
    Float64ArrayPrototype,
    Float32ArrayPrototype,
    Int32ArrayPrototype,
    Int16ArrayPrototype,
    Int8ArrayPrototype,
    Uint32ArrayPrototype,
    Uint16ArrayPrototype,
    Uint8ArrayPrototype,
    Uint8ClampedArrayPrototype,
    BigInt64ArrayPrototype,
    BigUint64ArrayPrototype,
}

impl NonArrayRealmIntrinsicSlot {
    const fn offset(self) -> u64 {
        match self {
            Self::ThrowTypeError => HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET,
            Self::TypeErrorPrototype => HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            Self::GeneratorFunctionConstructor => {
                HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET
            }
            Self::AsyncFunctionConstructor => {
                HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_CONSTRUCTOR_OFFSET
            }
            Self::AsyncGeneratorFunctionConstructor => {
                HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET
            }
            Self::ObjectPrototype => HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            Self::ArrayIteratorPrototype => HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET,
            Self::StringIteratorPrototype => HEAP_REALM_INTRINSICS_STRING_ITERATOR_PROTOTYPE_OFFSET,
            Self::MapIteratorPrototype => HEAP_REALM_INTRINSICS_MAP_ITERATOR_PROTOTYPE_OFFSET,
            Self::SetIteratorPrototype => HEAP_REALM_INTRINSICS_SET_ITERATOR_PROTOTYPE_OFFSET,
            Self::IteratorHelperPrototype => HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET,
            Self::IteratorPrototype => HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET,
            Self::IteratorFromWrapperPrototype => {
                HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET
            }
            Self::GeneratorPrototype => HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET,
            Self::GeneratorFunctionPrototype => {
                HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET
            }
            Self::AsyncIteratorPrototype => HEAP_REALM_INTRINSICS_ASYNC_ITERATOR_PROTOTYPE_OFFSET,
            Self::AsyncFunctionPrototype => HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET,
            Self::AsyncGeneratorPrototype => HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET,
            Self::AsyncGeneratorFunctionPrototype => {
                HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET
            }
            Self::NumberPrototype => HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
            Self::StringPrototype => HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
            Self::BooleanPrototype => HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
            Self::SymbolPrototype => HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET,
            Self::BigIntPrototype => HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
            Self::MapPrototype => HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET,
            Self::SetPrototype => HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET,
            Self::WeakMapPrototype => HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,
            Self::WeakSetPrototype => HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,
            Self::WeakRefPrototype => HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,
            Self::FinalizationRegistryPrototype => {
                HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET
            }
            Self::RegExpPrototype => HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET,
            Self::Float64ArrayPrototype => HEAP_REALM_INTRINSICS_FLOAT64_ARRAY_PROTOTYPE_OFFSET,
            Self::Float32ArrayPrototype => HEAP_REALM_INTRINSICS_FLOAT32_ARRAY_PROTOTYPE_OFFSET,
            Self::Int32ArrayPrototype => HEAP_REALM_INTRINSICS_INT32_ARRAY_PROTOTYPE_OFFSET,
            Self::Int16ArrayPrototype => HEAP_REALM_INTRINSICS_INT16_ARRAY_PROTOTYPE_OFFSET,
            Self::Int8ArrayPrototype => HEAP_REALM_INTRINSICS_INT8_ARRAY_PROTOTYPE_OFFSET,
            Self::Uint32ArrayPrototype => HEAP_REALM_INTRINSICS_UINT32_ARRAY_PROTOTYPE_OFFSET,
            Self::Uint16ArrayPrototype => HEAP_REALM_INTRINSICS_UINT16_ARRAY_PROTOTYPE_OFFSET,
            Self::Uint8ArrayPrototype => HEAP_REALM_INTRINSICS_UINT8_ARRAY_PROTOTYPE_OFFSET,
            Self::Uint8ClampedArrayPrototype => {
                HEAP_REALM_INTRINSICS_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET
            }
            Self::BigInt64ArrayPrototype => HEAP_REALM_INTRINSICS_BIGINT64_ARRAY_PROTOTYPE_OFFSET,
            Self::BigUint64ArrayPrototype => HEAP_REALM_INTRINSICS_BIGUINT64_ARRAY_PROTOTYPE_OFFSET,
        }
    }

    pub(crate) const fn for_typed_array_constructor(builtin: StandardBuiltinId) -> Option<Self> {
        Some(match builtin {
            StandardBuiltinId::Float64ArrayConstructor => Self::Float64ArrayPrototype,
            StandardBuiltinId::Float32ArrayConstructor => Self::Float32ArrayPrototype,
            StandardBuiltinId::Int32ArrayConstructor => Self::Int32ArrayPrototype,
            StandardBuiltinId::Int16ArrayConstructor => Self::Int16ArrayPrototype,
            StandardBuiltinId::Int8ArrayConstructor => Self::Int8ArrayPrototype,
            StandardBuiltinId::Uint32ArrayConstructor => Self::Uint32ArrayPrototype,
            StandardBuiltinId::Uint16ArrayConstructor => Self::Uint16ArrayPrototype,
            StandardBuiltinId::Uint8ArrayConstructor => Self::Uint8ArrayPrototype,
            StandardBuiltinId::Uint8ClampedArrayConstructor => Self::Uint8ClampedArrayPrototype,
            StandardBuiltinId::BigInt64ArrayConstructor => Self::BigInt64ArrayPrototype,
            StandardBuiltinId::BigUint64ArrayConstructor => Self::BigUint64ArrayPrototype,
            _ => return None,
        })
    }
}

/// Whether function allocation also creates the default own `prototype`
/// property. This policy is deliberately separate from semantic
/// constructability; realm bootstrap supplies a few intrinsic prototypes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionPrototypeMaterialization {
    Automatic,
    BootstrapSupplied,
}

impl RealmRecordLocal {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(i64)]
enum FunctionRealmOutcome {
    Resolved = 0,
    Revoked = 1,
    Invalid = 2,
}

/// The raw run-time result of `GetFunctionRealm` before its non-resolved
/// outcomes have been routed.
///
/// Its fields are intentionally private. A caller can only obtain the realm
/// local by consuming this value through
/// [`FunctionBuilder::emit_route_function_realm_result`], which handles both
/// `Revoked` and `Invalid` before returning a resolved witness.
#[must_use]
pub(crate) struct FunctionRealmResultLocals {
    realm_local: u32,
    outcome_local: u32,
}

/// A Wasm local whose `GetFunctionRealm` non-resolved outcomes have both been
/// handled according to an explicit route.
#[derive(Clone, Copy)]
#[must_use]
pub(crate) struct ResolvedFunctionRealmLocal(u32);

impl ResolvedFunctionRealmLocal {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

/// The ordinary-object intrinsic prototypes selected by
/// `GetPrototypeFromConstructor` in the shared construct path.
///
/// `%Array.prototype%` is deliberately absent because it has an Array layout
/// and a distinct representation tag. Keeping this domain closed prevents a
/// caller from pairing an arbitrary realm-intrinsic offset with an entry-realm
/// fallback.
#[derive(Clone, Copy)]
enum OrdinaryDefaultPrototype {
    Object,
    String,
    Number,
    Boolean,
}

impl OrdinaryDefaultPrototype {
    const fn offset(self) -> u64 {
        match self {
            Self::Object => HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            Self::String => HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
            Self::Number => HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
            Self::Boolean => HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
        }
    }
}

/// A populated ordinary-object prototype loaded from a realm already proven
/// by `GetFunctionRealm`.
///
/// The local is non-`Copy` and private so construction must consume it through
/// the operation that installs both its payload and Object representation tag.
#[must_use = "the resolved-realm prototype must be installed with its representation tag"]
struct ResolvedRealmOrdinaryPrototypeLocal(u32);

/// What a consumer does when `GetFunctionRealm` encounters a revoked Proxy.
///
/// `Invalid` is deliberately absent: every route traps for that internal
/// invariant failure. Promise job creation uses the current realm for a
/// revoked callback, while constructor/default-prototype consumers surface
/// the required TypeError and leave their enclosing control-flow region.
pub(crate) enum FunctionRealmRevokedRoute {
    UseCurrentRealm,
    ThrowTypeErrorAndReturn {
        payload_local: u32,
        tag_local: u32,
    },
    ThrowTypeErrorAndBranch {
        payload_local: u32,
        tag_local: u32,
        relative_depth: u32,
    },
}

fn is_canonical_array_index_name(name: &str) -> bool {
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    if name.len() > 1 && name.starts_with('0') {
        return false;
    }
    name.parse::<u64>()
        .is_ok_and(|index| index <= MAX_ARRAY_LENGTH - 1)
}

/// The destination local pair of an `Iterator.prototype` helper fast path.
///
/// # The measured defect, corrected
///
/// The paragraph that used to sit here blamed callee *acquisition*
/// (`emit_function_value_payload`) and it was wrong. Callee acquisition is
/// never reached on the receiver that fails, because **no code is emitted for
/// the call at all**.
///
/// `new S()`, for a class with heritage and no explicit constructor, is lowered
/// with `kind = Undefined` and a nullish `possible_kinds`. So
/// [`super::receiver_shape_targets_iterator_helper`] is false, the seven
/// bare-guard blocks (`find`, `reduce`, `take`, `map`, `every`, `some`,
/// `filter`) declined the call, and `emit_method_call`'s generic tail took its
/// statically-nullish shortcut — release the temps, return `Ok(())`, emit
/// nothing. `drop` and `flatMap` carry an extra `!receiver_is_array` disjunct
/// and were dispatched properly, which is the entire reason those two were
/// green on an otherwise identical fixture.
///
/// Measured, not argued: `lila::main` for a script whose only statement is
/// `new Source().flatMap(identity);` is 561,563 bytes, and for
/// `new Source().some(identity);`, `.map(identity);` and `.find(identity);` it
/// is 561,494 — the same number for all three, i.e. those three emit *nothing*
/// and differ from each other by nothing.
///
/// Nothing here says the generic tail is unreachable or unused: it remains the
/// right destination for primitive and dynamically-typed receivers and keeps
/// them. What changed is where an object-or-nullish-typed receiver goes for
/// these seven keys.
///
/// # What this type does and does not prove
///
/// It proves that a helper fast path handed its destination to
/// [`super::Emitter::emit_iterator_prototype_helper_method_call`], which is the
/// only function that consumes a [`MethodCallDestination`]. That is worth
/// having — an eleventh helper block cannot be added that forgets the
/// destination entirely.
///
/// It does **not** prove that a store was emitted, and it would not have caught
/// the real defect at all: the seven broken blocks never constructed a
/// destination, because they never called the emitter. A witness type can only
/// constrain the paths that reach it, and "the arm silently declined to handle
/// the call" is a path that does not. The assertion that closes *that* hole is
/// an emitted-module one, and it lives in
/// `crates/lila-aot-wasm/tests/iterator_helper_dispatch.rs`.
///
/// It is also not yet an invariant over `emit_method_call` as a whole. That
/// function still returns `Result<(), EmitError>` with ~50 other `return`
/// paths, so an eleventh fast path that returns without storing remains
/// writable. Making the claim real means threading a `MethodCallDestination`
/// through `emit_method_call` itself — an internal
/// `emit_method_call_into(.., destination) -> Result<DestinationWritten, _>`
/// with the public wrapper constructing the destination, so `new` need not be
/// visible outside this file. That is a ~50-site mechanical rewrite of a
/// 1,400-line emitter function and `docs/rust-rewrite/batch-workflow.md`
/// requires rung G (golden byte-diff) for any refactor of this crate, so it is
/// left for a lane with build access. Note when doing it that a mechanical
/// conversion mints a proof for every *existing* path: it constrains future
/// code, it does not audit present code.
///
/// Constructor and witness are `pub(super)`, not `pub(crate)`: forging the
/// proof with `MethodCallDestination::new(p, t).written()` is still one
/// expression, but only from inside this file.
mod method_call_destination {
    /// The `(payload, tag)` locals a method call must store its result into.
    /// Moved rather than copied so the receiving code path has to account for
    /// it.
    pub(super) struct MethodCallDestination {
        payload_local: u32,
        tag_local: u32,
    }

    /// Proof that stores into both locals of a [`MethodCallDestination`] have
    /// been emitted on every path that **completes normally** out of the
    /// emitter.
    ///
    /// The qualifier is load-bearing and was added when it stopped being
    /// redundant. `emit_iterator_prototype_helper_method_call` now calls
    /// `emit_propagate_throw_from_locals_if_needed` after compiling the
    /// receiver, and that emits a branch-to-handler / `return` before either
    /// destination local has been written (`control_flow.rs`). An abrupt exit
    /// carries its value in `completion_local` / `result_local` instead, so the
    /// destination pair is not read on that path — but "every path" would be a
    /// false statement about the emitted code, and this type is the
    /// compiler-enforced half of the batch's "the callee must write its
    /// destination" invariant. It must not over-claim.
    #[must_use]
    pub(super) struct DestinationWritten(());

    impl MethodCallDestination {
        pub(super) fn new(payload_local: u32, tag_local: u32) -> Self {
            Self {
                payload_local,
                tag_local,
            }
        }

        pub(super) fn payload_local(&self) -> u32 {
            self.payload_local
        }

        pub(super) fn tag_local(&self) -> u32 {
            self.tag_local
        }

        /// Consume the destination, witnessing that the code just emitted
        /// stores into both of its locals. Call this only immediately after
        /// the instruction sequence that performs those stores.
        pub(super) fn written(self) -> DestinationWritten {
            DestinationWritten(())
        }
    }

    impl DestinationWritten {
        /// Discharge the proof into the `()` that `emit_method_call` and its
        /// fast paths return.
        pub(super) fn discharge(self) {}
    }
}

use self::method_call_destination::{DestinationWritten, MethodCallDestination};

/// The `Iterator.prototype` methods `emit_method_call` has a static-key fast
/// path for.
///
/// `toArray` is deliberately absent: it has no fast path, reaches the generic
/// tail, and is measured correct there. The generic tail is the oracle for all
/// of these — a fast path may only be *faster*, never *different*.
///
/// Every helper's property name and builtin id live on this one enum, and both
/// mappings are exhaustive `match`es with no `_` arm, so adding an eleventh
/// helper is a compile error rather than a silently half-wired fast path. Ten
/// hand-copied blocks of which nine were wrong and one was right is exactly the
/// drifted-parallel-tables shape `AGENTS.md` warns about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum IteratorHelper {
    Map,
    Filter,
    FlatMap,
    Take,
    Drop,
    Some,
    Every,
    Find,
    Reduce,
    ForEach,
}

impl IteratorHelper {
    /// The builtin this helper's property resolves to. Used by the guard, so
    /// the guard and the emission cannot name different builtins.
    pub(crate) fn builtin(self) -> StandardBuiltinId {
        match self {
            Self::Map => StandardBuiltinId::IteratorPrototypeMap,
            Self::Filter => StandardBuiltinId::IteratorPrototypeFilter,
            Self::FlatMap => StandardBuiltinId::IteratorPrototypeFlatMap,
            Self::Take => StandardBuiltinId::IteratorPrototypeTake,
            Self::Drop => StandardBuiltinId::IteratorPrototypeDrop,
            Self::Some => StandardBuiltinId::IteratorPrototypeSome,
            Self::Every => StandardBuiltinId::IteratorPrototypeEvery,
            Self::Find => StandardBuiltinId::IteratorPrototypeFind,
            Self::Reduce => StandardBuiltinId::IteratorPrototypeReduce,
            Self::ForEach => StandardBuiltinId::IteratorPrototypeForEach,
        }
    }

    /// The property name read off the receiver. This is the same string the
    /// guard passes to `read_static_heap_shape_property` and the same one
    /// `intrinsics/iterator.rs` installs on `Iterator.prototype`; all eleven
    /// are interned unconditionally by `StringPool::collect`
    /// (`data.rs`), so `strings.payload` cannot miss.
    ///
    /// It is also what each helper block in `emit_method_call` matches its
    /// `PropertyKeyIr::StaticString` against. That was a bare literal per block
    /// until batch 6, which meant every block named its helper twice from two
    /// independent sources — once as `"take"` in the guard and once as
    /// `IteratorHelper::Take` in the emission — and batch 6 added a second
    /// emission site to each of seven blocks, doubling the pairs that could
    /// disagree. A disagreement compiles cleanly and silently dispatches the
    /// wrong helper or none at all, which is precisely the defect class this
    /// family has already shipped twice.
    pub(crate) fn property_name(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Filter => "filter",
            Self::FlatMap => "flatMap",
            Self::Take => "take",
            Self::Drop => "drop",
            Self::Some => "some",
            Self::Every => "every",
            Self::Find => "find",
            Self::Reduce => "reduce",
            Self::ForEach => "forEach",
        }
    }
}

/// Whether the receiver's static heap shape resolves `helper`'s property name
/// to `helper`'s `Iterator.prototype` builtin.
///
/// This was spelled out nine times inline, once per fast path, with the name
/// and the builtin written as two independent literals; a mismatched pair
/// compiled cleanly and silently disabled the fast path. Both now come off
/// [`IteratorHelper`].
fn receiver_shape_targets_iterator_helper(receiver: &TypedExpr, helper: IteratorHelper) -> bool {
    receiver
        .heap_shape
        .as_deref()
        .and_then(|shape| read_static_heap_shape_property(shape, helper.property_name()))
        .is_some_and(|property| match property {
            ObjectShapeProperty::Data(info) => info
                .function_targets
                .contains(&helper.builtin().function_id()),
            ObjectShapeProperty::Accessor { .. } => false,
        })
}

/// Whether a receiver that has exhausted a helper block's *receiver-specific*
/// alternatives must still be dispatched through
/// [`super::Emitter::emit_iterator_prototype_helper_method_call`] rather than
/// falling through to `emit_method_call`'s generic tail.
///
/// # The receiver this exists for, measured rather than reasoned
///
/// `new S()` where `S` is a class **with heritage and no explicit
/// constructor** — the shape all thirteen iterator fixtures use — is lowered
/// with `kind = Undefined` and a `possible_kinds` contained in
/// [`KindSet::NULLISH`]. That is a `lila-ir` defect, not a fact about the
/// program, and it is what every symptom in this family reduces to:
///
/// | probe (`lila inspect`, script result kind) | result |
/// |---|---|
/// | `class C { m(){} } new C();` | `object` |
/// | `class F { constructor(){} } new F();` | `object` |
/// | `class E extends Iterator { constructor(){super();} } new E();` | `object` |
/// | `class D extends Iterator { } new D();` | **`undefined`** |
///
/// Downstream of that, `emit_method_call`'s generic tail takes its
/// statically-nullish shortcut: the receiver "can only be undefined or null",
/// so the runtime kind dispatch is dead code, and the arm releases its temps
/// and returns `Ok(())` **having written neither destination local**. The
/// runtime value is an ordinary object, the emitted nullish check does not
/// fire, and the caller reads whatever the scratch pair happened to hold. Every
/// measured face of this family is that one hole:
///
/// * `typeof new Source().take(2)` answering `number`, and answering different
///   types in different programs — stale scratch, not a wrong result;
/// * `.some(cb)` returning with `calls-0` — no call was emitted at all;
/// * `TypeError: value is not callable` when the stale pair is then used as the
///   receiver of `.toArray()`;
/// * `helper::value_to_string` trapping on it.
///
/// Two independent measurements pin the mechanism. Emitted-size attribution
/// (`LILA_EMIT_SIZE_REPORT_PATH`) over two programs differing in one
/// identifier gives `lila::main` = 557,233 bytes for `new Source().drop(1)`
/// and 557,156 for `new Source().take(1)`: the failing helper emits **77 bytes
/// fewer**, i.e. it emits nothing where the working one emits a call. And
/// `new D().test(1)` — an ordinary method on the same receiver — is correct,
/// because `lowering.rs` only builds a `CallMethod` for `test` when
/// `possible_kinds.contains(Object)`, which this receiver fails, so it takes
/// the property-read path instead and never meets the hole.
///
/// `drop` and `flatMap` were green throughout for exactly one reason: their
/// guards carry a `!receiver_is_array` disjunct, so a non-array receiver
/// reaches the dispatch whatever the shape guard says.
///
/// # Why this predicate is narrower than `drop`'s disjunct
///
/// `!receiver_is_array` is also true for statically primitive receivers, and
/// the generic tail does real work for those that this dispatch does not:
/// per-kind prototype lookups for `String`/`Number`/`Boolean`/`Symbol`/`BigInt`
/// receivers. Restricting the fall-back to receivers whose kind set is
/// contained in `{Object, Function} ∪ NULLISH` leaves every one of those on the
/// tail, byte for byte, while covering the mistyped-`undefined` receiver above
/// and ordinary object receivers.
///
/// **What that does *not* do, stated because the earlier wording claimed
/// otherwise:** it does not leave `Dynamic` receivers on the tail. The
/// predicate partitions on `possible_kinds`, and `ValueKind::Dynamic` is what
/// `KindSet::as_value_kind` returns for *any* non-singleton kind set
/// (`ir.rs:402`), so a `cond ? {} : undefined` receiver carries
/// `kind == Dynamic` with `possible_kinds == {Object, Undefined}` — which is
/// inside the set above and is routed here. That is sound for these seven keys
/// rather than accidental: the tail's `Dynamic` arm differs from its `Object`
/// arm only by pre-resolving `toString`/`valueOf`/`toLocaleString` against the
/// number/string/bigint prototypes, and `runtime_number_builtin`,
/// `runtime_string_builtin` and `runtime_bigint_builtin` are all `None` for
/// `find`/`reduce`/`take`/`map`/`every`/`some`/`filter`. A receiver that can
/// still be a *primitive* is a different matter, and that is what the
/// `receiver.kind` conjunct below is for.
///
/// The nullish half of that set is not a licence to skip
/// `RequireObjectCoercible`: a genuinely nullish receiver is indistinguishable
/// from the mistyped one here, so
/// [`super::Emitter::emit_iterator_prototype_helper_method_call`] performs the
/// 7.2.1 check itself before the property read.
///
/// The `Array` heap-shape exclusion is redundant against the kind set and is
/// kept because it costs nothing and makes "arrays keep their own fast paths" a
/// claim local to this function rather than one distributed across seven call
/// sites.
///
/// # The `receiver.kind` conjunct is the L5 guard, not decoration
///
/// `KindSet::EMPTY.is_subset_of(anything)` is `true` (`ir.rs:370` —
/// `self.0 & !other.0 == 0`), so a kind-set test alone routes a receiver whose
/// `possible_kinds` is *empty* into an object-shaped `[[Get]]` regardless of
/// what its `kind` says. This repository already tracks that exact trap for the
/// identical guard shape on `{Array}`: see ledger **L5** on
/// `IntactnessPremise::ArrayIteratorIntact`
/// (`lila-ir/src/iterator_obligations.rs`). Nothing in the type system
/// forbids an empty `possible_kinds` — `lowering.rs` filters on
/// `!= KindSet::EMPTY` in the class-heritage path precisely because such
/// `ValueInfo`s get built — and a `kind == ValueKind::String` receiver that
/// arrived here would skip the tail's String-prototype routing and run
/// `emit_object_read` against a String tag.
///
/// So the predicate also tests `receiver.kind` against the same domain. On
/// every receiver this fall-back is meant to move the conjunct is already true
/// (`Object`, `Function`, `Undefined`, `Null`, or `Dynamic` per the paragraph
/// above), so it changes no emitted byte; on an `EMPTY` kind set with a
/// primitive `kind` it falls to the tail exactly as before.
///
/// # This set is deliberately WIDER than the repair needs. Two claims, kept apart
///
/// **The minimal repair is `possible_kinds.is_subset_of(KindSet::NULLISH)`.**
/// That is exactly the receiver the tail gets wrong — the statically-nullish
/// shortcut in `emit_method_call`, which returns having emitted no call. Every
/// other receiver this predicate captures reaches the tail's
/// `Object | Function | Dynamic` arm, which emits the same `[[Get]]` plus
/// `emit_function_handle_call_with_argv` this dispatch does, so on a *callable*
/// property the two are behaviourally equivalent and moving them repairs
/// nothing.
///
/// **What the wider set buys is a second, separate improvement, and it is the
/// only reason to keep it:** on a NON-callable property the two differ. The
/// tail's callee check is `i64.eq` against the `Function` tag followed by
/// `Instruction::Unreachable` — `({}).map(1)` traps the module. This dispatch
/// reaches `emit_function_handle_call_with_argv_inner`, whose own check throws
/// `TypeError: value is not callable`. Turning a Wasm trap into a spec-shaped
/// `TypeError` for seven of the most common method names in JavaScript is worth
/// having; treating it as a side effect of a bug fix is not, which is why it is
/// stated here rather than left to be re-derived.
///
/// The cost, stated so the next batch does not have to re-discover it: every
/// `{Object, Function}` receiver of these seven names moves off the tail for the
/// whole corpus, so any argument that banked rung-1c verdicts survive this
/// change has to enumerate those call sites. If the `TypeError` improvement is
/// ever given up, narrow the body to the `NULLISH` subset test alone — it still
/// repairs the mistyped class receiver, and it leaves every already-correct
/// object receiver on the tail byte for byte.
///
/// Neither claim is witnessed by a test today. `({}).map(1)` throwing a
/// `TypeError` rather than trapping is unfixtured, and the differential in
/// `crates/lila-aot-wasm/tests/iterator_helper_dispatch.rs` cannot see it
/// (it compares sizes, and both programs there have callable properties).
fn receiver_needs_dynamic_helper_dispatch(receiver: &TypedExpr) -> bool {
    let dispatchable = KindSet::from_kind(ValueKind::Object)
        .union(KindSet::from_kind(ValueKind::Function))
        .union(KindSet::NULLISH);
    receiver.possible_kinds.is_subset_of(dispatchable)
        && matches!(
            receiver.kind,
            ValueKind::Object
                | ValueKind::Function
                | ValueKind::Undefined
                | ValueKind::Null
                | ValueKind::Dynamic
        )
        && !matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)))
}

#[cfg(test)]
mod async_generator_topology_tests {
    use super::*;

    const PUBLIC_TOPOLOGY_SOURCES: &[&str] = &[
        r#"
            const asyncFunctionPrototype = Object.getPrototypeOf(async function () {});
            const asyncGenerator = async function* stream() {};
            const asyncGeneratorFunctionPrototype = Object.getPrototypeOf(asyncGenerator);
            const asyncGeneratorPrototype = asyncGeneratorFunctionPrototype.prototype;

            Object.getPrototypeOf(asyncGeneratorFunctionPrototype) === asyncFunctionPrototype
                && Object.getPrototypeOf(asyncGenerator.prototype) === asyncGeneratorPrototype;
        "#,
        r#"
            const asyncGeneratorFunctionPrototype = Object.getPrototypeOf(async function* () {});
            const asyncGeneratorPrototype = asyncGeneratorFunctionPrototype.prototype;
            const asyncIteratorPrototype = Object.getPrototypeOf(asyncGeneratorPrototype);
            const receiver = {};

            asyncIteratorPrototype[Symbol.asyncIterator].call(receiver) === receiver
                && asyncGeneratorPrototype[Symbol.toStringTag] === "AsyncGenerator"
                && asyncGeneratorFunctionPrototype[Symbol.toStringTag] === "AsyncGeneratorFunction";
        "#,
    ];

    pub(crate) const PUBLIC_RUNTIME_SOURCES: &[&str] = &[
        r#"
            let started = false;
            async function* values() {
                started = true;
                yield 1;
            }
            const iterator = values();
            if (started) throw new Error("async-generator call must be lazy");
            Object.getPrototypeOf(iterator) === values.prototype;
        "#,
        r#"
            async function* values() { yield 1; }
            const iterator = values();
            const next = iterator.next();
            const returned = iterator.return(2);
            const thrown = iterator.throw(3);
            next instanceof Promise && returned instanceof Promise && thrown instanceof Promise;
        "#,
        r#"
            const order = [];
            async function* empty() {}
            const iterator = empty();
            const first = iterator.next().then(result => {
                order.push("first");
                if (!result.done || result.value !== undefined) throw result;
            });
            const second = iterator.next().then(result => {
                order.push("second");
                if (!result.done || result.value !== undefined) throw result;
            });
            const returned = iterator.return(Promise.resolve(4)).then(result => {
                order.push("return");
                if (!result.done || result.value !== 4) throw result;
            });
            Promise.all([first, second, returned]).then(() => {
                if (order.join(",") !== "first,second,return") throw order;
            });
        "#,
        r#"
            async function* empty() {}
            const fulfilled = empty();
            const rejected = empty();
            const reason = new Error("return rejected");
            Promise.all([fulfilled.next(), rejected.next()]).then(() => Promise.all([
                fulfilled.return(Promise.resolve(4)).then(result => {
                    if (!result.done || result.value !== 4) throw result;
                }),
                rejected.return(Promise.reject(reason)).then(
                    () => { throw new Error("return must reject"); },
                    error => { if (error !== reason) throw error; },
                ),
            ]));
        "#,
        r#"
            const order = [];
            let release;
            const returnValue = new Promise(resolve => { release = resolve; });
            const reason = new Error("queued throw");
            async function* empty() {}
            const iterator = empty();
            iterator.next().then(() => {
                const returned = iterator.return(returnValue).then(result => {
                    order.push("return");
                    if (!result.done || result.value !== 7) throw result;
                });
                const next = iterator.next().then(result => {
                    order.push("next");
                    if (!result.done || result.value !== undefined) throw result;
                });
                const thrown = iterator.throw(reason).then(
                    () => { throw new Error("throw must reject"); },
                    error => {
                        order.push("throw");
                        if (error !== reason) throw error;
                    },
                );
                release(7);
                return Promise.all([returned, next, thrown]);
            }).then(() => {
                if (order.join(",") !== "return,next,throw") throw order;
            });
        "#,
        r#"
            let started = false;
            async function* values(source) {
                started = true;
                await source;
                yield source;
            }
            const pending = values(1).next();
            if (!started) throw new Error("first request must start body");
            pending instanceof Promise;
        "#,
        r#"
            const reason = new Error("body throw");
            async function* complete() {}
            async function* fail() { throw reason; }
            Promise.all([
                complete().next().then(result => {
                    if (!result.done || result.value !== undefined) throw result;
                }),
                fail().next().then(
                    () => { throw new Error("body throw must reject"); },
                    error => { if (error !== reason) throw error; },
                ),
            ]);
        "#,
    ];

    #[test]
    fn syntax_created_async_generator_functions_use_async_generator_intrinsic_roots() {
        assert_eq!(PUBLIC_TOPOLOGY_SOURCES.len(), 2);
        assert_eq!(
            syntax_function_object_prototype_global_index(FunctionExecutionKind::Async),
            Some(ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX)
        );
        assert_eq!(
            syntax_function_object_prototype_global_index(FunctionExecutionKind::AsyncGenerator),
            Some(ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX)
        );
        assert_eq!(
            syntax_function_instance_prototype_global_index(FunctionExecutionKind::AsyncGenerator),
            Some(ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX)
        );
        assert_eq!(
            syntax_function_instance_prototype_global_index(FunctionExecutionKind::Async),
            None
        );
    }

    #[test]
    fn async_generator_runtime_regressions_cover_lazy_calls_requests_and_fifo_completion() {
        assert_eq!(PUBLIC_RUNTIME_SOURCES.len(), 7);
        assert!(PUBLIC_RUNTIME_SOURCES[0].contains("call must be lazy"));
        assert!(PUBLIC_RUNTIME_SOURCES[1].contains("instanceof Promise"));
        assert!(PUBLIC_RUNTIME_SOURCES[2].contains("first,second,return"));
        assert!(PUBLIC_RUNTIME_SOURCES[3].contains("return rejected"));
        assert!(PUBLIC_RUNTIME_SOURCES[4].contains("return,next,throw"));
        assert!(PUBLIC_RUNTIME_SOURCES[5].contains("first request must start body"));
        assert!(PUBLIC_RUNTIME_SOURCES[6].contains("body throw must reject"));
    }
}

fn syntax_function_object_prototype_global_index(
    execution_kind: FunctionExecutionKind,
) -> Option<u32> {
    match execution_kind {
        FunctionExecutionKind::Ordinary => None,
        FunctionExecutionKind::Generator => Some(GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX),
        FunctionExecutionKind::Async => Some(ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX),
        FunctionExecutionKind::AsyncGenerator => {
            Some(ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX)
        }
    }
}

fn syntax_function_instance_prototype_global_index(
    execution_kind: FunctionExecutionKind,
) -> Option<u32> {
    match execution_kind {
        FunctionExecutionKind::Generator => Some(GENERATOR_PROTOTYPE_GLOBAL_INDEX),
        FunctionExecutionKind::AsyncGenerator => Some(ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX),
        FunctionExecutionKind::Ordinary | FunctionExecutionKind::Async => None,
    }
}

fn helper_store_i64_local_at_offset(
    function: &mut Function,
    object_local: u32,
    offset: u64,
    value_local: u32,
) {
    function.instruction(&Instruction::LocalGet(object_local));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(value_local));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
}

fn helper_store_i64_const_at_offset(
    function: &mut Function,
    object_local: u32,
    offset: u64,
    value: i64,
) {
    function.instruction(&Instruction::LocalGet(object_local));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Const(value));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
}

pub(crate) fn emit_array_alloc_helper_function(heap_alloc_function_index: u32) -> Function {
    const LEN_LOCAL: u32 = 0;
    const ARRAY_LOCAL: u32 = 1;
    const BUFFER_LOCAL: u32 = 2;
    const CAP_LOCAL: u32 = 3;
    const SIZE_LOCAL: u32 = 4;
    const SCRATCH_LOCAL: u32 = 5;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 5));

    function.instruction(&Instruction::I64Const(HEAP_ARRAY_RECORD_SIZE as i64));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(ARRAY_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
    function.instruction(&Instruction::I64LeU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::LocalSet(SIZE_LOCAL));
    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));

    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_PTR_OFFSET, BUFFER_LOCAL);
    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_LEN_OFFSET, LEN_LOCAL);
    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_CAP_OFFSET, CAP_LOCAL);
    function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        ARRAY_LOCAL,
        HEAP_PROTOTYPE_OFFSET,
        SCRATCH_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        ARRAY_LOCAL,
        HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
        ValueKind::Array.tag() as i64,
    );

    for (offset, value) in [
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET, -1),
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET, 0),
    ] {
        helper_store_i64_const_at_offset(&mut function, ARRAY_LOCAL, offset, value);
    }

    function.instruction(&Instruction::LocalGet(ARRAY_LOCAL));
    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::End);
    function
}

pub(crate) fn emit_function_object_alloc_helper_function(
    heap_alloc_function_index: u32,
    object_append_data_property_function_index: u32,
) -> Function {
    const TABLE_INDEX_LOCAL: u32 = 0;
    const ENV_HANDLE_LOCAL: u32 = 1;
    const FLAGS_LOCAL: u32 = 2;
    const TO_STRING_PAYLOAD_LOCAL: u32 = 3;
    const LENGTH_KEY_LOCAL: u32 = 4;
    const LENGTH_PAYLOAD_LOCAL: u32 = 5;
    const NAME_KEY_LOCAL: u32 = 6;
    const NAME_PAYLOAD_LOCAL: u32 = 7;
    const DESCRIPTOR_KIND_LOCAL: u32 = 8;
    const OBJECT_LOCAL: u32 = 9;
    const BUFFER_LOCAL: u32 = 10;
    const SCRATCH_LOCAL: u32 = 11;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 3));

    function.instruction(&Instruction::I64Const(HEAP_FUNCTION_OBJECT_SIZE as i64));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(OBJECT_LOCAL));
    function.instruction(&Instruction::I64Const(
        (MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE) as i64,
    ));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, HEAP_PTR_OFFSET, BUFFER_LOCAL);
    helper_store_i64_const_at_offset(&mut function, OBJECT_LOCAL, HEAP_LEN_OFFSET, 0);
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_CAP_OFFSET,
        MIN_HEAP_CAPACITY as i64,
    );

    function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_PROTOTYPE_OFFSET,
        SCRATCH_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
        ValueKind::Object.tag() as i64,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TABLE_INDEX_OFFSET,
        TABLE_INDEX_LOCAL,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_ENV_HANDLE_OFFSET,
        ENV_HANDLE_LOCAL,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_FLAGS_OFFSET,
        FLAGS_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
        ValueKind::Undefined.tag() as i64,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
        0,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
        TO_STRING_PAYLOAD_LOCAL,
    );

    function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_DEFINING_REALM_OFFSET,
        SCRATCH_LOCAL,
    );

    for (global_index, offset) in [
        (
            ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
        ),
        (
            DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
        ),
        (
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
        ),
        (
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
        ),
        (
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
        ),
    ] {
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    for (_, global_index, offset) in error_realm_prototype_entries() {
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    for (constructor_global_index, offset) in [
        (
            FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_FLOAT64_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_FLOAT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT16_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT8_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT16_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            BIGINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BIGINT64_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            BIGUINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BIGUINT64_ARRAY_PROTOTYPE_OFFSET,
        ),
    ] {
        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        function.instruction(&Instruction::LocalGet(SCRATCH_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
        0,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        0,
    );

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::LocalGet(LENGTH_KEY_LOCAL));
    function.instruction(&Instruction::LocalGet(LENGTH_PAYLOAD_LOCAL));
    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::Call(
        object_append_data_property_function_index,
    ));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::LocalGet(NAME_KEY_LOCAL));
    function.instruction(&Instruction::LocalGet(NAME_PAYLOAD_LOCAL));
    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::Call(
        object_append_data_property_function_index,
    ));

    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::I64Const(
        OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
    ));
    function.instruction(&Instruction::I64And);
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    helper_store_i64_const_at_offset(&mut function, OBJECT_LOCAL, HEAP_CAP_OFFSET, 0);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::End);
    function
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_class_definition_payload(
        &mut self,
        class: &ClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_meta = self
            .functions
            .get(&class.constructor_function_id)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unknown class constructor `{}`",
                    class.constructor_function_id
                ))
            })?
            .clone();
        if let Some(name_binding) = &class.name_binding {
            self.push_scope();
            self.emit_enter_lexical_environment(&name_binding.environment, function)?;
        }
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let heritage_payload_local = self.reserve_temp_local();
        let heritage_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let computed_field_key_count = class
            .element_plan
            .definitions
            .iter()
            .filter_map(|definition| match definition {
                ClassElementDefinitionIr::ComputedFieldKey { slot, .. } => Some(*slot + 1),
                ClassElementDefinitionIr::PublicMethod(_)
                | ClassElementDefinitionIr::PrivateMethod(_) => None,
            })
            .max()
            .unwrap_or(0);
        let class_element_context_local = class
            .element_plan
            .static_elements
            .iter()
            .any(|element| match element {
                ClassStaticElementIr::Field(field) => field.init_function_id.is_some(),
                ClassStaticElementIr::Block(_) => true,
            })
            .then(|| self.reserve_temp_local());
        let field_keys_local = (computed_field_key_count > 0).then(|| self.reserve_temp_local());
        let class_private_scope = class
            .private_name_ids
            .values()
            .next()
            .copied()
            .map(PrivateNameId::class_scope);
        debug_assert!(class
            .private_name_ids
            .values()
            .all(|private_name_id| Some(private_name_id.class_scope()) == class_private_scope));
        let private_environment_local = Some(self.reserve_temp_local());

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(heritage_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(heritage_tag_local));
        if let Some(heritage) = &class.heritage {
            self.compile_expr_to_locals(
                heritage,
                heritage_payload_local,
                heritage_tag_local,
                function,
            )?;
        }

        match class.heritage_kind {
            ClassHeritageKind::Constructable => {
                if class.heritage.is_none() {
                    return Err(EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing class heritage",
                    ));
                }
                function.instruction(&Instruction::LocalGet(heritage_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(heritage_payload_local));
                function.instruction(&Instruction::Else);
                self.emit_is_constructor_i32(heritage_tag_local, heritage_payload_local, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    "TypeError",
                    "class extends value is not a constructor or null",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ClassHeritageKind::Null | ClassHeritageKind::None => {}
        }

        self.emit_function_value_payload(&constructor_meta, function)?;
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        if let Some(private_environment_local) = private_environment_local {
            self.emit_current_private_environment_to_local(key_local, function);
            if let Some(class_private_scope) = class_private_scope {
                self.emit_heap_alloc_const(
                    HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET
                        + class.private_name_ids.len() as u64 * HEAP_PRIVATE_ENV_SLOT_SIZE,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(private_environment_local));
                self.store_i64_local_at_offset(
                    private_environment_local,
                    HEAP_PRIVATE_ENV_PARENT_OFFSET,
                    key_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    private_environment_local,
                    HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
                    class_private_scope as u64,
                    function,
                );
            } else {
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalSet(private_environment_local));
            }
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                key_local,
                function,
            );
            self.store_i64_local_at_offset(
                key_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local,
                function,
            );
            self.active_private_environment_locals
                .push(private_environment_local);
        }
        if let Some(field_keys_local) = field_keys_local {
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                key_local,
                function,
            );
            self.emit_heap_alloc_const(
                ENV_SLOT_BASE_OFFSET + computed_field_key_count as u64 * ENV_SLOT_SIZE,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(field_keys_local));
            self.store_i64_const_at_offset(field_keys_local, ENV_PARENT_OFFSET, 0, function);
            self.store_i64_local_at_offset(
                key_local,
                HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                field_keys_local,
                function,
            );
        }
        if class.heritage_kind == ClassHeritageKind::Constructable {
            function.instruction(&Instruction::LocalGet(heritage_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_function_flags(constructor_local, flags_local, function);
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::I64Const(
                FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR as i64,
            ));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(flags_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_FLAGS_OFFSET,
                flags_local,
                function,
            );
            function.instruction(&Instruction::Else);
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                heritage_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                heritage_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        if class.heritage_kind == ClassHeritageKind::Constructable {
            function.instruction(&Instruction::LocalGet(heritage_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                heritage_payload_local,
                heritage_tag_local,
                heritage_payload_local,
                heritage_tag_local,
                prototype_key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(value_payload_local),
                Some(value_tag_local),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        } else if class.heritage_kind == ClassHeritageKind::Null {
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        } else {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        // Constructors are allocated before their `.prototype` exists.  Now
        // that the exact instance home object has been created, complete the
        // immutable class-function context used by direct constructor `super`.
        self.store_class_function_home_object(
            constructor_local,
            prototype_payload_local,
            ValueKind::Object,
            function,
        );
        if let Some(class_element_context_local) = class_element_context_local {
            self.emit_alloc_class_execution_context(
                self.current_env_local,
                Some((constructor_local, ValueKind::Function)),
                class_element_context_local,
                function,
            )?;
            if let Some(private_environment_local) = private_environment_local {
                self.store_i64_local_at_offset(
                    class_element_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                    private_environment_local,
                    function,
                );
            }
        }
        self.emit_object_define_data(
            constructor_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            prototype_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        let mut static_private_method_brands = BTreeSet::new();
        for definition in &class.element_plan.definitions {
            let (function_id, placement, kind, private_name_id) = match definition {
                ClassElementDefinitionIr::PublicMethod(method) => {
                    let compiled_key_local =
                        self.compile_object_key_to_local(&method.key, function)?;
                    function.instruction(&Instruction::LocalGet(compiled_key_local));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.release_temp_local(compiled_key_local);
                    (&method.function_id, method.placement, method.kind, None)
                }
                ClassElementDefinitionIr::PrivateMethod(method) => (
                    &method.function_id,
                    method.placement,
                    method.kind,
                    Some(method.private_name_id),
                ),
                ClassElementDefinitionIr::ComputedFieldKey { slot, key } => {
                    let field_keys_local =
                        field_keys_local.expect("computed field key cache must be allocated");
                    self.compile_object_key_to_locals(key, key_local, value_tag_local, function)?;
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET
                            + *slot as u64 * ENV_SLOT_SIZE
                            + ENV_SLOT_PAYLOAD_OFFSET,
                        key_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET + *slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_TAG_OFFSET,
                        value_tag_local,
                        function,
                    );
                    continue;
                }
            };
            let target_local = match placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            let meta = self.functions.get(function_id).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unknown class method `{function_id}`"
                ))
            })?;
            self.emit_class_function_value_payload(
                meta,
                target_local,
                private_environment_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            match kind {
                ClassMethodKindIr::Method => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_method_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_data(
                            target_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    }
                }
                ClassMethodKindIr::Getter => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_getter_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_accessor(
                            target_local,
                            key_local,
                            Some((value_payload_local, value_tag_local)),
                            None,
                            function,
                        )?;
                    }
                }
                ClassMethodKindIr::Setter => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_setter_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_accessor(
                            target_local,
                            key_local,
                            None,
                            Some((value_payload_local, value_tag_local)),
                            function,
                        )?;
                    }
                }
            }
            if placement == ClassMethodPlacementIr::Static {
                if let Some(private_name_id) = private_name_id {
                    static_private_method_brands.insert(private_name_id);
                }
            }
        }

        if let Some(name_binding) = &class.name_binding {
            let storage = self
                .lookup_current_scope_binding(&name_binding.storage_name)
                .expect("class name environment must expose its binding");
            self.write_binding_from_locals(
                storage,
                constructor_local,
                constructor_tag_local,
                function,
            );
        }
        for private_name_id in static_private_method_brands {
            self.emit_private_name_token_to_local(private_name_id, key_local, function)?;
            self.emit_private_brand_add(
                constructor_local,
                constructor_tag_local,
                key_local,
                function,
            )?;
        }

        for static_element in &class.element_plan.static_elements {
            match static_element {
                ClassStaticElementIr::Field(field) => {
                    self.load_i64_to_local_from_offset(
                        constructor_local,
                        HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                        value_tag_local,
                        function,
                    );
                    self.emit_class_field_key_to_local(
                        &field.key,
                        value_tag_local,
                        key_local,
                        function,
                    );
                    if let Some(init_function_id) = &field.init_function_id {
                        let meta = self.functions.get(init_function_id).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unknown class field init `{init_function_id}`"
                            ))
                        })?;
                        if meta.class_element_execution_kind
                            != ClassElementExecutionKind::StaticFieldInitializer
                        {
                            return Err(EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: class field init `{init_function_id}` has invalid execution kind"
                            )));
                        }
                        self.emit_direct_class_element_js_call(
                            meta,
                            class_element_context_local
                                .expect("static initializer context must exist"),
                            Some((constructor_local, Some(constructor_tag_local))),
                            &[],
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(value_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(value_tag_local));
                    }
                    if let ClassFieldKeyIr::Private(private_name_id) = &field.key {
                        self.emit_private_name_token_to_local(
                            *private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_field_add(
                            constructor_local,
                            constructor_tag_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_enumerable_data(
                            constructor_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    }
                }
                ClassStaticElementIr::Block(block) => {
                    let meta = self.functions.get(&block.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown class static block `{}`",
                            block.function_id
                        ))
                    })?;
                    if meta.class_element_execution_kind != ClassElementExecutionKind::StaticBlock {
                        return Err(EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: class static block `{}` has invalid execution kind",
                            block.function_id
                        )));
                    }
                    self.emit_direct_class_element_js_call(
                        meta,
                        class_element_context_local.expect("static block context must exist"),
                        Some((constructor_local, Some(constructor_tag_local))),
                        &[],
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
            }
        }

        if private_environment_local.is_some() {
            self.active_private_environment_locals.pop();
        }
        if class.name_binding.is_some() {
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
        }
        function.instruction(&Instruction::LocalGet(constructor_local));
        if let Some(private_environment_local) = private_environment_local {
            self.release_temp_local(private_environment_local);
        }
        if let Some(field_keys_local) = field_keys_local {
            self.release_temp_local(field_keys_local);
        }
        if let Some(class_element_context_local) = class_element_context_local {
            self.release_temp_local(class_element_context_local);
        }
        self.release_temp_local(flags_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(heritage_tag_local);
        self.release_temp_local(heritage_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn normalize_derived_constructor_result(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Nested arrows carry the same activation metadata so their lexical
        // `this`/`super` reads reach the owner invocation, but they remain
        // ordinary calls. Only the actual derived [[Construct]] body applies
        // the special object/undefined return normalization.
        if !self.is_derived_constructor {
            return Ok(());
        }
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(self.result_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        // An object returned explicitly from a derived constructor wins, even
        // when `super()` was never evaluated.
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_get_derived_this_to_locals(self.result_local, self.result_tag_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            "TypeError",
            "derived constructor may only return object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_get_derived_this_to_locals(self.result_local, self.result_tag_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_adapt_call_this_arg(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Function.prototype.call/apply thisArg adaptation failed",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_load_bound_function_record(
        &mut self,
        record_local: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        self_payload_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
            self_payload_local,
            function,
        );
    }

    pub(crate) fn emit_concat_argv_payloads(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_len_local = self.reserve_temp_local();
        let rhs_len_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let lhs_index_local = self.reserve_temp_local();
        let rhs_index_local = self.reserve_temp_local();
        let dst_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_LEN_OFFSET,
            lhs_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_LEN_OFFSET,
            rhs_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));

        self.emit_alloc_array_with_len_local(
            total_len_local,
            dst_payload_local,
            dst_buffer_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            lhs_payload_local,
            lhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            rhs_payload_local,
            rhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(dst_index_local);
        self.release_temp_local(rhs_index_local);
        self.release_temp_local(lhs_index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(rhs_len_local);
        self.release_temp_local(lhs_len_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_array_with_len_local(
        &mut self,
        len_local: u32,
        payload_local: u32,
        buffer_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            return Ok(());
        }
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(payload_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            payload_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_exotic_slots(payload_local, function);

        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_bound_function_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Bound function objects dispatch through `[[BoundFunctionInvoke]]`'s
        // funcref-table slot, so its real body must be emitted.
        self.functions
            .record_standard_builtin(StandardBuiltinId::BoundFunctionInvoker);
        let meta = self
            .functions
            .get(&StandardBuiltinId::BoundFunctionInvoker.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `[[BoundFunctionInvoke]]`",
                )
            })?;
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BOUND_FUNCTION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );

        self.emit_load_function_constructable_flag(target_payload_local, flags_local, function);
        self.emit_heap_alloc_const(HEAP_FUNCTION_OBJECT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
            object_local,
            function,
        );
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            meta.table_index as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            flags_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload(meta.to_string_value.as_str()) as u64,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        for (_, _, offset) in error_realm_prototype_entries() {
            self.load_i64_to_local_from_offset(
                target_payload_local,
                offset,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(object_local, offset, self.scratch_local, function);
        }
        self.copy_function_realm_typed_array_prototypes(
            target_payload_local,
            object_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(flags_local);
        self.release_temp_local(record_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_function_or_proxy_construct_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            return self.emit_function_handle_construct_with_argv(
                callee_payload_local,
                callee_tag_local,
                new_target_payload_local,
                new_target_tag_local,
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            );
        }

        if self.outline_proxy_construct {
            if let Some(helper) = self.proxy_construct_helper_function_index() {
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::LocalGet(new_target_payload_local));
                function.instruction(&Instruction::LocalGet(new_target_tag_local));
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::LocalGet(argv_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                return Ok(());
            }
        }

        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_key_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let argv_tag_local = self.reserve_temp_local();
        let trap_args_payload_local = self.reserve_temp_local();
        let proxy_type_error_prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_construct_with_argv(
            current_payload_local,
            current_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET,
            proxy_type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            proxy_type_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("construct")));
        function.instruction(&Instruction::LocalSet(trap_key_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_break_current_completion_if_throw(2, function);

        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_array_like_snapshot_payload(
            argv_local,
            argv_tag_local,
            trap_args_payload_local,
            "Reflect.construct argumentsList must be an array",
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_proxy_call_helper_leave_throw_completion(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[
                (target_payload_local, target_tag_local),
                (trap_args_payload_local, argv_tag_local),
                (new_target_payload_local, new_target_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_break_current_completion_if_throw(3, function);
        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy construct trap returned non-object",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy construct trap is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_type_error_prototype_local);
        self.release_temp_local(trap_args_payload_local);
        self.release_temp_local(argv_tag_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(trap_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_construct_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let proto_is_object_local = self.reserve_temp_local();
        let instance_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let call_completion_local = self.reserve_temp_local();
        let callee_constructable_local = self.reserve_temp_local();
        let callee_flags_local = self.reserve_temp_local();
        let construct_this_payload_local = self.reserve_temp_local();
        let construct_this_tag_local = self.reserve_temp_local();
        let array_buffer_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let object_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ObjectConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let data_view_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::DataViewConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let proxy_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ProxyConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let aggregate_error_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::AggregateErrorConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let suppressed_error_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::SuppressedErrorConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let number_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::NumberConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let string_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::StringConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let boolean_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::BooleanConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let direct_returning_constructor_table_indices: Vec<i64> = [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
            StandardBuiltinId::SharedArrayBufferConstructor,
            StandardBuiltinId::PromiseConstructor,
            StandardBuiltinId::MapConstructor,
            StandardBuiltinId::WeakMapConstructor,
            StandardBuiltinId::WeakSetConstructor,
            StandardBuiltinId::WeakRefConstructor,
            StandardBuiltinId::FinalizationRegistryConstructor,
            StandardBuiltinId::AsyncDisposableStackConstructor,
            StandardBuiltinId::SetConstructor,
            StandardBuiltinId::TemporalZonedDateTimeConstructor,
        ]
        .into_iter()
        .filter_map(|builtin| {
            self.functions
                .get(&builtin.function_id())
                .map(|meta| meta.table_index as i64)
        })
        .collect();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_load_function_flags(callee_payload_local, callee_flags_local, function);
        function.instruction(&Instruction::LocalGet(callee_flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(callee_constructable_local));
        function.instruction(&Instruction::LocalGet(callee_constructable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );

        // Derived constructors provide their receiver through `super()`, so
        // their [[Construct]] path must not inspect newTarget.prototype or
        // allocate a base receiver first. Their function body already
        // normalizes its result according to the derived-constructor rules.
        function.instruction(&Instruction::LocalGet(callee_flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_DERIVED_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        if let Some(array_buffer_constructor_table_index) = array_buffer_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_buffer_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(data_view_constructor_table_index) = data_view_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(data_view_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(proxy_constructor_table_index) = proxy_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(proxy_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(aggregate_error_constructor_table_index) =
            aggregate_error_constructor_table_index
        {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(
                aggregate_error_constructor_table_index,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(suppressed_error_constructor_table_index) =
            suppressed_error_constructor_table_index
        {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(
                suppressed_error_constructor_table_index,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        for table_index in direct_returning_constructor_table_indices {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        // Ordinary [[Construct]] performs exactly one observable Get on the
        // original newTarget. In particular, do this before inspecting an
        // internal Proxy/bound representation: a Proxy's own get trap is not
        // replaceable by a read on its target.
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            proto_key_local,
            proto_payload_local,
            proto_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            proto_payload_local,
            proto_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proto_is_object_local));
        // A primitive result selects the intrinsic from GetFunctionRealm of
        // the original newTarget. Do this only after the observable Get above:
        // the get trap may revoke a Proxy, which GetFunctionRealm must then
        // reject rather than silently using the current realm.
        function.instruction(&Instruction::LocalGet(proto_is_object_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let prototype_realm_result =
            self.emit_get_function_realm(new_target_payload_local, new_target_tag_local, function);
        let prototype_realm = self.emit_route_function_realm_result(
            prototype_realm_result,
            FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch {
                payload_local,
                tag_local,
                // revoked if, primitive-prototype if, outer construct block
                relative_depth: 2,
            },
            function,
        )?;

        let ordinary_prototype = self.emit_load_required_resolved_realm_ordinary_prototype(
            prototype_realm,
            OrdinaryDefaultPrototype::Object,
            function,
        );
        self.emit_install_resolved_realm_ordinary_prototype(
            ordinary_prototype,
            proto_payload_local,
            proto_tag_local,
            function,
        );
        if let Some(string_constructor_table_index) = string_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(string_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            let ordinary_prototype = self.emit_load_required_resolved_realm_ordinary_prototype(
                prototype_realm,
                OrdinaryDefaultPrototype::String,
                function,
            );
            self.emit_install_resolved_realm_ordinary_prototype(
                ordinary_prototype,
                proto_payload_local,
                proto_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(array_constructor_table_index) = array_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_required_resolved_realm_array_prototype(
                prototype_realm,
                proto_payload_local,
                proto_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(number_constructor_table_index) = number_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(number_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            let ordinary_prototype = self.emit_load_required_resolved_realm_ordinary_prototype(
                prototype_realm,
                OrdinaryDefaultPrototype::Number,
                function,
            );
            self.emit_install_resolved_realm_ordinary_prototype(
                ordinary_prototype,
                proto_payload_local,
                proto_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(boolean_constructor_table_index) = boolean_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(boolean_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            let ordinary_prototype = self.emit_load_required_resolved_realm_ordinary_prototype(
                prototype_realm,
                OrdinaryDefaultPrototype::Boolean,
                function,
            );
            self.emit_install_resolved_realm_ordinary_prototype(
                ordinary_prototype,
                proto_payload_local,
                proto_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        // `GetFunctionRealm` reserves its opaque result after the constructor
        // frame locals above. Release that proof as soon as its last emitted
        // use is complete so the temporary-local stack remains strictly LIFO.
        self.release_resolved_function_realm_local(prototype_realm);

        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(proto_payload_local),
            Some(proto_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(instance_local));

        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::LocalSet(construct_this_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(construct_this_tag_local));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(construct_this_payload_local));
        function.instruction(&Instruction::LocalGet(construct_this_tag_local));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        for (constructor_table_index, primitive_tag, boxed_kind) in [
            (
                number_constructor_table_index,
                ValueKind::Number,
                BOXED_PRIMITIVE_KIND_NUMBER,
            ),
            (
                string_constructor_table_index,
                ValueKind::String,
                BOXED_PRIMITIVE_KIND_STRING,
            ),
            (
                boolean_constructor_table_index,
                ValueKind::Boolean,
                BOXED_PRIMITIVE_KIND_BOOLEAN,
            ),
        ] {
            if let Some(constructor_table_index) = constructor_table_index {
                function.instruction(&Instruction::LocalGet(table_index_local));
                function.instruction(&Instruction::I64Const(constructor_table_index));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(call_tag_local));
                function.instruction(&Instruction::I64Const(primitive_tag.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_store_boxed_primitive_metadata(
                    instance_local,
                    boxed_kind,
                    call_payload_local,
                    call_tag_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
        }

        if let Some(array_constructor_table_index) = array_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_local_at_offset(
                call_payload_local,
                HEAP_PROTOTYPE_OFFSET,
                proto_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                call_payload_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                proto_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if let Some(object_constructor_table_index) = object_constructor_table_index {
            // With a distinct newTarget, Object's construct path must select
            // the pre-created receiver. In particular it must not preserve an
            // object argument returned by Object(value).
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(object_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            self.emit_tagged_payload_same_value_i32(
                new_target_tag_local,
                new_target_payload_local,
                callee_tag_local,
                callee_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(instance_local));
            function.instruction(&Instruction::LocalSet(call_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_tag_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_heap_object_like_tag_i32(call_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(construct_this_tag_local);
        self.release_temp_local(construct_this_payload_local);
        self.release_temp_local(callee_flags_local);
        self.release_temp_local(callee_constructable_local);
        self.release_temp_local(call_completion_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(instance_local);
        self.release_temp_local(proto_is_object_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    pub(crate) fn copy_function_realm_typed_array_prototypes(
        &self,
        source_function_local: u32,
        target_function_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (builtin, _) in typed_array_constructor_bytes_per_element_entries() {
            let offset = typed_array_realm_prototype_offset(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing typed array realm prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.load_i64_to_local_from_offset(
                source_function_local,
                offset,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(
                target_function_local,
                offset,
                self.scratch_local,
                function,
            );
        }
        Ok(())
    }

    pub(crate) fn store_typed_array_realm_prototype_locals(
        &self,
        object_local: u32,
        prototype_locals: &[(StandardBuiltinId, u32)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (builtin, prototype_local) in prototype_locals {
            let offset = typed_array_realm_prototype_offset(*builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing typed array realm prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.store_i64_local_at_offset(object_local, offset, *prototype_local, function);
        }
        Ok(())
    }

    /// Materialize a builtin function value inside a branch that is provably dead
    /// in this module (its guarding heap-shape/kind cannot exist here), without
    /// forcing the builtin's real body through the emission fixpoint. The written
    /// funcref points at the shared stub table slot, which is fine because the
    /// branch can never execute. See `FunctionMetaRegistry::suppress_recording`.
    pub(crate) fn emit_function_value_payload_unrecorded(
        &mut self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.functions.set_recording_suppressed(true);
        let result = self.emit_function_value_payload(meta, function);
        self.functions.set_recording_suppressed(previous);
        result
    }

    /// Emit the function object denoted by one compiler-owned identity.
    ///
    /// Most identities have a Wasm function meta and are materialized on
    /// demand. Constructors and derived Function intrinsics instead have one
    /// canonical object allocated by realm bootstrap; those must be loaded,
    /// not re-emitted (and the dynamic-source identities intentionally have no
    /// backend meta at all).
    pub(crate) fn emit_function_identity_payload(
        &mut self,
        function_id: &FunctionId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(global_index) = preallocated_function_value_global_index(function_id) {
            function.instruction(&Instruction::GlobalGet(global_index));
            return Ok(());
        }

        let meta = self.functions.get(function_id).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: unknown function value `{function_id}`"
            ))
        })?;
        self.emit_function_value_payload(meta, function)
    }

    /// Emits parameter zero for a standard builtin call or function object.
    /// Created-realm standard builtins carry a self-backed realm record in
    /// their environment slot. A user function's nonzero environment is a
    /// lexical-environment allocation with a different layout and must never
    /// be interpreted as realm metadata by a builtin.
    fn emit_standard_builtin_realm_env_argument(&self, function: &mut Function) {
        if self
            .function_id
            .as_ref()
            .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
            .is_some()
        {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
    }

    pub(crate) fn emit_function_value_payload(
        &mut self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_value_payload_with_prototype_materialization(
            meta,
            FunctionPrototypeMaterialization::Automatic,
            function,
        )
    }

    pub(crate) fn emit_function_value_payload_with_prototype_materialization(
        &mut self,
        meta: &WasmFunctionMeta,
        prototype_materialization: FunctionPrototypeMaterialization,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // This is the choke point that makes a builtin's funcref-table slot
        // reachable at runtime (a function object now carries it), so its real
        // body must be emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        let function_object_alloc_function_index =
            self.function_object_alloc_function_index.ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing function object helper",
                )
            })?;
        let is_html_dda = meta.host_builtin == Some(HostBuiltinId::HTMLDDA);
        let flags = (if meta.protocol.is_constructable() {
            FUNCTION_FLAG_CONSTRUCTABLE
        } else {
            0
        }) | if meta.protocol.class_kind() == ClassFunctionKind::Constructor {
            FUNCTION_FLAG_CLASS_CONSTRUCTOR
        } else {
            0
        } | if meta.is_derived_constructor {
            FUNCTION_FLAG_DERIVED_CONSTRUCTOR
        } else {
            0
        } | if meta.is_synthetic_default_derived_constructor {
            FUNCTION_FLAG_SYNTHETIC_DEFAULT_DERIVED_CONSTRUCTOR
        } else {
            0
        } | if meta.class_heritage_kind == ClassHeritageKind::Null {
            FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR
        } else {
            0
        } | if meta.uses_super {
            FUNCTION_FLAG_USES_SUPER
        } else {
            0
        } | if meta.this_before_super {
            FUNCTION_FLAG_THIS_BEFORE_SUPER
        } else {
            0
        } | if meta.strict { FUNCTION_FLAG_STRICT } else { 0 }
            | if is_html_dda {
                FUNCTION_FLAG_IS_HTMLDDA
            } else {
                0
            }
            | if meta.protocol.execution_kind() == FunctionExecutionKind::Generator {
                FUNCTION_FLAG_GENERATOR
            } else {
                0
            }
            | if meta.protocol.execution_kind() == FunctionExecutionKind::Async {
                FUNCTION_FLAG_ASYNC
            } else {
                0
            }
            | if meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator {
                FUNCTION_FLAG_ASYNC_GENERATOR
            } else {
                0
            };
        let object_local = self.reserve_temp_local();
        let named_context_local = meta.is_named_expression.then(|| self.reserve_temp_local());
        let function_context_local = meta
            .has_function_context()
            .then(|| self.reserve_temp_local());
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let proto_value_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();

        if let Some(named_context_local) = named_context_local {
            self.emit_heap_alloc_const(ENV_SLOT_BASE_OFFSET + ENV_SLOT_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(named_context_local));
            self.store_i64_local_at_offset(
                named_context_local,
                ENV_PARENT_OFFSET,
                self.current_env_local,
                function,
            );
        }
        if let Some(function_context_local) = function_context_local {
            self.emit_alloc_class_execution_context(
                named_context_local.unwrap_or(self.current_env_local),
                None,
                function_context_local,
                function,
            )?;
            if meta.captures_private_environment {
                let private_environment_local = self.reserve_temp_local();
                self.emit_current_private_environment_to_local(private_environment_local, function);
                self.store_i64_local_at_offset(
                    function_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                    private_environment_local,
                    function,
                );
                self.release_temp_local(private_environment_local);
            }
        }
        function.instruction(&Instruction::I64Const(meta.table_index as i64));
        if let Some(function_context_local) = function_context_local {
            function.instruction(&Instruction::LocalGet(function_context_local));
        } else if let Some(named_context_local) = named_context_local {
            function.instruction(&Instruction::LocalGet(named_context_local));
        } else if meta.standard_builtin.is_some() {
            self.emit_standard_builtin_realm_env_argument(function);
        } else {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
        }
        function.instruction(&Instruction::I64Const(flags as i64));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(meta.to_string_value.as_str()),
        ));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::F64Const(Ieee64::from(meta.length as f64)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(meta.runtime_name()),
        ));
        function.instruction(&Instruction::I64Const(
            crate::objects::object_data_descriptor_kind(false, false, meta.length_name_configurable)
                as i64,
        ));
        function.instruction(&Instruction::Call(function_object_alloc_function_index));
        function.instruction(&Instruction::LocalSet(object_local));
        if meta.has_function_context() {
            let function_context_local =
                function_context_local.expect("function context must be allocated");
            self.store_i64_local_at_offset(
                function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
                object_local,
                function,
            );
        }
        if let Some(named_context_local) = named_context_local {
            self.store_i64_local_at_offset(
                named_context_local,
                ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
                object_local,
                function,
            );
            self.store_i64_const_at_offset(
                named_context_local,
                ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
                ValueKind::Function.tag() as u64,
                function,
            );
        }

        if !meta.length_name_configurable {
            self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, 0, function);
        }

        if let Some(prototype_global_index) =
            syntax_function_object_prototype_global_index(meta.protocol.execution_kind())
        {
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
        }

        let instance_prototype_global_index =
            syntax_function_instance_prototype_global_index(meta.protocol.execution_kind());
        if prototype_materialization == FunctionPrototypeMaterialization::Automatic
            && !is_html_dda
            && (meta.protocol.is_constructable() || instance_prototype_global_index.is_some())
        {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(instance_prototype_global_index.unwrap_or(OBJECT_PROTOTYPE_GLOBAL_INDEX)),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                proto_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                true,
                false,
                false,
                function,
            )?;

            if !matches!(
                meta.protocol.execution_kind(),
                FunctionExecutionKind::Generator | FunctionExecutionKind::AsyncGenerator
            ) {
                function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(proto_value_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(proto_tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_local,
                    key_local,
                    proto_value_local,
                    proto_tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
            }
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_value_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        if let Some(function_context_local) = function_context_local {
            self.release_temp_local(function_context_local);
        }
        if let Some(named_context_local) = named_context_local {
            self.release_temp_local(named_context_local);
        }
        self.release_temp_local(object_local);
        Ok(())
    }

    /// Materialize a function and attach its defining realm before exposing
    /// the destination local to the caller.
    ///
    /// Synthetic realm bootstrap must use this choke point instead of
    /// allocating under `CURRENT_REALM` and repairing the function header in
    /// a separate statement that a new builtin can forget.
    pub(crate) fn emit_function_value_payload_in_realm(
        &mut self,
        meta: &WasmFunctionMeta,
        defining_realm: RealmRecordLocal,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_value_payload_in_realm_with_prototype_materialization(
            meta,
            FunctionPrototypeMaterialization::Automatic,
            defining_realm,
            function_object_local,
            function,
        )
    }

    /// Materialize the created realm's `%Array%` constructor without first
    /// allocating the ordinary automatic prototype that realm bootstrap will
    /// replace with its initialized Array exotic.
    pub(crate) fn emit_realm_array_constructor_value_payload(
        &mut self,
        defining_realm: RealmRecordLocal,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        self.emit_function_value_payload_in_realm_with_prototype_materialization(
            &meta,
            FunctionPrototypeMaterialization::BootstrapSupplied,
            defining_realm,
            function_object_local,
            function,
        )
    }

    fn emit_function_value_payload_in_realm_with_prototype_materialization(
        &mut self,
        meta: &WasmFunctionMeta,
        prototype_materialization: FunctionPrototypeMaterialization,
        defining_realm: RealmRecordLocal,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_value_payload_with_prototype_materialization(
            meta,
            prototype_materialization,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        self.emit_store_function_defining_realm(
            function_object_local,
            defining_realm.index(),
            function,
        );
        Ok(())
    }

    pub(crate) fn reserve_realm_array_prototype_local(
        &mut self,
    ) -> ReservedRealmArrayPrototypeLocal {
        ReservedRealmArrayPrototypeLocal(self.reserve_temp_local())
    }

    /// Consume reserved storage and initialize it with the Array exotic
    /// layout required by a created realm's `%Array.prototype%`.
    pub(crate) fn emit_initialize_realm_array_prototype(
        &mut self,
        reserved: ReservedRealmArrayPrototypeLocal,
        object_prototype_local: u32,
        function: &mut Function,
    ) -> Result<RealmArrayPrototypeLocal, EmitError> {
        let length_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        self.emit_alloc_array_payload_with_length(length_local, reserved.0, function)?;
        self.store_i64_local_at_offset(
            reserved.0,
            HEAP_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            reserved.0,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.release_temp_local(length_local);
        Ok(RealmArrayPrototypeLocal(reserved.0))
    }

    pub(crate) fn emit_store_realm_array_prototype(
        &mut self,
        realm: RealmRecordLocal,
        prototype: &RealmArrayPrototypeLocal,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_local_at_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            prototype.0,
            function,
        );
        self.release_temp_local(intrinsics_local);
    }

    pub(crate) fn emit_define_realm_array_prototype_data_with_flags(
        &mut self,
        prototype: &RealmArrayPrototypeLocal,
        key: &str,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let enumerable_local = self.reserve_temp_local();
        let configurable_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(i64::from(writable)));
        function.instruction(&Instruction::LocalSet(writable_local));
        function.instruction(&Instruction::I64Const(i64::from(enumerable)));
        function.instruction(&Instruction::LocalSet(enumerable_local));
        function.instruction(&Instruction::I64Const(i64::from(configurable)));
        function.instruction(&Instruction::LocalSet(configurable_local));
        self.emit_array_define_named_data_descriptor(
            prototype.0,
            key_local,
            payload_local,
            tag_local,
            writable_local,
            enumerable_local,
            configurable_local,
            None,
            None,
            None,
            None,
            None,
            function,
        )?;
        self.release_temp_local(configurable_local);
        self.release_temp_local(enumerable_local);
        self.release_temp_local(writable_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    /// Install the two `%Array%` / `%Array.prototype%` links using the
    /// representation and attributes required by the intrinsic registry.
    pub(crate) fn emit_bind_realm_array_constructor_prototype(
        &mut self,
        constructor_local: u32,
        prototype: &RealmArrayPrototypeLocal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype.0,
            function,
        );
        function.instruction(&Instruction::I64Const(
            self.strings
                .static_builtin_property_key_payload("prototype"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            constructor_local,
            key_local,
            prototype.0,
            tag_local,
            false,
            false,
            false,
            function,
        )?;

        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_define_realm_array_prototype_data_with_flags(
            prototype,
            "constructor",
            constructor_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn release_realm_array_prototype_local(
        &mut self,
        prototype: RealmArrayPrototypeLocal,
    ) {
        self.release_temp_local(prototype.0);
    }

    fn emit_alloc_class_execution_context(
        &mut self,
        lexical_env_local: u32,
        home_object: Option<(u32, ValueKind)>,
        context_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(HEAP_CLASS_FUNCTION_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(context_local));
        self.store_i64_local_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
            lexical_env_local,
            function,
        );
        self.store_i64_const_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
            0,
            function,
        );
        if let Some((home_object_local, home_object_kind)) = home_object {
            self.store_i64_local_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                home_object_local,
                function,
            );
            self.store_i64_const_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                home_object_kind.tag() as u64,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
        }
        self.store_i64_const_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
            0,
            function,
        );
        Ok(())
    }

    /// Materialize a class member function and attach the exact object on
    /// which the member is being defined as its [[HomeObject]].
    fn emit_class_function_value_payload(
        &mut self,
        meta: &WasmFunctionMeta,
        home_object_local: u32,
        private_environment_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        debug_assert_ne!(meta.protocol.class_kind(), ClassFunctionKind::None);
        let function_local = self.reserve_temp_local();
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(function_local));
        let home_object_tag = if meta.is_static_class_member {
            ValueKind::Function
        } else {
            ValueKind::Object
        };
        self.store_class_function_home_object(
            function_local,
            home_object_local,
            home_object_tag,
            function,
        );
        if let Some(private_environment_local) = private_environment_local {
            let context_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                function_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                context_local,
                function,
            );
            self.store_i64_local_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local,
                function,
            );
            self.release_temp_local(context_local);
        }
        function.instruction(&Instruction::LocalGet(function_local));
        self.release_temp_local(function_local);
        Ok(())
    }

    fn store_class_function_home_object(
        &mut self,
        function_local: u32,
        home_object_local: u32,
        home_object_tag: ValueKind,
        function: &mut Function,
    ) {
        let context_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            context_local,
            function,
        );
        self.store_i64_local_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
            home_object_local,
            function,
        );
        self.store_i64_const_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
            home_object_tag.tag() as u64,
            function,
        );
        self.release_temp_local(context_local);
    }

    pub(crate) fn emit_alloc_realm_record(
        &mut self,
        realm_id: u64,
        agent_id: u64,
        realm_local: u32,
        function: &mut Function,
    ) -> Result<RealmRecordLocal, EmitError> {
        let intrinsics_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_REALM_INTRINSICS_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(intrinsics_local));
        for offset in (0..HEAP_REALM_INTRINSICS_RECORD_SIZE).step_by(8) {
            self.store_i64_const_at_offset(intrinsics_local, offset, 0, function);
        }

        self.emit_heap_alloc_const(HEAP_REALM_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(realm_local));
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_ID_OFFSET, realm_id, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_AGENT_ID_OFFSET, agent_id, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_GLOBAL_OBJECT_OFFSET, 0, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_GLOBAL_THIS_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_HOST_HOOKS_OFFSET, 0, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_MODULE_REGISTRY_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            0,
            function,
        );
        self.release_temp_local(intrinsics_local);
        Ok(RealmRecordLocal(realm_local))
    }

    fn emit_store_function_defining_realm(
        &self,
        function_object_local: u32,
        realm_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_type_error_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::TypeErrorPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_non_array_realm_intrinsic(
        &mut self,
        realm_local: u32,
        slot: NonArrayRealmIntrinsicSlot,
        value_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_local_at_offset(intrinsics_local, slot.offset(), value_local, function);
        self.release_temp_local(intrinsics_local);
    }

    pub(crate) fn emit_store_current_realm_global_intrinsic(
        &mut self,
        value_global_index: u32,
        slot: NonArrayRealmIntrinsicSlot,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::GlobalGet(value_global_index));
        function.instruction(&Instruction::LocalSet(value_local));
        self.emit_store_non_array_realm_intrinsic(realm_local, slot, value_local, function);
        self.release_temp_local(value_local);
        self.release_temp_local(realm_local);
    }

    /// Publish the entry realm's already-initialized Array exotic. This is
    /// intentionally hard-coded so the generic non-Array slot domain cannot
    /// be bypassed with the Array offset and an arbitrary local.
    pub(crate) fn emit_store_current_realm_array_prototype_global(
        &mut self,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.store_i64_local_at_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        self.release_temp_local(prototype_local);
        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_store_realm_array_iterator_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::ArrayIteratorPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_string_iterator_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::StringIteratorPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_iterator_helper_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::IteratorHelperPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_iterator_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::IteratorPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_iterator_from_wrapper_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::IteratorFromWrapperPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_object_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_non_array_realm_intrinsic(
            realm_local,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_load_realm_intrinsic_prototype_or_global(
        &mut self,
        realm_local: u32,
        intrinsic_offset: u64,
        fallback_global_index: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let fallback_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(fallback_global_index));
        function.instruction(&Instruction::LocalSet(fallback_local));
        self.emit_load_realm_intrinsic_prototype_or_local(
            realm_local,
            intrinsic_offset,
            fallback_local,
            result_local,
            function,
        );
        self.release_temp_local(fallback_local);
    }

    pub(crate) fn emit_load_realm_intrinsic_prototype_or_local(
        &mut self,
        realm_local: u32,
        intrinsic_offset: u64,
        fallback_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        let candidate_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(fallback_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            intrinsic_offset,
            candidate_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_local);
        self.release_temp_local(intrinsics_local);
    }

    /// Load a required ordinary-object intrinsic from a realm proven by
    /// `GetFunctionRealm`.
    ///
    /// A resolved ECMAScript realm always has an intrinsic record and every
    /// intrinsic in [`OrdinaryDefaultPrototype`]. Missing backend bootstrap
    /// state is therefore an internal invariant failure, never permission to
    /// substitute an entry-realm global.
    fn emit_load_required_resolved_realm_ordinary_prototype(
        &mut self,
        realm: ResolvedFunctionRealmLocal,
        intrinsic: OrdinaryDefaultPrototype,
        function: &mut Function,
    ) -> ResolvedRealmOrdinaryPrototypeLocal {
        let prototype_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(realm.index()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            intrinsic.offset(),
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        ResolvedRealmOrdinaryPrototypeLocal(prototype_local)
    }

    /// Consume a required ordinary-object prototype and install its payload
    /// and representation tag as one transition.
    fn emit_install_resolved_realm_ordinary_prototype(
        &mut self,
        prototype: ResolvedRealmOrdinaryPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(prototype.0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(prototype.0);
    }

    /// Load the populated `%Array.prototype%` slot from a realm already
    /// proven by `GetFunctionRealm`, preserving its Array representation.
    /// Missing realm bootstrap state is an internal invariant failure.
    pub(crate) fn emit_load_required_resolved_realm_array_prototype(
        &mut self,
        realm: ResolvedFunctionRealmLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(realm.index()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(intrinsics_local);
    }

    /// Implements GetFunctionRealm's recursive bound/proxy traversal without
    /// performing any user-visible property access.
    ///
    /// Constructor callers invoke this only after their observable
    /// `Get(newTarget, "prototype")`; Promise jobs invoke it on the already
    /// captured callback. The returned locals are opaque until a caller
    /// consumes them through [`Self::emit_route_function_realm_result`].
    pub(crate) fn emit_get_function_realm(
        &mut self,
        source_payload_local: u32,
        source_tag_local: u32,
        function: &mut Function,
    ) -> FunctionRealmResultLocals {
        let realm_local = self.reserve_temp_local();
        let outcome_local = self.reserve_temp_local();
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let proxy_handler_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::I64Const(FunctionRealmOutcome::Invalid as i64));
        function.instruction(&Instruction::LocalSet(outcome_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_function_flags(current_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            current_tag_local,
            function,
        );
        // inner if, outer function-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Resolved as i64,
        ));
        function.instruction(&Instruction::LocalSet(outcome_local));
        function.instruction(&Instruction::End);
        // function-tag if, loop, exit block
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(FunctionRealmOutcome::Revoked as i64));
        function.instruction(&Instruction::LocalSet(outcome_local));
        // revoked if, proxy if, object-tag if, loop, exit block
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        // proxy if, object-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Unknown/non-callable representation: retain the explicit Invalid
        // outcome. Validated newTarget values must reach an ordinary function.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_handler_local);
        self.release_temp_local(record_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        FunctionRealmResultLocals {
            realm_local,
            outcome_local,
        }
    }

    /// Consume a raw GetFunctionRealm result, route a revoked Proxy according
    /// to the caller's closed policy, and trap an invalid callable
    /// representation before exposing the realm local.
    pub(crate) fn emit_route_function_realm_result(
        &mut self,
        result: FunctionRealmResultLocals,
        revoked_route: FunctionRealmRevokedRoute,
        function: &mut Function,
    ) -> Result<ResolvedFunctionRealmLocal, EmitError> {
        function.instruction(&Instruction::LocalGet(result.outcome_local));
        function.instruction(&Instruction::I64Const(FunctionRealmOutcome::Revoked as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        match revoked_route {
            FunctionRealmRevokedRoute::UseCurrentRealm => {
                function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(result.realm_local));
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                payload_local,
                tag_local,
            } => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "cannot get function realm from a revoked Proxy",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch {
                payload_local,
                tag_local,
                relative_depth,
            } => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "cannot get function realm from a revoked Proxy",
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Br(relative_depth));
            }
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result.outcome_local));
        function.instruction(&Instruction::I64Const(FunctionRealmOutcome::Invalid as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(result.outcome_local);
        Ok(ResolvedFunctionRealmLocal(result.realm_local))
    }

    pub(crate) fn release_resolved_function_realm_local(
        &mut self,
        realm: ResolvedFunctionRealmLocal,
    ) {
        self.release_temp_local(realm.index());
    }

    pub(crate) fn emit_load_function_defining_realm_type_error_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_throw_type_error(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(THROW_TYPE_ERROR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_array_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_map_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_MAP_ITERATOR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_set_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_SET_ITERATOR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_string_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_STRING_ITERATOR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_iterator_helper_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm_local,
            HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET,
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
            result_local,
            function,
        );
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm_local,
            HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET,
            ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            result_local,
            function,
        );
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_iterator_from_wrapper_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm_local,
            HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET,
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
            result_local,
            function,
        );
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_object_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_object_fields(
        &mut self,
        function_object_local: u32,
        env_local: u32,
        table_index_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            table_index_local,
            function,
        );
    }

    pub(crate) fn emit_start_async_generator_body(
        &mut self,
        activation_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let function_object_local = self.reserve_temp_local();
        let function_environment_local = self.reserve_temp_local();
        let this_payload_local = self.reserve_temp_local();
        let this_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let body_payload_local = self.reserve_temp_local();
        let body_tag_local = self.reserve_temp_local();
        let body_completion_local = self.reserve_temp_local();
        let body_aux_local = self.reserve_temp_local();
        let body_status_local = self.reserve_temp_local();
        let resolved_return_payload_local = self.reserve_temp_local();
        let resolved_return_tag_local = self.reserve_temp_local();

        for (offset, destination_local) in [
            (HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET, function_object_local),
            (
                HEAP_ASYNC_GENERATOR_FUNCTION_ENV_OFFSET,
                function_environment_local,
            ),
            (HEAP_ASYNC_GENERATOR_THIS_PAYLOAD_OFFSET, this_payload_local),
            (HEAP_ASYNC_GENERATOR_THIS_TAG_OFFSET, this_tag_local),
            (HEAP_ASYNC_GENERATOR_ARGC_OFFSET, argc_local),
            (HEAP_ASYNC_GENERATOR_ARGV_OFFSET, argv_local),
        ] {
            self.load_i64_to_local_from_offset(
                activation_local,
                offset,
                destination_local,
                function,
            );
        }
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            table_index_local,
            function,
        );

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            body_status_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_status_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_STATE_INITIALIZING as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_EXECUTING,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_RUNNING,
            function,
        );

        function.instruction(&Instruction::LocalGet(function_environment_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(activation_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            body_payload_local,
            body_tag_local,
            body_completion_local,
            body_aux_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
            body_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
            body_tag_local,
            function,
        );

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            body_status_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_status_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_BODY_STATUS_YIELD as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let delegate_record_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            delegate_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(delegate_record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_generator_yield_reactions(
            activation_local,
            body_payload_local,
            body_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let resume_body_local = self.reserve_temp_local();
        self.emit_complete_async_generator_yield(
            activation_local,
            body_payload_local,
            body_tag_local,
            resume_body_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(resume_body_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(2));
        self.release_temp_local(resume_body_local);
        function.instruction(&Instruction::End);
        self.release_temp_local(delegate_record_local);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(body_status_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_BODY_STATUS_RUNNING as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(body_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_COMPLETE,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_DRAINING_QUEUE,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_aux_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RETURN_VALUE_ALREADY_AWAITED as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(body_completion_local));
        self.emit_complete_async_generator_step(
            activation_local,
            body_payload_local,
            body_tag_local,
            body_completion_local,
            true,
            function,
        )?;
        self.emit_drain_async_generator_queue(activation_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_async_generator_await_return_reactions(
            activation_local,
            body_payload_local,
            body_tag_local,
            resolved_return_payload_local,
            resolved_return_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(body_completion_local));
        self.emit_complete_async_generator_step(
            activation_local,
            resolved_return_payload_local,
            resolved_return_tag_local,
            body_completion_local,
            true,
            function,
        )?;
        self.emit_drain_async_generator_queue(activation_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(body_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_THROW,
            function,
        );
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(body_completion_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(body_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(body_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(body_tag_local));
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
            body_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
            body_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(body_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_COMPLETE,
            function,
        );
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(body_completion_local));
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_DRAINING_QUEUE,
            function,
        );
        self.emit_complete_async_generator_step(
            activation_local,
            body_payload_local,
            body_tag_local,
            body_completion_local,
            true,
            function,
        )?;
        self.emit_drain_async_generator_queue(activation_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(resolved_return_tag_local);
        self.release_temp_local(resolved_return_payload_local);
        self.release_temp_local(body_status_local);
        self.release_temp_local(body_aux_local);
        self.release_temp_local(body_completion_local);
        self.release_temp_local(body_tag_local);
        self.release_temp_local(body_payload_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(this_tag_local);
        self.release_temp_local(this_payload_local);
        self.release_temp_local(function_environment_local);
        self.release_temp_local(function_object_local);
        Ok(())
    }

    pub(crate) fn emit_load_function_flags(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            result_local,
            function,
        );
    }

    pub(crate) fn emit_load_function_constructable_flag(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.emit_load_function_flags(function_object_local, result_local, function);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(result_local));
    }

    pub(crate) fn emit_function_handle_call(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_or_proxy_call_leave_throw_completion(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_proxy_call_helper_leave_throw_completion(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let helper = self
            .proxy_call_helper_function_index()
            .expect("proxy-call helper index must exist");
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(payload_local, tag_local, function);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    /// `emit_function_or_proxy_call_leave_throw_completion` plus the throw
    /// propagation, for callers that want the throw routed to the active
    /// handler rather than left in the completion tuple.
    ///
    /// Was `..._with_throw_extra_depth`: the trailing `u32` declared how many
    /// raw frames the caller had open, which the branch arithmetic could not
    /// see. It can see them now, so there is nothing to declare.
    pub(crate) fn emit_function_or_proxy_call_with_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            args,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)
    }

    /// See [`Self::emit_function_or_proxy_call_with_throw_propagation`] for why
    /// this no longer takes a depth.
    pub(crate) fn emit_function_handle_call_with_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            PropagateCallThrow::ToActiveHandler,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            PropagateCallThrow::ToActiveHandler,
            function,
        )
    }

    pub(crate) fn emit_function_handle_call_with_argv_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            PropagateCallThrow::LeaveInCompletion,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            true,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_leave_throw_completion(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            false,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_inner(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        return_on_throw: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_function_handle_call_with_argv_without_throw_propagation(
                callee_payload_local,
                callee_tag_local,
                Some((this_payload_local, Some(this_tag_local))),
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            )?;
            if return_on_throw {
                self.emit_return_current_completion_if_throw(function);
            }
            return Ok(());
        }

        if self.outline_proxy_call {
            if let Some(helper) = self.proxy_call_helper_function_index() {
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::LocalGet(argv_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                if return_on_throw {
                    self.emit_return_current_completion_if_throw(function);
                }
                return Ok(());
            }
        }

        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_key_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let argv_tag_local = self.reserve_temp_local();
        let trap_args_payload_local = self.reserve_temp_local();
        let proxy_type_error_prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            current_payload_local,
            current_tag_local,
            Some((this_payload_local, Some(this_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET,
            proxy_type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            proxy_type_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("apply")));
        function.instruction(&Instruction::LocalSet(trap_key_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        } else {
            self.emit_break_current_completion_if_throw(2, function);
        }

        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_array_like_snapshot_payload(
            argv_local,
            argv_tag_local,
            trap_args_payload_local,
            "Reflect.construct argumentsList must be an array",
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_proxy_call_helper_leave_throw_completion(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[
                (target_payload_local, target_tag_local),
                (this_payload_local, this_tag_local),
                (trap_args_payload_local, argv_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy apply trap is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(1));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_type_error_prototype_local);
        self.release_temp_local(trap_args_payload_local);
        self.release_temp_local(argv_tag_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(trap_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_with_argv_inner(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        propagate_throw: PropagateCallThrow,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_function_call {
            if let Some(helper) = self.function_call_helper_function_index() {
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                match this_locals {
                    Some((this_payload_local, Some(this_tag_local))) => {
                        function.instruction(&Instruction::LocalGet(this_payload_local));
                        function.instruction(&Instruction::LocalGet(this_tag_local));
                    }
                    Some((this_payload_local, None)) => {
                        function.instruction(&Instruction::LocalGet(this_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    }
                    None => {
                        function.instruction(&Instruction::GlobalGet(
                            SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX,
                        ));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    }
                }
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::LocalGet(argv_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                match propagate_throw {
                    PropagateCallThrow::ToActiveHandler => {
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.set_completion_kind(CompletionKind::Normal, function);
                    }
                    PropagateCallThrow::LeaveInCompletion => {
                        function.instruction(&Instruction::LocalGet(self.completion_local));
                        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                        function.instruction(&Instruction::I64Ne);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.set_completion_kind(CompletionKind::Normal, function);
                        function.instruction(&Instruction::End);
                    }
                }
                return Ok(());
            }
        }

        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let call_this_payload_local = self.reserve_temp_local();
        let call_this_tag_local = self.reserve_temp_local();
        let can_call_generator = self
            .functions
            .values()
            .any(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Generator);
        let can_call_async_generator = self
            .functions
            .values()
            .any(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator);
        let can_call_async = self
            .functions
            .values()
            .any(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Async);
        let proxy_revocable_table_index = self
            .functions
            .get(&StandardBuiltinId::ProxyRevocable.function_id())
            .map(|meta| meta.table_index as i64);

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        self.emit_load_function_flags(callee_payload_local, flags_local, function);

        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_CLASS_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            if let Some(this_tag_local) = this_tag_local {
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(FUNCTION_FLAG_STRICT as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(FUNCTION_FLAG_STRICT as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_value_to_object_locals(
                    this_payload_local,
                    this_tag_local,
                    call_this_payload_local,
                    call_this_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
            }
        } else {
            function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(call_this_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_this_tag_local));
        }
        if let Some(proxy_revocable_table_index) = proxy_revocable_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(proxy_revocable_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_payload_local));
            function.instruction(&Instruction::LocalSet(call_this_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_this_tag_local));
            function.instruction(&Instruction::End);
        }
        if can_call_generator {
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::I64Const(FUNCTION_FLAG_GENERATOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            let generator_prototype_local = self.reserve_temp_local();
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(GENERATOR_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(payload_local));
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                OBJECT_INTERNAL_BRAND_GENERATOR,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_STATE_OFFSET,
                GENERATOR_STATE_SUSPENDED_START,
                function,
            );
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_GENERATOR_FUNCTION_OFFSET,
                callee_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_GENERATOR_THIS_PAYLOAD_OFFSET,
                call_this_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_GENERATOR_THIS_TAG_OFFSET,
                call_this_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_GENERATOR_ARGC_OFFSET,
                argc_local,
                function,
            );
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_GENERATOR_ARGV_OFFSET,
                argv_local,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_KIND_OFFSET,
                GENERATOR_RESUME_KIND_NORMAL,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_DELEGATE_RECORD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(payload_local, HEAP_GENERATOR_ENV_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_INITIALIZED_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                GENERATOR_RESUME_STATE_INITIALIZING,
                function,
            );
            let initialization_payload_local = self.reserve_temp_local();
            let initialization_tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::LocalGet(call_this_payload_local));
            function.instruction(&Instruction::LocalGet(call_this_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results(
                initialization_payload_local,
                initialization_tag_local,
                function,
            );
            self.emit_propagate_throw_from_locals_if_needed(
                initialization_payload_local,
                initialization_tag_local,
                function,
            )?;
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                0,
                function,
            );
            let generator_prototype_tag_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                callee_payload_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                generator_prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                callee_payload_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                generator_prototype_tag_local,
                function,
            );
            self.emit_is_heap_object_like_tag_i32(generator_prototype_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::GlobalGet(GENERATOR_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(generator_prototype_local));
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_PROTOTYPE_OFFSET,
                generator_prototype_local,
                function,
            );
            self.release_temp_local(generator_prototype_tag_local);
            self.release_temp_local(initialization_tag_local);
            self.release_temp_local(initialization_payload_local);
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
            self.release_temp_local(generator_prototype_local);
            function.instruction(&Instruction::End);
        }

        if can_call_async_generator {
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::I64Const(FUNCTION_FLAG_ASYNC_GENERATOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            let async_generator_activation_local = self.reserve_temp_local();
            let async_generator_prototype_local = self.reserve_temp_local();
            let async_generator_prototype_tag_local = self.reserve_temp_local();

            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(payload_local));
            self.store_i64_const_at_offset(
                payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR,
                function,
            );
            self.emit_heap_alloc_const(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(async_generator_activation_local));
            for (offset, source_local) in [
                (HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET, callee_payload_local),
                (HEAP_ASYNC_GENERATOR_FUNCTION_ENV_OFFSET, callee_env_local),
                (
                    HEAP_ASYNC_GENERATOR_THIS_PAYLOAD_OFFSET,
                    call_this_payload_local,
                ),
                (HEAP_ASYNC_GENERATOR_THIS_TAG_OFFSET, call_this_tag_local),
                (HEAP_ASYNC_GENERATOR_ARGC_OFFSET, argc_local),
                (HEAP_ASYNC_GENERATOR_ARGV_OFFSET, argv_local),
            ] {
                self.store_i64_local_at_offset(
                    async_generator_activation_local,
                    offset,
                    source_local,
                    function,
                );
            }
            for (offset, value) in [
                (HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET, 0),
                (
                    HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
                    ASYNC_GENERATOR_STATE_SUSPENDED_START,
                ),
                (
                    HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
                    ASYNC_GENERATOR_RESUME_STATE_INITIALIZING,
                ),
                (HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET, 0),
                (
                    HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                ),
                (
                    HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
                    ASYNC_GENERATOR_RESUME_KIND_NORMAL,
                ),
                (HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET, 0),
                (
                    HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
                    ASYNC_GENERATOR_BODY_STATUS_IDLE,
                ),
                (HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET, 0),
                (
                    HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                ),
                (HEAP_ASYNC_GENERATOR_INITIALIZED_OFFSET, 0),
                (HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET, 0),
            ] {
                self.store_i64_const_at_offset(
                    async_generator_activation_local,
                    offset,
                    value,
                    function,
                );
            }
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,
                async_generator_activation_local,
                function,
            );

            let initialization_payload_local = self.reserve_temp_local();
            let initialization_tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::LocalGet(call_this_payload_local));
            function.instruction(&Instruction::LocalGet(call_this_tag_local));
            function.instruction(&Instruction::LocalGet(async_generator_activation_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results(
                initialization_payload_local,
                initialization_tag_local,
                function,
            );
            self.emit_propagate_throw_from_locals_if_needed(
                initialization_payload_local,
                initialization_tag_local,
                function,
            )?;
            self.release_temp_local(initialization_tag_local);
            self.release_temp_local(initialization_payload_local);

            self.load_i64_to_local_from_offset(
                callee_payload_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                async_generator_prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                callee_payload_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                async_generator_prototype_tag_local,
                function,
            );
            self.emit_is_heap_object_like_tag_i32(async_generator_prototype_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::GlobalGet(
                ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(async_generator_prototype_local));
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_PROTOTYPE_OFFSET,
                async_generator_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);

            self.release_temp_local(async_generator_prototype_tag_local);
            self.release_temp_local(async_generator_prototype_local);
            self.release_temp_local(async_generator_activation_local);
            function.instruction(&Instruction::End);
        }

        if can_call_async {
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::I64Const(FUNCTION_FLAG_ASYNC as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            let async_promise_payload_local = self.reserve_temp_local();
            let async_promise_record_local = self.reserve_temp_local();
            let async_activation_local = self.reserve_temp_local();
            let async_body_payload_local = self.reserve_temp_local();
            let async_body_tag_local = self.reserve_temp_local();

            function.instruction(&Instruction::GlobalGet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_alloc_promise_with_prototype(
                self.scratch_local,
                async_promise_payload_local,
                async_promise_record_local,
                function,
            )?;
            self.emit_heap_alloc_const(HEAP_ASYNC_ACTIVATION_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(async_activation_local));
            for (offset, source_local) in [
                (HEAP_ASYNC_FUNCTION_ENV_OFFSET, callee_env_local),
                (HEAP_ASYNC_FUNCTION_TABLE_INDEX_OFFSET, table_index_local),
                (HEAP_ASYNC_THIS_PAYLOAD_OFFSET, call_this_payload_local),
                (HEAP_ASYNC_THIS_TAG_OFFSET, call_this_tag_local),
                (HEAP_ASYNC_ARGC_OFFSET, argc_local),
                (HEAP_ASYNC_ARGV_OFFSET, argv_local),
                (
                    HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET,
                    async_promise_payload_local,
                ),
                (HEAP_ASYNC_PROMISE_RECORD_OFFSET, async_promise_record_local),
            ] {
                self.store_i64_local_at_offset(
                    async_activation_local,
                    offset,
                    source_local,
                    function,
                );
            }
            for (offset, value) in [
                (HEAP_ASYNC_RESUME_STATE_OFFSET, 0),
                (HEAP_ASYNC_RESUME_PAYLOAD_OFFSET, 0),
                (
                    HEAP_ASYNC_RESUME_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                ),
                (HEAP_ASYNC_RESUME_KIND_OFFSET, ASYNC_RESUME_KIND_FULFILL),
                (HEAP_ASYNC_ENV_OFFSET, 0),
                (HEAP_ASYNC_INITIALIZED_OFFSET, 0),
                (HEAP_ASYNC_COMPLETED_OFFSET, 0),
                (HEAP_ASYNC_PENDING_COMPLETION_HEAD_OFFSET, 0),
                (HEAP_ASYNC_PENDING_COMPLETION_DEPTH_OFFSET, 0),
            ] {
                self.store_i64_const_at_offset(async_activation_local, offset, value, function);
            }

            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::LocalGet(call_this_payload_local));
            function.instruction(&Instruction::LocalGet(call_this_tag_local));
            function.instruction(&Instruction::LocalGet(async_activation_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results(async_body_payload_local, async_body_tag_local, function);

            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_settle_promise_record(
                async_promise_record_local,
                PROMISE_STATE_REJECTED,
                async_body_payload_local,
                async_body_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_resolve_promise_record(
                async_promise_payload_local,
                async_promise_record_local,
                async_body_payload_local,
                async_body_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.store_i64_const_at_offset(
                async_activation_local,
                HEAP_ASYNC_COMPLETED_OFFSET,
                1,
                function,
            );
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(async_promise_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
            self.release_temp_local(async_body_tag_local);
            self.release_temp_local(async_body_payload_local);
            self.release_temp_local(async_activation_local);
            self.release_temp_local(async_promise_record_local);
            self.release_temp_local(async_promise_payload_local);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(call_this_payload_local));
        function.instruction(&Instruction::LocalGet(call_this_tag_local));
        self.emit_undefined_new_target(function);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        self.set_completion_kind(CompletionKind::Normal, function);
        if self.outline_function_call {
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
        } else {
            function.instruction(&Instruction::ReturnCallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
        }
        self.store_call_results(payload_local, tag_local, function);
        match propagate_throw {
            PropagateCallThrow::ToActiveHandler => {
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.set_completion_kind(CompletionKind::Normal, function);
            }
            PropagateCallThrow::LeaveInCompletion => {
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.set_completion_kind(CompletionKind::Normal, function);
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "class constructor cannot be invoked without `new`",
            payload_local,
            tag_local,
            function,
        )?;
        match propagate_throw {
            PropagateCallThrow::ToActiveHandler => {
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            PropagateCallThrow::LeaveInCompletion => {}
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(call_this_tag_local);
        self.release_temp_local(call_this_payload_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    /// Captures the two values that SuperCall obtains before evaluating its
    /// argument list: the current invocation's `new.target` and the active
    /// constructor's current [[Prototype]].  Keeping this phase separate from
    /// construction lets expression lowering preserve the observable order
    /// when an argument mutates the class heritage.
    pub(crate) fn emit_prepare_super_construct_to_locals(
        &mut self,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        ctor_payload_local: u32,
        ctor_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.lexical_derived_activation.is_none() {
            return Err(EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: super outside derived constructor",
            ));
        }

        let active_function_payload_local = self.reserve_temp_local();
        let active_function_tag_local = self.reserve_temp_local();
        self.emit_get_derived_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        self.emit_get_derived_active_function_to_locals(
            active_function_payload_local,
            active_function_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            active_function_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            ctor_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            active_function_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ctor_tag_local,
            function,
        );
        self.release_temp_local(active_function_tag_local);
        self.release_temp_local(active_function_payload_local);
        Ok(())
    }

    pub(crate) fn emit_super_construct_with_prepared_arg_vector(
        &mut self,
        ctor_payload_local: u32,
        ctor_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        self.emit_function_or_proxy_construct_with_argv(
            ctor_payload_local,
            ctor_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            call_payload_local,
            call_tag_local,
            function,
        )?;
        // Construct consumes the base constructor's completion.  Its produced
        // receiver is an ordinary value for the rest of the derived body.
        self.set_completion_kind(CompletionKind::Normal, function);
        // Binding is intentionally after Construct.  A second `super()` must
        // still perform the base construction before its duplicate-bind
        // ReferenceError, and a failed construction leaves `this` unbound.
        self.emit_bind_derived_this_from_locals(
            call_payload_local,
            call_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        let owner_function_id = self
            .lexical_derived_activation
            .expect("derived super call must have activation metadata")
            .owner_function_id
            .clone();
        let constructor_meta = self.functions.get(&owner_function_id).cloned().ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: unknown derived constructor `{owner_function_id}`"
            ))
        })?;
        if constructor_meta.class_instance_element_plan.is_some() {
            let active_function_payload_local = self.reserve_temp_local();
            let active_function_tag_local = self.reserve_temp_local();
            let class_context_local = self.reserve_temp_local();
            self.emit_get_derived_active_function_to_locals(
                active_function_payload_local,
                active_function_tag_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                active_function_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                class_context_local,
                function,
            );
            self.emit_initialize_instance_elements(
                &constructor_meta,
                class_context_local,
                call_payload_local,
                call_tag_local,
                function,
            )?;
            self.release_temp_local(class_context_local);
            self.release_temp_local(active_function_tag_local);
            self.release_temp_local(active_function_payload_local);
        }
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        Ok(())
    }

    /// SuperCall entry point for callers whose argument vector already
    /// exists, notably the synthetic default derived constructor.
    pub(crate) fn emit_super_construct_with_arg_vector(
        &mut self,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ctor_payload_local = self.reserve_temp_local();
        let ctor_tag_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        self.emit_prepare_super_construct_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            ctor_payload_local,
            ctor_tag_local,
            function,
        )?;
        self.emit_super_construct_with_prepared_arg_vector(
            ctor_payload_local,
            ctor_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(ctor_tag_local);
        self.release_temp_local(ctor_payload_local);
        Ok(())
    }

    pub(crate) fn store_call_results(
        &self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn store_call_results_to(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalSet(aux_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
    }

    pub(crate) fn emit_arguments_has_index_i32(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            entry_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entry_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_array_has_index_i32(arguments_local, index_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_rest_array_payload(
        &mut self,
        start_index: usize,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rest_len_local = self.reserve_temp_local();
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(rest_len_local));

        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, rest_len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, self.scratch_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_exotic_slots(array_local, function);

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
        ] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        self.release_temp_local(rest_len_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_object_payload(
        &mut self,
        protocol: &PresentArgumentsObjectProtocol,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arguments_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.emit_heap_alloc_const(HEAP_ARGUMENTS_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(arguments_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(arguments_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            ValueKind::Number.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET,
            0,
            function,
        );
        for offset in [
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
        ] {
            self.store_i64_const_at_offset(arguments_local, offset, 0, function);
        }
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        match protocol {
            PresentArgumentsObjectProtocol::Mapped(_) => self.store_i64_local_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                self.current_env_local,
                function,
            ),
            PresentArgumentsObjectProtocol::Unmapped(_) => self.store_i64_const_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                0,
                function,
            ),
        }
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_OWN_PROPERTY
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_CONFIGURABLE,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.load_i64_to_local_from_offset(
            self.class_function_context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
            iterator_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_tag_local));
        match protocol {
            PresentArgumentsObjectProtocol::Mapped(_) => {
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
                    ARRAY_DESCRIPTOR_OWN_PROPERTY
                        | OBJECT_DESCRIPTOR_DATA
                        | OBJECT_DESCRIPTOR_WRITABLE
                        | OBJECT_DESCRIPTOR_CONFIGURABLE,
                    function,
                );
                self.store_i64_local_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
                    iterator_payload_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
                    iterator_tag_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
                    0,
                    function,
                );
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                    function,
                );
            }
            PresentArgumentsObjectProtocol::Unmapped(_) => {
                self.emit_load_function_defining_realm_throw_type_error(
                    iterator_payload_local,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
                    ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
                    function,
                );
                self.store_i64_local_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
                self.store_i64_local_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    arguments_local,
                    HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
            }
        }

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PAYLOAD_OFFSET] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }
        function.instruction(&Instruction::I64Const(ARRAY_DESCRIPTOR_NORMAL_DATA as i64));
        match protocol {
            PresentArgumentsObjectProtocol::Mapped(plan) => {
                for entry in plan.entries().iter().copied() {
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(entry.argument_index_i64()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                    // 10.4.4.2/10.4.4.3: the mapping is not a descriptor *kind*, it
                    // is an orthogonal exotic flag with a payload. `DescriptorFlags`
                    // makes bit 5 and the bits-32..63 slot index inseparable, which
                    // the hand-built `ARGUMENTS_DESCRIPTOR_MAPPED as i64 | ((slot as
                    // i64) << 32)` did not: either half could be written without the
                    // other. The `const _` in `heap.rs` proves this reproduces that
                    // word for slot 7; this is what makes the proof load-bearing on
                    // the product path rather than an assertion about unused types.
                    function.instruction(&Instruction::I64Const(
                        DescriptorWord::of_data(false, false, false)
                            .with_flags(DescriptorFlags {
                                array_own_property: false,
                                mapped: Some(entry.mapped_slot()),
                            })
                            .as_i64(),
                    ));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::I64Or);
                }
            }
            PresentArgumentsObjectProtocol::Unmapped(_) => {}
        }
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            dst_entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            self.scratch_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arguments_local));
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arguments_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_length(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PROTOTYPE_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_object_read(
            getter_payload_local,
            getter_tag_local,
            arguments_local,
            arguments_tag_local,
            self.scratch_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            getter_payload_local,
            getter_tag_local,
            Some((arguments_local, Some(arguments_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_length_write(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();
        let write_succeeded_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_OWN_PROPERTY
                | OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE
                | OBJECT_DESCRIPTOR_CONFIGURABLE,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            setter_payload_local,
            setter_tag_local,
            arguments_local,
            arguments_tag_local,
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_succeeded_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to arguments.length", function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(write_succeeded_local);
        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_callee_read(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_object_read(
            prototype_local,
            prototype_tag_local,
            arguments_local,
            arguments_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            getter_payload_local,
            getter_tag_local,
            arguments_local,
            arguments_tag_local,
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_callee_write(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();
        let write_succeeded_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_OWN_PROPERTY
                | OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE
                | OBJECT_DESCRIPTOR_CONFIGURABLE,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            setter_payload_local,
            setter_tag_local,
            arguments_local,
            arguments_tag_local,
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(write_succeeded_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to arguments.callee", function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(write_succeeded_local);
        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_is_concat_spreadable_read(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_arguments_is_concat_spreadable_write(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        function.instruction(&Instruction::Else);
        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_arguments_read(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();

        self.emit_arguments_descriptor_kind_for_index(
            arguments_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_data_read(
            arguments_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_array_index_get(
            arguments_local,
            index_local,
            arguments_local,
            arguments_tag_local,
            payload_local,
            tag_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    fn emit_arguments_data_read(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let env_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let mapped_slot_local = self.reserve_temp_local();

        self.emit_arguments_descriptor_kind_for_index(
            arguments_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(ARGUMENTS_DESCRIPTOR_MAPPED as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        // The reader of the bits-32..63 mapped-slot payload. `MappedSlot::SHIFT`
        // rather than a bare literal, so the writer in
        // `emit_arguments_descriptor_kind_for_index` and both readers move
        // together.
        function.instruction(&Instruction::I64Const(MappedSlot::SHIFT as i64));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(mapped_slot_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapped_slot_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapped_slot_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(mapped_slot_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(env_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_parameter_map_write(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let mapped_slot_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.emit_arguments_descriptor_kind_for_index(
            arguments_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        // The reader of the bits-32..63 mapped-slot payload. `MappedSlot::SHIFT`
        // rather than a bare literal, so the writer in
        // `emit_arguments_descriptor_kind_for_index` and both readers move
        // together.
        function.instruction(&Instruction::I64Const(MappedSlot::SHIFT as i64));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(mapped_slot_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapped_slot_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapped_slot_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        self.release_temp_local(mapped_slot_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(env_local);
    }

    pub(crate) fn emit_arguments_descriptor_kind_for_index(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            capacity_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(buffer_local);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_arguments_store_index_entry(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let indexed_extent_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            indexed_extent_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            capacity_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            arguments_local,
            buffer_local,
            indexed_extent_local,
            capacity_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for (offset, local) in [
            (HEAP_ARRAY_PAYLOAD_OFFSET, payload_local),
            (HEAP_ARRAY_TAG_OFFSET, tag_local),
            (HEAP_ARRAY_SETTER_PAYLOAD_OFFSET, setter_payload_local),
            (HEAP_ARRAY_SETTER_TAG_OFFSET, setter_tag_local),
            (HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET, descriptor_kind_local),
        ] {
            self.store_i64_local_at_offset(entry_local, offset, local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(indexed_extent_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(indexed_extent_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            indexed_extent_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(indexed_extent_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_write(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let write_succeeded_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let indexed_extent_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();

        self.emit_arguments_descriptor_kind_for_index(
            arguments_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            indexed_extent_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            capacity_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        if self.outline_proxy_call {
            self.emit_function_or_proxy_call_leave_throw_completion(
                setter_payload_local,
                setter_tag_local,
                arguments_local,
                self.scratch_local,
                &[(payload_local, tag_local)],
                setter_result_payload_local,
                setter_result_tag_local,
                function,
            )?;
        } else {
            self.emit_function_handle_call(
                setter_payload_local,
                setter_tag_local,
                Some((arguments_local, Some(self.scratch_local))),
                &[(payload_local, tag_local)],
                setter_result_payload_local,
                setter_result_tag_local,
                function,
            )?;
        }
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            arguments_local,
            buffer_local,
            indexed_extent_local,
            capacity_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(indexed_extent_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(indexed_extent_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            indexed_extent_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_succeeded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(write_succeeded_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to arguments index", function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_succeeded_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(ARGUMENTS_DESCRIPTOR_MAPPED as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_parameter_map_write(
            arguments_local,
            index_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(indexed_extent_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(write_succeeded_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn current_function_meta(&self) -> Option<&WasmFunctionMeta> {
        self.function_id
            .as_ref()
            .and_then(|function_id| self.functions.get(function_id))
    }

    pub(crate) fn emit_load_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self
            .current_function_meta()
            .is_some_and(WasmFunctionMeta::has_class_execution_context)
        {
            let home_object_local = self.reserve_temp_local();
            let home_object_tag_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                home_object_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                home_object_tag_local,
                function,
            );
            self.emit_load_super_base_from_home_object(
                home_object_local,
                home_object_tag_local,
                payload_local,
                tag_local,
                function,
            );
            self.release_temp_local(home_object_tag_local);
            self.release_temp_local(home_object_local);
            return Ok(());
        }
        if self.lexical_derived_activation.is_some() {
            // A constructor's SuperProperty reference is based on its
            // [[HomeObject]], the constructor's `.prototype` object. Arrows
            // lexically enclosed by that constructor share the same base; the
            // arrow call ABI's `this` parameter is not the home object and is
            // deliberately ignored here.
            let active_function_payload_local = self.reserve_temp_local();
            let active_function_tag_local = self.reserve_temp_local();
            let class_context_local = self.reserve_temp_local();
            let home_object_local = self.reserve_temp_local();
            let home_object_tag_local = self.reserve_temp_local();
            self.emit_get_derived_active_function_to_locals(
                active_function_payload_local,
                active_function_tag_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                active_function_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                class_context_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                home_object_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                home_object_tag_local,
                function,
            );
            self.emit_load_super_base_from_home_object(
                home_object_local,
                home_object_tag_local,
                payload_local,
                tag_local,
                function,
            );
            self.release_temp_local(home_object_tag_local);
            self.release_temp_local(home_object_local);
            self.release_temp_local(class_context_local);
            self.release_temp_local(active_function_tag_local);
            self.release_temp_local(active_function_payload_local);
            return Ok(());
        }

        let Some(home_object) = self.lookup_binding(LEXICAL_HOME_OBJECT_NAME) else {
            return Err(EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: super outside class method",
            ));
        };
        let home_object_local = self.reserve_temp_local();
        let home_object_tag_local = self.reserve_temp_local();
        self.read_binding_to_locals(
            home_object,
            home_object_local,
            home_object_tag_local,
            function,
        )?;
        self.emit_load_super_base_from_home_object(
            home_object_local,
            home_object_tag_local,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(home_object_tag_local);
        self.release_temp_local(home_object_local);
        Ok(())
    }

    fn emit_load_super_base_from_home_object(
        &mut self,
        home_object_local: u32,
        home_object_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            home_object_local,
            HEAP_PROTOTYPE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(home_object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            home_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_throw_if_null_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "super property access on null base",
            payload_local,
            tag_local,
            function,
        )?;
        // `emit_throw_runtime_error` has already recorded the throw completion
        // and result locals. Dispatch it through the normal completion path so
        // active `finally` blocks run before the throw reaches its handler (or
        // returns from the function). The dispatch is nested inside this
        // null-test `if`; the sink counts that frame, so the branch immediate
        // needs no correction here.
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_call_args_vector(
        &mut self,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(u32, u32), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if args
            .iter()
            .all(|arg| !matches!(arg.expr, ExprIr::SpreadArgument(_)))
        {
            let mut evaluated_args = Vec::with_capacity(args.len());
            for arg in args {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(arg, payload_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                evaluated_args.push((payload_local, tag_local));
            }
            self.emit_pre_evaluated_arg_vector(&evaluated_args, argc_local, argv_local, function)?;
            for (payload_local, tag_local) in evaluated_args.into_iter().rev() {
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
            }
            return Ok((argc_local, argv_local));
        }

        self.emit_pre_evaluated_arg_vector(&[], argc_local, argv_local, function)?;

        for arg in args {
            let ExprIr::SpreadArgument(spread) = &arg.expr else {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(arg, payload_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_array_write(argv_local, argc_local, payload_local, tag_local, function)?;
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(argc_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                continue;
            };

            let source_payload_local = self.reserve_temp_local();
            let source_tag_local = self.reserve_temp_local();
            let method_payload_local = self.reserve_temp_local();
            let method_tag_local = self.reserve_temp_local();
            let iterator_payload_local = self.reserve_temp_local();
            let iterator_tag_local = self.reserve_temp_local();
            let next_payload_local = self.reserve_temp_local();
            let next_tag_local = self.reserve_temp_local();
            let result_payload_local = self.reserve_temp_local();
            let result_tag_local = self.reserve_temp_local();
            let done_payload_local = self.reserve_temp_local();
            let done_tag_local = self.reserve_temp_local();
            let value_payload_local = self.reserve_temp_local();
            let value_tag_local = self.reserve_temp_local();
            let key_local = self.reserve_temp_local();

            self.compile_expr_to_locals(
                &spread.value,
                source_payload_local,
                source_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                source_payload_local,
                source_tag_local,
                function,
            )?;
            self.compile_nullish_tagged_i32(source_tag_local, function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Spread argument is not iterable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);

            // `GetMethod(source, @@iterator)`. The key is the well-known
            // symbol, not the string "Symbol.iterator": a static-string key
            // reads an ordinary string-named property and never finds the
            // symbol-keyed method, which made every spread argument report
            // "not iterable". Building the key as a Symbol-kinded expression is
            // the same encoding `yield*` delegation uses.
            let runtime_source = TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags()
                        .without(ValueKind::Undefined)
                        .without(ValueKind::Null),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::Undefined,
            );
            let iterator_symbol_key = TypedExpr::from_info(
                ValueInfo::new(ValueKind::Symbol),
                ExprIr::String("Symbol.iterator".to_string()),
            );
            self.compile_property_read_from_locals(
                &runtime_source,
                &PropertyKeyIr::StringExpr(Box::new(iterator_symbol_key)),
                source_payload_local,
                source_tag_local,
                method_payload_local,
                method_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                method_payload_local,
                method_tag_local,
                function,
            )?;
            self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Spread argument is not iterable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);

            self.emit_function_or_proxy_call_leave_throw_completion(
                method_payload_local,
                method_tag_local,
                source_payload_local,
                source_tag_local,
                &[],
                iterator_payload_local,
                iterator_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                iterator_payload_local,
                iterator_tag_local,
                function,
            )?;
            self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Spread iterator method must return object",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::I64Const(self.strings.payload("next")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                iterator_payload_local,
                iterator_tag_local,
                iterator_payload_local,
                iterator_tag_local,
                key_local,
                next_payload_local,
                next_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                next_payload_local,
                next_tag_local,
                function,
            )?;
            self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Spread iterator next must be callable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);

            let iterator_exit = self.open_frame(ControlFrameKind::Block, function);
            let iterator_loop = self.open_frame(ControlFrameKind::Loop, function);

            self.emit_function_or_proxy_call_leave_throw_completion(
                next_payload_local,
                next_tag_local,
                iterator_payload_local,
                iterator_tag_local,
                &[],
                result_payload_local,
                result_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                result_payload_local,
                result_tag_local,
                function,
            )?;
            self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Spread iterator next result must be object",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::I64Const(self.strings.payload("done")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                result_payload_local,
                result_tag_local,
                result_payload_local,
                result_tag_local,
                key_local,
                done_payload_local,
                done_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
            self.emit_branch_if_to_target(iterator_exit, function);

            function.instruction(&Instruction::I64Const(self.strings.payload("value")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                result_payload_local,
                result_tag_local,
                result_payload_local,
                result_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            self.emit_array_write(
                argv_local,
                argc_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(argc_local));
            self.emit_branch_to_target(iterator_loop, function);

            self.pop_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);

            for local in [
                key_local,
                value_tag_local,
                value_payload_local,
                done_tag_local,
                done_payload_local,
                result_tag_local,
                result_payload_local,
                next_tag_local,
                next_payload_local,
                iterator_tag_local,
                iterator_payload_local,
                method_tag_local,
                method_payload_local,
                source_tag_local,
                source_payload_local,
            ] {
                self.release_temp_local(local);
            }
        }

        Ok((argc_local, argv_local))
    }

    pub(crate) fn emit_direct_js_call(
        &mut self,
        meta: &WasmFunctionMeta,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if meta.protocol.class_kind() != ClassFunctionKind::Constructor {
            self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        }
        self.emit_direct_js_call_with_argv(
            meta,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_direct_js_call_with_argv(
        &mut self,
        meta: &WasmFunctionMeta,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_direct_js_call_with_environment(
            meta,
            None,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_direct_class_element_js_call(
        &mut self,
        meta: &WasmFunctionMeta,
        class_context_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        debug_assert_ne!(
            meta.class_element_execution_kind,
            ClassElementExecutionKind::None
        );
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_direct_js_call_with_environment(
            meta,
            Some(class_context_local),
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_class_field_key_to_local(
        &mut self,
        key: &ClassFieldKeyIr,
        class_context_local: u32,
        key_local: u32,
        function: &mut Function,
    ) {
        match key {
            ClassFieldKeyIr::Public(key) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                function.instruction(&Instruction::LocalSet(key_local));
            }
            ClassFieldKeyIr::ComputedPublic(slot) => {
                let field_keys_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    class_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                    field_keys_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    field_keys_local,
                    ENV_SLOT_BASE_OFFSET + *slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_PAYLOAD_OFFSET,
                    key_local,
                    function,
                );
                self.release_temp_local(field_keys_local);
            }
            ClassFieldKeyIr::Private(private_name_id) => {
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(&private_data_key(*private_name_id)),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
            }
        }
    }

    pub(crate) fn emit_initialize_instance_elements(
        &mut self,
        constructor_meta: &WasmFunctionMeta,
        class_context_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(plan) = constructor_meta.class_instance_element_plan.clone() else {
            return Ok(());
        };
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let private_environment_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            class_context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
            private_environment_local,
            function,
        );
        self.active_private_environment_locals
            .push(private_environment_local);

        for private_name_id in plan.private_method_brands {
            self.emit_private_name_token_to_local(private_name_id, key_local, function)?;
            self.emit_private_brand_add(
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                function,
            )?;
        }

        for field in plan.fields {
            self.emit_class_field_key_to_local(
                &field.key,
                class_context_local,
                key_local,
                function,
            );
            if let Some(init_function_id) = &field.init_function_id {
                let initializer_meta = self.functions.get(init_function_id).cloned().ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unknown class field init `{init_function_id}`"
                    ))
                })?;
                if initializer_meta.class_element_execution_kind
                    != ClassElementExecutionKind::InstanceFieldInitializer
                {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: class field init `{init_function_id}` has invalid execution kind"
                    )));
                }
                self.emit_direct_class_element_js_call(
                    &initializer_meta,
                    class_context_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    &[],
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
            }
            if let ClassFieldKeyIr::Private(private_name_id) = field.key {
                self.emit_private_name_token_to_local(private_name_id, key_local, function)?;
                self.emit_private_field_add(
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            } else {
                self.emit_object_define_enumerable_data(
                    receiver_payload_local,
                    key_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            }
        }

        self.active_private_environment_locals.pop();
        self.release_temp_local(private_environment_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_direct_js_call_with_environment(
        &mut self,
        meta: &WasmFunctionMeta,
        environment_local: Option<u32>,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // A direct call into a builtin's body requires its real body to be
        // emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        if meta.protocol.class_kind() == ClassFunctionKind::Constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.active_throw_target() {
                self.emit_branch_to_target(target, function);
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            if meta.standard_builtin.is_some() {
                self.emit_standard_builtin_realm_env_argument(function);
            } else if let Some(environment_local) = environment_local {
                function.instruction(&Instruction::LocalGet(environment_local));
            } else {
                function.instruction(&Instruction::LocalGet(self.current_env_local));
            }
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                function.instruction(&Instruction::LocalGet(this_payload_local));
                if let Some(this_tag_local) = this_tag_local {
                    function.instruction(&Instruction::LocalGet(this_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                }
            } else {
                self.emit_default_this_for_known_strictness(meta.strict, function);
            }
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        }

        Ok(())
    }

    pub(crate) fn emit_indirect_call_from_locals(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, u32)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let default_this_payload_local = self.reserve_temp_local();
        let default_this_tag_local = self.reserve_temp_local();

        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;

        let (this_payload_local, this_tag_local) =
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                (this_payload_local, this_tag_local)
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(default_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(default_this_tag_local));
                (default_this_payload_local, default_this_tag_local)
            };

        self.emit_function_or_proxy_call_with_argv_without_throw_propagation(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(default_this_tag_local);
        self.release_temp_local(default_this_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        static_regexp_compilation: Option<&StaticRegExpCompilation>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if args.is_empty()
            && static_regexp_compilation.is_none()
            && self
                .current_function_meta()
                .is_some_and(|meta| meta.protocol.class_kind() == ClassFunctionKind::Constructor)
        {
            if let (ExprIr::FunctionValue(function_id), Some(this_arg)) = (&callee.expr, this_arg) {
                if matches!(this_arg.expr, ExprIr::This) {
                    let initializer_meta =
                        self.functions.get(function_id).cloned().filter(|meta| {
                            meta.class_element_execution_kind
                                == ClassElementExecutionKind::InstanceFieldInitializer
                        });
                    if let Some(initializer_meta) = initializer_meta {
                        let this_payload_local = self.reserve_temp_local();
                        let this_tag_local = self.reserve_temp_local();
                        self.compile_expr_to_locals(
                            this_arg,
                            this_payload_local,
                            this_tag_local,
                            function,
                        )?;
                        self.emit_direct_class_element_js_call(
                            &initializer_meta,
                            self.class_function_context_local,
                            Some((this_payload_local, Some(this_tag_local))),
                            &[],
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(this_tag_local);
                        self.release_temp_local(this_payload_local);
                        return Ok(());
                    }
                }
            }
        }

        let string_match_function_id = StandardBuiltinId::StringPrototypeMatch.function_id();
        let string_split_function_id = StandardBuiltinId::StringPrototypeSplit.function_id();
        let string_slice_function_id = StandardBuiltinId::StringPrototypeSlice.function_id();
        if callee.function_targets.len() == 1
            && (callee.function_targets.contains(&string_match_function_id)
                || callee.function_targets.contains(&string_split_function_id)
                || callee.function_targets.contains(&string_slice_function_id))
        {
            if let Some(this_arg) = this_arg {
                if callee.function_targets.contains(&string_match_function_id) {
                    return self.emit_string_match_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                if callee.function_targets.contains(&string_slice_function_id) {
                    return self.emit_string_slice_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if let (
            ExprIr::PropertyRead {
                key: PropertyKeyIr::StaticString(name),
                ..
            },
            Some(this_arg),
        ) = (&callee.expr, this_arg)
        {
            let string_or_undefined = KindSet::from_kind(ValueKind::String)
                .union(KindSet::from_kind(ValueKind::Undefined));
            if name == "split" && this_arg.possible_kinds.is_subset_of(string_or_undefined) {
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        let reflect_define_property_function_id =
            StandardBuiltinId::ReflectDefineProperty.function_id();
        let object_define_property_function_id =
            StandardBuiltinId::ObjectDefineProperty.function_id();
        let is_reflect_define_property_access = matches!(
            &callee.expr,
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(name),
            } if name == "defineProperty"
                && matches!(
                    &target.expr,
                    ExprIr::GlobalPropertyRead { name } | ExprIr::Identifier(name)
                        if name == REFLECT_NAME
                )
        );
        if is_reflect_define_property_access
            && callee.function_targets.len() == 1
            && (callee
                .function_targets
                .contains(&reflect_define_property_function_id)
                || callee
                    .function_targets
                    .contains(&object_define_property_function_id))
        {
            let callee_payload_local = self.reserve_temp_local();
            let callee_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
            let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
            let meta = self
                .functions
                .get(&reflect_define_property_function_id)
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                    )
                })?;
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            self.emit_default_this_for_known_strictness(meta.strict, function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }

        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let default_this_payload_local = self.reserve_temp_local();
        let default_this_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;

        let this_locals = if let Some(this_arg) = this_arg {
            let this_payload_local = self.reserve_temp_local();
            let this_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;
            Some((this_payload_local, this_tag_local))
        } else {
            None
        };
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        if let Some(StaticRegExpCompilation::InvalidSyntax { message }) = static_regexp_compilation
        {
            self.emit_throw_runtime_error(
                SYNTAX_ERROR_NAME,
                message,
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                self.release_temp_local(this_tag_local);
                self.release_temp_local(this_payload_local);
            }
            self.release_temp_local(default_this_tag_local);
            self.release_temp_local(default_this_payload_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }

        let (this_payload_local, this_tag_local) =
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                (this_payload_local, this_tag_local)
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(default_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(default_this_tag_local));
                (default_this_payload_local, default_this_tag_local)
            };

        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        if let Some(StaticRegExpCompilation::Program(program)) = static_regexp_compilation {
            if callee
                .function_targets
                .contains(&StandardBuiltinId::RegExpPrototypeCompile.function_id())
            {
                if let Some((this_payload_local, _)) = this_locals {
                    self.emit_regexp_program_slots(this_payload_local, Some(program), function);
                }
            } else {
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::GlobalGet(REGEXP_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_program_slots(payload_local, Some(program), function);
                function.instruction(&Instruction::End);
            }
        }

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            self.release_temp_local(this_tag_local);
            self.release_temp_local(this_payload_local);
        }
        self.release_temp_local(default_this_tag_local);
        self.release_temp_local(default_this_payload_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }

    pub(crate) fn emit_tail_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<bool, EmitError> {
        let Some(function_helper) = self.function_call_helper_function_index() else {
            return Ok(false);
        };
        let Some(proxy_helper) = self.proxy_call_helper_function_index() else {
            return Ok(false);
        };

        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let this_payload_local = self.reserve_temp_local();
        let this_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            callee_payload_local,
            callee_tag_local,
            function,
        )?;
        if let Some(this_arg) = this_arg {
            self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(
                this_payload_local,
                this_tag_local,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(this_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(this_tag_local));
        }
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::ReturnCall(function_helper));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::ReturnCall(proxy_helper));
        function.instruction(&Instruction::End);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(this_tag_local);
        self.release_temp_local(this_payload_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(true)
    }

    fn emit_custom_array_named_method_call(
        &mut self,
        receiver: &TypedExpr,
        key: &PropertyKeyIr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        let key_local = self.compile_object_key_to_local(key, function)?;
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            callee_payload_local,
            callee_tag_local,
            function,
        )?;
        self.release_temp_local(key_local);
        self.emit_propagate_throw_from_locals_if_needed(
            callee_payload_local,
            callee_tag_local,
            function,
        )?;

        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    /// The one emission shape for every `Iterator.prototype` helper fast path.
    ///
    /// The callee is acquired by an ordinary `[[Get]]` of the helper's property
    /// name off the receiver — the same acquisition the generic tail uses.
    /// Because the read is dynamic, this is *not* an iterator-specific
    /// emission: it is a generic method call for any object receiver, which is
    /// why `drop` and `flatMap` route every non-array receiver here
    /// unconditionally and are correct on receivers that are not `Iterator`
    /// subclasses at all. That is what makes it safe for
    /// [`super::receiver_needs_dynamic_helper_dispatch`] to send the other
    /// seven helpers here too.
    ///
    /// # Abrupt completions and RequireObjectCoercible
    ///
    /// Four checks below were all missing, and their absence is why routing
    /// more receivers here would otherwise have traded one defect for another.
    /// Three of them are the ones the structurally identical
    /// [`Self::emit_custom_array_named_method_call`] already performed; each is
    /// a no-op unless `completion_local` holds a throw, so they cost one
    /// comparison on the ordinary path:
    ///
    /// 1. after the receiver is compiled — otherwise a receiver expression that
    ///    threw is used as the base of the `[[Get]]`;
    /// 2. after the `[[Get]]` — otherwise a throwing accessor's completion is
    ///    treated as the callee, the arguments are still evaluated for their
    ///    side effects, and the user sees `value is not callable` in place of
    ///    the error they threw;
    /// 3. after the call, matching the tail.
    ///
    /// **(3) IS STATICALLY DEAD, and calling it load-bearing was wrong.**
    /// [`Self::emit_function_handle_call_with_argv`] hard-codes
    /// `PropagateCallThrow::ToActiveHandler`, and *both* arms that implement it
    /// — the outlined-helper path and the inline path — already call
    /// `emit_propagate_throw_from_locals_if_needed` and then
    /// `set_completion_kind(CompletionKind::Normal)`. Control reaches (3) only
    /// with `completion_local == NORMAL`, so the `i64.eq`/`if`/`end` it emits
    /// can never take its true arm. The generic tail carries the identical dead
    /// check after its own call, so this is a copy of an existing no-op rather
    /// than a repair; it is kept for symmetry with the tail and so that a future
    /// switch to `PropagateCallThrow::LeaveInCompletion` here is safe, and for
    /// no other reason. (1) and (2) are the two that are merely *unwitnessed*,
    /// which is a weaker statement than dead — see below.
    ///
    /// The fourth is 7.2.1 RequireObjectCoercible, between (1) and (2). It is a
    /// *runtime* tag test rather than a static one for the reason spelled out
    /// on [`super::receiver_needs_dynamic_helper_dispatch`]: the receiver that
    /// makes this dispatch necessary is one `lila-ir` mistypes as
    /// `undefined`, so no static test can separate it from a program that
    /// really wrote `undefined.take(1)`. It is the only one of the four with a
    /// reachable true arm on the ordinary path.
    ///
    /// `wasm_iterator_helper_class_receiver_abrupt_dispatch.js` answers `ok` on
    /// the *pre-repair* compiler, because its receivers are ordinary objects
    /// that the generic tail already handled. **It pins that the routing change
    /// preserves the tail's abrupt-completion behaviour; it does not witness
    /// checks (1)-(3),** and an earlier version of this comment claimed it did.
    /// Measured: the fixture still answers `ok` with all three deleted — and for
    /// (3) no fixture ever could, per the paragraph above. In this
    /// emitter's configuration they are redundant —
    /// [`Self::emit_object_read`] routes to `emit_object_read_ordinary`, which
    /// propagates on the outlined-helper path (the variant that does *not* is
    /// spelled `emit_object_read_without_throw_propagation`, and is not what is
    /// called here), and `emit_function_handle_call_with_argv` propagates a
    /// callee throw itself, as the comment at its call site below says.
    ///
    /// They are not dead in general: on the inlined read path
    /// `AccessorThrowRouting::BreakToOrdinaryReadExit` leaves the throw for the
    /// caller, and that configuration is what (2) exists for. Nothing in the
    /// CLI corpus is known to reach it through this emitter, so the honest
    /// statement is "defensive, load-bearing on one configuration, unwitnessed
    /// by the fixture" rather than "the fixture would catch their removal".
    /// A probe that forces the inlined read is the missing coverage.
    ///
    /// Property installation is not at risk here. `planning.rs` roots
    /// `IteratorConstructor` for every helper in this family, and
    /// `intrinsics/iterator.rs` installs all eleven together, so the dynamic
    /// read finds a real function object.
    ///
    /// Every temp this needs is reserved here, so the emitted sequence never
    /// depends on `self.scratch_local` / `self.result_tag_local` — which
    /// matters because `expressions.rs` hands `emit_method_call` the scratch
    /// local as its destination.
    fn emit_iterator_prototype_helper_method_call(
        &mut self,
        helper: IteratorHelper,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        destination: MethodCallDestination,
        function: &mut Function,
    ) -> Result<DestinationWritten, EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        // 7.2.1 RequireObjectCoercible, checked at run time rather than
        // statically. It has to be here, not at the call sites, because the
        // receiver this dispatch exists for is one `lila-ir` types as
        // `undefined` while the runtime value is an ordinary object — so a
        // static nullish test cannot tell "the compiler is wrong about this
        // receiver" from "the program really wrote `undefined.take(1)`", and
        // only the tag decides. The generic tail spells the same check the same
        // way; before this the dispatch had none at all, so `drop` and `flatMap`
        // read a property off `undefined` instead of throwing.
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot read properties of null or undefined",
            destination.payload_local(),
            destination.tag_local(),
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            destination.payload_local(),
            destination.tag_local(),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            self.strings.payload(helper.property_name()),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            callee_payload_local,
            callee_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            callee_payload_local,
            callee_tag_local,
            function,
        )?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        // Writes both destination locals on every path, and propagates a
        // callee throw to the active handler (`PropagateCallThrow::ToActiveHandler`),
        // which is what makes a callback throw reach a user `catch`.
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            destination.payload_local(),
            destination.tag_local(),
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            destination.payload_local(),
            destination.tag_local(),
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(key_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        // Witnesses the normal-completion path only, which is exactly what
        // `DestinationWritten` claims: the three propagate calls above can each
        // emit an abrupt exit before `emit_function_handle_call_with_argv`
        // writes the pair, and those exits carry their value in the completion
        // locals rather than in the destination.
        Ok(destination.written())
    }

    pub(crate) fn emit_method_call(
        &mut self,
        receiver: &TypedExpr,
        key: &PropertyKeyIr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let custom_array_named_get = matches!(
            receiver.heap_shape.as_deref(),
            Some(HeapShape::Array(shape)) if shape.prototype.is_some()
        ) && matches!(
            key,
            PropertyKeyIr::StaticString(name)
                if name != "length" && !is_canonical_array_index_name(name)
        );
        if custom_array_named_get {
            return self.emit_custom_array_named_method_call(
                receiver,
                key,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "push")
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
        {
            return self.emit_array_push_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "toLocaleString")
            && receiver.possible_kinds.contains(ValueKind::Array)
        {
            return self.emit_array_direct_builtin_method_call(
                StandardBuiltinId::ArrayPrototypeToLocaleString,
                "Array.prototype.toLocaleString",
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "toString")
            && args.is_empty()
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
        {
            return self.emit_array_join_method_call(
                receiver,
                &[],
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "reverse") {
            return self.emit_array_reverse_method_call(
                receiver,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "split") {
            return self.emit_string_split_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "match") {
            return self.emit_string_match_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "substring") {
            return self.emit_string_substring_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "slice") {
            let receiver_is_string = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String));
            let receiver_has_string_slice = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "slice"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::StringPrototypeSlice.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_string || receiver_has_string_slice {
                return self.emit_string_slice_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            let receiver_is_array = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array));
            let receiver_has_array_slice = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "slice"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeSlice.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_array || receiver_has_array_slice {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::ArrayPrototypeSlice,
                    "Array.prototype.slice",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "charAt") {
            return self.emit_string_char_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "charCodeAt") {
            return self.emit_string_char_code_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "codePointAt") {
            return self.emit_string_code_point_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "at")
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String))
        {
            return self.emit_string_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "pop") {
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let len_local = self.reserve_temp_local();
            let index_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Array.prototype.pop receiver is not array",
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                len_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_array_read(
                receiver_payload_local,
                index_local,
                payload_local,
                tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                index_local,
                function,
            );
            function.instruction(&Instruction::End);
            self.release_temp_local(index_local);
            self.release_temp_local(len_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "splice") {
            return self.emit_array_direct_builtin_method_call(
                StandardBuiltinId::ArrayPrototypeSplice,
                "Array.prototype.splice",
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "sort") {
            return self.emit_array_direct_builtin_method_call(
                StandardBuiltinId::ArrayPrototypeSort,
                "Array.prototype.sort",
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "spliceFromArray") {
            return self.emit_array_splice_from_array_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), "keys" | "entries" | "values") || name == LILA_STATIC_GENERATOR_VALUES_METHOD)
        {
            let kind = match key {
                PropertyKeyIr::StaticString(name) if name == "keys" => ARRAY_ITERATOR_KIND_KEYS,
                PropertyKeyIr::StaticString(name) if name == "entries" => {
                    ARRAY_ITERATOR_KIND_ENTRIES
                }
                _ => ARRAY_ITERATOR_KIND_VALUES,
            };
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            self.emit_array_iterator_create_from_locals(
                receiver_payload_local,
                receiver_tag_local,
                kind,
                payload_local,
                tag_local,
                function,
            )?;
            if matches!(key, PropertyKeyIr::StaticString(name) if name == LILA_STATIC_GENERATOR_VALUES_METHOD)
            {
                self.emit_object_define_bool_data(
                    payload_local,
                    LILA_STATIC_GENERATOR_ITERATOR_SLOT,
                    true,
                    function,
                )?;
            }
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "concat") {
            let receiver_is_string = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String));
            let receiver_has_string_concat = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "concat"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::StringPrototypeConcat.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_string || receiver_has_string_concat {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::StringPrototypeConcat,
                    "String.prototype.concat",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            return self.emit_array_concat_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "flat") {
            return self.emit_array_flat_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        // `flatMap` is handled once, here. There used to be a second
        // `name == "flatMap"` block further down whose guard was
        // `receiver_is_iterator || !receiver_is_array` and whose trailing
        // array call was therefore unreachable: this block already returned for
        // every array receiver, so `!receiver_is_array` always held by the time
        // control got there and the disjunction was constant `true`. Nothing
        // between the two blocks matches the key `"flatMap"`.
        //
        // Two separate claims, and only the first is "behaviour-preserving":
        //
        // 1. BLOCK SELECTION is unchanged by the fold. Same receiver
        //    classification, same two destinations, no interleaved `"flatMap"`
        //    key.
        // 2. CALLEE ACQUISITION on the non-array branch is NOT unchanged. It
        //    was a static reference to `IteratorPrototypeFlatMap`; it is now an
        //    ordinary `[[Get]]` of `"flatMap"` off the receiver. That is the
        //    repair, and it is the correct semantics — but it applies to every
        //    receiver this branch takes, not only to `Iterator` subclasses.
        //    `receiver_is_array` here tests `receiver.kind`, not
        //    `possible_kinds` (unlike `drop` below), so a `Dynamic` receiver
        //    takes this branch. Two paths are consequently unfixtured:
        //    `function f(x, g) { return x.flatMap(g); }` called with an array,
        //    and a primitive receiver such as `(5).flatMap(g)`, which used to
        //    throw a clean `TypeError` from the builtin body and now reaches
        //    `emit_object_read`. Neither is covered by
        //    `wasm_iterator_helper_class_receiver_flat_map.js` (literal array +
        //    `class extends Iterator`), and rung 1c is the only gate that would
        //    see them.
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::FlatMap.property_name())
        {
            let receiver_is_array = receiver.kind == ValueKind::Array
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            if receiver_is_array {
                return self.emit_array_flat_map_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            return self
                .emit_iterator_prototype_helper_method_call(
                    IteratorHelper::FlatMap,
                    receiver,
                    args,
                    MethodCallDestination::new(payload_local, tag_local),
                    function,
                )
                .map(DestinationWritten::discharge);
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "at") {
            return self.emit_array_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "includes") {
            return self.emit_array_includes_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "indexOf") {
            return self.emit_array_index_of_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "lastIndexOf") {
            return self.emit_array_last_index_of_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Find.property_name())
        {
            let receiver_is_array = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Find);
            let receiver_has_typed_array_find = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "find"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::TypedArrayPrototypeFind.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            let receiver_has_array_find = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "find"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeFind.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Find,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            if receiver_has_typed_array_find {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::TypedArrayPrototypeFind,
                    "TypedArray.prototype.find",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            if receiver_is_array || receiver_has_array_find {
                return self.emit_array_find_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            // Every receiver-specific alternative has been ruled out above, so
            // a typed array (whose instance shape carries `find`) has already
            // returned and cannot reach here. What is left and still ordinary
            // is an object receiver, which the generic tail mishandles.
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Find,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findIndex") {
            let receiver_has_typed_array_find_index = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "findIndex"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::TypedArrayPrototypeFindIndex.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_has_typed_array_find_index {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::TypedArrayPrototypeFindIndex,
                    "TypedArray.prototype.findIndex",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            return self.emit_array_find_index_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Reduce.property_name())
        {
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Reduce);
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Reduce,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            // `reduce` has no array fast path in this function at all: an array
            // receiver reaches the generic tail and must keep doing so, which
            // is why the predicate is kind-restricted rather than
            // `!receiver_is_array`.
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Reduce,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Take.property_name())
        {
            // No `receiver_is_array` binding here, unlike `drop` below: this
            // block has no array fast path to protect, so the array case is
            // excluded by the kind restriction inside
            // `receiver_needs_dynamic_helper_dispatch` rather than by a local.
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Take);
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Take,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            // The A/B that identified the defect: this block and `drop`'s below
            // are otherwise identical, and `drop`'s extra disjunct is the only
            // reason its fixture is green and this one's was red.
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Take,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Drop.property_name())
        {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Drop);
            // Guard preserved verbatim, including the `|| !receiver_is_array`
            // disjunct that the plain-`receiver_is_iterator` helpers do not
            // have. Only the callee acquisition changes here.
            if receiver_is_iterator || !receiver_is_array {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Drop,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findLast") {
            let receiver_has_typed_array_find_last = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "findLast"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::TypedArrayPrototypeFindLast.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_has_typed_array_find_last {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::TypedArrayPrototypeFindLast,
                    "TypedArray.prototype.findLast",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            return self.emit_array_find_last_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findLastIndex") {
            let receiver_has_typed_array_find_last_index = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "findLastIndex"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info.function_targets.contains(
                        &StandardBuiltinId::TypedArrayPrototypeFindLastIndex.function_id(),
                    ),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_has_typed_array_find_last_index {
                return self.emit_array_direct_builtin_method_call(
                    StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
                    "TypedArray.prototype.findLastIndex",
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            return self.emit_array_find_last_index_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Map.property_name())
        {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Map);
            let receiver_has_array_map = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "map"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeMap.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Map,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            if receiver_is_array || receiver_has_array_map {
                return self.emit_array_map_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Map,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Every.property_name())
        {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Every);
            let receiver_has_array_every = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "every"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeEvery.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Every,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            if receiver_is_array || receiver_has_array_every {
                return self.emit_array_every_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Every,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Some.property_name())
        {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Some);
            let receiver_has_array_some = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "some"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeSome.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Some,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            if receiver_is_array || receiver_has_array_some {
                return self.emit_array_some_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Some,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Filter.property_name())
        {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator =
                receiver_shape_targets_iterator_helper(receiver, IteratorHelper::Filter);
            let receiver_has_array_filter = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "filter"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::ArrayPrototypeFilter.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Filter,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
            if receiver_is_array || receiver_has_array_filter {
                return self.emit_array_filter_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
            if receiver_needs_dynamic_helper_dispatch(receiver) {
                return self
                    .emit_iterator_prototype_helper_method_call(
                        IteratorHelper::Filter,
                        receiver,
                        args,
                        MethodCallDestination::new(payload_local, tag_local),
                        function,
                    )
                    .map(DestinationWritten::discharge);
            }
        }
        // `forEach` needs no fall-back disjunct, and the batch-5 story for why
        // it was already correct — "it was the one helper that acquired its
        // callee with an ordinary `[[Get]]`, and it is now the same call as the
        // other nine" — was wrong on both halves, which is why it no longer sits
        // here. Callee acquisition was never the defect (see this file's header:
        // for the failing receiver no code was emitted for the call at all), no
        // arm was converted, and "the other nine" does not describe anything:
        // there are eleven helpers, seven bare-guard blocks, `drop`/`flatMap` on
        // a different guard shape, and `toArray` never reaches this function.
        //
        // What is true at this head: lowering resolves `forEach` off the fixed
        // instance shape, so `receiver_shape_targets_iterator_helper` fires for
        // the class receiver and this guard alone is enough.
        // `wasm_iterator_helper_class_receiver_for_each.js` is what covers it.
        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::ForEach.property_name())
            && receiver_shape_targets_iterator_helper(receiver, IteratorHelper::ForEach)
        {
            return self
                .emit_iterator_prototype_helper_method_call(
                    IteratorHelper::ForEach,
                    receiver,
                    args,
                    MethodCallDestination::new(payload_local, tag_local),
                    function,
                )
                .map(DestinationWritten::discharge);
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), "trim" | "trimStart" | "trimLeft" | "trimEnd" | "trimRight"))
        {
            let trim_start = matches!(
                key,
                PropertyKeyIr::StaticString(name)
                    if matches!(name.as_str(), "trim" | "trimStart" | "trimLeft")
            );
            let trim_end = matches!(
                key,
                PropertyKeyIr::StaticString(name)
                    if matches!(name.as_str(), "trim" | "trimEnd" | "trimRight")
            );
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let string_local = self.reserve_temp_local();

            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "String.prototype method receiver is null or undefined",
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_value_to_string_payload(
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(string_local));
            self.emit_ecmascript_trim_payload_from_locals(
                string_local,
                trim_start,
                trim_end,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::End);

            self.release_temp_local(string_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        let string_html_builtin = match key {
            PropertyKeyIr::StaticString(name) => match name.as_str() {
                "anchor" => Some(StandardBuiltinId::StringPrototypeAnchor),
                "big" => Some(StandardBuiltinId::StringPrototypeBig),
                "blink" => Some(StandardBuiltinId::StringPrototypeBlink),
                "bold" => Some(StandardBuiltinId::StringPrototypeBold),
                "fixed" => Some(StandardBuiltinId::StringPrototypeFixed),
                "fontcolor" => Some(StandardBuiltinId::StringPrototypeFontcolor),
                "fontsize" => Some(StandardBuiltinId::StringPrototypeFontsize),
                "italics" => Some(StandardBuiltinId::StringPrototypeItalics),
                "link" => Some(StandardBuiltinId::StringPrototypeLink),
                "small" => Some(StandardBuiltinId::StringPrototypeSmall),
                "strike" => Some(StandardBuiltinId::StringPrototypeStrike),
                "sub" => Some(StandardBuiltinId::StringPrototypeSub),
                "substr" => Some(StandardBuiltinId::StringPrototypeSubstr),
                "substring" => Some(StandardBuiltinId::StringPrototypeSubstring),
                "sup" => Some(StandardBuiltinId::StringPrototypeSup),
                "match" => Some(StandardBuiltinId::StringPrototypeMatch),
                "matchAll" => Some(StandardBuiltinId::StringPrototypeMatchAll),
                "replace" => Some(StandardBuiltinId::StringPrototypeReplace),
                "replaceAll" => Some(StandardBuiltinId::StringPrototypeReplaceAll),
                "search" => Some(StandardBuiltinId::StringPrototypeSearch),
                "indexOf" => Some(StandardBuiltinId::StringPrototypeIndexOf),
                "lastIndexOf" => Some(StandardBuiltinId::StringPrototypeLastIndexOf),
                "at" => Some(StandardBuiltinId::StringPrototypeAt),
                "slice" => Some(StandardBuiltinId::StringPrototypeSlice),
                "split" => Some(StandardBuiltinId::StringPrototypeSplit),
                "padStart" => Some(StandardBuiltinId::StringPrototypePadStart),
                "padEnd" => Some(StandardBuiltinId::StringPrototypePadEnd),
                "repeat" => Some(StandardBuiltinId::StringPrototypeRepeat),
                "endsWith" => Some(StandardBuiltinId::StringPrototypeEndsWith),
                "includes" => Some(StandardBuiltinId::StringPrototypeIncludes),
                "startsWith" => Some(StandardBuiltinId::StringPrototypeStartsWith),
                "toLocaleLowerCase" => Some(StandardBuiltinId::StringPrototypeToLocaleLowerCase),
                "toLocaleUpperCase" => Some(StandardBuiltinId::StringPrototypeToLocaleUpperCase),
                "toLowerCase" => Some(StandardBuiltinId::StringPrototypeToLowerCase),
                "toUpperCase" => Some(StandardBuiltinId::StringPrototypeToUpperCase),
                "toString" => Some(StandardBuiltinId::StringPrototypeToString),
                "valueOf" => Some(StandardBuiltinId::StringPrototypeValueOf),
                "isWellFormed" => Some(StandardBuiltinId::StringPrototypeIsWellFormed),
                "toWellFormed" => Some(StandardBuiltinId::StringPrototypeToWellFormed),
                "trim" => Some(StandardBuiltinId::StringPrototypeTrim),
                "trimStart" | "trimLeft" => Some(StandardBuiltinId::StringPrototypeTrimStart),
                "trimEnd" | "trimRight" => Some(StandardBuiltinId::StringPrototypeTrimEnd),
                _ => None,
            },
            _ => None,
        };
        if let Some(builtin) = string_html_builtin {
            let receiver_is_string = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String));
            let receiver_has_string_builtin = match key {
                PropertyKeyIr::StaticString(name) => receiver
                    .heap_shape
                    .as_deref()
                    .and_then(|shape| read_static_heap_shape_property(shape, name))
                    .is_some_and(|property| match property {
                        ObjectShapeProperty::Data(info) => {
                            info.function_targets.contains(&builtin.function_id())
                        }
                        ObjectShapeProperty::Accessor { .. } => false,
                    }),
                _ => false,
            };
            if receiver_is_string || receiver_has_string_builtin {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                if matches!(
                    builtin,
                    StandardBuiltinId::StringPrototypeToLocaleLowerCase
                        | StandardBuiltinId::StringPrototypeToLocaleUpperCase
                        | StandardBuiltinId::StringPrototypeToLowerCase
                        | StandardBuiltinId::StringPrototypeToUpperCase
                ) {
                    self.emit_function_handle_call_with_argv_inner(
                        callee_payload_local,
                        callee_tag_local,
                        Some((receiver_payload_local, Some(receiver_tag_local))),
                        argc_local,
                        argv_local,
                        payload_local,
                        tag_local,
                        PropagateCallThrow::ToActiveHandler,
                        function,
                    )?;
                } else {
                    self.emit_function_handle_call_with_argv(
                        callee_payload_local,
                        callee_tag_local,
                        Some((receiver_payload_local, Some(receiver_tag_local))),
                        argc_local,
                        argv_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                }
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        if receiver.kind == ValueKind::BigInt {
            let builtin = match key {
                PropertyKeyIr::StaticString(name) => match name.as_str() {
                    "toString" => Some(StandardBuiltinId::BigIntPrototypeToString),
                    "toLocaleString" => Some(StandardBuiltinId::BigIntPrototypeToLocaleString),
                    "valueOf" => Some(StandardBuiltinId::BigIntPrototypeValueOf),
                    _ => None,
                },
                _ => None,
            };
            if let Some(builtin) = builtin {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot read properties of null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);
        // 7.2.1 RequireObjectCoercible / 13.3.6.2 EvaluateCall. A receiver that
        // can only be undefined or null always takes the throw above, so the
        // runtime kind dispatch below is statically dead: emitting it only
        // grows the function and can fail the whole module on a missing
        // primitive-prototype builtin that could never be reached.
        //
        // "Statically dead" is a claim about the *type*, and this compiler has
        // been wrong about that type: `new S()` for a class with heritage and no
        // explicit constructor lowers with `kind = Undefined` and a nullish
        // `possible_kinds` while the runtime value is an ordinary object. The
        // emitted nullish test then does not fire, control reaches this return,
        // and before the two stores below both destination locals kept whatever
        // the scratch pair last held — a corruption, not a wrong answer, and the
        // whole of the batch-5 `iterator_helpers` failure set.
        //
        // The stores do not make that receiver *work* — the routing in the
        // helper blocks above is what does — but they bound what a future
        // mistyping can cost to a well-defined `undefined` instead of unrelated
        // memory, and they are what lets this arm honour "no path out of
        // `emit_method_call` leaves the destination unwritten".
        if matches!(receiver.kind, ValueKind::Undefined | ValueKind::Null)
            && receiver.possible_kinds.is_subset_of(KindSet::NULLISH)
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.release_temp_local(flags_local);
            self.release_temp_local(table_index_local);
            self.release_temp_local(callee_env_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        let receiver_kind = if matches!(receiver.kind, ValueKind::Undefined | ValueKind::Null) {
            ValueKind::Dynamic
        } else {
            receiver.kind
        };
        match receiver_kind {
            ValueKind::Object | ValueKind::Function | ValueKind::Dynamic => {
                let runtime_number_builtin = if receiver_kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::NumberPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "toLocaleString" => {
                            Some(StandardBuiltinId::NumberPrototypeToLocaleString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::NumberPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let runtime_string_builtin = if receiver_kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::StringPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::StringPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let runtime_bigint_builtin = if receiver_kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::BigIntPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "toLocaleString" => {
                            Some(StandardBuiltinId::BigIntPrototypeToLocaleString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::BigIntPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(builtin) = runtime_number_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_string_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_bigint_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                if runtime_number_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
                if runtime_string_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
                if runtime_bigint_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
            }
            ValueKind::Array => {
                if matches!(
                    key,
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::StringExpr(_)
                ) {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    let own_found_local = self.reserve_temp_local();
                    let prototype_payload_local = self.reserve_temp_local();
                    let prototype_tag_local = self.reserve_temp_local();
                    self.emit_array_named_prop_read(
                        receiver_payload_local,
                        key_local,
                        callee_payload_local,
                        callee_tag_local,
                        Some(own_found_local),
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(own_found_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_PROTOTYPE_OFFSET,
                        prototype_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(prototype_payload_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Else);
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                        prototype_tag_local,
                        function,
                    );
                    self.emit_object_read(
                        prototype_payload_local,
                        prototype_tag_local,
                        receiver_payload_local,
                        receiver_tag_local,
                        key_local,
                        callee_payload_local,
                        callee_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(prototype_tag_local);
                    self.release_temp_local(prototype_payload_local);
                    self.release_temp_local(own_found_local);
                    self.release_temp_local(key_local);
                } else {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_array_read(
                        receiver_payload_local,
                        index_local,
                        callee_payload_local,
                        callee_tag_local,
                        function,
                    );
                    self.release_temp_local(index_local);
                }
            }
            ValueKind::Arguments => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.emit_arguments_read(
                    receiver_payload_local,
                    index_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            ValueKind::String => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Number => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Boolean => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Symbol => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            _ => {
                self.release_temp_local(flags_local);
                self.release_temp_local(table_index_local);
                self.release_temp_local(callee_env_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Err(EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: property access on non-object target inferred as {:?}",
                    receiver.kind
                )));
            }
        }

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_call(
        &mut self,
        name: &str,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.values().find(|meta| meta.name == name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: direct call to unknown top-level function `{name}`"
            ))
        })?;
        // A direct call into a builtin's body requires its real body to be
        // emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        let wasm_index = meta.wasm_index;
        let is_class_constructor = meta.protocol.class_kind() == ClassFunctionKind::Constructor;
        let uses_resumable_call_dispatch = matches!(
            meta.protocol.execution_kind(),
            FunctionExecutionKind::Generator
                | FunctionExecutionKind::Async
                | FunctionExecutionKind::AsyncGenerator
        );
        let is_strict = meta.strict;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let callee_table_index_local = self.reserve_temp_local();

        if uses_resumable_call_dispatch {
            if let Some(storage) = self.lookup_binding(name) {
                self.read_binding_to_locals(
                    storage,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
            } else {
                let key_local = self.reserve_temp_local();
                let global_object_local = self.reserve_temp_local();
                let global_object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(global_object_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(global_object_tag_local));
                self.emit_object_read(
                    global_object_local,
                    global_object_tag_local,
                    global_object_local,
                    global_object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(global_object_tag_local);
                self.release_temp_local(global_object_local);
                self.release_temp_local(key_local);
            }
            self.emit_function_handle_call_with_argv(
                callee_payload_local,
                callee_tag_local,
                None,
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(callee_table_index_local);
            self.release_temp_local(callee_env_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            return Ok(());
        }

        if is_class_constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.active_throw_target() {
                self.emit_branch_to_target(target, function);
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            if let Some(storage) = self.lookup_binding(name) {
                self.read_binding_to_locals(
                    storage,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
            } else {
                let key_local = self.reserve_temp_local();
                let global_object_local = self.reserve_temp_local();
                let global_object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(global_object_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(global_object_tag_local));
                self.emit_object_read(
                    global_object_local,
                    global_object_tag_local,
                    global_object_local,
                    global_object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
                self.release_temp_local(global_object_tag_local);
                self.release_temp_local(global_object_local);
                self.release_temp_local(key_local);
            }
            self.emit_default_this_for_known_strictness(is_strict, function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        }
        self.release_temp_local(callee_table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_pre_evaluated_arg_vector(
        &mut self,
        args: &[(u32, u32)],
        argc_local: u32,
        argv_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let capacity = (args.len() as u64).max(MIN_HEAP_CAPACITY);

        function.instruction(&Instruction::I64Const(args.len() as i64));
        function.instruction(&Instruction::LocalSet(argc_local));
        // Argument vectors are built at every call site with pre-evaluated
        // args; go through the shared array-alloc helper (which performs the
        // full ~30-store header/slot init once) instead of inlining that init
        // at each site.
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            function.instruction(&Instruction::I64Const(args.len() as i64));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(argv_local));
        } else {
            self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(argv_local));
            self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(buffer_local));
            self.store_i64_local_at_offset(argv_local, HEAP_PTR_OFFSET, buffer_local, function);
            self.store_i64_const_at_offset(
                argv_local,
                HEAP_LEN_OFFSET,
                args.len() as u64,
                function,
            );
            self.store_i64_const_at_offset(argv_local, HEAP_CAP_OFFSET, capacity, function);
            function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                argv_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(
                argv_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                ValueKind::Array.tag() as u64,
                function,
            );
            self.emit_init_array_exotic_slots(argv_local, function);
        }

        for (index, (arg_payload_local, arg_tag_local)) in args.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_TAG_OFFSET,
                *arg_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                *arg_payload_local,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                ARRAY_DESCRIPTOR_NORMAL_DATA,
                function,
            );
        }

        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_builtin_arg_to_locals(
        &mut self,
        index: usize,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let argc_local = self.argc_param_local();
        let argv_local = self.argv_param_local();
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(
            argv_local,
            self.scratch_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }
}
