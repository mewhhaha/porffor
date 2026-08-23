# T24 — Globals, native errors, Annex B and remaining host-visible builtins

**Status:** In progress — errors/globals/URI/host exotics are broad but not fully closed

**Parallel group:** Feature lane; split by errors, globals and Annex B  
**Depends on:** T04, T06, T07, T09, T10; dynamic evaluation uses T13; strings/RegExp use T18/T19  
**Blocks:** Remaining builtins/Annex B/harness portions of T26

## Current repository state

Native errors, global constants/functions, URI codecs, Annex B builtins,
IsHTMLDDA and AbstractModuleSource have dedicated runtime/backend paths and
many focused real-suite results. The Wasm backend now validates every
hand-written runtime-error name at its string boundary and immediately carries
`NativeErrorKind`; one exhaustive mapping owns both the global and per-realm
prototype locations, the realm snapshot entries derive from
`NativeErrorKind::ALL`, and the static `instanceof` fast path consumes the same
mapping. A new error family omitted from that authority is therefore a compile
error, and an invalid internal spelling can no longer silently fall back to
`%Object.prototype%`.

The 19 host-backed callables now also come from one macro-backed
`HostBuiltinId` row source. Each row must classify its global exposure and
realm scope, so the compiler derives complete/global/every-realm iteration and
name lookup without parallel lists or a raw `HTMLDDA` exclusion. Lowering,
AOT lookup/stub planning and created-realm installation consume that catalog.
The nineteenth row is the Test262-only realm-eval callable: T13 rejects every
resolved invocation as typed dynamic-source debt, and its defensive AOT body
exists only so the harness can store a valid function value, not as support.
`HostSurfacePolicy` is now the authority over those classifications. Product
compilation, including the CLI/default engine path, admits ECMAScript globals
and the deliberate `print`/`gc` extensions but does not resolve Test262-only
`__lila*` globals. The Test262 harness opts in explicitly, the policy is part
of the whole-program cache key, every IR lowering entry point receives it, and
agent workers inherit the root compilation's policy. The focused authority
acceptance requirement is therefore closed. CLI conformance fixtures use the
explicit `--host-surface test262` opt-in; product CLI compilation retains the
default `Product` policy. AOT raw-identifier fallbacks are also limited to the
IR-derived compiled-host set, so the backend cannot re-authorize a filtered
spelling through its complete stub registry. See
`docs/rust-rewrite/contracts/host-builtin-surface.md`.

Global declaration instantiation now has one typed IR authority as well.
`GlobalBindingPlan` owns a unique map instead of allowing lowering to append
duplicate predefined/`var`/function rows for AOT to collect last-wins. Property
initialization and declaration claims are separate: `var Infinity` retains the
immutable predefined value and descriptor, while duplicate source functions
carry the exact last `FunctionId`. Global lexical names stay in the separate
declarative-name set. Annex-B copies carry an exhaustive owner-binding versus
script-global target, and script-global writes reuse the plan's existing-
property policy rather than unconditionally overwriting raw object storage;
every mirrored Set then resynchronizes the frame cache from the authoritative
global property, including properties hardened after instantiation.
See `docs/rust-rewrite/contracts/global-binding-plan.md`.

`Error.prototype.toString` now has one object-representation admission and two
typed observable phases. The shared object-like predicate admits Object,
Function, Array and Arguments values, after which both `name` and `message`
use the same proxy-aware `[[Get]]` path. A private, `must_use`
`PreparedErrorNameLocal` is the only input accepted by the message/result
phase, so `Get(name)`, name defaulting and `ToString(name)` must be emitted
before `Get(message)`. This closes the prior Array/Arguments omission and the
prior early observation of `message`; name conversion mutation and abrupt
completion, all backend object representations, Proxy trap order, and
defining-realm TypeErrors from the ToPrimitive/ToString composite have a
focused durable witness. The shared operation layer exposes dedicated
current-function-realm conversion wrappers to this builtin. An opaque,
`must_use` primitive token carries their closed realm policy between
ToPrimitive and primitive ToString, while helper ABI parameter 2 forwards the
same two-word policy when ToPrimitive is outlined and parameter 6 forwards the
Realm environment. The existing main-realm wrappers remain separately fixed
policy surfaces, so T24 cannot silently select the wrong realm for a Symbol or
invalid conversion hook. See
`docs/rust-rewrite/contracts/error-prototype-tostring-phases.md`. This is a
bounded semantic closure only; the focused runtime and current-pin Error gates
remain deferred to the centralized verification pass.

`Error` construction now gives `OrdinaryCreateFromConstructor` one typed
owner.  The generic construct dispatcher sends the allocating Error builtin
directly to its body, preventing a second observable `NewTarget.prototype`
read and an unrelated `%Object.prototype%` preallocation.  A private,
non-`Copy`, `must_use` prototype witness couples payload and representation
tag; its only instance allocator uses the tagged prototype operation.  A
primitive prototype selects the required `%Error.prototype%` slot only after
`GetFunctionRealm(NewTarget)`, never a per-function snapshot or entry global.
Entry and created realms both publish that slot through the closed
realm-intrinsics domain, and created Error functions carry their active
identity so call-without-`new` still performs its active function's observable
prototype Get before selecting the same realm. A focused fixture pins fallback
identity, custom Object/Function/Array/Arguments prototypes, the explicit
one-Get path, abrupt and revocation routes, Error branding and message
installation; the immutable active intrinsic's common Get transition is
source-pinned. See
`docs/rust-rewrite/contracts/error-constructor-realm-prototype.md`.  This is a
static implementation checkpoint: focused runtime and current-pin verification
are deferred. At that checkpoint the adjacent native-error-family fallbacks
remained open; the next bounded seam addresses the six §20.5.5 families.

The six §20.5.5 NativeError constructors now retain their exact family through
the same construction boundary. A macro-backed, seven-kind
`ErrorMessageConstructorKind` is the single authority for `Error` plus
EvalError, RangeError, ReferenceError, SyntaxError, TypeError and URIError; each
row owns the builtin identity, active entry constructor, prototype global and
realm slot. The shared message/options/cause algorithm is emitted into every
typed body instead of erasing the family by wrapper-calling `Error`. The
construct dispatcher derives all seven direct-return entries from the closed
domain, so the typed body owns the sole observable `NewTarget.prototype` Get
and allocation. Primitive results select the matching required intrinsic only
after `GetFunctionRealm`; tagged Object, Function, Array and Arguments results
remain intact. Entry and created realms publish all seven slots, and created
constructors are self-backed for call-without-`new` active identity. The older
2026-08-13 Wasm artifact reported exactly these six failures in an otherwise
88/94 NativeErrors leaf; it selected the seam but is not current-SHA evidence.
The durable fixture and structural gate are recorded in
`docs/rust-rewrite/contracts/native-error-constructor-realm-prototypes.md`.
This remains a static-only checkpoint: focused runtime, the complete
NativeErrors leaf and current-pin verification are deferred. AggregateError now
has the narrow construction-phase integration described next; SuppressedError
construction is unchanged.

`AggregateError` construction now has explicit prepared-object phases for its
observable constructor and the two internal Promise.any origins. The constructor
allocates and brands first, then installs optional `message` and `cause` before
consuming the errors iterator; a private non-`Copy` token is the sole input to
the final `errors` installer. Promise.any uses a separate narrow producer that
performs no constructor message/options work and is called only for exhausted
input and the last reject element. This shares branded-object finalization
without routing Promise.any through the observable constructor or changing its
settlement algorithm. The focused
[`AggregateError` construction-phase contract](../docs/rust-rewrite/contracts/aggregate-error-construction-phases.md)
is implemented, independently reviewed and focused-verified as of 2026-08-23.
Under the shared eight-core and 22 GB cap, `cargo fmt --all -- --check`,
`cargo xc` and `git diff --check` are green; the AggregateError structure suite
passes `3/3`, the existing exact
`error_prototype_to_string_has_typed_ordered_observable_phases` library witness
passes `1/1`, and the constructor-properties and iterable-to-list CLI fixtures
each pass `1/1`. Four pinned AggregateError leaves and two pinned Promise.any
leaves pass both variants, for `12/12` Wasm-AOT executions with every failure
bucket at zero under `--jobs 1 --threads 1`. This does not close either full
tree, change Promise.any settlement semantics or change a published conformance
count.

Annex B `unescape` now materializes its result through one private UTF-16
output coordinator. Previously, each `%uXXXX` was encoded independently, so a
decoded lead/trail pair such as `%uD801%uDC01` became two WTF-8 surrogate
payloads and compared unequal to the equivalent astral String. The private,
non-`Copy`, `must_use` pending-lead witness delays the only ambiguous unit;
its consuming finalizer flushes a lone lead before it measures and packs the
completed output, while a following trail is combined into canonical UTF-8.
Decoded escapes and raw input share this path, including decoded/raw
boundaries, and raw astral scalars are projected through their two UTF-16 units
before materialization. The existing product fixture now pins paired and lone
code units directly, non-pairing, mixed-boundary, malformed/raw-multibyte and
raw-astral cases, while a structural test forbids the decoder from bypassing
the coordinator or packing before finalization. See
`docs/rust-rewrite/contracts/annexb-unescape-output.md`. This is a static-only
implementation freeze; focused runtime and current-pin Annex B gates remain
deferred to the centralized verification pass.

Error/global/Annex B metadata and legacy behavior still appears in other
exact-path materializations, dynamic-source cases remain visible exclusions,
and the full assigned trees lack current complete Wasm-AOT closure.

## Objective

Complete the foundational global properties/functions, native error families, Annex B extensions and standardized host-facing objects that do not belong to another feature lane. Keep ECMAScript semantics distinct from Test262 shell capabilities.

## Native Error objects

Implement the exact constructor/prototype behavior for every error family in the pinned suite:

- `Error`, `EvalError`, `RangeError`, `ReferenceError`, `SyntaxError`, `TypeError`, `URIError`;
- `AggregateError` and iterator consumption;
- `SuppressedError` and explicit-resource-management integration;
- any current standardized error additions in the pin.

Cover:

- call/construct and custom new target;
- realm-correct prototype fallback and cross-realm throws;
- `message`, `cause`, `errors`, `error` and `suppressed` own-property creation/order/descriptors;
- constructor/prototype `name`, `message`, `toString` and descriptors;
- arbitrary thrown JavaScript values through the T04 completion ABI;
- subclassing, proxies and side-effecting options/iterables.

A non-standard `stack` property may be offered as an extension only if it does not change standard own-key/descriptor tests or error construction order. Document and feature-gate its format.

## Global values and functions

Complete and install exact descriptors for:

- `globalThis`, `Infinity`, `NaN`, `undefined`;
- `isFinite`, `isNaN`, `parseFloat`, `parseInt`;
- `encodeURI`, `encodeURIComponent`, `decodeURI`, `decodeURIComponent`;
- Annex B `escape` and `unescape` where required;
- `eval` through T13;
- current standardized global constructors/functions not owned elsewhere.

URI algorithms must operate on UTF-16 code units/code points as specified, reject malformed surrogate/escape sequences with realm-correct `URIError`, and never delegate to platform URL encoding.

## Annex B grammar and semantics

Coordinate syntax/early errors with T07 and implement every Annex B feature enabled by the pinned suite, including as applicable:

- block-level function declaration web-legacy semantics and global instantiation interactions;
- catch-parameter/`var` compatibility exceptions;
- legacy octal literals/escapes and string/regexp grammar interactions;
- object-literal `__proto__` and `Object.prototype.__proto__` getter/setter;
- `Object.prototype.__defineGetter__`, `__defineSetter__`, `__lookupGetter__`, `__lookupSetter__`;
- String HTML wrapper methods;
- RegExp legacy `compile` and other Annex B RegExp semantics present in the pin;
- `escape`/`unescape`.

Do not enable Annex B semantics in modules or strict contexts where prohibited.

## Standardized host-facing objects

Implement or assign explicitly any pinned host-facing standard objects such as:

- `%AbstractModuleSource%` and source-phase import support, coordinated with T12;
- IsHTMLDDA behavior required by Test262, represented as a documented host exotic rather than ordinary truthy/falsy special cases scattered through operators;
- `$262` capabilities through T03, without installing Test262-only globals in normal product realms.

Host-only objects must use typed host capability interfaces and the same object/call/realm protocols as standard objects.

## Intrinsic/descriptor registry completion

Audit the declarative intrinsic registry from T06 against the current specification and suite manifest. Every global, constructor, prototype, method, accessor, symbol property and alias must have:

- owning realm/intrinsic ID;
- function `name` and `length`;
- callable/constructable flags;
- exact writable/enumerable/configurable attributes;
- constructor/prototype links and required identity aliases.

Generate a test that fails when Test262 references a known standard global absent from the registry.

## Acceptance criteria

- Full pinned native-error, global-function/global-value and Annex B trees are green.
- Error creation and throwing preserve exact realm/prototype/property order.
- URI functions pass malformed-surrogate, percent-escape and reserved-character cases.
- Annex B behavior is correctly gated by parse goal/strictness/context.
- IsHTMLDDA is implemented through one host-exotic contract, not operator-specific test hacks.
- Test262 shell globals are absent from ordinary product realms unless explicitly enabled.
- No focused static error/global/Annex B materialization remains for covered semantics.

## Required tests

```sh
cargo test -p lila-ir annex_b --quiet
cargo test -p lila-aot-wasm error_ --quiet
cargo test -p lila-aot-wasm global_ --quiet
cargo test -p lila-cli wasm_error --quiet
./target/debug/lila test262 run built-ins/Error --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/AggregateError --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run annexB --execution-backend wasm --timeout-ms 180000 --threads 4
```

Also run URI/global-value/global-function filters, SuppressedError, AbstractModuleSource and IsHTMLDDA-focused cases.
