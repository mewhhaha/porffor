# T09 — Functions, constructors, classes and private elements

**Status:** In progress — broad function/class support exists; full call/construct semantics remain

**Parallel group:** Core foundations  
**Depends on:** T04, T06, T08  
**Blocks:** T12-T15, T24

## Current repository state

The IR and Wasm backend contain explicit function metadata, call/construct
lowering, closures, bound functions, classes and private-element support, with
many focused fixtures. Class element definitions now carry the closed
`ClassMethodKindIr::{Method, Getter, Setter}` domain: a constructor or a
no-class-role function cannot enter a public/private method row, and the Wasm
definition emitter consumes the three cases exhaustively instead of rejecting
an impossible kind at runtime. The function lifecycle is now also a closed
`FunctionProtocolIr`: analysis, lowering signatures, `FunctionIr` and Wasm
metadata carry one of the reachable ordinary/arrow/resumable/class roles rather
than independently combining flavor, execution kind, constructability and
class role. Generated accessors cannot become resumable or constructable,
class constructors cannot lose `[[Construct]]`, and the backend derives its
runtime flags exhaustively from the same protocol. Backend prototype
materialization is a separate policy, so realm bootstrap no longer lies about
the constructability of GeneratorFunction, AsyncFunction or
AsyncGeneratorFunction while suppressing their automatically generated
`prototype` object. The exact matrix and boundary choices are recorded in
`docs/rust-rewrite/contracts/function-protocol.md`.

The built-in-function `Function.prototype.toString` materializer has been
removed. Both unchanged pinned sources now execute with the LocalMerged
native-function matcher; the broad case also retains the full
`wellKnownIntrinsicObjects.js` traversal. All four sloppy/strict Wasm-AOT
executions pass. The separate Sputnik materializer has also been removed. Its
nine unchanged pinned sources execute with the complete vendored harness, and
all 18 sloppy/strict Wasm-AOT executions pass. The generated inventory assigns
one T09 observation to the reduced native-function-matcher path gate. The
token-aware scanner exposes that previously uncounted selector; neither removed
materializer has returned.

Class auto-accessors now preserve their public/private and instance/static AST
kinds and lower through a closed descriptor-plus-backing plan. Every element
owns an inseparable generated getter/setter pair and a fresh typed backing name
which source private-name lookup cannot construct; the class private
environment separately records visible names and total slots. Definition-time
events install complete public or private accessor entries, while ordered
instance/static events initialize only the backing private field. The Wasm
fixture covers all four placements, literal/computed/string/numeric/Symbol
keys, detached and wrong receivers, descriptor flags and lengths, overwrite
order, inheritance and non-extensible receiver rejection. At `2026-08-22`, the
focused IR/backend/CLI gates are `1/1`, `1/1`, and `1/1`; the five raw pinned
grammar/control files pass `10/10`; and the public staging semantic file passes
`2/2`. The private staging file's ordinary semantics execute, but its two
literal-`eval` duplicate-name assertions remain `0/2` Runtime/Bug because the
dynamic-source boundary produces the wrong error constructor. A created-realm
class fixture also remains open. The design, evidence and nonclaims are
recorded in
[`class-auto-accessors.md`](../docs/rust-rewrite/contracts/class-auto-accessors.md).

Object-literal methods, getters and setters now extend that closed function
protocol without masquerading as class members. A public
`ObjectMethodFunctionIr` with private construction state is the only value the
six method/accessor `ObjectPropertyIr` rows accept, so every exhaustive IR and
AOT consumer must acknowledge the HomeObject-bearing lifecycle. The backend
pairs that carrier with the already allocated literal, stores the literal as
the function's `[[HomeObject]]` before property definition, and consumes the
invocation `this` as the distinct Receiver for super reads and writes. The
durable oracle covers method/getter/setter bodies, parameter-initializer super,
computed/static key order, detached alien receivers and later prototype
replacement. At clean pre-batch commit `304e4bbad3`, the exact five-file cohort
under `language/expressions/object` is `method.js`,
`method-definition/name-super-prop-body.js`,
`method-definition/name-super-prop-param.js`, `getter-super-prop.js`, and
`setter-super-prop.js`; it reported `0/10` sloppy/strict executions, all at the
object-literal-method NotImplemented boundary. The implementation, bounded
witnesses and fixture now pass the workspace/all-target check, `cargo xc`, the
focused IR invariant (`1/1`), the bounded structure executable (`5/5`), the
Wasm CLI fixture (`1/1` in 19.75s), and the exact cohort (`10/10`, zero
unsupported/crash/bug outcomes). Complete resumable object-method HomeObject
transport remains an explicit nonclaim of that batch.
Direct generator and async body/parameter controls are green, but they do not
establish complete suspension-safe or async-generator transport. Nested arrows
using an enclosing object method's `super` now have a separate verified
closed owner-role boundary. At clean pre-batch commit `039253d27`, exact
Test262 `prop-dot-obj-val-from-arrow.js` and
`prop-expr-obj-val-from-arrow.js` reported `0/4` sloppy/strict executions, all
at the object-literal-method Runtime/NotImplemented boundary. The
workspace/all-target check, focused IR invariant (`1/1`), bounded structure
executable (`4/4`), Wasm CLI fixture (`1/1` in 19.37s), and exact cohort (`4/4`,
zero unsupported/crash/bug outcomes) are now green. Its durable fixture covers the
paired lexical `this`/HomeObject capability, parameter-created and multiply
nested arrows, detached receivers and later prototype replacement. The two
boundaries are recorded in
`docs/rust-rewrite/contracts/object-literal-home-object.md` and
`docs/rust-rewrite/contracts/object-method-arrow-super.md`.

The adjacent non-resumable super-property mutation lifecycle now has a fused
contract and verified consumer oracle. It covers a computed key which
changes the HomeObject prototype during its sole coercion while the retained
base and detached alien receiver still reach the original getter and setter;
the exact compound and prefix traces are `key,getA,rhs,setA:3:true` and
`key,getA,setA:2:true`. The fixture also covers every prefix/postfix
increment/decrement form for Number and BigInt, strict failed Set, and derived
constructor uninitialized-`this` ordering. At near-HEAD `b0d1d1300`, the four
exact `language/expressions/super/prop-expr-*-putvalue-{increment,compound-assign}.js`
files reported `2/8`: the increment pair was `0/4`
Runtime/NotImplemented, the uninitialized-`this` compound file was `0/2`
Runtime/Bug, and the existing compound GetSuperBase guard was `2/2`. The debug
binary was four minutes older than the commit. Post-batch workspace check and
`cargo xc`, focused IR `1/1`, structure `5/5`, compiled Wasm fixture `1/1`,
exact cohort `8/8`, and both adjacent eight-execution order/control filters are
green with zero unsupported, crash or bug outcomes. Resumable, logical and
private mutation References are not claimed. The normative boundary is
`docs/rust-rewrite/contracts/super-property-reference-mutation.md`.

Private-element heap storage now has the closed five-row
`PrivateElementHeapKind` protocol. Receiver rows are either a brand or a field;
shared definition rows are a setter, method or getter. The entry writer accepts
only legal row variants instead of independently combining an optional
receiver, a raw integer kind and an optional value, and definition lookup has
the narrower three-kind domain. Private read and write trap compiler-owned
corrupt rows rather than treating an unknown kind as a brand. The stable wire
words and backend/spec boundary are recorded in
`docs/rust-rewrite/contracts/private-element-entry-protocol.md`.

`PrivateElementEntryLocals` is now one owned row with no incidental cloning,
copying, debugging, equality, or default capability. Its borrowed exhaustive projections
copy only raw locals before the owned writer preserves the existing validation,
allocation, storage, and Realm-list publication order. The exact 13 lexical mentions and
five product producers are guarded alongside five test rows, shape assertions,
and reverse local releases; the contract remains
`docs/rust-rewrite/contracts/private-element-entry-protocol.md`. The embedded
row unit is green `1/1`, the focused structure target is green `5/5`, and the
exact callable-definition, duplicate-installation, and non-extensible-receiver
CLI witnesses are green `3/3`; broader workspace and Test262 verification
remain centralized. Independent review is clean after exact-normalizing all
five producer wrappers and the complete writer lifecycle. The shared workspace
formatter, `cargo xc`, diff, module-boundary, and task-plan checks all pass;
broader Test262 verification remains deferred.

Arguments-object construction now has the closed backend protocol
`Absent | Present(Unmapped | Mapped(plan))`. Arrow functions have no own
binding; strict or non-simple ordinary functions are unmapped; sloppy simple
ordinary functions carry a prevalidated argument-index-to-environment-slot
plan. Missing mapped storage is rejected as malformed lowered IR instead of
silently changing the function to unmapped, duplicate names retain only their
last occurrence, and an empty simple list remains `Mapped(empty)`. The semantic
and storage boundaries are recorded in
`docs/rust-rewrite/contracts/arguments-object-construction-protocol.md`.

Ordinary property assignment now discovers the `length` and `callee` special
properties of a same-receiver Arguments object before entering the ordinary
named-property scan. Their stored accessors therefore run through the existing
Arguments writer instead of being missed by the shared OrdinarySet helper. The
restored mapped-descriptor fixture passes unchanged, and an independent
fixture covers captured setter bindings for both special properties. At
`2026-08-27`, the focused structure test, mapped-descriptor CLI regression,
special-accessor CLI regression, and neighboring callee CLI regression pass
`1/1` each with fresh explicit cache directories for the CLI runs.

Bound-function creation now preserves `[[BoundThis]]` as the exact tagged
ECMAScript value supplied to `bind`. A private two-source domain admits only
builtin argument zero and the compiler-owned Proxy revocation Object; sibling
modules cannot call the raw payload/tag allocator. Strict preservation and
sloppy substitution/boxing remain centralized in the target-call path, so a
strict primitive is not boxed during binding and a sloppy primitive receives a
fresh wrapper on each invocation. The boundary and its cross-realm nonclaim are
recorded in
`docs/rust-rewrite/contracts/bound-function-this-capture.md`.

Class-definition installation now preserves the non-configurable constructor
`prototype` invariant. Class constructors materialize their own `prototype`
data property with all three attributes false. Computed public static
`prototype` fields, methods/getters/setters and auto-accessors share one
property-key guard; field initializers run before the resulting TypeError,
while auto-accessor backing initialization remains after the failing accessor
definition. The durable class-element fixture passes `1/1`, the three exact
computed-field files pass `6/6`, and the adjacent nine-file
method/accessor/descriptor cohort passes `18/18` Wasm-AOT executions. This is a
bounded class-definition correction, not complete class closure.

The closed thirteen-member family of non-generic Boolean, Number, BigInt, and
String prototype methods now retains the acquired function object and the
reference base as separate `CallIndirect` operands. Shape analysis may identify
one of those targets, but that knowledge no longer authorizes a key-only
`CallMethod` whose backend fast path can replace the transferred function
according to the receiver and property name. A private
`NonGenericBuiltinMethod` domain owns the two Boolean methods, all six Number
methods, all three BigInt methods, and String `toString`/`valueOf`; generic
String methods keep their key-only fast paths. The receiver-materialization
boundary has a durable binding-identity witness proving the callee read and
`this_arg` share the same single evaluation. For each method, the witness covers
a valid same-brand boxed call including `Object(1n)` under an unrelated name,
plus an Object wrong-brand call under standard and unrelated destination names;
the six Number methods also cover a boxed-Boolean standard-name transfer so a
remembered Boolean value cannot fold away an overwritten callee. Because heap
shapes are copied by value rather than joined by an object-identity carrier, a
property write clears the complete pre-write Boolean fold set and invalidates
the copied heap shapes of every other binding in that set before updating the
precisely resolved target. A separate alias witness proves a write through a
copied boxed-Boolean binding leaves neither a literal fold nor a stale builtin
target on the original name. All 45 family calls pin the expected result kind
on both IR layers. Runtime evidence remains
narrower. Fresh baselines confirmed
`built-ins/Number/prototype/toString/S15.7.4.2_A4_T01.js` and
`built-ins/Number/prototype/valueOf/S15.7.4.4_A2_T01.js` failing in both modes.
After the final alias-safe fold repair, both complete five-file Number prefixes
pass 10/10. The ten Boolean files remain a separate bounded rerun gate. The
Number formatting methods share
`thisNumberValue`, while pinned BigInt and String tests prove their branded
extraction and realm contracts without covering every property-transfer shape.
Symbol and Date were audited but do not enter the domain because current
lowering already preserves their acquired callees through the general
indirect-call path. The boundary is recorded in
`docs/rust-rewrite/contracts/non-generic-builtin-method-callee-identity.md`.

Exact-context return facts are no longer applied to a source function whose
captures are absent from the context key. The finalized lowered body remains
authoritative for `FunctionIr` and aggregate return metadata instead of being
overwritten after lowering by a stale context signature. The positive captured
`ArrayBuffer` shape witness remains exact when no later effect invalidates it;
the mutation-before-call control proves that the body, function return targets
and call result all omit the stale intrinsic `ArrayBuffer.prototype.resize`
identity. Both focused regressions pass `1/1`, and the complete `lila-ir` unit
suite passes `892/892`. Context-sensitive captured-state specialization remains
future work.

The `%Function.prototype%[@@hasInstance]` source batch now gives the ordinary
algorithm and the `instanceof` operator a shared closed request domain rather
than a boolean-selected helper. The operator entry owns observable
`@@hasInstance` lookup and handler invocation; the ordinary entry owns callable
and primitive rejection, bound-target redispatch, observable `prototype` Get,
and Proxy-aware prototype-chain traversal. The exact intrinsic is installed
with its realm-local identity and all-false property attributes. `cargo xc`,
the five bounded structure checks and the CLI witness are green. The complete
intrinsic leaf passes 22/22 strict and sloppy Wasm-AOT executions, and the
adjacent four-file operator-hook prefix passes 8/8. These are focused results,
not a replacement for the complete current-pin publication.

The request's private runtime-state authority no longer derives cloning or
copying capability or relies on Rust discriminant order for its numeric code.
One borrowed exhaustive projection owns the existing operator 0 and ordinary 1
codes at the initial store, operator gate, absent-handler transition and bound
target redispatch. The bounded guard separately pins the raw Wasm `i64` local's
single comparison and three writes; this is source-equivalent encoding
hardening, not a claim that Wasm locals carry the Rust type. The structure
target passes `5/5`, the exact CLI consumer passes `1/1`, and four selected
intrinsic/operator Test262 leaves pass all `8/8` Wasm-AOT variants with every
failure bucket at zero. Independent dry re-review is clean after the first
reservation and complete reverse-release tail were pinned. The following
shared workspace compile, formatter, module-boundary, task-plan and diff gates
all pass.

Cross-realm Function construction remains an explicit dynamic-source
exclusion, and complete Function/class/private-element subtrees have not been
verified against the current pin without materializations. This remains an
active foundation task.

Function-object prototype allocation now consumes the capability-free
`FunctionPrototypeMaterialization::{Automatic, BootstrapSupplied}` policy
through an exhaustive two-arm projection instead of equality that let a future
policy inherit bootstrap's no-allocation default. The recursive, bounded guard
pins the six existing producers and the automatic allocation/store/publication
order. This is source-equivalent T09 hardening, recorded in
[`function-prototype-materialization.md`](../docs/rust-rewrite/contracts/function-prototype-materialization.md),
not new function or constructor behavior. The structure target passes `4/4`,
and the automatic-prototype and created-Realm bootstrap CLI witnesses each pass
`1/1`. Independent review hardened the guard to pin the exact allocation,
function-header stores, both property-publication rows and release order, then
finished clean. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings.

The function-object representation now distinguishes a compiler-owned builtin
closure capture from the environment handle used by Realm/error/proxy paths.
Promise's fourteen escaping internal functions use the new GC-visible capture
slot and self-backed environment identity through one typed materializer. Both
function allocation paths initialize the slot, while non-Promise functions
retain their existing environment representations. This is a function-header
ABI seam for T06/T14, not broader call/class closure.

Async invocation now derives one opaque execution-Realm context from the
callee function object before allocating its returned Promise. The same
authority is stored in the ordinary async activation and later consumed by
captured reactions; async generators recover it from their retained function
object. This closes the defining-Realm handoff at the call boundary without
changing PromiseResolve constructor catalogs or unrelated async builtins. The
shared semantic golden passes `2/2`
across 663 dumps; its three additions are the focused callback/async Realm
witnesses, and all 660 retained structural summaries match after expected
code-size and local-accounting normalization.

Async-generator request allocation now obtains the intrinsic Promise
constructor from the executing `next`, `return` or `throw` function object's
defining Realm. A private non-copyable constructor proof is the only input to
the request-specific capability wrapper; the generator activation's retained
function is deliberately not used as method Realm authority. Entry publication
self-backs all three request-method identities so their call ABI carries the
required defining-Realm proof. Created-Realm async-generator function
materialization remains outside this boundary. The focused runtime witness
passes `1/1`; the shared 664-dump semantic golden passes `2/2`, adds one
Temporal fixture, removes none and records only the strengthened async Realm
witness's intentional five-function structural expansion among retained dumps.

Five Function prototype methods now obtain their invocation receiver through
one paired Function prototype receiver authority. Its private non-copy carrier
can be constructed only from the builder's payload-and-tag `this` slots, so
`@@hasInstance`, `call`, `apply`, `bind`, and `toString`
cannot mix payload and tag sources or substitute `new.target` while creating
the receiver proof. The
recursive Rust-lexical
`function_prototype_receiver_ownership_structure` guard pins the sole paired
constructor, all five producers, the absence of raw receiver/new-target reads
inside those branches and every typed projection. This source-equivalent
hardening does not change callability, receiver adaptation, Proxy or Realm
behavior and does not claim wider T09 conformance. The receiver guard passes
`4/4`, the neighboring private-element ownership target passes `5/5`, and the
exact CLI test passes `1/1`, running the Function-builtin fixture through
Wasm-AOT with `boolean(true)`. The full focused evidence is recorded in the
contract.

Batch AV makes the outer family a private `FunctionBuiltin` with no derived
capabilities and exposes only eight fixed Function entries to standard
dispatch. Seven entries are public intrinsic operations; the eighth is the
separately named hidden bound-function invoker. The frozen 409-line
domain/emitter selection has SHA-256
`f922e7edf4c8c1626a9b40920c2a9f418c8b3badcce3c347ffb09b55109d2093`;
restoring only the former derive and visibility reproduces that source exactly.
`cargo xc` passes. The receiver-ownership, callable-prototype and
`Symbol.hasInstance` structure targets pass `4/4`, `8/8` and `5/5`; the exact
Function-builtin Wasm-AOT CLI fixture passes `1/1`. No Test262 or Wasm golden
was required for this source-equivalent boundary, which claims no new Function behavior,
conformance result or published-count change.

## Objective

Complete the ECMAScript call/construct model and class semantics, including metadata, parameter environments, `this` modes, inheritance and private elements. Async/generator execution engines are owned by T14/T15, but their function objects must use the interfaces defined here.

## Function object model

Represent all required internal slots and behavior for:

- ordinary functions, arrows, methods, getters/setters and concise methods;
- base and derived constructors;
- builtin functions and host functions;
- bound functions;
- generator/async/async-generator function objects;
- class constructors and field initializer functions.

Each function must retain realm, environment, private environment, source-text representation, strictness, `this` mode, constructor kind, home object and code identity.

## Call and construct

Implement shared `[[Call]]`/`[[Construct]]` paths with:

- ordinary call binding and lexical `this` for arrows;
- sloppy `this` substitution/boxing and strict preservation;
- `new.target`, constructor return-value rules and derived-constructor `this` initialization;
- `super()` and `super` property access through the home object;
- bound arguments/this, bound constructor forwarding and bound metadata;
- custom new target and realm-correct prototype fallback;
- callable/constructable proxy integration through T11.

## Parameters and `arguments`

- Function declaration instantiation.
- Simple/non-simple parameter lists, defaults, rest and destructuring.
- Correct parameter/body environment separation.
- Mapped and unmapped `arguments`, aliasing, iterator and property descriptors.
- Duplicate parameter and strict-mode interactions from T07.
- Function `name`, `length`, inferred names and `toString` source representation.

## Classes and private elements

- Heritage evaluation, `extends null`, constructor synthesis and prototype creation.
- Instance/static public fields, methods, accessors and static blocks.
- Private fields, methods and accessors; brand creation/checking and lexical private-name resolution.
- Correct ordering of computed names, decorators if standardized in the pin, field initializers and static initialization.
- Class name TDZ, immutable inner binding and strict semantics.
- `super` in fields/static blocks and cross-realm inheritance.

## Acceptance criteria

- All function kinds share one coherent metadata/call protocol.
- Arbitrary thrown values propagate through calls/constructors.
- Bound, proxy-wrapped and cross-realm constructors preserve new-target behavior.
- Parameter/default/rest/arguments aliasing and evaluation-order tests pass.
- Class fields/private elements/static blocks pass brand, ordering, inheritance and abrupt-completion tests.
- Function `name`, `length`, prototype-property presence and descriptors match Test262.
- No function family is implemented by source-text pattern matching.

## Required tests

```sh
cargo test -p lila-ir function_ --quiet
cargo test -p lila-aot-wasm function_ --quiet
cargo test -p lila-cli wasm_function --quiet
cargo test -p lila-cli wasm_class --quiet
```

Run real filters under `language/expressions/function`, `arrow-function`, `class`, `language/statements/function`, `built-ins/Function`, `Function/prototype`, and private-element feature groups.
