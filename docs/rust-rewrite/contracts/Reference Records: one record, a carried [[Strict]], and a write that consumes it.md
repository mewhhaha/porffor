# Contract: Reference Records — one record, a carried `[[Strict]]`, and a write that consumes it

Area owner: FORMALIZER lane, THEORY-FIRST campaign.
Branch: `claude/test-driven-rust-opus-pp6giw`. Tree read at `84e782506`.
Status: **specification only.** No source file was edited in producing this
document. It is the artefact the encoder implements verbatim and the dry-runner
checks against.

## 0. How to read this, and what is measured

Every count and line number below was obtained by reading the tree at
`84e782506`, not estimated. Where a number is derived rather than counted, it
says so. Where this contract **disagrees with the area brief**, the disagreement
is stated explicitly in §5 with the evidence, because the brief is what the
encoder would otherwise implement.

Three numbers to anchor on, all counted:

| Fact | Value | How counted |
|---|---|---|
| `ExprIr` variants | **77** | `ir.rs:1313`–`ir.rs:1630`, one `^    [A-Z]` line per variant |
| `unsupported_expr(` calls in `lowering.rs` | **130** | `grep -c` |
| `lowering.rs` length | **38,003** lines | `wc -l` |

Spec citations are to ECMA-262. Where the current edition renumbered a step,
both numberings are given, because the tree's own comments use the older one and
a reader matching comment to spec will otherwise conclude one of them is wrong.

---

## 1. Spec basis

### 1.1 The Reference Record is a four-field product (6.2.5)

6.2.5 defines the Reference Record specification type with exactly four fields:

| Field | Domain |
|---|---|
| `[[Base]]` | an ECMAScript language value, **or** an Environment Record, **or** `unresolvable` |
| `[[ReferencedName]]` | a String, **or** a Symbol, **or** a Private Name |
| `[[Strict]]` | a Boolean |
| `[[ThisValue]]` | an ECMAScript language value, **or** `empty` |

Two structural facts follow immediately and are the whole basis of this
contract:

**F1 — `[[Strict]]` is a field of the record, not of the running code.** It is
*populated* from the running code's strictness at the moment the Reference is
created (13.1.3 Identifier evaluation: "*Return ? ResolveBinding(StringValue of
Identifier)*", where ResolveBinding's `strict` argument is the strictness of the
running execution context's code; 13.3.2.1 / 13.3.3.1 member-expression
evaluation set `[[Strict]]` to `strict`, the parameter threaded from the
enclosing production). Once created, the record carries its own `[[Strict]]`
wherever it travels. Any implementation that recovers `[[Strict]]` by asking
"what is the strictness of the code I am currently emitting?" is computing a
*different quantity* that happens to agree whenever creation and consumption sit
in the same function body — and disagrees the moment they do not. That is
b361b4815 in one sentence (§3, MC2).

**F2 — `[[ThisValue]]` is separate from `[[Base]]` and is only non-`empty` for a
Super Reference.** 6.2.5.1 defines `IsSuperReference(V)` as "*if V.[[ThisValue]]
is not empty, return true*". GetThisValue(V) (6.2.5.4) returns `[[ThisValue]]`
for a Super Reference and `[[Base]]` otherwise. A `super.x` Reference therefore
**reads through the home object's prototype but sets with the receiver `this`**.
Collapsing the two fields is not a loss of tidiness; it changes which object
gets an own property.

### 1.2 The three predicates are total functions of the record (6.2.5.1)

```
IsPropertyReference(V)     ≡ V.[[Base]] is neither unresolvable nor an Environment Record
IsUnresolvableReference(V) ≡ V.[[Base]] is unresolvable
IsSuperReference(V)        ≡ V.[[ThisValue]] is not empty
IsPrivateReference(V)      ≡ V.[[ReferencedName]] is a Private Name
```

They are **total** and mutually determined by the two fields `[[Base]]` and
`[[ReferencedName]]`. The three-way split
`unresolvable | Environment Record | value` is a **closed partition of
`[[Base]]`**, and `IsSuperReference` is orthogonal to it (a Super Reference is
always a property reference). This is what makes the base an `enum` in §2 and
what makes a catch-all arm over it a defect rather than defensive coding.

### 1.3 GetValue (6.2.5.5) does not read `[[Strict]]`

```
1. If V is not a Reference Record, return V.
2. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
3. If IsPropertyReference(V) is true, then
   a. Let baseObj be ? ToObject(V.[[Base]]).
   b. If IsPrivateReference(V) is true, return ? PrivateGet(baseObj, V.[[ReferencedName]]).
   c. Return ? baseObj.[[Get]](V.[[ReferencedName]], GetThisValue(V)).
4. Else,
   a. Let base be V.[[Base]].
   b. Assert: base is an Environment Record.
   c. Return ? base.GetBindingValue(V.[[ReferencedName]], V.[[Strict]]).
```

Step 2 throws in **both** modes. This is load-bearing for the dry-run corpus:
it means an *unresolvable* reference is a bad oracle for a `[[Strict]]` defect,
because the GetValue side throws in sloppy code too and hides whether the
PutValue side was correct. §6 uses it deliberately, and §6 marks which corpus
entries are contaminated by it.

Step 4.c does pass `[[Strict]]`, to GetBindingValue. That path is TDZ, and TDZ
throws regardless of `S` for a `let`/`const` binding (9.1.1.1.6 step 3), so it
is not an observable `[[Strict]]` consumer here. **Out of scope** (§4.4).

### 1.4 PutValue (6.2.5.6) — the four consumers of `[[Strict]]`

Current-edition numbering, with the older numbering the tree's comments use
given in brackets:

```
1. If V is not a Reference Record, throw a ReferenceError exception.
2. If IsUnresolvableReference(V) is true, then                              [old 5]
   a. If V.[[Strict]] is true, throw a ReferenceError exception.            [old 5.a]   <-- CONSUMER A
   b. Let globalObj be GetGlobalObject().
   c. Perform ? Set(globalObj, V.[[ReferencedName]], W, false).
   d. Return unused.
3. If IsPropertyReference(V) is true, then                                  [old 6]
   a. Let baseObj be ? ToObject(V.[[Base]]).
   b. If IsPrivateReference(V) is true,
      return ? PrivateSet(baseObj, V.[[ReferencedName]], W).
   c. Let succeeded be ? baseObj.[[Set]](V.[[ReferencedName]], W, GetThisValue(V)).  [old 6.e]
   d. If succeeded is false and V.[[Strict]] is true,
      throw a TypeError exception.                                          [old 6.f]   <-- CONSUMER B
   e. Return unused.
4. Else,
   a. Let base be V.[[Base]].
   b. Assert: base is an Environment Record.
   c. Return ? base.SetMutableBinding(V.[[ReferencedName]], W, V.[[Strict]]).          <-- CONSUMER C
```

> **Numbering note for the encoder.** `ir.rs:1374` and `environments.rs:1247`
> both say "PutValue step 2.b" for consumer A. In the current edition the step
> is **2.a**; 2.b is `Let globalObj be GetGlobalObject()`. Correct both comments
> in the same patch, or leave them — but do not treat the existing text as
> authoritative when reading the code against the spec.

Plus the fourth consumer, in the `delete` operator (13.5.1.2):

```
1. Let ref be ? Evaluation of UnaryExpression.
3. If ref is not a Reference Record, return true.
4. If IsUnresolvableReference(ref) is true, then
   a. Assert: ref.[[Strict]] is false.
   b. Return true.
5. If IsPropertyReference(ref) is true, then
   a. Assert: IsPrivateReference(ref) is false.
   b. If IsSuperReference(ref) is true, throw a ReferenceError exception.    <-- uses F2
   c. Let baseObj be ? ToObject(ref.[[Base]]).
   d. Let deleteStatus be ? baseObj.[[Delete]](ref.[[ReferencedName]]).
   e. If deleteStatus is false and ref.[[Strict]] is true,
      throw a TypeError exception.                                           <-- CONSUMER D
   f. Return deleteStatus.
6. Else, ... return ? base.DeleteBinding(ref.[[ReferencedName]]).
```

**The area brief says `[[Strict]]` is load-bearing "in exactly three places".
It is four.** Consumer C — `SetMutableBinding(N, W, S)` — is the one the brief
omits, and it is the one whose treatment decides which IR variants need a field
at all. §1.5 settles it.

Step 4.a of `delete` is an *assertion*, not a check: `delete unresolvableName`
in strict code is an early SyntaxError (13.5.1.1), so the case cannot arise. The
implementation therefore owes nothing here.

### 1.5 Consumer C is statically decidable; A, B and D are not

9.1.1.1.5 Declarative Environment Record SetMutableBinding(N, V, S):

```
2. If envRec does not have a binding for N, then
   a. If S is true, throw a ReferenceError exception.
   b. Perform envRec.CreateMutableBinding(N, true) ... Return unused.
3. If the binding for N in envRec is a strict binding, set S to true.
4. If the binding for N in envRec has not yet been initialized,
   throw a ReferenceError exception.
5. Else if the binding for N in envRec is a mutable binding,
   change its bound value to V.
6. Else,
   a. Assert: this is an attempt to change the value of an immutable binding.
   b. If S is true, throw a TypeError exception.
```

Step 2 cannot fire for a Reference produced by ResolveBinding (the binding was
found, or the Reference would be unresolvable and we would be in PutValue
branch 2). Step 4 is TDZ — mode-independent. That leaves **step 6.b**: an
immutable binding, `S` true → TypeError; `S` false → silent no-op.

Whether a *resolved* binding is immutable is decided at **lowering** time in
this compiler: it is `BindingInfo.mode == BindingMode::Const`
(`lowering.rs:5`–`11`), plus the sloppy named-function-expression case tracked in
`sloppy_immutable_binding_storage_names`. The lowerer already does exactly this,
and already consults strictness while doing it:

- `lowering.rs:31004`–`31051` — `const` reassignment folds to
  `ExprIr::RuntimeThrow { TypeError, "assignment to immutable binding" }`.
- `lowering.rs:31026`–`31032` — the sloppy-immutable case returns the value
  unchanged when `!self.is_current_owner_strict()`.

**Therefore consumer C is fully discharged at lowering time and needs no IR
field.** Consumers A, B and D are not statically decidable — A because global
resolution can change at run time (`globalThis.x = ...` before the write), B and
D because `[[Set]]`/`[[Delete]]` returning `false` depends on runtime property
attributes, extensibility and Proxy traps. Those three, and only those three,
require `[[Strict]]` to survive into the IR.

This is the proof behind §5.1's deviation: adding a `strictness` field to
`AssignIdentifier`, `CompoundAssignIdentifier` or `UpdateIdentifier` produces a
field no backend arm can read, which AGENTS.md classifies as decoration.

### 1.6 Single-evaluation obligations (13.15.1, 13.15.2, 13.4)

13.15.2 EvaluateAssignment, for `LeftHandSideExpression AssignmentOperator
AssignmentExpression` (the compound case):

```
1. Let lref be ? Evaluation of LeftHandSideExpression.
2. Let lval be ? GetValue(lref).
3. Let rref be ? Evaluation of AssignmentExpression.
4. Let rval be ? GetValue(rref).
5. Let r be ? ApplyStringOrNumericBinaryOperator(lval, opText, rval).
6. Perform ? PutValue(lref, r).
7. Return r.
```

Two obligations, and they are different:

**O1 (single evaluation).** `LeftHandSideExpression` is evaluated **once**, at
step 1. The record produced there is the record consumed at both step 2 and step
6. If the base expression or the computed key has side effects, they run exactly
once. `a[idx()].v += 1` must call `idx` once.

**O2 (ordering).** LHS evaluation strictly precedes RHS evaluation. And for a
computed member expression, 13.3.3.1 evaluates the base, then the key
*expression*, but defers `ToPropertyKey` until the Reference is consumed — so
`base[prop()] = expr()` runs `prop()` before `expr()`, while
`base[objWithThrowingToString] = expr()` runs `expr()` before the `toString`.
This is exactly what
`test/language/expressions/assignment/target-member-computed-reference.js`
asserts, in its two halves.

13.15.1 (logical `&&=`, `||=`, `??=`) has the same shape with the PutValue
conditional on the short-circuit branch. 13.4 update expressions (`++`/`--`)
likewise: *"Let expr be ? Evaluation of UnaryExpression"* once, then GetValue,
ToNumeric, and PutValue on that one record.

### 1.7 Where the spec leaves latitude — the choices this contract makes

| # | Latitude | Choice | Why |
|---|---|---|---|
| **C1** | The spec's Reference is a runtime value; an AOT compiler may reify it, or may fuse Reference creation and consumption into one node. | **Fuse, and make the fused node the unit.** A `ReferenceRecord` exists only inside the lowerer; the IR carries fused nodes (`PropertyWrite`, `PropertyUpdate`, …) that each denote "one Reference, created and consumed". | A fused node makes O1 structural: there is no way to emit two evaluations of a base that appears once in the node. `PropertyUpdate` already works this way and is the model. |
| **C2** | Consumer C may be discharged statically or dynamically. | **Statically, at lowering** (§1.5). | It already is; formalising it lets §5.1 delete three fields the brief would add, and turns "which variants carry `[[Strict]]`" from taste into a theorem. |
| **C3** | `[[Strict]]` may be a `bool` threaded by convention or a distinct type. | **A two-inhabitant `enum Strictness`** with no `bool` conversion in the producing direction. | MC1 and MC2 are both "a `bool` in the wrong slot". A `bool` cannot make either a compile error. |
| **C4** | The strictness of code whose owner plan is missing. | **`Strictness::Strict`, plus a recorded `IrDiagnostic`.** Never a silent `false`. | A spurious throw fails loudly in the first test that touches it; a suppressed throw is invisible — the fae75423a story, where strict mode "was not enforced at all for unresolvable references" and no test noticed. Prefer the loud failure, and make it non-silent besides. |
| **C5** | Where the `ToPropertyKey` of a computed key runs relative to RHS evaluation. | **Preserve the spec order**: key *expression* before RHS, `ToPropertyKey` after. The `ReferenceRecord` pins the key expression's *value*, not its property-key coercion. | O2. ~~The tree already lowers this correctly~~ — **false; corrected at DISCREPANCY-FIXER stage, see below.** The contract must not regress the *compound-assignment* path, and must not claim the plain-assignment path is already right. |

> **C5's "the tree already lowers this correctly" is refuted.** It holds for the
> compound-assignment path, which reaches `reference_base_of_lowered_read`'s
> `SpecOperationIr::Get`/`GetV` arm and pins the key operand uncoerced. It is
> false for **plain assignment** (`ExprIr::PropertyWrite`), which is where
> `base[prop] = expr()` actually lands:
> `compile_property_write_to_locals` (`objects.rs:6343`) calls
> `compile_object_key_to_locals` at `:6364` — and again at `:6435`, `:6453`,
> `:6467` — *before* it compiles the value at `:6370`, and
> `compile_object_key_to_locals` performs the coercion itself
> (`emit_value_to_property_key_locals`, `objects.rs:9067`). So corpus entry 6's
> second half throws `Test262Error("property key evaluated")` instead of
> `DummyError`.
>
> **Pre-existing, and outside this landing's delta.** It is recorded here rather
> than fixed because the fix that is safe is the *typed* one, and it is not a
> local reordering: `PropertyKeyIr::StringExpr` conflates "the key operand" with
> "the property key", so the IR has nowhere to say the coercion is owed. The
> shape that closes it —
>
> ```rust
> enum PropertyKeyIr {
>     UncoercedExpr(Box<TypedExpr>),   // operand pinned, ToPropertyKey owed
>     CoercedExpr(Box<TypedExpr>),     // already a String | Symbol
>     …
> }
> ```
>
> — makes "this emitter coerced before the RHS" a named arm an author has to
> write, and is `E0004` at every emitter that matches the key. `PropertyKeyIr` is
> a shared enum matched across the whole backend, so this is its own lane with
> build access. The purely local alternative (evaluate the raw key operand,
> then the value, then `emit_value_to_property_key_locals`) touches four
> branches of `compile_property_write_to_locals` and two helpers shared with the
> *read* path, where the current order is correct — not work to do blind.
>
> **Owed:** a follow-up lane, named here so no other lane reads C5 as a
> guarantee. Ledger **L7**.
| **C6** | Whether an internal, compiler-synthesised binding write is modelled as a Reference. | **It is not.** Compiler temps are written with `ExprIr::AssignIdentifier`, which under §5.1 carries no `[[Strict]]` at all, so the question does not arise. | Avoids inventing a third `Strictness` inhabitant, or a `Strictness::Internal` that would be a lie at 27 construction sites. |

---

## 2. Type mapping

New module: **`crates/lila-ir/src/reference.rs`** (this lane's exclusive
file). Everything in §2.1–§2.6 lives there unless stated otherwise.

### 2.1 `Strictness` — invariant I1

> **I1.** The Reference's `[[Strict]]` is a two-valued domain that may not be
> confused with any other `bool` in a call, and may not be produced by accident.

```rust
/// `[[Strict]]` of a Reference Record (6.2.5).
///
/// Not a `bool`. The four consumers of this field (PutValue 2.a, PutValue 3.d,
/// PutValue 4.c, `delete` 5.e) each sit next to other boolean parameters —
/// `implicit`, `configurable`, `succeeded` — and a transposition between them
/// is silent under `bool` and was shipped once (b361b4815).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strictness {
    Sloppy,
    Strict,
}
```

Prohibited on this type, and each prohibition is a defect class:

| Prohibited | Because |
|---|---|
| `impl Default` | there is no defensible default; C4 exists precisely to force the decision |
| `impl From<bool>` / `impl From<Strictness> for bool` | reintroduces MC1 with one extra keystroke |
| `derive(PartialOrd, Ord)` | invites `strictness > other`, which means nothing |
| `as` casts, `#[repr(u8)]` | `strictness as i64` at an `I64Const` site is MC2 with the type system watching |

Exactly **two** exits, each named for the spec obligation it discharges, so the
call site reads as spec text:

```rust
impl Strictness {
    /// PutValue 2.a / 3.d and `delete` 5.e: does an observed failure throw?
    pub(crate) fn throws_on_failed_set(self) -> bool { matches!(self, Self::Strict) }

    /// The backend's runtime flag word for the outlined object-write helper
    /// (helper parameter 5, `emit.rs:3526`). Named so that a reader can check
    /// it against `object_write_strict_flag_local`'s contract.
    pub(crate) fn helper_flag_word(self) -> i64 { i64::from(self.throws_on_failed_set()) }
}
```

**Sole producer** — replaces `Lowerer::is_current_owner_strict`
(`lowering.rs:16916`–`16921`), whose `is_some_and(|owner| owner.strict)` is the
silent default C4 forbids:

```rust
impl Lowerer<'_> {
    /// `[[Strict]]` for a Reference created by the code currently being lowered.
    fn reference_strictness(&mut self) -> Strictness {
        match self.analysis.owner_plans.get(&self.current_owner_id) {
            Some(owner) if owner.strict => Strictness::Strict,
            Some(_) => Strictness::Sloppy,
            // C4: unknown strictness is not sloppy. Loud, and recorded.
            None => {
                self.record_missing_owner_plan_strictness();
                Strictness::Strict
            }
        }
    }
}
```

Note the signature change to `&mut self`. All six existing callers
(`lowering.rs:11438`, `11449`, `31029`, `31087`, `32779`, and the new
`lower_reference`) sit inside `&mut self` methods; verified by reading each
enclosing `fn`. `record_missing_owner_plan_strictness` pushes an `IrDiagnostic`
— it must not `panic!`, because the same accessor is reached from the three
`lower_generated_*` paths (`lowering.rs:14565`, `17785`, `19793`, `20193`) whose
owner-plan registration this lane does not own.

### 2.2 `ReferenceRecord` and `ReferenceBase` — invariants I2, I3, I4

> **I2.** `[[Base]]`'s three-way partition (§1.2) is closed; a consumer that
> handles four of five shapes must not compile.
> **I3.** A Reference cannot exist without a `[[Strict]]`.
> **I4.** A Super Reference cannot exist without a `[[ThisValue]]`.

```rust
/// A Reference Record (6.2.5), reified for the duration of one lowering step.
///
/// Fields are private and there is no public constructor: the only way to
/// obtain one is `Lowerer::lower_reference`, which is also the only place that
/// calls `Lowerer::reference_strictness`. That is what makes I3 hold — you
/// cannot build a record and forget to ask.
///
/// Deliberately NOT `Clone` and NOT `Copy`: see I5.
#[must_use = "a Reference Record that is neither read nor written has evaluated \
              its base and key for nothing"]
pub(crate) struct ReferenceRecord {
    base: ReferenceBase,
    strictness: Strictness,
}

/// The closed partition of `[[Base]]` from 6.2.5.1, refined by what this
/// compiler can prove at lowering time.
pub(crate) enum ReferenceBase {
    /// `[[Base]]` is an Environment Record and the binding is resolved to
    /// concrete storage. PutValue branch 4; consumer C, already discharged
    /// (§1.5), so this variant's write carries no runtime strictness.
    Binding { storage_name: String },

    /// `[[Base]]` is unresolvable, or is the global object. The two cases are
    /// not separable at compile time (C4 / `implicit`), so they share a variant
    /// and the backend performs the presence test — exactly what
    /// `emit_global_property_write_checked` (`environments.rs:1252`) already does.
    Global { name: String, implicit: bool },

    /// `IsPropertyReference` true, `IsPrivateReference` false,
    /// `IsSuperReference` false.
    Property { base: TypedExpr, key: PropertyKeyIr },

    /// `IsPrivateReference` true. PutValue 3.b: PrivateSet throws
    /// unconditionally on failure, so no `[[Strict]]` reaches the IR.
    Private { base: TypedExpr, private_name_id: PrivateNameId },

    /// `IsSuperReference` true. `[[Base]]` is the super base; `[[ThisValue]]`
    /// is the Receiver of `[[Set]]` (PutValue 3.c via GetThisValue).
    Super { key: PropertyKeyIr, this_value: SuperThisValue },
}
```

I4 is carried by a newtype whose only constructor is fallible and requires the
class context to exist:

```rust
/// `[[ThisValue]]` of a Super Reference. Cannot be `empty`: a `ReferenceBase::Super`
/// that does not know its receiver is not constructible.
pub(crate) struct SuperThisValue(TypedExpr);

impl SuperThisValue {
    /// The only constructor. `None` when `ClassLoweringContext` is absent,
    /// which is the `unsupported_expr("object literal method")` case at
    /// `lowering.rs:32810`.
    pub(crate) fn from_class_context(lowerer: &Lowerer<'_>) -> Option<Self> { … }
    pub(crate) fn receiver(&self) -> &TypedExpr { &self.0 }
}
```

I2 is carried by `ReferenceBase` being an `enum` matched exhaustively with no
`_` arm anywhere in `reference.rs` or in `lower_reference`'s consumers. Adding a
sixth base shape then produces **E0004 non-exhaustive patterns** at every
consumer, which is the point.

**`base_mut()` is deleted (DISCREPANCY-FIXER stage).** The encoder added
`ReferenceRecord::base_mut(&mut self) -> &mut ReferenceBase` for pinning, with a
doc comment stating "the shape of the base cannot be changed through this, only
its operands". `&mut ReferenceBase` permits whole-value assignment, so

```rust
*record.base_mut() = ReferenceBase::Global { name };
```

compiled and swapped a property Reference for a global one *after* its
`[[Strict]]` had been chosen — the exact confusion the single record exists to
prevent, with the type carrying none of the invariant the comment asserted. Its
one caller now goes through `ReferenceRecord::pin_operands` (§2.4), which reaches
`evaluated_base_mut()` / `computed_key_mut()` internally and hands the caller one
operand at a time.

### 2.3 Linearity: `read` borrows, `write` consumes — invariant I5

> **I5.** One Reference is written at most once, and a second write of the same
> Reference must not compile.

```rust
impl ReferenceRecord {
    pub(crate) fn strictness(&self) -> Strictness { self.strictness }

    /// GetValue (6.2.5.5). Borrows, because 13.15.2 needs GetValue *and then*
    /// PutValue on the same record.
    pub(crate) fn read(&self) -> ExprIr { … }

    /// PutValue (6.2.5.6). Takes `self` by value.
    pub(crate) fn write(self, value: TypedExpr, compose: Composition) -> PendingReferenceWrite { … }
}
```

`ReferenceRecord` is neither `Clone` nor `Copy`, so a second `write` is
**E0382 use of moved value**. This replaces the hand-maintained precaution the
tree relies on today: `build_property_reference_write` (`lowering.rs:32357`)
takes `&PropertyReference` and is a free-standing method that nothing stops
anyone calling twice.

`Composition` is the closed 2-element domain of how the write sits inside the
surrounding expression, matching the two shapes at `lowering.rs:32284`–`32326`:

```rust
pub(crate) enum Composition {
    /// The write is the whole expression: `ref = v`, `ref op= v`, `++ref`.
    Value,
    /// 13.15.1: `read op (ref = v)` — the write is the short-circuit branch.
    ShortCircuit { op: LogicalBinaryOp, read: TypedExpr },
}
```

Matched exhaustively. A third assignment shape then fails to build rather than
falling into a `_` arm that drops the read.

### 2.4 Pin discharge as a typestate — invariant I6

> **I6.** Every temporary the Reference pinned is materialised exactly once,
> around the whole compound expression, and forgetting to materialise it must
> not compile.

`lowering.rs:32251`–`32279` pins an effectful base and an effectful computed key
into temps; `lowering.rs:32328`–`32338` wraps the result in the corresponding
`MaterializeBinding` chain. Nothing connects the two: the wrap is a convention.

Make it a typestate. `lower_reference` returns the record **and** its pins as a
separate, non-`Clone` value, and the only way to turn a written reference back
into a `TypedExpr` is to spend the pins:

```rust
/// The `MaterializeBinding` bindings this Reference's evaluation created.
/// Not `Clone`, no `Default`, no public constructor; the only producer is
/// `ReferenceRecord::pin_operands` and the only consumer is `materialize`.
#[must_use]
pub(crate) struct ReferencePins(Vec<(String, TypedExpr)>);

/// A written Reference that has not yet had its pins materialised around it.
/// No public field, no `Deref`, no `Into<TypedExpr>`.
#[must_use]
pub(crate) struct PendingReferenceWrite(TypedExpr);

impl ReferencePins {
    /// The single exit. Wraps `write` in this Reference's pin chain, innermost
    /// pin last, matching `lowering.rs:32328`'s `.rev()`.
    pub(crate) fn materialize(self, write: PendingReferenceWrite) -> TypedExpr { … }
}
```

Now the failure modes are compile errors, not review comments:

- forget to materialise → you hold a `PendingReferenceWrite` where a `TypedExpr`
  is required → **E0308 mismatched types**;
- materialise twice → `ReferencePins` moved → **E0382**;
- materialise the *wrong* reference's write → still type-correct, and this is
  the one hole; see ledger **L3**.

**Corrected at DISCREPANCY-FIXER stage.** As first encoded, the E0308 forced *a*
materialise, not *the right* one, and the hole was wider than L3 said:
`ReferencePins` derived `Default` and exposed `pub(crate) fn none()`, so

```rust
let pins = self.pin_reference_operands(record.base_mut());
…
ReferencePins::none().materialize(record.write(value, compose))   // compiled
```

type-checked and silently discarded the real pin chain, leaving `pins` as a
merely-unused binding — a warning at most, and not even that once it is passed
anywhere, because `#[must_use]` on a struct does not fire for a value bound to a
name. Both constructors of an *empty* chain are gone. `ReferencePins` is now
produced only by

```rust
impl ReferenceRecord {
    /// The sole producer of `ReferencePins`. Needs a record to be called on, so
    /// no code path can hold a bare pin chain.
    pub(crate) fn pin_operands(
        &mut self,
        pin: impl FnMut(ReferenceOperand, &mut TypedExpr) -> Option<(String, TypedExpr)>,
    ) -> ReferencePins;
}

/// Which operand is being pinned. The record knows which operands *exist*
/// (exhaustively, over `ReferenceBase`); the caller names the temporary.
pub(crate) enum ReferenceOperand { Base, ComputedKey }
```

which also deletes `base_mut()` — see the note under §2.2 below.

### 2.5 `lower_reference` — invariant I7, and the death of the catch-all

> **I7.** A legal assignment target may not be silently downgraded to
> `unsupported_expr` because the lowered *read* had a shape the reconstruction
> did not anticipate.

```rust
impl Lowerer<'_> {
    /// Builds the Reference Record for an assignment or update target,
    /// **from the AST**, never by re-reading a lowered expression.
    fn lower_reference(
        &mut self,
        target: ReferenceTarget<'_>,
    ) -> Result<(ReferenceRecord, ReferencePins), UnsupportedTarget>;
}

/// The three AST positions that produce a Reference in this compiler.
pub(crate) enum ReferenceTarget<'a> {
    Assign(&'a AssignTarget),          // boa_ast, 4 variants
    Update(&'a UpdateTarget),          // boa_ast, 3 variants
    Access(&'a PropertyAccess),        // boa_ast, 3 variants
}

/// Why a syntactic target produced no Reference. Closed; no `Other`.
pub(crate) enum UnsupportedTarget {
    /// `f() = v` (Annex B). Not "unsupported": a runtime ReferenceError.
    /// Routes to `web_compat_call_assignment_reference_error` (`lowering.rs:30986`).
    WebCompatCall,
    /// Destructuring. Routes to `lower_pattern_assign` (`lowering.rs:31135`).
    Pattern,
    /// `#x` with no enclosing class private environment.
    PrivateNameNotInScope,
    /// `super.x` outside a class body.
    SuperOutsideClassContext,
}
```

The structural claim: `AssignTarget` has **4** variants (`Identifier`, `Access`,
`WebCompatCall`, `Pattern` — `boa_ast-0.21.1/src/expression/operator/assign/mod.rs:126`),
`UpdateTarget` has **3** (`Identifier`, `PropertyAccess`, `WebCompatCall` —
`.../update/mod.rs:129`), `PropertyAccess` has **3** (`Simple`, `Private`,
`Super` — `.../expression/access.rs:91`). Matching those exhaustively is a
10-arm obligation over closed AST domains. Matching a *lowered read* is an
open-ended obligation over **77** `ExprIr` variants, which is why both existing
reconstructions handle a handful and give up:

| Site | Function | Shapes matched | Catch-all |
|---|---|---|---|
| `lowering.rs:32210`–`32249` | `lower_property_reference_update` | 5 of 77 | `_ =>` at **32248** |
| `lowering.rs:32848`–`32872` | `lower_update` | 2 of 77 | `_ =>` at **32871**, plus a nested `_ =>` at **32867** |

The second is not in the area brief and is the more severe of the two: it
handles only `PropertyRead` and `SpecOperation{Get|GetV}`, so `super.x++`,
`#priv++` and a global-property `++` on a shape the lowerer specialised all fall
into `unsupported_expr("property update target")`. Both die under I7.

`UnsupportedTarget` is deliberately *not* an error type that maps to
`unsupported_expr`. Each variant names a real outcome, and two of the four are
success paths that were previously reachable only by falling out of a match.

### 2.6 `strictness_of` — invariant I8, the const-assert equivalent

> **I8.** Adding a new reference-shaped `ExprIr` variant must force an explicit
> decision about whether it carries `[[Strict]]` **and which of PutValue's two
> strict throws it can reach** (widened at DISCREPANCY-FIXER stage).

```rust
/// Which `[[Strict]]` (if any) an IR node carries, as a total function of the
/// 77-variant `ExprIr`.
///
/// This match has NO catch-all. Its only purpose is to fail to compile when a
/// variant is added: whoever adds `ExprIr::TypedArrayElementWrite` must decide,
/// at that moment, whether it is a PutValue site.
pub fn carried_put_value_failure(expr: &ExprIr) -> Option<(Strictness, PutValueFailure)> {
    match expr {
        ExprIr::PropertyWrite { strictness, .. }
        | ExprIr::PropertyUpdate { strictness, .. }
        | ExprIr::PropertyCompoundAssign { strictness, .. }
        | ExprIr::SuperPropertyWrite { strictness, .. }
        | ExprIr::GlobalPropertyWrite { strictness, .. }
        | ExprIr::GlobalPropertyUpdate { strictness, .. }
        | ExprIr::GlobalPropertyCompoundAssign { strictness, .. }
        | ExprIr::DeleteProperty { strictness, .. }
        | ExprIr::DeleteGlobalProperty { strictness, .. } => Some(*strictness),

        ExprIr::Undefined | ExprIr::ArrayHole | … => None,   // all 68 others, spelled out
    }
}
```

This earns its place under the AGENTS.md test: the plausible mistake ("a new
write node ships without `[[Strict]]`") becomes **E0004** in `reference.rs`,
a file whose whole subject is that question. It must be called from at least one
product path so it is not dead code — §4.3 gives it one.

**The return type was `Option<Strictness>` and it was too narrow — corrected at
DISCREPANCY-FIXER stage.** Its product call site is `infer_expr_throw_info`
(`lowering.rs`), which merges the throw shape of every node into
`infer_catch_binding_info`'s inferred type for a `catch` binding. With a bare
`Option<Strictness>` it attributed a **TypeError** instance to every node
carrying `Strictness::Strict` — including the three global-write variants, whose
PutValue step **2.a** raises a **ReferenceError**. So

```js
"use strict"; try { undeclaredXyz = 1 } catch (e) { /* e.name, e.constructor */ }
```

narrowed `e` from the previous `Dynamic` / `all_runtime_tags` default to a
TypeError-shaped object, complete with
`prototype: standard_error_prototype_shape(TypeErrorConstructor)` — a wrong
static answer for a value that is a ReferenceError. The encoder's note calling
this "wider … a correctness fix" was right about the merge and wrong about the
error type.

The function is now total over the *pair*:

```rust
/// Which spec error a failed PutValue on this node raises, when `[[Strict]]`
/// is `Strict`. Closed; matched exhaustively at the consumer.
pub enum PutValueFailure {
    /// PutValue 3.d, or `delete` 5.e. The base is resolved, so 2.a is
    /// unreachable and a TypeError is the only outcome.
    TypeErrorOnly,
    /// PutValue 2.a **or** 3.d. `ReferenceBase::Global` covers "the global
    /// object" and "unresolvable" alike, and which one holds is a runtime fact.
    TypeErrorOrReferenceError,
}
```

`GlobalPropertyWrite` / `GlobalPropertyUpdate` / `GlobalPropertyCompoundAssign`
return the second and the consumer merges both error shapes; the other six return
the first. `DeleteGlobalProperty` is deliberately in the *first* group: `delete`
step 4.a is an **assertion** that `[[Strict]]` is false for an unresolvable
Reference (13.5.1.1 makes `delete <identifier>` an early SyntaxError in strict
code), so the ReferenceError branch cannot arise for a delete. "A new global-write
node forgot that 2.a is a ReferenceError" is now `E0004` in `reference.rs`.

### 2.7 The IR field set — invariant I9

> **I9.** Every IR node that denotes a PutValue or `delete` whose failure is
> `[[Strict]]`-observable carries a `Strictness`, and no other node does.

Add `strictness: Strictness` to these **six**:

| Variant | `ir.rs` | Consumer | Currently |
|---|---|---|---|
| `PropertyWrite` | 1392–1396 | PutValue 3.d | **absent** |
| `PropertyUpdate` | 1397–1403 | PutValue 3.d | **absent** |
| `PropertyCompoundAssign` | 1404–1409 | PutValue 3.d | **absent** |
| `SuperPropertyWrite` | 1605–1608 | PutValue 3.d, Receiver = `[[ThisValue]]` | **absent** |
| `GlobalPropertyUpdate` | 1416–1421 | PutValue 2.a + 3.d | **absent** |
| `GlobalPropertyCompoundAssign` | 1427–1431 | PutValue 2.a + 3.d | **absent** |

Convert these **three** from `strict: bool` to `strictness: Strictness`:

| Variant | `ir.rs` | Consumer |
|---|---|---|
| `GlobalPropertyWrite` | 1370–1379 | PutValue 2.a + 3.d |
| `DeleteGlobalProperty` | 1446–1451 | `delete` 5.e |
| `DeleteProperty` | 1452–1456 | `delete` 5.e |

Add to **none** of: `AssignIdentifier`, `CompoundAssignIdentifier`,
`UpdateIdentifier` (§1.5, consumer C is discharged at lowering);
`PrivateWrite` (PutValue 3.b, PrivateSet throws unconditionally);
`OptionalPropertyChain` (read-only; not a valid assignment target — 13.3.9.1
early error).

`SuperPropertyWrite` additionally gains **`this_value: Box<TypedExpr>`**. See
§3, MC4b, and §5.3: the backend currently writes to the super base, which is
the wrong object.

### 2.8 The runtime-checked ledger

These are the only places where a test remains load-bearing. Each entry states
why a type cannot carry the invariant.

| # | Invariant | Why no type carries it | Mitigation |
|---|---|---|---|
| **L1** | The `Strictness` a lowering site passes is the strictness of *the code that created this Reference*, not of some other owner. | `reference_strictness()` reads `self.current_owner_id`, ambient mutable state set at `lowering.rs:14565`, `17785`, `19793`, `20193`. Making the owner a parameter threaded through ~600 lowering methods is a different lane's refactor. | `lower_reference` is the **only** caller of `reference_strictness` for reference construction; a lane-local `grep` gate (`reference_strictness` appears exactly once outside `delete` lowering) makes drift visible. Dry-run 8.14.4-8-b_1 / _2 (§6) is the behavioural oracle. |
| **L2** | Owner-plan lookup actually succeeds for every owner the lowerer visits. | C4 makes the miss loud and recorded, but "the diagnostic list is empty" is a runtime property. | `record_missing_owner_plan_strictness` pushes an `IrDiagnostic`; assert the diagnostic count is zero over the fixture corpus. |
| **L3** | `ReferencePins::materialize` is spent on the write of *its own* Reference, not a sibling's. **Shrunk at DISCREPANCY-FIXER stage:** the *empty* chain is no longer constructible (no `Default`, no `none()`, sole producer `ReferenceRecord::pin_operands`), so what remains is only "two live records at once, chains crossed". | Distinguishing two live pairs needs a lifetime brand (`GhostToken`-style), which is more machinery than the one nesting case in the tree justifies. | `pin_operands` needs a record to be called on, and no lowering function holds two live `(record, pins)` pairs at once — checkable by reading the one call site (`lower_property_reference_update`). |
| **L4** | The backend's emitted strict guard matches the `Strictness` on the node. | The IR/Wasm boundary is `i64` words; `helper_flag_word` is the last typed point. | The b361b4815 oracle pair (§6, corpus 4/5) is precisely this test, and it is a byte-identical source pair differing only in the directive prologue. |
| **L5** | `ExprIr::PropertyWrite`'s new field is actually *read* by the backend rather than merely present. | Rust does not warn on an unread field of a public enum variant. | §4.3 stage 3 is not optional: the field and its consumer land together, and dry-run ADVERSARIAL-MC3 (§6) fails until they do. **The entry fired at ENCODER stage and is now closed for all nine variants** — see the note below this table. |
| **L6** | `SuperPropertyWrite`'s Receiver is `[[ThisValue]]`, not the super base (MC4b, §5.3). | The IR node has no `this_value` field and the backend has no receiver parameter to thread it into; S5 was deferred. | **Open for writes.** The related `delete super.x` refusal is closed independently by the fused delete-super plan below: deletion never consumes the base or receiver, so fixing it does not pretend to supply the missing write receiver. |
| **L7** | `ToPropertyKey` on a computed key runs *after* the RHS on the plain-assignment path (C5, corrected). | `PropertyKeyIr::StringExpr` conflates the key operand with the property key, so the IR cannot record that the coercion is owed; the typed fix changes a shared enum matched across the whole backend. | Open, and **pre-existing** — not a regression of this landing. Corpus entry 6's second half is the oracle. Follow-up lane with build access; see C5. |
| **L8** | The runtime strictness guard's block depth and the `Br` immediates emitted inside it. | Wasm label depths are `u32` immediates computed against a control stack the raw `If`/`Else` instructions in these guards are not on. No type distinguishes "depth relative to the guard" from "depth relative to the frame". | `RUNTIME_STRICT_GUARD_BLOCK_DEPTH` and `NON_EXTENSIBLE_THROW_EXTRA_DEPTH` are named once in `objects.rs` and added at every branch inside the guard, so the two arms of one helper cannot disagree — which is how the defect arose. The behavioural oracle is the fixture pair `wasm_reference_strictness_putvalue_{strict,sloppy}.js`; a wrong depth is a wasm validation failure or a throw caught by the wrong handler. |

### T08 suspended ordinary-property Reference amendment

`lhs = yield rhs` evaluates the LeftHandSideExpression before the yielded RHS
and performs PutValue only after a normal resume. The existing synchronous
generator implementation already follows the temporal half of that rule: its
activation stores the evaluated target payload/tag and normalized key
payload/tag before compiling `rhs`, and the resume branch reloads those words.
It nevertheless erased one of 6.2.5's fields in IR:
`GeneratorResumeModeIr::AssignProperty { target, key }` had no `[[Strict]]`.
The two resume consumers then invoked raw `emit_object_write`, bypassing the
Reference strictness guard used by ordinary `ExprIr::PropertyWrite`.

The bounded replacement is:

```rust
pub struct SuspendedPropertyReferenceIr { /* private */ }

pub enum SuspendedPropertyReferenceUse<'a> {
    Ordinary {
        base_and_receiver: &'a TypedExpr,
        key: &'a PropertyKeyIr,
        strictness: Strictness,
    },
}

pub enum GeneratorResumeModeIr {
    // ...
    AssignProperty(SuspendedPropertyReferenceIr),
}
```

The single constructor is crate-private. `base_and_receiver` is one value on
purpose: for an ordinary property Reference, 6.2.5.3 returns `[[Base]]` as the
receiver. Storing two expressions for that case would make a disagreement
representable. A future Super Reference, whose `[[Base]]` and
`[[ThisValue]]` differ, must add another use-view variant; every AOT match then
fails exhaustiveness until it persists and consumes the distinct receiver.

A single backend helper matches that view to perform both halves of the
suspension protocol. On the suspend state it evaluates base, applies
ToPropertyKey, and stores the existing activation words. On normal resume it
reloads those words and calls `emit_object_write` inside
`with_reference_strictness(strictness, ...)`. Plain `yield` and `yield*` call
the same helper rather than spelling raw slot access twice. Throw/return resume
dispatch still precedes PutValue, so an abrupt resume never writes.

No asynchronous claim is made. Async-generator property assignment is already
rejected before emission and the async activation has no corresponding four
slots. Supporting it requires an async-generator ABI/layout change and changes
to both plain-yield and delegation dispatch; it remains an explicit T08/T15
gap. Private and super assignment targets at a yield remain explicit lowering
gaps too.

### T08 delete-super Reference amendment

`delete super[key]` is a smaller lifecycle than a general Super Reference, but
it is not a bare `RuntimeThrow`. SuperProperty evaluation first obtains
`actualThis` through `GetThisBinding`; only after that succeeds does it evaluate
the computed expression and apply `GetValue`. It deliberately retains that raw
value rather than applying `ToPropertyKey`. Delete then recognizes a Super
Reference and throws `ReferenceError` before `ToObject`, `ToPropertyKey`, or
`[[Delete]]`.

That order has three observable boundaries:

1. an uninitialized derived-constructor `this` throws before the key expression;
2. an abrupt computed expression wins over delete's `ReferenceError`;
3. a normally produced object key is not coerced and no proxy delete trap runs.

The lowering contract is one private, consuming plan:

```rust
pub(crate) struct DeleteSuperReferencePlan { /* private */ }

enum DeleteSuperReferencedName {
    Static,
    Uncoerced(Box<TypedExpr>),
}
```

Its sole constructor accepts the current-this `ValueInfo` and a
`PropertyKeyIr`. It constructs `ExprIr::This` itself, so a caller cannot pass
the super base in place of `actualThis`. The conversion is exhaustive:
`StaticString`/`ArrayLength` become `Static`, while
`StringExpr`/`ArrayIndex` surrender their raw operand to `Uncoerced`. A new key
representation is therefore `E0004`, not an implicit coercion policy.

`into_reference_error(self)` destructures every private field and exhaustively
consumes the two referenced-name states into:

```text
MaterializeBinding(actualThis,
  MaterializeBinding(raw computed value,  // absent for a static name
    RuntimeThrow(ReferenceError)))
```

The sequence must not use `ExprIr::Comma`. The generic Wasm comma consumer
compiles its left operand and then its right operand without the explicit
abrupt-completion propagation performed by `MaterializeBinding`; a throwing key
could otherwise be overwritten by the terminal ReferenceError. The two bound
values are deliberately unread. Their private fixed names contain `.` and
cannot collide with source bindings, while every materialization owns a lexical
scope.

The plan does not store `[[Base]]`: `GetSuperBase` asserts an ordinary home
object and uses a non-abrupt `[[GetPrototypeOf]]`, and delete does not inspect
the resulting object/null value before throwing. It does not store `[[Strict]]`
because both strict and non-strict Super References take the same unconditional
throw. This is the C1 fused-node choice applied before durable IR; it is not a
general Super Reference and does not close L6's write receiver.

Private fields plus the sole constructor make an omitted current-this/key
argument `E0061`; the two exhaustive matches make a new key or lifecycle state
`E0004`; destructuring `Self` without `..` makes an unconsumed added field
`E0027`. Rust still permits a deliberate `let _ = plan`, so static review bans
`_`/`..` in these matches and the durable lowering fixture proves the sole
production call consumes the plan. The structural unit pins the exact
actualThis → raw key → ReferenceError tree. The Wasm fixture additionally
covers pre-`super()` ordering, a throwing key, absent key coercion, null base,
and absence of a proxy `deleteProperty` trap.

No claim is made for object-literal-method `super` deletion, whose home-object
context is not yet represented by this class-context lowering path. The pinned
`super-property-topropertykey.js` object-literal case therefore remains
unsupported. Nor is a claim made for `SuperPropertyWrite`, super
compound/update targets, suspended super assignment, the plain-assignment
`ToPropertyKey` gap in L7, or the complete T08/Test262 matrix.

**L5, closed.** At ENCODER stage the field was read for six of the nine variants:
`GlobalPropertyUpdate` and `GlobalPropertyCompoundAssign` bound `strictness: _`
with an explanatory comment, and their write-back called the *unchecked*
`emit_global_property_write`. A field constructed at three sites and read at zero
is what invariant I9 prohibits, and the honest comment did not change that. Both
arms now route their write-back through
`FunctionBuilder::emit_reference_global_property_write`, which spends the carried
`[[Strict]]` on both of PutValue's strict throws — 2.a via
`emit_global_property_write_checked`'s presence test and 3.d via the runtime
guard `with_reference_strictness` installs. The observable difference:
`"use strict"; var g = "a"; g -= 1;` on a non-writable global no longer silently
no-ops, and `"use strict"; delete globalThis.g; g++;` no longer silently creates
a property. `planning.rs` gained the matching `+ REFERENCE_STRICTNESS_FLAG_LOCALS`
budget entries at all three global-write arms.

---

## 3. The mistake-class table

| # | Mistake | Today | Compile error after this contract | Type/variant involved |
|---|---|---|---|---|
| **MC1** | Hardcode `strict: false` at a reference-write construction site. | **Measured:** 6 `ExprIr::GlobalPropertyWrite` constructions in `lowering.rs` — `30634`, `30846`, `30946`, `31099`, `32183`, `32402`. Exactly **one** (`31099`) derives it, from `let strict = self.is_current_owner_strict();` at `31087`. **Five** write the literal `strict: false` (`30638`, `30850`, `30950`, `32187`, `32406`). | **E0308 mismatched types: expected `Strictness`, found `bool`** at each of the five. | `Strictness` (no `From<bool>`, §2.1) |
| **MC2** | Pass the wrong strictness to a shared/outlined emitter. | Shipped as b361b4815: the outlined object-write helper was emitted with `strict=true`, so every read-only / non-extensible / proxy-`set`-false write threw regardless of caller mode. The mechanism was a bare positional `bool` in helper parameter 5 (`emit.rs:3526`, `objects.rs:14684`). | **E0308** at the helper boundary once the parameter is `Strictness`; and `helper_flag_word()` is the single named conversion, so a raw `i64::from(bool)` at the `I64Const` site no longer type-checks against a `Strictness`. **Not built at ENCODER stage; built at DISCREPANCY-FIXER stage** — see below. | `Strictness::helper_flag_word` |
| **MC3** | Write a reference-shaped IR node with nowhere to record `[[Strict]]`. | **LIVE, VERIFIED.** `ExprIr::PropertyWrite { target, key, value }` (`ir.rs:1392`–`1396`) has no strict field. The backend arm (`expressions.rs:344`) calls `compile_property_write_payload(target, key, value, function)` (`objects.rs:5770`) — no strictness parameter — and the guard deep inside reads the **ambient** `object_write_strict_flag_local` / `is_current_function_strict()` (`objects.rs:14684`, `environments.rs:754`). `"use strict"; const o = Object.freeze({x:1}); o.x = 2;` has no IR that can express the required TypeError as a property of *the reference*. Same for `PropertyUpdate`, `PropertyCompoundAssign`, `SuperPropertyWrite`, `GlobalPropertyUpdate`, `GlobalPropertyCompoundAssign`. | **E0063 missing field `strictness` in initializer** at each of the counted construction sites (§4.2), and **E0027 pattern does not mention field `strictness`** at each backend arm that binds all fields (§4.3). | the six `ExprIr` variants of §2.7 |
| **MC3′** | Fix MC3 by hardcoding `strictness: Strictness::Strict` on `PropertyWrite` — b361b4815 repeated one layer up. | n/a | Not a compile error. **This is the one mistake the types do not catch**, and it is why the sloppy control (§6 corpus 12) is mandatory and why L1 exists. | ledger L1/L4 |
| **MC4a** | Reconstruct a Reference by pattern-matching a lowered read and fall into `_`, downgrading a legal target to `unsupported_expr`. | Two sites, not one: `lowering.rs:32248` (5 of 77 shapes) and `lowering.rs:32871` + `32867` (2 of 77). Any new read specialisation added anywhere in the 38,003-line lowering silently removes compound assignment or `++` for that shape, with no compile error. | Both reconstructions are deleted. `lower_reference` matches `AssignTarget` (4) / `UpdateTarget` (3) / `PropertyAccess` (3) exhaustively; a new AST shape is **E0004**, and a new `ExprIr` read shape is *irrelevant* because the record is built from the AST. | `ReferenceTarget`, `UnsupportedTarget` |
| **MC4b** | Drop `[[ThisValue]]` from a Super Reference. | `PropertyReference::Super { key }` (`lowering.rs:80`–`82`) carries no this-value, and so does `ExprIr::SuperPropertyWrite { key, value }` (`ir.rs:1605`). The backend (`expressions.rs:1445`–`1470`) calls `emit_object_write(super_base_local, …)` — it writes **to the super base**, i.e. the home object's prototype, not to `this`. PutValue 3.c requires `GetThisValue(V)` as the Receiver. The lowerer's own comment at `lowering.rs:32385`–`32389` says the write goes to `this`; the backend disagrees. | `ReferenceBase::Super` cannot be constructed without a `SuperThisValue`, whose only constructor is `from_class_context` (§2.2) → **E0063** at construction, **E0027** at every backend arm that binds `SuperPropertyWrite`'s fields. | `SuperThisValue`, `ReferenceBase::Super` |
| **MC5** | Evaluate the Reference twice, re-running an effectful base or computed key. | `PropertyReference::read_ir` (`lowering.rs:89`–`105`) and `build_property_reference_write` (`lowering.rs:32357`) are separately callable; the pinning at `32251`–`32279` and the `MaterializeBinding` wrap at `32328`–`32338` are joined only by convention. | Second write → **E0382 use of moved value: `record`** (`write(self, …)`, and `ReferenceRecord` is not `Clone`). Forgotten pin discharge → **E0308** (a `PendingReferenceWrite` where a `TypedExpr` is wanted). Double discharge → **E0382** on `ReferencePins`. | `ReferenceRecord::write`, `ReferencePins`, `PendingReferenceWrite` |
| **MC6** *(new)* | Add a new reference-shaped `ExprIr` variant and forget `[[Strict]]` entirely. | Not covered by MC1–MC5: a brand-new variant has no construction site to break. | **E0004 non-exhaustive patterns** in `carried_put_value_failure` (§2.6). | `carried_put_value_failure` |
| **MC7** *(new, DISCREPANCY-FIXER)* | Add a new global-write variant and forget that PutValue **2.a** is a **ReferenceError**, not a TypeError. | The `Option<Strictness>` return of `carried_strictness` could not express the distinction; every strict write contributed a TypeError shape to the enclosing `catch` binding's inferred type. | **E0004** in `carried_put_value_failure`, whose arms return a `PutValueFailure` the consumer matches exhaustively (§2.6). | `PutValueFailure` |

**MC2, actually discharged (DISCREPANCY-FIXER stage).** At ENCODER stage the
compile error MC2 names had not been built: `emit_global_property_write_checked`
(`environments.rs`), `emit_global_property_delete` (`environments.rs`) and
`compile_delete_property_i32` (`objects.rs`) all still took `strict: bool`, and
the call sites converted with `.throws_on_failed_set()`. Nothing stopped

```rust
self.emit_global_property_write_checked(name, p, t, self.is_current_function_strict(), function)
```

from compiling at either of its call sites — b361b4815 verbatim, one layer out.
All three parameters are now `strictness: Strictness` with the conversion moved
inside. The second half of the claim — "`helper_flag_word` is the single named
conversion to a machine word" — was also false while
`i64::from(self.is_current_function_strict())` sat at the `I64Const` site in
`emit_object_write`. That expression is now
`FunctionBuilder::ambient_object_write_strict_flag_word()`, named for what it is:
the *ambient* mode, correct only for writes the spec does not route through a
Reference Record (property installation, class field definition, internal helper
writes), and visibly not `Strictness::helper_flag_word`.

---

## 4. The retrofit map

### 4.1 Order

The stages are ordered so that each one ends at a `cargo check`-clean tree, and
so that the compile errors that arrive are attributable.

```
S0  reference.rs: Strictness, ReferenceBase, ReferenceRecord, SuperThisValue,
    ReferencePins, PendingReferenceWrite, Composition, ReferenceTarget,
    UnsupportedTarget, carried_strictness.           new file; compiles alone
S1  reference_strictness() replaces is_current_owner_strict();
    the 3 existing bool fields become Strictness.     ~9 sites, mechanical
S2  the 6 new strictness fields on ExprIr.            E0063/E0027 storm; §4.2/§4.3
S3  backend consumption of PropertyWrite's field.     the scope extension, §4.5
S4  lower_reference replaces both reconstructions.    deletes 2 catch-alls
S5  SuperPropertyWrite gains this_value.              MC4b
```

S5 is separable and may be deferred to a follow-up lane; S1–S4 are one landing,
because S2 without S3 is ledger item L5's decoration.

### 4.2 Construction sites in `lila-ir` — E0063

`ExprIr::` occurrences of the nine assignment-shaped variants, counted at
`84e782506`:

| File | Sites | With `..` (unaffected) | Without `..` |
|---|---|---|---|
| `crates/lila-ir/src/lowering.rs` | 36 | 9 | **27** (26 constructions → E0063; 1 pattern at `12332` → E0027) |
| `crates/lila-ir/src/ir.rs` | 9 | 7 | **2** (both patterns, in the AST-stat visitor: `2957`, `3294`) |
| `crates/lila-ir/src/early_errors.rs` | 9 | 9 | **0** |
| `crates/lila-ir/src/lib.rs` | 10 | 10 | **0** |

The 26 `lowering.rs` construction sites, by line:
`9799`, `13126`, `16538`, `16605`, `16635`, `18113`, `30502`, `30625`, `30749`,
`30755`, `30837`, `30937`, `31075`, `31118`, `32173`, `32370`, `32395`, `32511`,
`32634`, `32665`, `32702`, `32764`, `32819`, `32875`, `32964`, `32971`.

Under §5.1's deviation, the sites constructing `AssignIdentifier`,
`CompoundAssignIdentifier` and `UpdateIdentifier` — `9799`, `13126`, `16538`,
`16605`, `16635`, `18113`, `30625`, `30749`, `30837`, `30937`, `31075`, `32173`,
`32964` (13 of the 26) — **do not change**, which is the practical payoff of
proving consumer C static. The remaining **13** gain a `strictness:` field.

Two facts worth recording because they are cheap to assume and wrong:

- **`early_errors.rs` needs no edit.** All 9 of its references to these variants
  are in the exhaustive `expr_contains_this_before_super` match and every one
  uses a `..` rest pattern (`147`, `152`, `161`, `254`, `256`–`259`, `262`).
  The area brief lists it as owned; it is owned, and the correct action in it is
  *nothing*. (§5.4 notes an unrelated pre-existing defect found there.)
- **`lib.rs` needs no edit**, and is not in the brief's `files_owned`. All 262
  of its `ExprIr::` references sit inside `#[cfg(test)]` (module opens at
  `lib.rs:142`), and all 10 that touch these variants use `..`. Verified by
  reading each: `1745`, `2036`, `2283`, `4006`, `4501`, `4535`, `4741`, `5676`,
  `8148`, `8497`, `9143`, `9513`. Had any bound all fields, the lane would have
  needed a file it does not own.

### 4.3 Backend match arms — E0027

All `ExprIr` occurrences in `lila-aot-wasm` are **patterns**; the backend
never constructs IR. Counted over the same nine variants:

| File | Arms | With `..` | Without `..` → **E0027** |
|---|---|---|---|
| `crates/lila-aot-wasm/src/planning.rs` | 59 | 48 | **11** — `2834`, `2903`, `3241`, `3289`, `3466`, `4747`, `4846`, `6612`, `6812`, `7476`, `7974` |
| `crates/lila-aot-wasm/src/expressions.rs` | 19 | 5 | **14** — `292`, `344`, `347`, `366`, `383`, `458`, `565`, `633`, `1445`, `2654`, `2717`, `2727`, `2745`, `2862` |
| `crates/lila-aot-wasm/src/data.rs` | 9 | 6 | **3** — `2936`, `2955`, `3337` |

Under §5.1, the arms binding only `AssignIdentifier` /
`CompoundAssignIdentifier` / `UpdateIdentifier` fields (`planning.rs:3241`;
`expressions.rs:292`, `383`, `458`, `2654`; `data.rs:2955`) do not change.
The rest — **8** in `planning.rs`, **9** in `expressions.rs`, **2** in `data.rs`
— gain `strictness` or `..`.

`planning.rs` and `data.rs` arms are *analysis* passes (temp-local counting,
string interning, function-table use). They should take `..`, not the field.
`expressions.rs` arms are the emitters and must take the field, because they are
where S3 lands. `carried_strictness` (§2.6, invariant I8) gets its product call
site here: `planning.rs`'s throw-analysis pass uses it to decide whether a node
can throw a strict-mode TypeError, which is a real question it already asks
approximately.

### 4.4 What stays untouched, and why it is stated

Stating the boundary is what makes it auditable later.

| Not touched | Reason |
|---|---|
| `tdz_scopes`, `mark_tdz_binding`, `clear_tdz_binding`, `is_tdz_binding`, `is_tdz_binding_storage_name`, the `$tdz.` sentinel, `BindingInfo.storage_name` classification | 9.1.1.1 Environment Record binding lifecycle is a separate deferred area. GetValue 4.c and SetMutableBinding step 4 are TDZ paths and throw mode-independently (§1.3, §1.5), so this lane owes them nothing. This holds **even inside functions this lane rewrites**, notably `lower_assign` (`lowering.rs:30400`). |
| Object Environment Records and `with` — `active_with_objects`, `lower_with_scoped_identifier_write` (`lowering.rs:31109`) | 9.1.1.2 is its own lifecycle; `lower_assign` returns early at `30423`–`30425` before any Reference is built. |
| GetValue's `ToObject`-on-primitive-base path | runtime concern; PutValue 3.a likewise. |
| The Proxy `[[Set]]` trap result | runtime. This contract fixes *whether a `false` result is observable*, not what produces it. |
| Whether a write **succeeds** | out of scope by construction; only observability of failure is in scope. |
| `ExprIr::OptionalPropertyChain` (`ir.rs:1388`) | read-only, never a PutValue target. |
| `crates/lila-ir/src/lowering_helpers.rs` | **listed in the brief's `files_owned`, but it contains zero `ExprIr::` references and its only `Reference` hit is `PropertyDefinition::IdentifierReference` at line 1911, an unrelated AST shape.** No edit. |
| `crates/lila-aot-wasm/src/objects.rs` — *as a match site* | its only reference to these variants is a doc comment at `objects.rs:6094`. The brief is right about that. It is **not** right that objects.rs needs no edit at all; see §4.5. |
| `crates/lila-aot-wasm/src/builtins/{intl_datetimeformat,temporal*,emitted_function,runtime_helpers}.rs` | batch 2 concurrency hold. None contains a reference to these variants. |

### 4.5 The scope extension S3 requires, stated so it can be refused

**This contract cannot make MC3's fix load-bearing inside the brief's
`files_owned` list.** The evidence:

- `ExprIr::PropertyWrite`'s emitter arm is `expressions.rs:344` (owned), which
  calls `compile_property_write_payload` at **`objects.rs:5770`** (not owned),
  which calls `compile_property_write_to_locals` at **`objects.rs:6316`** (not
  owned), which reaches `emit_object_write` at **`objects.rs:6360`** and
  **`objects.rs:6428`**.
- The strictness those two calls actually use comes from
  `self.object_write_strict_flag_local` — a `Option<u32>` **runtime local index**
  on the emitter (`emit.rs:302`), set to `Some(5)` inside the outlined helper
  (`emit.rs:3526`) and otherwise `None`, in which case the guard falls back to
  the compile-time ambient `self.is_current_function_strict()`
  (`environments.rs:754`, read at `objects.rs:14684`, `14746`, `14788`, `14831`,
  `15506`).

So a `strictness` field on `PropertyWrite` reaches nothing unless the parameter
crosses into `objects.rs`. The **minimal, bounded** extension is:

| File | Edit | Size |
|---|---|---|
| `objects.rs` | `compile_property_write_payload` (5770) and `compile_property_write_to_locals` (6316) each take one added `strictness: Strictness` parameter; the two `emit_object_write` calls at 6360 and 6428 are wrapped in the existing scoped-override idiom so the guard sees the record's value rather than `is_current_function_strict()` | 2 signatures, 2 call sites, 1 scope wrapper |
| `emit.rs` | one compile-time companion to `object_write_strict_flag_local`, e.g. `reference_strictness_override: Option<Strictness>`, consulted by `objects.rs:14684`'s `None` arm before falling back to the ambient | 1 field, 1 initialiser, 1 read |
| `control_flow.rs` | `8957` is the third caller of `compile_property_write_to_locals` (a destructuring property write, which *is* a Reference write) and must pass its own `Strictness` | 1 call site |

That is **3 files, 5 signatures/fields, 4 call sites** — measured, not estimated.
Neither `objects.rs`, `emit.rs` nor `control_flow.rs` is on batch 2's hold list.
If the campaign declines the extension, S2 must be reduced to the five variants
whose emitters live in owned files and `PropertyWrite` must be dropped from
§2.7 — which abandons MC3, the only conformance gap in this area that spans
whole test262 families. **State the choice; do not let it be decided by which
file an encoder happened to open.**

### 4.5.1 The scoped override was chosen, and it moved a latent defect onto the product path

The encoder took neither branch: it generalised the existing scoped-override
idiom as `FunctionBuilder::with_reference_strictness` inside the *owned*
`expressions.rs`, so the parameter never has to cross into `objects.rs`. That is
correct and cheaper than §4.5's parameter threading — but it has a consequence
§4.5 could not have named, and the dry run caught it.

Before the landing, `object_write_strict_flag_local` was `Some(_)` only inside
emitted helper/builtin bodies (`emit.rs:3526`, `emit.rs:3659`,
`objects.rs:6251`, `objects.rs:14880`), where `is_main()` is false and
`emit_throw_runtime_error_to_active_handler`'s `extra_depth` is **dead code** —
`builtins/errors.rs:674-680` only reaches `emit_branch_to_target` under
`is_main() && active_throw_target().is_some()`. `with_reference_strictness` makes
the `Some` arm of those guards live in `main`, for every property write.

That arm opens one extra `If(BlockType::Empty)` the `None` arm does not, and
`emit_branch_to_target` adds `extra_depth` straight to a Wasm `Br` immediate
(`control_flow.rs:811`). Two sites were wrong in opposite directions:

- `emit_object_write_set_failure_else` forwarded the caller's `extra_depth`
  unchanged into a guard it had just opened;
- `emit_object_write_non_extensible_failure` compensated its sloppy
  abandon-branch with `Br(sloppy_br_depth + 1)` while leaving its *throw's*
  `extra_depth` at the inline value `5` in both arms. The two cannot both be
  right.

Symptom: `"use strict"; try { a.length = 0 } catch (e) {}` in **top-level script
code** with a non-writable `length` branches one label too shallow — the wrong
handler, or a module that fails validation. Nothing inside a function body shows
it.

Fixed by naming the quantity once: `RUNTIME_STRICT_GUARD_BLOCK_DEPTH` (and
`NON_EXTENSIBLE_THROW_EXTRA_DEPTH` for the bare `5` written twice) in
`objects.rs`, added at **every** branch emitted inside the guard. Ledger **L8**.
The behavioural oracle is a new fixture pair,
`crates/lila-cli/tests/fixtures/wasm_reference_strictness_putvalue_{strict,sloppy}.js`,
whose failing writes sit inside a **top-level** `try` — the shape that makes the
`extra_depth` live — with tests in `crates/lila-cli/tests/cli/language.rs`.
This is the one item in this area that can produce invalid or mis-branching Wasm,
so it is the first thing to verify with an actual build.

---

## 5. Deviations from the area brief, with evidence

The brief is a good survey; these five points are where following it verbatim
would produce a defect or a decoration.

### 5.1 Nine variants → six. Three of the brief's nine must NOT get the field.

The brief's `contract_scope` (e) lists `AssignIdentifier`,
`CompoundAssignIdentifier` and `UpdateIdentifier` among the variants "verified
missing" a strictness field. They are missing it, and they should stay missing
it: §1.5 shows PutValue consumer C is decided at lowering time, and the lowerer
already decides it (`lowering.rs:31004`–`31051`). A field on those three would be
constructed at 13 sites and read by zero backend arms. AGENTS.md: *"If it does
not [become a compile error], the type is decoration and a plain function is
better."* Adding it would also be actively misleading — a future reader would
assume the backend honours it.

Conversely the brief omits `DeleteProperty` and `DeleteGlobalProperty` from the
`Strictness` conversion even though it names `delete` as consumer D and those
two already carry `strict: bool` (`ir.rs:1450`, `1455`). They are included here.

### 5.2 One catch-all → two. `lower_update` is the worse one.

The brief names `lowering.rs:32248`. `lower_update` (`lowering.rs:32828`) has an
independent reconstruction at `32848`–`32872` with its own `_ =>` at **32871**
and a nested `_ =>` at **32867**, matching **2** of 77 shapes against the other's
5. Deleting only the first leaves `super.x++` and `#priv++` unreachable with no
compile error to say so.

### 5.3 `SuperPropertyWrite` does not merely *lack* `[[ThisValue]]` — it writes to the wrong object.

`expressions.rs:1445`–`1470` emits `emit_object_write(super_base_local,
super_base_tag_local, key_local, …)`. `super_base_local` is loaded by
`emit_load_super_base` (`functions.rs:7460`), which walks the home object's
`[[Prototype]]`. PutValue 3.c requires the Receiver to be `GetThisValue(V)` =
`[[ThisValue]]`. The machinery for a split target/receiver already exists —
`emit_ordinary_set_result_with_receiver_fallback` (`emit.rs:3682`) takes both —
so this is a wiring defect, not a missing capability. The lowerer's comment at
`lowering.rs:32385`–`32389` asserts the write goes to `this`, which is what makes
it a genuine bug rather than a design choice: two parts of the tree hold
contradictory beliefs.

### 5.4 Two corpus entries do not test what the brief says they test.

Corrected in §6, but flagged here because they change what the encoder should
expect:

- `11.13.1-4-27-s.js` and `11.13.1-4-3-s.js` do **not** reach any
  `GlobalPropertyWrite` site. Both do `var global = this;` then
  `global.undefined = 42`. `is_global_this_expr` (`lowering.rs:29243`) matches
  **only** the literal identifier `globalThis` (`GLOBAL_THIS_NAME`,
  `names.rs:53`), so `global.undefined` lowers to `ExprIr::PropertyWrite` with
  target `Identifier("global")`. They are **MC3** oracles, not MC1 oracles.
- `11.13.2-6-s.js` wraps its subject in `eval("…")`. AGENTS.md classifies `eval`
  as an explicit Wasm-AOT unsupported dynamic-code-generation case, so this test
  cannot exercise the `strict: false` literal at `lowering.rs:32187` on the
  product path at all. **The ADVERSARIAL-MC1 trace (§6, corpus 13) is the only
  reachability proof for that literal.** The brief's instruction to "trace
  `lower_identifier_arithmetic_general` into its global arm" is right; the test
  it names cannot do it.

### 5.5 `lowering_helpers.rs` and `lib.rs` are misfiled in opposite directions.

`lowering_helpers.rs` is in `files_owned` and needs no edit (0 `ExprIr::`
references). `lib.rs` is not in `files_owned` and has 262 — all in `#[cfg(test)]`
and all using `..`, so it happens to need no edit either, but the lane should
know it is one binding-pattern away from needing a file it does not own.

---

## 6. Dry-run corpus, with corrected traces

Every path below was confirmed to exist under
`/home/user/lila/test262/vendor/test262/`.

| # | Case | Class | Trace, corrected |
|---|---|---|---|
| 1 | `expressions/assignment/11.13.1-4-27-s.js` | **MC3** (brief said MC1) | `var global = this; global.undefined = 42` → `PropertyWrite{target: Identifier("global")}` via `lower_property_assign` (`32519`) → construction at `32634`. Strict → TypeError. §5.4. |
| 2 | `expressions/assignment/11.13.1-4-3-s.js` | **MC3** (brief said MC1) | Same shape, `global.Infinity`. Second non-writable global; checks a `KindSet` proof does not reroute it. |
| 3 | `expressions/compound-assignment/11.13.2-6-s.js` | **inert** on the product path | `eval("_11_13_2_6 <<= 1")`. §5.4. Keep it in the corpus as a spec-exec-backend oracle only. |
| 4 | `expressions/assignment/8.14.4-8-b_1.js` | **MC2** sloppy half | `flags:[noStrict]`, non-writable inherited property; must silently no-op, `o.hasOwnProperty('bar') === false`. |
| 5 | `expressions/assignment/8.14.4-8-b_2.js` | **MC2** strict half | Byte-identical body, `flags:[onlyStrict]`; must throw TypeError. **The b361b4815 oracle**: one emitted write helper, two callers, opposite required outcomes. The dry run must show the outcome is a function of a `Strictness` carried from the caller, not of a constant in the helper. Ledger L1/L4. |
| 6 | `expressions/assignment/target-member-computed-reference.js` | **MC5 + O2** | Two halves: `base[prop()] = expr()` must throw `DummyError` from `prop()` (LHS before RHS); `base[objWithThrowingToString] = expr()` must throw `DummyError` from `expr()` (`ToPropertyKey` after both). Choice C5. Traces `lower_property_reference_update`'s `Get`/`GetV` arm (`32225`–`32240`) and the pins at `32251`–`32279`. |
| 7 | `expressions/delete/super-property.js` | **MC4a + MC4b boundary** | `delete super.x` must throw ReferenceError (13.5.1.2 step 5.b). The fused delete-super plan now evaluates `actualThis`, evaluates a raw computed key when present, and throws without coercion or deletion. This closes the former `lower_delete` refusal without claiming MC4b's still-open `SuperPropertyWrite` receiver. |
| 8 | `expressions/assignment/non-simple-target.js` | **negative control, parser-level** | `1 = 1`, `negative: {phase: parse, type: SyntaxError}`. Boa's `AssignTarget::from_expression` (`boa_ast .../assign/mod.rs:141`) returns `None`, so this **never reaches `lower_reference`**. It controls that the *parser* still rejects, not that `lower_reference` does. State this, or the dry-runner will look for a lowering arm that cannot exist. |
| 9 | `expressions/assignment/assignment-operator-calls-putvalue-lref--rval-.js` | **T08 Object Environment follow-up** | The case's subject is a `with` scope: 9.1.1.2.5 `ObjectEnvironmentRecord.SetMutableBinding` re-checks `HasProperty` at *write* time, reached via PutValue 4.c, after the RHS deletes the binding. The T08 follow-up below replaces the old pre-RHS `Conditional { then: PropertyWrite }` shortcut with a consuming `WithEnvironmentReferencePlan`: initial `HasBinding`/unscopables resolution selects one materialized binding object, RHS evaluation happens once in that selected branch, and `SetMutableBinding` re-checks `HasProperty` on the same object before strict ReferenceError or checked Set. Corpus 14 remains the canonical ordinary-property single-record trace for I5. |
| 10 | `expressions/assignment/11.13.1-1-s.js` | **MC3** | `Object.defineProperty(obj,"prop",{writable:false})`, then `obj.prop = 20` → TypeError, `obj.prop === 10`. A **resolvable, non-global** property reference: the cleanest MC3 oracle, uncontaminated by §1.3's both-modes GetValue throw. |
| 11 | ADVERSARIAL MC3 (strict): `"use strict"; const o = Object.freeze({x:1}); o.x = 2;` | **MC3 acceptance** | Must go from *"no IR node can express the throw"* to *"the throw is emitted"*. This is the S2+S3 acceptance criterion and the reason §4.5's extension is not optional. |
| 12 | ADVERSARIAL MC3 (sloppy control): same source minus the directive; no throw, `o.x === 1` | **MC3′ guard** | Guards against fixing MC3 by hardcoding `Strictness::Strict`. Mandatory, per MC3′ and ledger L1. |
| 13 | ADVERSARIAL MC1 (reachability), **corrected at DISCREPANCY-FIXER stage**: `"use strict"; var g = "a"; Object.defineProperty(globalThis,'g',{writable:false}); g -= 1;` | **MC1 reachability proof** | The entry as first written (`Object.defineProperty(globalThis,'g',{value:1,…}); g += 1;` with no prior `var g`) **does not reach the arm it names.** `Object.defineProperty` registers nothing in `self.global_properties`, so `global_property_is_proven_present` (`lowering.rs:16977`) is false; with no binding either, `needs_general_form` (`lowering.rs:30664`) is false and the source falls into the `unsupported … unbound identifier 'g'` arm at `lowering.rs:30752`. `lower_identifier_arithmetic_general` is never entered. The replacement satisfies the actual reachability condition — `proven_present == true` via the `var`, and a recorded kind (`String`) that does not match the specialised Number fast path — so it takes the `needs_general_form` branch at `lowering.rs:30689`, reaches `lowering.rs:32211`, and carries `Strictness::Strict` where the tree at `84e782506` wrote `strict: false`. **That is the real MC1 behavioural delta of this landing, and it had no covering trace.** Must throw TypeError; `g` must still be `"a"`. |
| 14 | ADVERSARIAL MC5: `let n = 0; const a = [{v:1}]; const idx = () => { n++; return 0; }; a[idx()].v += 1;` → `n === 1`, `a[0].v === 2` | **MC5 acceptance** | Trace that the `ReferenceRecord` owns the pin for `a[idx()]` and that `write` consuming the record makes a second emission of `idx()` an **E0382**, not a review comment. |
| 15 | ADVERSARIAL MC1-unresolvable: `"use strict"; undeclaredXyz = 1;` → ReferenceError | **MC1, PutValue 2.a** | Folds in open ledger item R7. Already handled at `lowering.rs:31099`; the trace confirms S1 does not regress the one site that was correct. |

---

## 7. Acceptance criteria

The encoder's work is done when all of the following hold. Each is checkable
without running the conformance suite except where noted.

1. `crates/lila-ir/src/reference.rs` exists and defines exactly the items of
   §2, with no `_` arm over `ReferenceBase`, `Composition`, `UnsupportedTarget`
   or `ExprIr` (in `carried_strictness`).
2. `grep -rn "is_current_owner_strict" crates/` returns **0**.
3. `grep -rn "strict: bool" crates/lila-ir/src/ir.rs` returns **0**.
4. `grep -rn "PropertyReference" crates/` returns **0** — the enum at
   `lowering.rs:71`, its `read_ir` at `88`, and `build_property_reference_write`
   at `32357` are all deleted, not wrapped.
5. Neither `lowering.rs:32248` nor `lowering.rs:32871`/`32867` exists in any
   form: `lower_property_reference_update` and `lower_update`'s property branch
   both obtain their record from `lower_reference`.
6. `carried_put_value_failure` has at least one non-test call site (§4.3).
7. The six variants of §2.7 carry `strictness: Strictness`; `AssignIdentifier`,
   `CompoundAssignIdentifier`, `UpdateIdentifier`, `PrivateWrite` and
   `OptionalPropertyChain` do **not**.
8. `ReferenceRecord`, `ReferencePins` and `PendingReferenceWrite` derive neither
   `Clone` nor `Copy`, and `PendingReferenceWrite` has no public field, no
   `Deref` and no `Into<TypedExpr>`.
9. `cargo check -p lila-ir && cargo check -p lila-aot-wasm` is clean
   (rung 0; 1–5 s and 15–40 s per `batch-workflow.md`).
10. **Behavioural, and therefore last and elsewhere:** corpus 11 throws, corpus
    12 does not, corpus 13 (as corrected) throws, corpus 14 reports `n === 1`.
    These four are the tests ledger entries L1, L4 and L5 leave load-bearing;
    nothing in the type system can replace them.
11. **Added at DISCREPANCY-FIXER stage, and the highest priority of the ten:**
    `cargo test -p lila-cli --test cli language::` passes the new fixture
    pair `wasm_reference_strictness_putvalue_{strict,sloppy}.js`. They put a
    failing strict property write inside a **top-level** `try`, which is the only
    shape that exercises the runtime strictness guard's `Br` immediate (§4.5.1,
    ledger L8) — the one item in this area that can emit invalid Wasm.
12. `grep -rn "base_mut\|ReferencePins::none\|derive(Debug, Default)] *$" crates/lila-ir/src/reference.rs`
    returns **0**: the empty pin chain and the whole-base mutable accessor are
    both gone (§2.2, §2.4).
13. `grep -rn "strict: bool" crates/lila-aot-wasm/src/{environments,objects}.rs`
    returns **0** for the three Reference-consuming emitters of MC2.

Rung G (`emit_golden` + `diff -r`) is **not** applicable: this is feature work
under `batch-workflow.md`'s rule, and the emitted bytes are supposed to change
for every strict-mode property write in the fixture corpus.

---

## 8. T08 Object Environment Record `SetMutableBinding` follow-up

### 8.1 The defect

The former `lower_with_scoped_identifier_write` evaluated an initial
`HasBinding`/`Symbol.unscopables` condition and put the RHS and a direct
`PropertyWrite` in its true branch. That preserved the first half of assignment
order, but it silently treated the initial `HasBinding` result as authority to
write later. `ObjectEnvironmentRecord.SetMutableBinding` instead calls
`HasProperty(bindingObject, N)` after the RHS has run. If that second query says
the property is absent, strict code throws `ReferenceError`; sloppy code still
performs the observable query and then calls checked `Set`. Either query may be
a Proxy trap and may complete abruptly.

Nested `with` exposed a second instance of the same shortcut. A lowering-local
stack cannot represent the ordered Environment Record chain: a declarative
record introduced inside `with` must stop lookup before the outer Object
Environment Record, while an Object Environment Record introduced inside that
declarative record must still be queried first. A fresh function lowerer also
cannot infer the Object Environment Records surrounding the function's
definition. ResolveBinding therefore needs the analyzed environment cursor
chain, not a flat list of objects owned by one lowerer.

### 8.2 Closed representation

Analysis registers `EnvironmentKind::WithObject` only after scanning the object
expression, then pushes that cursor while scanning the body. Its private
`WithObjectBindingName` is derived from the stable `EnvironmentId` and contains
`.` so source text cannot spell it. Every source function defined under that
cursor unconditionally captures each surrounding with-object binding. Existing
owned slots, capture hops, lexical-environment materialization and closure
capture then carry the Object Environment Records into a fresh nested lowerer;
there is no new closure ABI or backend operation.

`ObjectEnvironmentBindingObject` is the only representation admitted to the
ordered lowering chain. Its materialized-with source contains the storage name
and type information of the already-materialized synthetic binding. Cloning it
can only create another identifier read of that binding; it cannot re-evaluate
the source object expression or substitute an arbitrary effectful `TypedExpr`.
Its distinct compiler-owned global-object source is used by the global Object
Environment Reference plan and cannot enter the ordered `with` chain through a
source expression.

The ordered chain uses four closed position types: current Object entry depth,
current declarative binding depth, captured Object cursor depth and captured
declarative binding cursor depth. Current and captured depths are distinct
newtypes. One exhaustive cross-product decides whether an Object Environment
Record precedes the already-located declarative fallback. Thus a new position
class is E0004, and passing a current depth where a captured depth is required
is E0308. The lexical/global fallback is located once and carried into PutValue;
selection and emission cannot silently resolve different bindings.

`WithEnvironmentResolution` binds that object to the one private temporary used
while evaluating its initial `Symbol.unscopables` query. A non-empty
`WithEnvironmentReferencePlan` owns one innermost resolution and zero or more
outer resolutions, the referenced name and `Strictness`. The plan is neither
`Clone` nor `Copy`, and its `put_value` exit consumes it. Thus an empty
environment chain is not constructible, a second write is E0382, a `bool`
cannot stand in for `[[Strict]]`, and a new strictness state makes its
exhaustive PutValue match E0004. Section 9 adds the distinct consuming
`get_value` exit without reopening construction.

The consuming exit builds one fixed tree:

1. only Object Environment Records which precede the located declarative or
   global fallback survive selection, in exact inner-to-outer order;
2. initial binding visibility is tested from innermost to outermost;
3. only the selected branch evaluates and materializes the RHS;
4. `HasProperty` is re-run on that exact selected binding object;
5. strict absence throws `ReferenceError`, while presence reaches a checked
   strict Set;
6. sloppy mode materializes the recheck so its abrupt completion is propagated,
   then reaches a checked sloppy Set regardless of the Boolean result;
7. lexical/global fallback is reachable only after every initial resolution
   misses.

The with expression is evaluated into an outer temporary before the compiler
enters the `WithObject` lexical environment and initializes its hidden binding.
Otherwise a closure created by the object expression would capture an
Environment Record which does not exist yet in ECMA-262.

The RHS and sloppy recheck use `MaterializeBinding`, not `Comma`, because the
former is the existing IR boundary that propagates abrupt completion before
entering its body. Fixed temporary names contain `.` and therefore cannot
collide with source bindings.

### 8.3 Boundaries and regressions

The structural contract checks strict and sloppy tree shapes, same-object
initial/recheck/write identity, current/captured declarative interleaving,
production hidden captures, RHS placement and the absence of Set on a strict
missing binding. The Wasm fixture makes strict closure capture load-bearing
with a predeclared global sentinel, invokes an escaping closure after leaving
`with`, checks repeated entries retain distinct objects, and adds declarative
shadowing to the observable Proxy order, abrupt-recheck, sloppy-recreation,
abrupt-RHS, unscopables and nested-object cases.

This follow-up covers only plain identifier `=` through `with` in scripts and
ordinary source functions. Direct identifier GetValue is the separate lifecycle
in §9. Compound, logical, update and destructuring writes, Global Object
Environment Records, generated class/helper execution contexts, the
super-write `[[ThisValue]]` gap and deferred computed-key coercion remain
explicit debt. A resumable owner which would capture a `WithObject` environment
is rejected explicitly rather than pretending the existing suspension
activation can re-enter it. The change adds no backend operation, IR variant or
closure ABI and makes no status-count or full-T08 claim.

---

## 9. T08 Object Environment Record `GetBindingValue` follow-up

### 9.1 The defect

The read path did not share §8's ordered Environment Record selection. It asked
`OrderedWithEnvironmentChain::innermost_binding_object`, tested that one
object's `HasBinding`, and sent a miss directly to the non-`with` fallback.
That has three observable wrong answers from one lifecycle shortcut:

1. a declarative binding introduced inside a `with` body did not cut off the
   outer Object Environment Record;
2. an inner Object Environment miss never continued to an outer Object
   Environment Record; and
3. the selected object was read with a bare property Get, omitting
   `ObjectEnvironmentRecord.GetBindingValue`'s second `HasProperty` query.

The third point is not redundant with `HasBinding`. The `@@unscopables` getter
can delete the property between the two operations, and a Proxy makes both
queries independently observable or abrupt. `GetBindingValue(N, S)` returns
`undefined` after a missing recheck when `S` is false and throws
`ReferenceError` when `S` is true. The strict case is reachable through an
ordinary strict function created inside sloppy `with` code and carrying the
Object Environment Record through the existing capture chain.

### 9.2 Closed selection and consuming GetValue

`OrderedWithEnvironmentChain::select_preceding` is the only read/write
selection exit. It takes the already-located declarative fallback and returns
`Option<SelectedWithEnvironmentObjects>`. `None` means no Object Environment
Record precedes that fallback. The selected form has a required `innermost`
field and an `outer` vector, so an empty chain is not representable. The old
`innermost_binding_object` accessor is deleted: a caller cannot bypass
declarative cutoff or outer chaining by requesting one raw object.

`SelectedWithEnvironmentObjects::into_reference_plan` consumes the selection,
allocates one unscopables temporary per object, and performs the one reversal
needed to build the nested conditionals in inner-to-outer execution order. It
is the sole external producer of `WithEnvironmentReferencePlan`; the raw
binding-object read, `binding_visible`, `WithEnvironmentResolution` constructor
and plan constructor are private to the Reference module.

The existing non-`Clone`, non-`Copy` plan now has two consuming exits:
`get_value` and `put_value`. Both use the same selected objects, referenced name
and carried `Strictness`, so read and write cannot silently disagree about
which Environment Record chain ResolveBinding traversed. Choosing either exit
spends the plan; a second GetValue or PutValue is E0382.

The GetValue exit builds this fixed tree:

1. initial `HasBinding` (HasProperty, then `@@unscopables`) is evaluated from
   the innermost selected object outward;
2. an initial miss enters only the next outer resolution, and all misses reach
   the declarative/global/unresolvable fallback;
3. the selected branch re-runs `HasProperty` on the exact same materialized
   binding object;
4. an abrupt recheck propagates without performing Get;
5. presence performs a property Get with that binding object as receiver;
6. absence returns `undefined` for `Strictness::Sloppy` and throws
   `ReferenceError` for `Strictness::Strict`.

The lowerer locates the declarative fallback before selecting Object
Environment Records, just as the write path does. The materialized-with source
of `ObjectEnvironmentBindingObject` still names only the stable hidden binding
created after the `with` expression was evaluated, so no part of either query
can re-evaluate or substitute the source object expression.

### 9.3 Proof and boundary

The structural proof covers a non-empty selection, declarative cutoff,
inner-to-outer conditional nesting, same-object initial query/recheck/Get,
strict missing `ReferenceError` and sloppy missing `undefined`. The Wasm fixture
makes the four-operation Proxy trace (`has`, unscopables Get, recheck `has`,
value Get), outer fallback, declarative shadowing, deleted-during-unscopables
strict/sloppy outcomes and abrupt recheck observable. The pinned Test262
oracles are `get-binding-value-idref-with-proxy-env.js`,
`has-binding-idref-with-proxy-env.js`, `binding-blocked-by-unscopables.js`, and
the sloppy/strict-mode
`get-mutable-binding-binding-deleted-in-get-unscopables*.js` pair.
Node 24/V8 does not expose that second query, so a host-engine run is not an
oracle for this edge; the current ECMA-262 algorithm and the pinned Test262
tests agree on the four-operation sequence above.

The `typeof unresolvableName` fast path is used only when no selected Object
Environment Record can bind the name. When one can, the plan uses `undefined`
as its terminal value only for a genuinely unresolvable fallback; any selected
record still runs GetBindingValue before `typeof` applies to the result. This
preserves 13.5.3's exemption without bypassing Object Environment resolution or
turning a selected record's strict missing recheck into `undefined`.

This follow-up claims direct identifier GetValue, including the operand of
`typeof`. The adjacent direct non-eval identifier-call contract now preserves
`WithBaseObject` through a consuming plan which produces the selected callee
and receiver from one binding object; observable with selection forces its
ordinary fallback callee to remain Dynamic. Optional/property/super/eval calls,
generated class/helper contexts and resumable captured Object Environment
Records remain explicit debt. Compound/logical/update/destructuring and delete
operations keep their separately recorded boundaries. No new backend
operation, closure ABI, complete subtree, or pinned-matrix closure is claimed.
