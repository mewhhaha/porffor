# Shared operation descriptor and abrupt-routing contract

This note is the current design boundary for the next T04 migration wave. It
supplements the catalog-evidence contract; it does not replace that contract's
iterator witnesses or its explicit gap ledger.

## One declaration

For every expression-shaped shared abstract operation `o`, Lila records one
descriptor

```text
D(o) = (name, family, operand domain, normal result, abrupt capability)
```

in the `spec_operations!` declaration in `lila-ir/src/operations.rs`. The same
macro row declares the `SpecOperationIr` variant and its canonical `ALL` entry.
There is no default for any descriptor column. Adding a variant without
choosing a domain, result or abrupt capability is therefore rejected by Rust's
macro matcher.

The operand domain is semantic rather than a bare count. For example,
`ValuePair`, `ObjectAndPropertyKey` and `ObjectAndSourceValue` each contain two
operands, but they are not interchangeable claims. Each closed domain owns its
arity predicate. The Wasm backend checks that predicate at both shared entry
points before dispatching an emitter arm, including the two variadic domains
for `Call` and `Construct`.

The descriptor is the source of `name`, `family`, `normal_result` and
`abrupt`; the parallel exhaustive matches that formerly supplied those columns
have been deleted. `SpecOperationCatalogEntry` is still the evidence-bearing
view used by the catalog. Its implementation claim remains no stronger than
the existing `EmitterEvidence` contract.

## Abrupt capability

An expression-shaped shared operation has one of two capabilities:

- `Infallible`: no abrupt completion can escape the operation.
- `MayThrow`: a JavaScript throw may escape the operation.

Return, break and continue are not missing variants here. They are
statement-shaped completion effects and remain in `CompletionAbruptKind` and
the iterator statement rows. If another expression-operation capability is
ever required, adding it must also update the exhaustive conversion to
completion kinds and every backend routing match.

The type records capability, not proof that an emitter arm is correct. That
cross-crate proof remains T04 ledger L2. The first backend consumers therefore
use a deliberately private `MayThrowOperation`: its const constructor rejects
an `Infallible` descriptor, and migrated wrappers always finish routing before
returning to their caller.

## Migrated abrupt routes

Four bounded slices now use typed routing:

1. `GetMethod` invokes `GetV` through a wrapper that routes the tagged thrown
   value to the active in-function handler.
2. `Number.prototype.toFixed` loads argument zero and applies `ToNumber`
   through a wrapper that returns the current completion from the builtin
   function on throw.
3. Every shared `ToPrimitive` entry point, including the object- and
   function-specialized lower seam, requires every caller to pass a
   `ToPrimitiveAbruptRoute`. The closed routes are active-handler propagation,
   current-function return and iterator-close-and-return with its complete local
   witness. Its match is exhaustive, and both former byte-identical
   `_without_throw_propagation` entry points have been deleted.

   The raw-helper route is no longer an enum case. Private raw emitters return a
   `#[must_use]` `PendingToPrimitiveCompletion` whose fields are private and
   whose exits consume it. Exact numeric and string composite consumers emit
   their existing completion guards; a dedicated runtime-helper wrapper emits
   the complete four-slot tuple without exposing either the token or its
   locals. This module denies `unused_must_use`, turning an omitted internal
   continuation into a build error. A specialized caller can no longer invoke
   OrdinaryToPrimitive and merely hope a later statement notices its
   completion.
4. The sole primitive-to-string emitter requires a
   `PrimitiveToStringAbruptRoute`. Its three routes mirror the observable
   consumers: reach the active in-function handler, return the current
   function, or close a named iterator while preserving the incoming throw and
   then return. The route is consumed at the Symbol branch, which is the only
   abrupt branch of primitive ToString. The old
   `_to_local_without_throw_return` copy is deleted; its only consumer now uses
   the same emitter with the active-handler route.

   This closes a real ownership mismatch rather than merely renaming the
   helper. The shared `SpecOperationIr::ToString` arms and the object branch of
   `String(value)` selected active-handler routing for `ToPrimitive`, then
   called a primitive helper whose Symbol branch unconditionally returned the
   whole function. An object whose coercion hook normally returned a Symbol
   therefore escaped an enclosing `try` even though an object whose hook threw
   reached that `catch`. Array element stringification had the same split.
   With a required route, all three cases use the active handler and a new call
   site cannot silently inherit function-return behavior.

The complete primitive-to-string route inventory is deliberately small and
explicit: shared ToString, ToPropertyKey, `String(value)`, array-element
stringification and the String matching fallback use the active handler;
Object.fromEntries and Object.groupBy carry their existing iterator-close
witnesses; JSON and the outlined ToString helper return their current function.
Changing any of those policies is therefore a typed call-site edit, and adding
a route is an exhaustive-match edit.

The ToPrimitive migration preserves each existing coercion and abrupt-routing
order. It also closes one real omitted route: Temporal month-code coercion now
returns the exact value thrown by a user coercion hook before testing whether
the normal result is a String. No general claim is made for the remaining
property reads or roughly 130 builtin `ToNumber` sites; they remain migration
work and continue to use their existing local routing sequences.

## Non-claims

- The completion ABI is not yet an `exnref`/typed-reference ABI.
- The operation catalog does not yet prove that every emitter arm performs the
  routing declared by its descriptor.
- Property internal-method dispatch and proxy correctness still depend on T10
  and T11.
- Feature-local primitive-to-number and full value-to-string conversions remain
  open migration work. The object/function ToPrimitive and primitive-ToString
  seams themselves are closed: raw completion ownership cannot cross either
  boundary, and every internal composite must consume its pending token or
  choose an abrupt route.

The cheapest meaningful integration checkpoint is:

```sh
cargo check -p lila-ir -p lila-aot-wasm
```

That should be followed by the focused `operations_` tests already required by
T04 plus
`wasm_backend_temporal_month_code_preserves_toprimitive_throw_identity` and the
existing Object.fromEntries/Object.groupBy iterator-close regressions, plus
`run_wasm_backend_routes_primitive_to_string_throws_to_the_active_handler`.
This wave was dry-written and intentionally does not claim any command has run.
