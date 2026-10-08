# Conditional await expression ownership

Status: 2026-10-03 source implementation. Compilation, Wasm validation and
execution of the authored regressions are pending. This does not close T14 or
change published conformance results.

The predecessor expression-statement path armed an unconditional prefix without
the admission used by declarations and returns. It could evaluate an await from
an unchosen logical or conditional branch. The source walk also omitted New
arguments and import options. The actual ordinary prefix entry now consumes one
private source-admission carrier; expression statements and yielded await
targets use that entry, and statement-head prefix arming requires the same
carrier. Declarations, returns, destructuring and class operands retain their
existing calls to the shared entry. Constructor arguments and both import
operands participate in the same walk; the import specifier's raw value is
retained before a later suspending option expression.

In a plain async function outside loop bodies or inside a checked awaited while
condition with an eager body, `?:` arms may suspend.
One branch or both may await, and finite nested conditionals reuse the same
owner. The test's prefix runs before branch entry. The existing
`AsyncFunctionIfPlanIr` and Wasm dispatcher evaluate the residual test once,
store the selected branch in disjoint activation-state ranges, and join before
later operands. A private completed-arm constructor checks each actual prefix
against the existing same-activation state census. A non-copyable result owner
allocates the activation binding and creates its declaration, both final arm
writes and its resumed read. The result consumer accepts checked arms rather
than independently supplied statement vectors or result names.

Branch facts start after condition evaluation. Each checked arm owns a private
lowering scope for its compiler-generated temporaries, which remain in the
activation slot registry and its retained prefix. Popping that scope before the
join preserves one common outer source-binding domain even when only one arm
awaits or nested arms allocate different slot counts. The else arm starts from
the same outer facts as the then arm, and the ordinary conservative join runs
before subsequent expressions are lowered. The result retains merged value information. Const
initialization still occurs after the selected arm completes, preserving the
existing pending-binding and TDZ owner. Call, New and tagged-template arguments
consume the existing evaluated Reference and captured argument-list owners.
Direct outer-await targets stage their inner expression before the outer Await.

No new continuation IR variant or dispatcher was added. Existing exhaustive
StatementIr consumers already visit both AsyncFunctionIf arms, including
environment/data/function planning, early errors, throw inference and source
flow. The shared emitter's statement sequence recognizes its entry/exit states,
guards each prefix segment and commits the join after normal branch completion.
Existing try/finally and Promise reactions continue to own rejection, arbitrary
throw identity and execution Realm. Class computed-name/heritage operand
prefixes use the same async statement sequence; nested function bodies retain
separate activation ownership.

The existing source visitors were inspected at their real scope boundaries:
Boa's Await/Yield Contains excludes nested functions and observes computed
class names/heritage. The current-activation suspension visitor additionally
observes static blocks, static initializers and decorators while excluding
instance initializers and nested function bodies. The shared admission requires
an actual current-activation suspension as well as the ordinary Await shape; it
does not infer activation ownership from a nested function's source text.

The subsequent [logical value owner](logical-await-expression-ownership.md)
uses this same completed-arm/result seam for `&&`, `||` and `??`, retaining the
left GetValue once and publishing its original skipped value. That source change
and its authored controls remain uncompiled and unexecuted.

The subsequent [optional property owner](optional-property-await-ownership.md)
uses this same arm/result seam for checked property-only chains. If statically
nullish nested keys erase every actual suspension, the completed-arm state
checks select ordinary If and restore the original entry; both writes and the
result binding remain owned. These sources also remain uncompiled and unexecuted.

The [logical assignment owner](logical-assignment-await-reference-ownership.md)
now shares this arm/result seam outside loops, retaining the actual declarative
binding or captured ordinary property Reference across selected RHS suspension.
Runtime/with/global/unresolved References and suspended LHS/private/super targets
retain their own boundaries. Grouped optional callees/tags with awaited
property keys use the [terminal Reference owner](grouped-optional-reference-await-ownership.md)
outside all loops. Direct delete and target-only awaited callee/tag References,
suspended optional Call/private/super links, super() suspension,
mixed yield/await targets and async-generator branch values remain explicit
refusals. Bounded ordinary-generator values use the separate
[generator branch owner](plain-generator-value-branch-ownership.md). The existing statically nullish optional exception remains.
The [checked awaited while owner](plain-async-awaited-while-condition.md) now
admits these value prefixes in its condition. Its private scoped context retains
loop depth and restores ordinary admission before its eager body; its constructor
checks exact recursive branch association and the shared ready state. Awaited
bodies and other loop heads remain refused because their lifecycle owners do
not admit these branch segments.

Authored controls cover exact state ranges and result-cell association, nested
and single-awaiting arms, Call/New/tag and outer-await targets, the original
logical mis-hoist plus New/import operand omissions, and separate nested
function activations. Three paired sloppy/strict Wasm-AOT sources check selected
branches, retained receiver/callee and operand order, async scheduling, nested
joins, branch facts, const TDZ, class names and throw/rejection Realm/identity
through an awaited finally. All remain unexecuted.

The intended conditional evaluation follows the [conditional operator](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-conditional-operator),
with suspension and rejection handled by the existing [Await](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#await)
implementation. No interpreter, second object model, GC bridge or new host ABI
was introduced.
