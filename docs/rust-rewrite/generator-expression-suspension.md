# Yields nested in synchronous generator expressions

A synchronous generator used to accept a `yield` only in a short list of
positions: a statement-level `yield`, `x = yield`, `return yield`, and a few
hand-counted composites (calls, simple property reads and writes, array and
object literals, class operands). Anything else — a `yield` inside a
destructuring pattern, a binary or unary operand, a template substitution, a
computed object key, a `#x in` operand — was refused before lowering with
"generator body: … cannot be flattened into a linear suspension plan" or, for a
generator expression, the reason-less "generator suspension".

## Staged evaluation

`ScriptLowerer::lower_staged_generator_expression`
(`crates/lila-ir/src/lowering/generator_staging.rs`) evaluates an expression
one operand at a time. Every operand the specification evaluates before a
`yield` is stored in an activation-owned binding, the `yield` becomes its own
`StatementIr::GeneratorYield`, and the expression is finished after it. The
backend's resumable statement dispatcher re-enters after the `yield` with the
partially evaluated operands intact and evaluates nothing twice.

Most forms reuse their ordinary lowering. The operands up to the last
suspending one are evaluated first and *pinned* by AST node, so when the
ordinary lowering reaches the same node it reads the retained value back. This
keeps every coercion and typing decision of the ordinary path: binary and
relational operators (`in`, `instanceof`, equality), comma, unary operators
other than `delete`, `#x in`, a condition-only `?:`, a target-only optional
chain without call links, private property reads and writes, `new`, and object
literals with computed keys, methods and accessors. A computed object key that
is retained across a later `yield` is converted with ToPropertyKey before it is
retained, as 13.2.5.5 orders it.

Forms whose ordinary lowering would move an observable step past a later
suspension have explicit staged lowerings: a template literal appends each
converted substitution to an activation-owned string; a call retains its
callee's GetValue and `this`; an object literal of literal keys and spreads
copies each spread at its own position; `x op= yield` retains GetValue of `x`
before the right-hand side.

## Destructuring assignment

`pattern = value` evaluates the value, then runs 13.15.5.2 against it and yields
the value. A pattern without a `yield` is still one `ArrayDestructure` or
`ObjectDestructure`.

An **array** pattern that suspends keeps its Iterator Record in three
activation slots and runs as steps of the new `ExprIr::ResumableArrayDestructuring`
(`ResumableArrayDestructuringIr`):

- `Open` performs GetIterator and sets `[[Done]]` to false;
- `Elements` runs consecutive non-suspending elements through the ordinary
  element emitter. A suspending element is split: its target Reference's
  operands are evaluated into activation bindings first; when its Initializer
  or nested pattern suspends, a binding-target element stores the iterator
  value (or the rest Array) first;
- a suspending Initializer is a one-branch `StatementIr::GeneratorIf` around its
  single plain `yield`, taken only when the value is undefined;
- the elements run inside a synthesized generator try/catch/finally. The catch
  block runs `Close(Throw)` — IteratorClose whose own abrupt completion is
  discarded — and rethrows; the finally block runs `Close(NormalOrReturn)`,
  whose errors and non-object results propagate. Both are guarded by
  `[[Done]]`, which every step publishes before its completion leaves it.

So `generator.return()` or `generator.throw()` while the pattern is suspended
closes the iterator exactly as 13.15.5.2 step 3 requires, and an exhausted or
throwing iterator is not closed. The emitter is
`compile_resumable_array_destructuring`
(`crates/lila-aot-wasm/src/control_flow/resumable_array_destructuring.rs`); the
pattern carries the one-inhabitant `ResumableArrayPatternProtocol` witness and
the new `EmissionSite::ResumableArrayDestructuring` in the iterator-protocol
catalog.

An **object** pattern that suspends has no iterator to close. After
RequireObjectCoercible, each suspending property evaluates its key (with
ToPropertyKey) and target Reference into activation bindings, reads the
property with GetV, runs its Initializer (a one-branch `GeneratorIf` if it
suspends) and performs PutValue or the nested pattern.

## Admission

`crates/lila-ir/src/generator_staging_plan.rs` walks the same forms in the same
order and allocates the resume states the lowering consumes; its tests check
the planned state count against the states the lowered IR uses. A refused
shape reports the construct that has no staged evaluation order, appended to
the statement-level reason, and generator expressions, generator methods and
generator IIFEs now report that reason instead of "generator suspension".

The shapes that remain refused, each with its own `StagedYieldRejection`:

| Rejection | Shapes | Why |
| --- | --- | --- |
| `ConditionalOperand` | `a && (yield)`, a `?:` branch, logical assignment, a `?.` link | the `yield` runs on only some paths and needs a branching resume point |
| `UnstagedReference` | `delete`, `++`/`--`, `o.p += yield`, `super` References | the Reference itself would have to survive the suspension |
| `UnstagedArgumentList` | spread arguments with a `yield`, tagged templates | no staged argument list |
| `UnstagedForm` | `import()`, `super()`, optional calls | no staged evaluation order |
| `DestructuringInitializer` | `[x = yield (yield)] = v`, `[x = yield* g()] = v` | a one-branch resume point holds exactly one plain `yield` |
| `DestructuringObjectRest` | `({ a = yield, ...r } = v)` | the excluded keys would have to survive the suspension |
| `ObjectLiteralOperand` | `{ ...a, [yield]: 1 }`, `{ a, [yield]: 1 }` | a spread or shorthand before a later `yield` has no retained operand |
| `ClassOperand` | a class key whose `yield` needs a structured resume point | class evaluation prefixes resume linearly |

Async generators keep their preplanned resume states; the structured
destructuring lowering is synchronous-generator only and refuses explicitly
("async generator destructuring assignment whose pattern suspends has no
preplanned structured resume point"). Suspending `var` initializers, `for-of`,
`switch` and the other statement kinds listed by
`GeneratorPlanRejection::YieldInUnsupportedStatement` are unchanged. In
particular the 24 `language/statements/for-of/dstr/*-yield-expr.js` executions
put the suspending pattern in a `for-of` head; they now report
`YieldInUnsupportedStatement` instead of the reason-less "generator
suspension" and need a resumable synchronous `for-of` in plain generators.

## Verification

`cargo test -p lila-ir generator_staging_plan` checks admission against the
lowered IR for every newly admitted shape and every rejection message.
`cargo test -p lila-engine --test aot_generator_expression_suspension` executes
the shapes through Wasmtime with exact evaluation-order traces, including
iterator closing on `return()` and `throw()` resumption, an exhausted iterator,
abrupt and non-object close results, and ToString order in templates.
