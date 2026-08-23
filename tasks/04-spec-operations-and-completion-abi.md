# T04 — Shared ECMAScript operations and completion ABI

**Status:** In progress — shared catalogs exist; migration is incomplete

**Parallel group:** Foundation  
**Depends on:** T02  
**Blocks:** Most semantic feature tasks

## Current repository state

`lila-ir/src/operations.rs` and
`lila-aot-wasm/src/operations.rs` provide shared operation catalogs and
emitters, while the backend has explicit ABI and control-flow modules. The 29
expression-shaped `SpecOperationIr` rows now come from one typed descriptor
declaration containing the name, family, operand domain, normal result and
abrupt capability. The backend validates that closed operand domain before
dispatch, and the former parallel family/result/abrupt matches are gone.

Typed abrupt routing now covers `GetV` inside `GetMethod`, the `ToNumber` of
`Number.prototype.toFixed` argument zero, and every caller of the shared tagged
`ToPrimitive` emitter. The sole tagged emitter requires a closed
`ToPrimitiveAbruptRoute`: route to the active handler, return the current
function, or close a named iterator and return. Adding a route requires an
exhaustive match update, and a new caller cannot omit the decision. The
duplicate tagged `_without_throw_propagation` entry point is gone.

The same route is also mandatory at the lower object/function-specialized
ToPrimitive seam. Its byte-identical `_without_throw_propagation` twin is gone,
and the former generic raw-completion route is gone. Private raw emitters now
return a `#[must_use]` `PendingToPrimitiveCompletion` with private fields. Every
internal numeric/string composite consumes that token in its exact guarded
continuation; the runtime-helper generator reaches only a dedicated wrapper
that emits all four ABI result slots. `unused_must_use` is denied in the module,
so a new internal raw call that omits its continuation fails to build. Array
element stringification selects active-handler routing before coercion.

Primitive ToString now has the same closed ownership rule. Its sole emitter
requires a `PrimitiveToStringAbruptRoute`: active handler, current-function
return, or iterator-close-and-return with a complete local witness. The former
raw `_to_local_without_throw_return` copy is gone. Every consumer names its
policy, and adding a policy requires an exhaustive match update. This fixes the
shared `SpecOperationIr::ToString`, `String(object)` and array-element paths:
when an object's coercion hook returns a Symbol, the resulting TypeError now
reaches an enclosing catch just like a value thrown by the hook, instead of
unconditionally returning the whole function. Object.fromEntries and
Object.groupBy retain their iterator-close-before-return discipline.

The exceptional `ToLength` seam now has a similarly closed, deliberately
bounded owner set. The two RegExp execution paths must propagate a conversion
throw to the active in-function handler, while Array.fromAsync's array-like path
must reject and return its already-created promise. Those three consumers call
one routed emitter with an exhaustive `ToLengthAbruptRoute`; the former
`_without_throw_return` twin and the three caller-side completion checks are
gone. A throw is routed immediately after `ToNumber`, before the infallible
numeric normalization step, so a new exceptional caller cannot accidentally
continue matching, mutate state, or escape a promise-returning algorithm without
naming its completion owner. The ordinary `ToLength` wrapper and its 56 callers
retain their existing current-function policy and remain outside this bounded
migration.

The proxy-aware `Call` dispatcher now encodes its remaining internal
two-policy choice as a private, exhaustive `ProxyCallThrowRouting` domain:
return the current function's completion tuple, or leave the throw in that
tuple for the caller to inspect. The raw dispatcher is private to
`functions.rs`; its two named wrappers fix one variant each, and the outlined
runtime-helper generator reaches it through the leave-completion wrapper rather
than selecting a raw boolean from `emit.rs`. This domain is deliberately
separate from `PropagateCallThrow::ToActiveHandler`, which may branch to an
active in-function handler instead of returning the current function.

The shared primitive `ToNumber` emitter now encodes its internal two-policy
choice as a private, exhaustive `PrimitiveToNumberThrowRouting` domain. Its two
named wrappers fix either current-function return or leaving the throw in the
completion tuple for an enclosing composite; only those wrappers can reach the
raw emitter. Both the BigInt and Symbol TypeError branches consume the typed
policy immediately after creating the error and before emitting the existing
placeholder NaN, preserving their instruction order while making boolean
inversion impossible.

The value-to-BigInt Number-admission seam now carries a crate-private,
two-variant `BigIntNumberPolicy` instead of `allow_number: bool`. The
crate-visible value helper and its private primitive helper require that
policy; the value helper performs the same number-hinted `ToPrimitive` and
forwards the policy unchanged, while only the primitive Number branch projects
it through an exhaustive match. The two
`SpecOperationIr::ToBigInt` projections, typed-data low-word conversion and
three Temporal epoch-nanosecond conversions explicitly reject Number. Only the
`%BigInt%` function selects `NumberToBigInt`, retaining integral conversion and
non-integral `RangeError` behavior. The implementation, source contract and
bounded six-reject/one-admit mutation guard are independently reviewed. Under
the shared eight-core cap, `cargo xc` is green, the structural guard passes
`2/2`, the exact BigInt minimal-validation CLI witness passes `1/1`, and the
exact TypedArray `with` CLI witness passes `1/1` while exercising Number
rejection by a BigInt typed-data write. This verifies the bounded policy seam;
no broad BigInt, Temporal or Test262 refresh or conformance gain is claimed.

The shared ECMAScript string-trim core now carries a private, exhaustive
`EcmaTrimMode::{Start, End, Both}` instead of independent `trim_start` and
`trim_end` Booleans. That is the complete `TrimString` `where` domain, so the
former unowned neither-end state is unrepresentable. Three named wrappers own
the only raw-core entries: String-to-BigInt selects Both; the static String
method fast path maps `trim` to Both, `trimStart`/`trimLeft` to Start and
`trimEnd`/`trimRight` to End; and the standard-builtin dispatcher applies the
same mapping to its three builtin identities. The existing receiver coercion,
abrupt-completion, scan, slice and temporary-local order is unchanged. The
normative source contract, implementation and hardened caller/alias mutation
guard are independently reviewed. Under the shared eight-core cap, `cargo xc`
is green, the structural guard passes `2/2`, and the exact String trim and
arbitrary-precision BigInt string fixtures each pass `1/1`. No broad String,
BigInt or Test262 refresh or conformance gain is claimed.

The synchronous DisposableStack value-return seam now names its remaining
two-policy choice with a private, exhaustive
`DisposableStackReturnDisposition`: return the current function from the early
nullish `use()` branch, or fall through after a completed `use()` / `adopt()`
path installs the normal result. The former raw Boolean is gone, so a new caller
must name that lifecycle decision and cannot silently transpose an unlabeled
Boolean or omit the choice. This closes one feature-local completion-routing
invariant; it does not migrate the stack to a shared completion operation or
change the tuple ABI. The implementation, source
contract and bounded caller-map guard pass the capped `cargo xc` gate, the
exact structural witness (`1/1`) and the existing exact CLI lifecycle fixture
(`1/1`). This verifies the routing-only seam; the 76-file inventory and broad
DisposableStack cohorts were not refreshed, and no conformance gain is
claimed.

The ArrayBuffer slice bound-normalization seam now carries the private, closed
`ArrayBufferSliceBound::{Start, End}` role instead of a caller-selected argument
index and default Boolean. Exhaustive projection fixes `Start` to argument zero
and default zero and `End` to argument one and the entry byte length; the sole
grouped body for ordinary, shared, and immutable slice writes `start_local`
before `end_local`. The implementation and strengthened caller/order guard are
independently reviewed. Under the shared eight-core cap,
`cargo fmt --all -- --check` and `cargo xc` are green, the structural guard
passes `3/3`, the exact species-capture CLI fixture passes `1/1`, and the exact
`start-default-if-undefined.js` and `end-default-if-absent.js` Test262 leaves
each pass `2/2` Wasm-AOT executions with all failure buckets zero under
`--jobs 1 --threads 1`. This verifies only the bound-role invariant: no broad
ArrayBuffer/Test262 refresh, shared-operation migration, copy-policy change, or
conformance gain is claimed.

The earlier Proxy `Call` and primitive `ToNumber` migrations are likewise
invariant-only rewrites. Their former boolean selections already chose the
correct policies, all existing public wrapper call sites are unchanged, and the
policy-dependent emission points retain their exact return/leave branch and
instruction order. Focused source contracts pin each closed variant set,
exhaustive projection, private raw entry and named-wrapper route. Their static
source/diff/rustfmt gates are green; compile and the existing Proxy apply,
callable-trap abrupt-completion, JSON reviver and numeric-conversion runtime
fixtures remain queued behind centralized verification. No `Call`/Proxy or
`ToNumber` conformance gain, completion-ABI redesign or `exnref` migration is
claimed.

This migration also fixes the Temporal month-code coercion path: a user value
thrown by `toString` now escapes unchanged instead of being overwritten by the
later non-String TypeError check. Existing coercion and iterator-close order is
otherwise unchanged. These wrappers do not make the remaining property and
builtin-coercion sites authoritative: feature
emitters still contain substantial local coercion, property and completion
logic, and the large Test262 materialization layer shows that shared operations
are not yet authoritative across every family. The Wasm completion convention
also remains the existing tuple/current-completion mechanism rather than the
target `exnref` design.

The descriptor and migration boundary are specified in
[`docs/rust-rewrite/operation-descriptors.md`](../docs/rust-rewrite/operation-descriptors.md).
Keep new cross-family semantics in the shared operation layer and delete local
copies only as callers migrate.

## Objective

Create one spec-shaped implementation path for common ECMAScript abstract operations and one uniform ABI for normal/throw/return/break/continue completions. Remove feature-local copies whose subtle differences cause evaluation-order, proxy, realm and abrupt-completion failures.

## Required operation families

### Conversion and comparison

- `Type`, `IsCallable`, `IsConstructor` and `IsPropertyKey`.
- `ToPrimitive` with correct hint and `@@toPrimitive` ordering.
- `ToBoolean`, `ToNumeric`, `ToNumber`, `ToBigInt`, `ToString`, `ToObject` and `ToPropertyKey`.
- `ToIntegerOrInfinity`, `ToLength`, `ToIndex`, integer/uint conversions and clamping.
- `SameValue`, `SameValueZero`, strict equality, abstract equality and abstract relational comparison.

### Object and invocation operations

- `Get`, `GetV`, `Set`, `HasProperty`, `HasOwnProperty`, `DeletePropertyOrThrow`.
- `CreateDataProperty`, `CreateDataPropertyOrThrow`, `DefinePropertyOrThrow` and descriptor conversion.
- `GetMethod`, `Call`, `Construct`, `OrdinaryCreateFromConstructor`, `SpeciesConstructor` and `ArraySpeciesCreate`.
- Iterator acquisition/step/value/close operations, with sync/async variants exposed for T14/T15.

### Completion model

Define a Rust representation and Wasm calling convention for:

- normal value;
- throw with value and realm-correct error identity;
- return;
- break/continue with optional target;
- empty completion and completion-value updates.

The convention must work across user functions, builtins, proxy traps, host imports and nested `try/finally` without relying on unstructured scratch globals.

## Design constraints

- Operations must preserve observable order and stop immediately on abrupt completion.
- Object operations must dispatch through the internal-method protocol from T10; static-shape fast paths require guards proving no observable trap/accessor/prototype difference.
- Avoid a runtime interpreter. These are compiler-emitted helpers or specialized Wasm functions generated from typed operation IR.
- Design the Wasm-level completion convention from the experimental Wasmtime lower bound: `exnref` exception handling, typed function references and reference types are available and may carry throw/abrupt paths. Do not maintain a second completion mechanism for runtimes that lack them.
- Keep operation signatures stable enough for feature modules to depend on them. Version or feature-gate ABI changes rather than silently changing tuple layout.
- Emit structured diagnostics when an operation cannot yet lower; do not panic.

## Implementation sequence

1. Write a catalog mapping operation name to spec inputs, outputs and possible abrupt completions.
2. Introduce typed operation nodes/helpers in `lila-ir`.
3. Introduce shared Wasm helper generation and a registry that emits each helper once per module.
4. Convert representative property access, builtin argument coercion and tagged `ToPrimitive` paths.
5. Migrate remaining call sites incrementally, deleting old helpers as coverage moves.
6. Add operation-level differential tests against `spec-exec` using side-effecting coercion objects and proxies.

## Acceptance criteria

- There is one authoritative implementation for each listed operation or an explicit tracked gap.
- Side-effect/evaluation-order tests cover success and abrupt paths for every conversion family.
- Nested calls and builtins can propagate arbitrary thrown JavaScript values, not only error-name strings.
- `try/catch/finally`, proxy traps and cross-realm errors consume the same completion ABI.
- Representative Array, String, TypedArray, Date and Proxy tests use the shared operations rather than local coercion code.
- No operation silently maps unsupported object input to a primitive default.

## Required tests

```sh
cargo test -p lila-ir operations_ --quiet
cargo test -p lila-aot-wasm operations_ --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli wasm_ --quiet
```

Run real Test262 coercion-order cases from several builtins plus `language/statements/try`, `built-ins/Proxy`, and `built-ins/Object` to verify cross-family behavior.
