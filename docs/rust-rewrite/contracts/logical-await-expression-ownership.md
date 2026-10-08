# Logical await expression ownership

Status: 2026-10-03 source implementation. Compilation, Wasm validation and
execution of the authored controls are pending. T14 and published conformance
results remain open.

Plain async functions outside loop bodies and inside checked awaited while
conditions with eager bodies admit awaited logical
values `&&`, `||` and `??`, including finite combinations with `?:`, Call/New/tag
arguments, outer Await targets, declarations, returns and ordinary property
assignment values. The existing checked prefix admission still excludes
unsupported optional Reference/link forms,
mixed yield/await and async-generator protocols. Bounded ordinary-generator
values use the separate [generator owner](plain-generator-value-branch-ownership.md).
Checked property-only optional values
now compose through the [same branch seam](optional-property-await-ownership.md).
Grouped callees/tags with awaited keys use its
[terminal Reference mode](grouped-optional-reference-await-ownership.md) outside
all loops; the checked while condition still admits only the value route.
Statically nullish optional exceptions remain unchanged.

The private `EvaluatedLogicalLeft` constructor lowers the actual left source
once and appends its completed GetValue to an activation binding before branch
selection. Its consumers can only derive the condition from that binding or
consume it as the skipped arm's original value. `&&` and `||` use the existing
If truthiness owner; neither returns a Boolean in place of the operand. `??`
uses the pure existing IR predicate `(saved === null) || (saved === undefined)`.
This adds no coercion hook and preserves every non-nullish operand, including
the existing Test262 HTMLDDA host value.

The actual logical entry and the existing conditional entry share one private
branch lowering function. Its closed arm source lowers an expression, consumes
the retained-left owner, publishes undefined or consumes a checked optional
property tail, or completes an awaited RHS before consuming its captured logical
assignment write. All routes use the checked private scope and completed-arm
carrier; its entry/ready states
feed the same non-copyable conditional result owner. That owner associates the
declaration, both writes, resumed read and `AsyncFunctionIfPlanIr` join. The
selected RHS alone enters its Await segments. A skipped RHS creates no Promise
reaction or thenable job. The original left value remains activation-owned
through later suspension. When static optional shorting removes every actual
suspension, the checked result owner uses ordinary If and restores the entry.

Arm-local compiler temporaries are removed from the outer source-binding fact
domain before the existing conservative fact join; their activation slots stay
registered. The captured-left and result cells exist in the common outer domain.
Existing Promise reactions, abrupt-completion and try/finally owners continue to
carry rejection and arbitrary throw identity. Existing invocation and property
Reference owners retain receiver/callee or base/key before a suspending value.
No continuation IR variant, planner route or Wasm dispatcher was added.

The cross-entry census includes switch discriminants/selectors: their current
prefix constructor accepts the existing If state spans, and the dispatcher
resumes only the reached selector before committing CaseBlock matching.
The [checked awaited while condition](plain-async-awaited-while-condition.md)
retains loop depth and installs a private branch scope only while staging that
condition. Its constructor validates exact recursive value arm association,
and an eager prefix left by erased awaits stays within each iteration. Ordinary
loop extraction still lacks If-owned segments; awaited bodies and other loop
heads retain their explicit boundaries.

Four IR controls inspect captured-value association, strict nullish predicates,
state ranges, awaited-left order, nested composition and remaining boundaries.
Three paired strict/sloppy Wasm-AOT sources check original primitive/object/
Symbol/BigInt/HTMLDDA values, zero skipped effects/jobs, getter and thenable order,
retained Call/New/tag/property References, TDZ, nested joins and cross-Realm
throw/rejection identity through finally. The existing switch target gains a
selector-order control. Old refusal inputs now exercise compound or optional Call
boundaries. These sources have not been executed.

The value and short-circuit rules follow the [binary logical operators](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-binary-logical-operators).
Suspension continues through the existing [Await](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#await)
implementation. The earlier conditional ownership contract's plain-logical
refusal is superseded by this scope, and its blanket optional refusal by the
property-only extension. The [logical assignment owner](logical-assignment-await-reference-ownership.md)
now admits declarative/ordinary-property RHS suspension outside loops through
this seam. Its remaining Reference boundaries, awaited bodies, other loop heads
and mixed/async-generator protocols remain open. Ordinary-generator loop
branches remain outside the bounded generator value owner.
