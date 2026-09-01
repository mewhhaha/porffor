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

The current token-aware shortcut inventory assigns 5 observations to T24.
That census includes exact rewrite calls, source contract guards and selector
tables omitted by the historical line-oriented checkpoints below.

The 19 host-backed callables now also come from one macro-backed
`HostBuiltinId` row source. Each global row classifies its exposure, and that
closed exposure derives realm scope, so the compiler derives
complete/global/every-realm iteration and name lookup without parallel lists or
a raw `HTMLDDA` exclusion. Lowering,
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

Read-modify-write operations now honor that same authority. Arithmetic
compound assignment on a script-global `var` lowers to a global-property
operation in both the main owner and nested functions; the main frame cannot
read a stale left operand after a nested call mutates the property. The public
IR owner regression and the existing nested-callback ToLength CLI witness each
pass `1/1`, and the shared `cargo xc` checkpoint is green.

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

Created-realm Error-constructor inheritance now consumes that same seven-kind
authority exhaustively. `Error` retains its existing internal prototype, while
the six NativeError kinds share the exact existing stores for the created-realm
`Error` constructor and Function tag. Removing `PartialEq`/`Eq` from
`ErrorMessageConstructorKind` prevents equality plus a default branch from
silently classifying a later family. The strengthened structural gate pins the
seven rows, exact bounded match and materialization/self-backing/inheritance/
realm-slot/public-prototype order. This is an invariant-only change: the CLI
realm fixture passes `1/1`, and the unchanged six exact NativeError `proto.js`
leaves pass all `12/12` sloppy/strict Wasm-AOT executions with every failure
bucket at zero. The older broad structural unit also passes after its stale
`%TypeError.prototype%` diagnostic-name expectation was aligned with the
existing `TypeError.prototype` heap and module registry spelling; runtime
layout and behavior are unchanged.

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

The cause-options argument role is now a one-shot, non-derived
`ErrorCauseOptionsArgument`. Its consuming exhaustive projection preserves the
exact message-error index `1` and AggregateError index `2`, while making a
second by-value observation fail to compile. The Rust-lexical AggregateError
guard pins the six type uses, two local uses, three exact producers and the
complete cause-installer instruction and release order; it also follows the
current Promise.any allocation-context boundary without changing Promise
production. This is source-equivalent ownership hardening and does not expand
Error or AggregateError support. The structure target passes `3/3`, and the
exact Error and AggregateError constructor-properties CLI witnesses pass
`2/2`.

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

Boolean constructor and prototype-method metadata, plus the two legacy
conversion cases, no longer pass through handwritten Test262 source. The five
metadata files pass their unchanged sources and full `propertyHelper` harness
for 10/10 variants, and `S9.2_A6_T1.js` passes 2/2. `S9.2_A1_T1.js` remains
0/2 with the pre-existing explicit `NotImplemented/Runtime` result at
`eval("var x")`; it is neither green nor an expected failure. Removing these
rewrite authorities retired eight T24-owned semantic shortcut observations
without changing the T13 capability boundary.

The shared Boolean prototype receiver path now carries its result policy
through the private, non-derived
`BooleanPrototypeOperation::{ToString, ValueOf}` domain. The two public builtin
variants forward their named operation explicitly; unchanged primitive/boxed
Boolean validation precedes one exhaustive result match, so a future operation
cannot inherit `valueOf` or `toString` behavior from equality plus an `else`
default. The exact invariant and scoped non-claims are recorded in
[`boolean-prototype-operation.md`](../docs/rust-rewrite/contracts/boolean-prototype-operation.md).
The bounded structure target passes `4/4`, the exact boxed-builtin CLI owner
passes `1/1`, and four current-pin `toString`/`valueOf` leaves pass all `8/8`
sloppy/strict Wasm-AOT executions with every failure bucket at zero.
Independent dry review is clean.

Batch AS makes the outer family choice a private `BooleanBuiltin` and exposes
only three fixed Boolean entries to standard dispatch. The frozen 58-line
domain/emitter selection has SHA-256
`48961edd05a7a1789538b92ad90ed76232fad5156cec5144214122dd4c52eaab`;
restoring only the former enum and emitter visibility reproduces that source
exactly. At the 2026-08-28 Batch AS checkpoint, `cargo xc` is green, the
strengthened structure target passes `4/4`, the exact boxed-builtin CLI owner
passes `1/1`, and the four selected leaves pass all `8/8` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. This source-equivalent
boundary claims no new Boolean behavior, broader conformance or published
conformance-count change.

The `Infinity`, `NaN` and `undefined` descriptor cases also execute their
unchanged sources and full `propertyHelper` harness for 6/6 variants. Removing
their stale rewrite authority retired four more T24-owned semantic shortcut
observations.

The `Error.isError` descriptor case now executes its unchanged source and full
`propertyHelper` harness for 2/2 variants. Removing its stale rewrite authority
retired two more T24-owned semantic shortcut observations.

The Annex B `escape` and `unescape` descriptor, `length` and `name` cases now
execute their unchanged sources and full `propertyHelper` harness for 12/12
variants. Removing their stale rewrite authority retired four more T24-owned
semantic shortcut observations.

The six `Error.prototype` property, `length` and `name` metadata cases now
execute their unchanged sources and full `propertyHelper` harness for 12/12
variants. Removing their stale rewrite authority retired seven more T24-owned
semantic shortcut observations.

`built-ins/AggregateError/order-of-args-evaluation.js` now executes its
unchanged source and complete `promiseHelper.js` for 2/2 sloppy/strict
variants. Removing its reduced-helper branch retired one more T24-owned
semantic shortcut observation.

`built-ins/Error/message_property.js` now executes its unchanged source and
full `propertyHelper.js` for 2/2 sloppy/strict variants. Removing its inline
rewrite retired one more T24-owned semantic shortcut observation.

`built-ins/Error/cause_property.js` now executes its unchanged source and full
`propertyHelper.js` for 2/2 sloppy/strict variants. Removing its inline rewrite
retired one more T24-owned semantic shortcut observation.

`built-ins/Error/prop-desc.js` now executes its unchanged source and full
`propertyHelper.js` for 2/2 sloppy/strict variants. Removing its inline rewrite
retired one more T24-owned semantic shortcut observation.

`built-ins/Error/instance-prototype.js` now executes its unchanged source and
full `propertyHelper.js` for 2/2 sloppy/strict variants. Removing its inline
rewrite retired one more T24-owned semantic shortcut observation.

`built-ins/Error/prototype/no-error-data.js` now executes its unchanged source
and full harness for 2/2 sloppy/strict variants. Removing its exact branch from
the shared Error prototype rewrite retired one more T24-owned semantic shortcut
observation.

`built-ins/Error/prototype/S15.11.3.1_A1_T1.js` now executes its unchanged
source and full `propertyHelper.js` harness for 2/2 sloppy/strict variants.
Removing its exact branch from the shared Error prototype rewrite retired one
more T24-owned semantic shortcut observation.

`built-ins/Error/prototype/S15.11.3.1_A2_T1.js` now executes its unchanged
source and full harness for 2/2 sloppy/strict variants. Removing its exact
branch from the shared Error prototype rewrite retired one more T24-owned
semantic shortcut observation.

`built-ins/Error/prototype/S15.11.3.1_A3_T1.js` now executes its unchanged
source and full `propertyHelper.js` harness for 2/2 sloppy/strict variants.
Removing its exact branch from the shared Error prototype rewrite retired one
more T24-owned semantic shortcut observation.

`built-ins/Error/prototype/S15.11.3.1_A4_T1.js` now executes its unchanged
source and full harness for 2/2 sloppy/strict variants. Removing its exact
branch from the shared Error prototype rewrite retired one more T24-owned
semantic shortcut observation.

`built-ins/Error/prototype/S15.11.4_A1.js` now executes its unchanged source
and full harness for 2/2 sloppy/strict variants. Removing its exact branch from
the shared Error prototype rewrite retired one more T24-owned semantic shortcut
observation.

`built-ins/Error/prototype/S15.11.4_A2.js` now executes its unchanged source
and full harness for 2/2 sloppy/strict variants. Removing its exact branch from
the shared Error prototype rewrite retired one more T24-owned semantic shortcut
observation.

`built-ins/Error/prototype/S15.11.4_A3.js` now executes its unchanged source
and full harness for 2/2 sloppy/strict variants. Removing its exact branch from
the shared Error prototype rewrite retired one more T24-owned semantic shortcut
observation.

`built-ins/Error/prototype/S15.11.4_A4.js` now executes its unchanged source
and full harness for 2/2 sloppy/strict variants. Removing its exact branch from
the shared Error prototype rewrite retired one more T24-owned semantic shortcut
observation.

`built-ins/Error/prototype/constructor/S15.11.4.1_A1_T2.js` now executes its
unchanged source and full harness for 2/2 sloppy/strict variants. Removing its
exact branch from the shared Error prototype rewrite retired one more T24-owned
semantic shortcut observation.

`built-ins/Error/prototype/toString/called-as-function.js` now executes its
unchanged source and full harness for 2/2 sloppy/strict variants. Removing its
exact branch from the shared Error prototype rewrite retired one more T24-owned
semantic shortcut observation.

`built-ins/Error/prototype/toString/invalid-receiver.js` now executes its
unchanged source and full harness for 2/2 sloppy/strict variants. Removing its
last exact branch and the now-unused shared Error prototype rewrite entrypoint
retired two more T24-owned semantic shortcut observations; the generated
inventory retains 25 T24 observations.

`built-ins/AggregateError/length.js` now executes its unchanged source and full
`propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic harness
and descriptor backend already preserve the required value and attribute
checks, so removing the stale exact materialization retired one more T24-owned
semantic shortcut observation; the generated inventory retains 24 T24
observations.

`built-ins/AggregateError/name.js` now executes its unchanged source and full
`propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic harness
and descriptor backend already preserve the required value and attribute
checks, so removing the stale exact materialization retired one more T24-owned
semantic shortcut observation; the generated inventory retains 23 T24
observations.

`built-ins/AggregateError/prop-desc.js` now executes its unchanged source and
full `propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic
harness and descriptor backend already preserve the global binding type and
attribute checks, so removing the stale exact materialization retired one more
T24-owned semantic shortcut observation; the generated inventory retains 22
T24 observations.

`built-ins/SuppressedError/length.js` now executes its unchanged source and
full `propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic
harness and descriptor backend already preserve the required value and
attribute checks, so removing the stale exact materialization retired one more
T24-owned semantic shortcut observation; the generated inventory retains 21
T24 observations.

`built-ins/SuppressedError/name.js` now executes its unchanged source and full
`propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic harness
and descriptor backend already preserve the required value and attribute
checks, so removing the stale exact materialization retired one more T24-owned
semantic shortcut observation; the generated inventory retains 20 T24
observations.

`built-ins/SuppressedError/prop-desc.js` now executes its unchanged source and
full `propertyHelper.js` harness for 2/2 sloppy/strict variants. The generic
harness and descriptor backend already preserve the global binding type and
attribute checks, so its stale exact materialization is gone. Its multiline
predicate was not a separate generated observation in that line-oriented
scanner, leaving the historical inventory at 320 exact entries, 242 semantic
shortcuts and 20 T24 observations.

`built-ins/Error/isError/errors.js` now executes its unchanged source and full
assert harness for 2/2 sloppy/strict variants, including the pinned
`SuppressedError` assertion that the stale rewrite omitted. Removing the exact
predicate and its reduced assert retired two T24-owned semantic shortcut
observations; the generated inventory retains 318 exact entries, 240 semantic
shortcuts and 18 T24 observations.

`built-ins/Error/isError/non-error-objects.js` now executes its unchanged
source and full assert harness for 2/2 sloppy/strict variants, including the
pinned `SuppressedError` constructor assertion that the stale rewrite omitted.
Removing its reduced assert retired one T24-owned semantic shortcut
observation; the generated inventory retains 317 exact entries, 239 semantic
shortcuts and 17 T24 observations.

The six `built-ins/NativeErrors/*/length.js` files now execute their unchanged
sources and full `propertyHelper.js` harness for 12/12 sloppy/strict variants.
The generic harness and descriptor backend already preserve every constructor
length value and attribute check, so removing the shared exact length branch
retired one T24-owned semantic shortcut observation; the generated inventory
retains 316 exact entries, 238 semantic shortcuts and 16 T24 observations.

The six `built-ins/NativeErrors/*/name.js` files now execute their unchanged
sources and full `propertyHelper.js` harness for 12/12 sloppy/strict variants.
The generic harness and descriptor backend already preserve every constructor
name value and attribute check, so removing the shared exact name branch
retired one T24-owned semantic shortcut observation; the generated inventory
retains 315 exact entries, 237 semantic shortcuts and 15 T24 observations.

The six `built-ins/NativeErrors/*/prop-desc.js` files now execute their
unchanged sources and full `propertyHelper.js` harness for 12/12 sloppy/strict
variants. The generic harness and descriptor backend already preserve every
global constructor binding value and attribute check, so removing the shared
exact global-descriptor branch retired one T24-owned semantic shortcut
observation; the generated inventory retains 314 exact entries, 236 semantic
shortcuts and 14 T24 observations.

The six `built-ins/NativeErrors/*/prototype.js` files now execute their
unchanged sources and full assert plus `propertyHelper.js` harness for 12/12
sloppy/strict variants. The generic harness and descriptor backend already
preserve prototype identity and every constructor `prototype` attribute check,
so removing the shared exact branch retired one T24-owned semantic shortcut
observation; the generated inventory retains 313 exact entries, 235 semantic
shortcuts and 13 T24 observations.

The six `built-ins/NativeErrors/*/prototype/constructor.js` files now execute
their unchanged sources and full assert plus `propertyHelper.js` harness for
12/12 sloppy/strict variants. The generic harness and descriptor backend
already preserve constructor identity and every prototype `constructor`
attribute check, so removing the shared exact branch retired one T24-owned
semantic shortcut observation; the generated inventory retains 312 exact
entries, 234 semantic shortcuts and 12 T24 observations.

The six `built-ins/NativeErrors/*/prototype/message.js` files now execute their
unchanged sources and full assert plus `propertyHelper.js` harness for 12/12
sloppy/strict variants. The generic harness and descriptor backend already
preserve the empty-string value and every prototype `message` attribute check,
so removing the shared exact branch retired one T24-owned semantic shortcut
observation; the generated inventory retains 311 exact entries, 233 semantic
shortcuts and 11 T24 observations.

The six `built-ins/NativeErrors/*/prototype/name.js` files now execute their
unchanged sources and full `propertyHelper.js` harness for 12/12 sloppy/strict
variants. The generic harness and descriptor backend already preserve every
prototype name value and attribute check. Removing the final exact branch also
deleted the now-unused NativeError path parser, rewriter and dispatcher entry,
retiring three T24-owned semantic shortcut observations; the generated
inventory retains 308 exact entries, 230 semantic shortcuts and 8 T24
observations.

The Annex B `String.prototype` HTML helpers, `substr`, and `trimLeft`/`trimRight`
aliases now execute all 48 metadata files as unchanged pinned sources with the
full `propertyHelper.js` harness for 96/96 sloppy/strict variants. The generic
harness and descriptor backend preserve the method descriptors, arities, and
names, including the `trimStart`/`trimEnd` alias names. Removing the obsolete
shared rewriter and dispatcher entry retired four T24-owned semantic shortcut
observations; the generated inventory retains 304 exact entries, 226 semantic
shortcuts and 4 T24 observations.

The exponentiation `order-of-evaluation.js` and `bigint-toprimitive.js` cases
now execute their unchanged pinned sources and full Test262 assertion harness
for 4/4 sloppy/strict variants. Removing their shared reduced `assert.throws`
and `assert.sameValue` injection retired two T24-owned semantic shortcut
observations; the generated inventory retains 302 exact entries, 224 semantic
shortcuts and 2 T24 observations.

The global URI dispatcher now carries encode/decode direction together with
the existing `UriCodecKind` in one closed `UriBuiltin` value. All six standard
producer spellings remain unchanged; the four URI codec spellings use named
associated constants. One exhaustive match owns string coercion, Annex B
dispatch and URI codec forwarding without `if` or `unreachable!` branches. The
bounded `uri_builtin_codec_domain_structure` target pins the four producer
mappings and the single exhaustive consumer. The structure target passes
`2/2`; the focused paired URI/component and Annex B escape/unescape CLI
fixtures each pass `1/1`, and `cargo xc` is green. The Wasm golden and broad
conformance gates remain deferred to the centralized verification pass. The
647-artifact Wasm golden has an empty recursive pre/post diff.

The URI codec identity is now capability-free. `UriBuiltin` consumes its
closed operation once, URI encoding borrows the codec choice at its repeated
unescaped-code-point projection, and URI decoding consumes the choice through
an exhaustive `Uri`/`Component` match. The former equality branch and both
enums' `Clone`, `Copy`, `PartialEq` and `Eq` capabilities are gone, so a future
codec cannot silently inherit Component decoding. The strengthened
`uri_builtin_codec_domain_structure` guard pins both exhaustive projections
and the capability boundary. This source-equivalent closure changes no URI
algorithm or published conformance count. Its bounded contract is
`docs/rust-rewrite/contracts/uri-codec-capability.md`.

Batch AP makes `UriBuiltin`, its codec constants and the raw URI compiler
private to `builtins/uri.rs`. The standard dispatcher sees only six fixed
semantic wrappers and cannot import, construct or pass the raw operation
policy. The strengthened URI structure target and adjacent Annex B
output-coordination guard pass `4/4` and `3/3`, both exact URI and Annex B CLI
controls pass `1/1`, and `cargo xc` is green. This source-equivalent boundary
changes no URI algorithm and claims no new behavior or conformance result.

Annex B nested-function analysis now carries its direct-declaration decision
as the private, non-`Copy`
`AnnexBDirectFunctionCollection::{Skip, Record}` domain. The owner body and
already-grouped switch cases select `Skip`; ordinary blocks plus try, catch and
finally bodies select `Record`. One exhaustive match records direct functions
before recursive traversal, while the switch still aggregates every case
before visiting any case. This is an invariant-only migration with no emitted-IR,
Wasm or conformance change; the focused contract and recursive producer guard
live in
`docs/rust-rewrite/contracts/annex-b-direct-function-collection.md`.
Its structure and existing IR witnesses pass `3/3`, the CLI witness passes
`1/1`, and the three exact Wasm-AOT Test262 leaves pass `3/3` with all failure
buckets zero. `cargo xc` and repository checks are green, and independent dry
review is clean.

The coercing global numeric predicates now retain their family decision in the
private, non-derived `GlobalNumericBuiltin::{IsFinite, IsNaN}` domain. Removing
its unused clone, copy, debug and equality capabilities prevents a later
predicate from acquiring either result policy through an equality/default
shortcut. Both exact standard producers and the two exhaustive emitter
matches preserve the existing coercion, infinity checks, Boolean publication
and local order. The bounded structure target passes `4/4`; the exact
`isFinite` false-for-NaN/infinities and `isNaN` true-for-NaN leaves pass all
`4/4` sloppy/strict Wasm-AOT executions with every failure bucket at zero. See
[`global-numeric-builtin-capability.md`](../docs/rust-rewrite/contracts/global-numeric-builtin-capability.md).
This source-equivalent checkpoint does not close either full tree or change a
published conformance count. Independent review, the shared workspace compile
and all repository gates are green.

Batch AR makes the raw global numeric domain and family emitter private to
`global_numeric.rs`. Standard dispatch sees only fixed `isFinite` and `isNaN`
entries and cannot import, construct or pass `GlobalNumericBuiltin`. The frozen
47-line domain/emitter body has SHA-256
`3057db4769633e0293b564bd3e61383677777bb91780936f92b2dd21fb80cda2`;
normalizing only the narrowed visibilities reproduces that hash exactly. At the
2026-08-28 Batch AR checkpoint, `cargo xc` is green, the strengthened structure
target passes `4/4`, and the exact pinned `isFinite` and `isNaN` leaves pass all
`4/4` sloppy/strict Wasm-AOT executions with every failure bucket at zero. This
source-equivalent boundary claims no new global numeric behavior, broader
conformance or published conformance-count change.

The Error-family builtin dispatcher now enters eleven fixed operations whose
private, non-derived `ErrorBuiltin` authority selects nine exact
`NativeErrorKind` constructors, `Error.isError` and
`Error.prototype.toString`. The sole raw consumer exhausts the three outer rows
and all nine constructor families without equality, wildcard or default
routing, so adding a family or observing a dispatch value twice requires an
explicit source change. The source-equivalent invariant and its 16-mention
Rust-lexical census are recorded in
[`error-builtin-dispatch-ownership.md`](../docs/rust-rewrite/contracts/error-builtin-dispatch-ownership.md).
The focused structure target passes `4/4`; runtime witnesses remain with the
existing family fixtures, and this checkpoint does not claim broader T24 or
Error-tree completion.

Batch AQ makes the raw `ErrorBuiltin`, its `NativeErrorKind` constructor
selection and the raw error emitter private to `builtins/errors.rs`. Eleven
fixed sibling-visible entries mean standard dispatch can no longer import,
construct or pass the raw policy. At the 2026-08-28 Batch AQ checkpoint,
`cargo xc` is green, the strengthened ownership structure target passes `4/4`,
and the exact constructor-properties, cross-realm `Error.isError` and
`Error.prototype.toString` CLI controls pass `3/3`. This source-equivalent
boundary claims no new Error behavior and no Batch AQ Test262, semantic-golden
or published conformance-count result.

Runtime-created native errors now use `NativeErrorKind` as the sole authority
for their published diagnostic name. The private paired name/message publisher
no longer accepts an interchangeable raw string: both message-bearing paths
forward the kind already used for object and prototype selection, while the
message-less path names `NativeErrorKind::TypeError` directly. Publication
retains its existing position after object creation and before the Throw
completion. The bounded authority and producer law is recorded in
[`thrown-error-diagnostic-kind-authority.md`](../docs/rust-rewrite/contracts/thrown-error-diagnostic-kind-authority.md).
This does not change emitted Wasm or the separate property-reading path for
user-thrown values, and it does not claim broader Error-tree or T24 closure.

`built-ins/Error/isError/errors-other-realm.js` now executes its unchanged
pinned source with the active LocalMerged `sta.js` Realm capability and the
LocalMerged `assert.js` same-value prelude for both sloppy and strict variants.
Both Wasm-AOT executions pass (`2/2`) with every failure and non-success bucket
at zero, including the pinned `SuppressedError` assertion that the stale rewrite
omitted. Removing the exact source rewrite retired one semantic observation;
that checkpoint's token-aware inventory had 403 entries, 255 semantic shortcuts
and 19 T24-owned observations. The createRealm string-pool boundary now interns
its host-published `evalScript` key even when user source does not spell that
key, and the Proxy Realm witness is guarded against masking this invariant.

The fourteen exact AggregateError and SuppressedError core-property branches
now preserve their pinned sources. A 14-source matrix pins both sloppy and
strict bytes, rejects every other rewrite/unsupported route, and verifies the
applicable LocalMerged and VendoredHarness prelude order. Six raw Wasm-AOT
cohorts pass `36/36`: all `28/28` executions owned by the retired branches plus
eight adjacent prototype controls, with every failure and non-success bucket at
zero. Removing those branches leaves 389 exact observations, 241 semantic
shortcuts and 5 T24-owned observations. Those five remaining observations are
explicit `eval` or cross-Realm Function-constructor substitutions owned by the
T13 dynamic-source boundary; they cannot honestly execute unchanged in emitted
Wasm.

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

The host-builtin catalog now carries only `HostBuiltinExposure` for each global;
one private exhaustive projection derives `EveryRealm` for ECMAScript globals
and `EntryRealmOnly` for product extensions and Test262 capabilities. Exposure
and realm scope can therefore no longer contradict each other in a catalog
row. `host_builtin_surface_domain_structure` passes `3/3`, both focused
`lila-ir` catalog/policy tests pass `1/1`, and the existing Wasm created-realm
builtin-function-prototype CLI witness passes `1/1`. The shared workspace
compile and every repository policy gate pass, and all 648 Wasm-golden
artifacts are byte-identical to the post-Iterator baseline. No broader
conformance run was performed for this catalog-only invariant.
