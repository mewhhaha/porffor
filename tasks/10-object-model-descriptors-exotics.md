# T10 — Object model, descriptors and exotic-object protocol

**Status:** In progress — canonical descriptor lattice is consumed; exotic closure remains

**Parallel group:** Core foundations  
**Depends on:** T04, T05, T06  
**Blocks:** T11, T16-T24

## Current repository state

`lila-ir` now owns one closed ECMA-262 6.2.6 descriptor lattice: six typed
fields, three presence states, validation before classification, the
data/accessor/generic partition and two complete stored kinds. The ordinary
object `ValidateAndApplyPropertyDescriptor` emitter consumes that lattice, and
the array named-property validator now does too. Its data/accessor entry points
construct `ValidatedDescriptor<WasmLocals>` values, while the validator derives
kind-change checks from `classify`/`KindTerms`; the former parallel
`requested_data_descriptor: bool`, six positional field fragments and
hand-written four-field kind-presence fold are gone. Heap descriptor values and
masks are also distinct typed domains, so an accessor word cannot acquire a
`[[Writable]]` bit through their constructors.

Array-index `[[DefineOwnProperty]]` now crosses the Object builtin boundary as
one `ValidatedDescriptor<WasmLocals>`. Dense and sparse index storage project
their current data/getter/setter carriers into the same typed compatibility
validator used by array named properties; validation therefore completes
before any element, accessor, descriptor-word or length mutation. Generic
descriptors preserve the existing kind, omitted fields preserve the existing
value/accessor, kind transitions use `undefined`/false defaults, and
non-configurable comparisons use tagged `SameValue` (including NaN, signed
zero and object/function identity). Indexed descriptor materialization matches
the stored kind and exposes raw getter/setter identity without invoking either.
This lane owns the 27 observed current-pin
Array witnesses ending 190, 192, 193, 202, 207, 212–214, 227–230, 233–242,
244, 245 and 260–262.

Arguments-index `[[DefineOwnProperty]]` now crosses the same builtin boundary
as one `ValidatedDescriptor<WasmLocals>` and projects current indexed storage
through `StoredDescriptorLocals` into the shared compatibility validator. Its
private non-`Copy` `ArgumentsIndexMappingLocals` captures both mapping presence
and the bits-32..63 environment slot before any descriptor mutation; mapped
reads, post-define writes and mapping restoration consume that retained role,
so a nonzero slot cannot silently become slot zero after the descriptor word
is replaced. Accessor conversion and `[[Writable]]: false` detach the mapping,
generic updates retain the complete mapping, and validation finishes before
the first indexed or ParameterMap store. Creating an absent index also checks
the Arguments non-extensible flag before either store. Indexed descriptor
materialization now exposes raw Arguments getter/setter identity rather than
invoking or flattening the accessor. Dynamically tagged Arguments named writes also enter
an Arguments-aware ordinary `[[Set]]` route: own and inherited accessor or
non-writable semantics run before fresh creation, while actual named updates
use Arguments named-property storage instead of treating the indexed-entry
buffer as an ordinary object property table. The bounded contract is recorded
in `docs/rust-rewrite/contracts/arguments-index-descriptor-exotic.md`; the exact
current-pin witnesses ending 279 and 280 now pass 4/4 Wasm-AOT executions on
the current checkout.
Absent indexed assignment now preserves the direct existing-own/mapped path but
routes a missing own descriptor through prototype `[[Set]]` before bounded
receiver-side indexed creation, including inherited setter/read-only and
non-extensible outcomes. Special `length`/`callee` writes and
`Symbol.isConcatSpreadable` coercion/delete remain explicit follow-up audit
surfaces rather than claims of this lane.

Arguments-object `length` writes now take the same closed
`PropertyDescriptorKind` domain rather than an `accessor: bool`. The Generic
arm preserves the existing data/accessor kind (and data-only writability) while
applying only the requested attributes, so a generic update can no longer
silently turn an accessor `length` back into a data property. Its backing value
is a tagged ECMAScript value rather than a coerced integer, its getter and
setter are stored independently, and the read, write and
`GetOwnPropertyDescriptor` paths exhaustively follow the stored kind. Updating
one accessor field preserves the omitted peer, while a real kind conversion
initializes omitted fields to `undefined` instead of reviving stale storage.
The remaining arguments `callee` and `length` attribute tables also carry
`DescriptorMask` values rather than raw `u64` words. They can test or apply only
the three named attribute masks, and cannot accidentally receive a complete
stored descriptor word at that boundary.

Wasm-AOT `[[HasProperty]]` now has one full crate-visible entry seam and a
private dispatcher over the closed, runtime-consumed
`ObjectInternalMethodBranch` order: Proxy, integer-indexed TypedArray, Array,
arguments, boxed String and Ordinary.
The match is exhaustive, so adding a declared representation without emitting
its branch is a compile error. Function's `prototype` internal slot is part of
the Ordinary branch and is checked on every prototype step. Array builtins no
longer call an ordinary-only bypass. Array and arguments misses, ordinary
prototype traversal, and absent Proxy `has` traps all restart the same dispatch
with the actual next payload and tag; boxed String virtual misses continue into
that object's ordinary storage. Proxy `has` also accepts callable Proxy trap
values. A durable Wasm-AOT regression covers each branch, nested absent-trap
targets, a TypedArray Symbol own property and non-canonical-key prototype
reclassification. The current-pin focused inventory is 58 Test262 files/105
execution variants across Proxy `has` (26/43) and integer-indexed HasProperty
(32/62). On 2026-08-24, the exact AOT controls passed `2/2`, the durable engine
runtime control passed `1/1`, and those filters passed the full `43/43` and
`62/62` Wasm-AOT variants respectively. The combined `105/105` run had every
failure bucket at zero. This is focused evidence for the closed HasProperty
dispatcher, not a claim that the complete Proxy, Object or TypedArray trees are
green.

The bounded Proxy invariant consumers now share a typed direct-target
`[[GetOwnProperty]]` fact and the existing `[[IsExtensible]]` operation. The
fact keeps presence separate from the descriptor word, so an all-false
descriptor cannot masquerade as absence, and exhaustively covers the same
integer-indexed, Array, arguments, boxed-String, Function-special and ordinary
representation order as `[[HasProperty]]`. A false `has` result and a true
`deleteProperty` result both accept absence, reject a non-configurable property,
and check extensibility only for a present configurable property. The former
raw Array/ordinary delete scan and direct `HEAP_CAP_OFFSET` test are gone.

Proxy `[[Set]]` truthy-result validation now consumes a richer typed projection
from that same direct-own-descriptor authority rather than maintaining another
representation scan. `DirectOwnDescriptorProjectionLocals` is a closed Rust
domain: the value-free fact and the complete Proxy-Set projection share one
exhaustive `ObjectInternalMethodBranch` loop. The latter carries distinct fact,
descriptor-data and accessor-setter locals, while target, property key and
incoming value are separate typed roles at both Set call sites. Array length
and indices, mapped arguments data, arguments special/accessor slots,
boxed-String virtual values, Function-special and ordinary storage are observed
without allocating a public descriptor object or invoking a getter. Missing
setters normalize to tagged `undefined`, and the invariant tests exactly that
state rather than requiring a Function tag, so callable Proxy setters remain
valid. Ordinary entries precede virtual fallbacks, preserving a Function
`prototype` entry's later `writable: false` transition while keeping the
DataView/intrinsic and generic internal-slot fallbacks ordered behind it. The
former Object/Function/arguments raw entry scan is gone.

The three user-facing own-descriptor predicates now share one closed compiler
domain. `Object.hasOwn`, `Object.prototype.hasOwnProperty` and
`Object.prototype.propertyIsEnumerable` exhaustively select their input source,
observable conversion order and result projection, then make exactly one call
through the canonical `Object.getOwnPropertyDescriptor` metadata. The static
builtin still performs `ToObject` before `ToPropertyKey`; both prototype
methods still perform `ToPropertyKey` before `ToObject`. Their wrappers no
longer contain Array, arguments, boxed-String, Proxy or ordinary heap scans, so
valid integer-indexed TypedArray elements can no longer disappear from only the
prototype predicates. The enumerable projection reads the materialized
descriptor's own data field and never invokes the target property getter. The
bounded contract is recorded in
`docs/rust-rewrite/contracts/own-descriptor-predicates.md`.
The runtime bootstrap planner now records
`Object.prototype.hasOwnProperty`'s direct dependency on
`Object.getOwnPropertyDescriptor`, and the existing focused planning test
inventories that entry point. Previously the dependency was masked by the
foundational Object-constructor chain and by the combined runtime fixture's
`Object.hasOwn` calls; this closes an architectural reachability gap rather
than claiming a reproduced runtime failure.
On 2026-08-24, the isolated planner invariant and exact CLI fixture passed
`1/1` each. The six direct conversion-order Test262 leaves passed all `12/12`
raw Wasm-AOT variants with every failure bucket at zero. This focused result
does not turn the masked planner omission into a historical runtime failure or
claim complete Object/descriptor closure.

`Object.prototype.toLocaleString` now has one typed `Invoke` path. A private
receiver-role value keeps the exact original receiver distinct from the
current-function-Realm object used only for GetV lookup. General `IsCallable`
validation consumes those roles and produces a private non-`Copy` invocation
token; its sole ownership-consuming call is Proxy-aware and passes the exact
original receiver with no arguments. Nullish and non-callable failures use the
running built-in's Realm. The durable source and CLI regressions cover strict
primitive getter and method receivers, callable Proxy `apply`, the downstream
Array path, and created-realm TypeErrors. At current pin
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the exact `onlyStrict` inventory,
and therefore the exact four-execution inventory, is:

- `built-ins/Object/prototype/toLocaleString/primitive_this_value.js`;
- `built-ins/Object/prototype/toLocaleString/primitive_this_value_getter.js`;
- `built-ins/Array/prototype/toLocaleString/primitive_this_value.js`; and
- `built-ins/Array/prototype/toLocaleString/primitive_this_value_getter.js`.

On 2026-08-24, the batch-wide `cargo check` and `cargo xc` gates were green.
The central verifier passed the three-test
`object_to_locale_string_invoke_structure` target at `3/3`, the exact
`language_numerics::run_wasm_backend_succeeds_for_object_to_locale_string_invoke_fixture`
CLI test at `1/1`, and one Wasm-AOT execution for each listed leaf at `4/4`
total, with every failure bucket at zero. The exact commands, stale-baseline
disclosure and nonclaims remain recorded in
`docs/rust-rewrite/contracts/object-to-locale-string-invoke.md`.

This is direct-target closure only. The fact deliberately marks a nested Proxy
target as handled without treating its own storage as the target descriptor;
the recursive Proxy descriptor-record protocol remains T11 work. The complete
`[[Delete]]` and `[[Set]]` dispatch, trap lookup and fallback paths also remain
separate from the full `[[HasProperty]]` dispatcher. Proxy `[[Get]]` retains its
older value-bearing invariant scan.

Class constructors now install their own `prototype` data property with the
class-specific all-false attribute tuple. Computed public static class elements
use an explicit key guard before definition, because the current
`Presence::Present` complete-descriptor paths intentionally omit the run-time
step-4 compatibility checks and therefore cannot enforce that non-configurable
entry by themselves. The exact descriptor witness passes `2/2` Wasm-AOT
executions. This is a bounded T09 consumer correction and does not close LN10.

This is still a foundation, not task closure. Array application paths,
remaining arguments special/named descriptors, several builtin/exotic emitters and lowering shape
facts still consume derived raw words or parallel positional forms. The
`Presence::Present` step-4 exemption remains the explicit LN10 obligation in
the ordinary and array-named consumers, the ordinary `Object.defineProperty`
adapter still relies on its emitted run-time 6.2.6.5 step-9 check, and the
shortcut audit still finds path/source-dependent materializations. The new
Array-index structural regression has received only rustfmt/diff/static checks;
its focused Rust and Test262 execution remains deferred. The workspace check
for `lila-ir` and `lila-aot-wasm` and the focused array descriptor CLI fixture
were green at the earlier descriptor checkpoint;
the HasProperty and Proxy-Set batches have not rerun them. The new Arguments
indexed checkpoint passed its structural tests 4/4, focused CLI fixture 1/1,
and exact Test262 279/280 variants 4/4 in the centralized verification lane.
The subsequently added Arguments-as-indexed-prototype setter/read-only witness
exposed a dropped Arguments tag in ordinary prototype mutation/observation on
its first focused CLI run. After the bounded tag-preservation repair, the full
fixture including explicit prototype-identity checks passes 1/1; the structural
contract remains 4/4 and exact Test262 279/280 remain 4/4. The focused
Proxy-Set direct-descriptor fixture is written but has not run while the shared
verification lane owns Cargo and Test262. The focused own-descriptor-predicate
fixture, strengthened bootstrap-planning checkpoint and six-file current-pin
cohort are green at `1/1`, `1/1` and `12/12`, respectively. The
`Object.prototype.toLocaleString` Invoke lane is green at `3/3` for its
structure target, `1/1` for its exact CLI fixture and `4/4` for its four
current-pin `onlyStrict` Wasm-AOT leaves, with every failure bucket at zero. A
complete current-pin Wasm-AOT Object/descriptor subtree run has not been
performed.

## Objective

Implement the ECMAScript object internal-method model and exact property descriptor semantics as a reusable runtime/compiler layer. Arrays, typed arrays, strings, module namespaces and proxies should extend this protocol rather than bypass it with unrelated representations.

## Internal methods

Define an explicit dispatch contract for:

- `[[GetPrototypeOf]]`, `[[SetPrototypeOf]]`;
- `[[IsExtensible]]`, `[[PreventExtensions]]`;
- `[[GetOwnProperty]]`, `[[DefineOwnProperty]]`;
- `[[HasProperty]]`, `[[Get]]`, `[[Set]]`, `[[Delete]]`;
- `[[OwnPropertyKeys]]`;
- optional `[[Call]]` and `[[Construct]]` integration for callable objects.

Ordinary objects should use optimized implementations. Exotic objects register overrides while retaining shared invariant checks.

## Property descriptors

- Represent absent descriptor fields distinctly from fields containing `undefined`/`false`.
- Implement data/accessor/generic descriptor classification, `CompletePropertyDescriptor`, `IsCompatiblePropertyDescriptor` and `ValidateAndApplyPropertyDescriptor`.
- Preserve getter/setter identity and callable validation.
- Enforce non-configurable/non-writable transitions exactly.
- Implement `FromPropertyDescriptor` and `ToPropertyDescriptor` with observable property access order.

## Ordinary object behavior

- Prototype traversal, receiver-aware accessors and assignment.
- Prototype-cycle detection.
- Integer-index/string/symbol own-key ordering.
- Extensibility, seal/freeze/integrity-level operations.
- Object literal definitions, computed keys, methods/accessors/spread and `__proto__` semantics.
- `Object` constructor/static/prototype methods and exact descriptors.

## Exotic protocol targets

Create extension points for:

- arrays (T16);
- string wrapper objects (T18);
- arguments objects (T09);
- integer-indexed typed arrays (T17);
- module namespace objects (T12);
- immutable-prototype and host-defined objects;
- proxies (T11), which must wrap and validate any target implementation.

## Optimization constraints

Static shapes and direct offsets are allowed only when guards prove that prototypes, descriptors, accessors, proxies and symbols cannot make the shortcut observable. A deoptimization/fallback path must execute the same internal operation.

## Acceptance criteria

- All property operations route through the explicit internal-method API or a proven guarded fast path.
- Descriptor conversion and redefinition order tests pass with side-effecting/proxy descriptors.
- Own-key ordering is correct for numeric strings, ordinary strings and symbols.
- Object integrity methods handle primitives, proxies and exotics correctly.
- Prototype mutation/cycle and receiver-aware setter cases pass.
- Feature modules can add an exotic implementation without editing a giant central match.
- Object and descriptor Test262 subtrees reach zero failures before this task is closed.

## Required tests

```sh
cargo test -p lila-ir object_ --quiet
cargo test -p lila-aot-wasm object_ --quiet
cargo test -p lila-cli wasm_object --quiet
./target/debug/lila test262 run built-ins/Object --execution-backend wasm
./target/debug/lila test262 run built-ins/Reflect --execution-backend wasm
```

Include tests with accessors, symbols, proxies, inherited properties, non-extensible targets and cross-realm descriptor functions.
