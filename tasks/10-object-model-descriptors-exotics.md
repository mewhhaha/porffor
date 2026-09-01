# T10 — Object model, descriptors and exotic-object protocol

**Status:** In progress — canonical descriptor lattice is consumed; exotic closure remains

**Parallel group:** Core foundations  
**Depends on:** T04, T05, T06  
**Blocks:** T11, T16-T24

## Current repository state

`Object.prototype.toString` and the intrinsic fallback used by
`Array.prototype.toString` now share the existing typed, Proxy-aware `IsArray`
authority before either reads `@@toStringTag`. Both emitters cover the complete
builtin-tag set, including boxed primitives and the Error, Date and RegExp
internal brands, instead of inferring Array identity from the outer value tag.
The focused product fixture covers direct and nested Proxy-wrapped Arrays and
revoked Proxies; the unchanged pinned Array fallback case passes both ordinary
Wasm-AOT variants with its full harness.

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

The `DescriptorSourceText` builder now names all six static attribute choices:
`writable`/`non_writable`, `enumerable`/`non_enumerable`, and
`configurable`/`non_configurable`. Its public surface has no boolean parameter,
so explicit false remains a present field without leaving an unlabelled flag at
the call site. The module-namespace accessor emitter selects `enumerable()` and
`non_configurable()` directly; generated descriptor text and field order are
unchanged. The bounded contract is
`docs/rust-rewrite/contracts/descriptor-source-text-attribute-selection.md`.
The recursive structure target passes `4/4`, the explicit-false rendering
witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green.

Static stored descriptor attributes now cross their crate-visible boundary as
one closed `StoredPropertyAttributes::{Data, Accessor}` domain. Each producer
names its variant and fields instead of passing positional booleans, while the
Accessor variant cannot represent `[[Writable]]`. One exhaustive projection
delegates to the existing `DescriptorWord` constructors, preserving the exact
stored bits. The bounded contract and recursive guard are recorded in
`docs/rust-rewrite/contracts/stored-property-attributes.md`. The recursive
structure target passes `4/4`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. Semantic goldens were not
rerun for this source-equivalent type hardening.

Ordinary accessor definitions now cross their three object-definition entries
as one nonempty
`AccessorDescriptorLocals::{Getter, Setter, GetterAndSetter}` domain. Distinct
`AccessorGetterLocals` and `AccessorSetterLocals` roles retain the tagged
payload/tag carrier and make endpoint transposition a type error; the domain has
no empty state and no independent accessor-kind Boolean. One exhaustive match
projects `[[Get]]` and `[[Set]]` presence before descriptor validation. The
bounded contract and recursive guard are recorded in
`docs/rust-rewrite/contracts/accessor-descriptor-local-roles.md`. The structure
target passes `4/4`, the three exact CLI behavior controls pass `3/3`, and the
shared `cargo xc` gate is green. No semantic golden, broad descriptor suite or
Test262 baseline was rerun for this source-equivalent type hardening.

The two ordinary-object `Object.defineProperty` branches now construct one
closed `ObjectDefinePropertyDescriptorLocals::{Data, Accessor}` value. Each
variant carries exactly its four legal run-time-presence fields, and one
ownership-consuming exhaustive projection produces the validated descriptor.
The opposite descriptor side is structurally absent, so a mixed carrier/presence
pair cannot cross the branch boundary. The former sixteen-argument
`emit_object_define_entry` adapter and `presence_from_positional` translator are
gone; the only remaining `from_runtime_checked()` Wasm source obligation is the
distinct Arguments `callee` boundary. The focused contract and recursive guard
are recorded in
`docs/rust-rewrite/contracts/object-define-property-descriptor-roles.md`.
The structure target passes `4/4`, the exact object-descriptor CLI fixture
passes `1/1`, and the shared `cargo xc` gate is green. No semantic golden,
broad descriptor suite or Test262 baseline was rerun for this source-equivalent
boundary closure.

The stored-attribute closure now has no alternate raw constructor path.
`StoredPropertyAttributes` lives beside `DescriptorWord`; its sole exhaustive
projection can call the now heap-private positional constructors, while all
fourteen data and two accessor producers outside `heap.rs` must name their
variant and every legal field. Accessor producers cannot spell `writable`, and
future external calls to `DescriptorWord::of_data` or `of_accessor` fail to
compile. The strengthened recursive guard and existing bounded contract remain
`docs/rust-rewrite/contracts/stored-property-attributes.md`. Batch R uses dry
source checks during implementation. Its focused structure target passes
`4/4`, `cargo xc` is green, and semantic goldens were not rerun for this
source-equivalent type hardening.

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

`ProxyRevocationRoute` is now a crate-private, capability-free one-shot
authority. Its ten exact producers select the existing
current-function-Realm, active-handler, object-mutation-Realm-to-active-handler
or current-completion policy, and the shared live-slot reader consumes that
choice through one exhaustive match before exposing target or handler locals.
The route census is recorded in
[`proxy-revocation-route-ownership.md`](../docs/rust-rewrite/contracts/proxy-revocation-route-ownership.md);
the prior eight-producer ownership target passed `4/4`, and exact
define-property, get-prototype and delete-property Wasm-AOT fixtures each
passed `1/1`. Verification of the expanded ten-producer census, the
SetPrototypeOf Realm-aware active-handler fixture and the direct Reflect Set
current-function-Realm fixture is pending. Adding the direct Reflect producer
preserves its existing route; SetPrototypeOf separately corrects its revoked
error Realm. Neither closes the broader task.

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

The module-boundary guard now makes that ownership durable by pinning the
complete Proxy-Set projection, typed target/key/incoming roles, its sole direct
descriptor projection, the exact `SameValue` and undefined-setter consumers,
and one active exact CLI registration. On 2026-08-24, the CLI witness passed
`1/1`. At current Test262 pin
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the six selected unrewritten Proxy
Set invariant files passed all `12/12` ordinary Wasm-AOT executions with every
failure bucket at zero. This closes evidence for the post-trap direct-target
projection only; trap lookup/fallback, recursive Proxy targets, module
namespaces, the full 27-file/54-variant Proxy Set subtree and complete
TypedArray `[[Set]]` remain open.

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
separate from the full `[[HasProperty]]` dispatcher. Proxy `[[Get]]` and
`[[Set]]` have typed direct-target projections, but neither closes recursive
descriptor lookup through a Proxy target.

Class constructors now install their own `prototype` data property with the
class-specific all-false attribute tuple. Computed public static class elements
retain the explicit key guard introduced while known-present compatibility was
still open. The exact descriptor witness passes `2/2` Wasm-AOT executions.

LN10 is now closed. The ordinary and stored descriptor validators project each
field through the private closed
`DescriptorCompatibilityPredicate::{Never, Always, AtRuntime}` domain.
`Presence::Absent` emits no check, `Presence::Present` emits the check
unconditionally, and `Presence::Runtime` gates it on the supplied Wasm local.
The same projection consumes `KindTerms`, so a statically-known data/accessor
side can no longer skip kind-change validation. The predicate has no copy,
debug, equality or default capability, and its mapping is exhaustive. Existing
entry validation remains inside the successful property lookup, so missing
property creation is unchanged.

The bounded regression passed its six structure checks and exact CLI fixture.
The fixture covers ordinary objects and non-index Array named properties across
attribute rejection, `SameValue` for `NaN` and signed zero, accessor identity,
kind changes, absent carry-over, configurable updates and missing-property
creation. The computed static `prototype` case remains guarded earlier and does
not reach the `Presence::Present` validator route. Seven selected pinned
`Object.defineProperty` leaves (`15.2.3.6-4-85`,
`-87`, `-12`, `-14`, `-277`, `-281` and `-285`) passed `14/14`
strict/non-strict Wasm-AOT executions with every failure bucket zero. There is
currently no user-reachable Array named-property operation that both supplies a
known-present descriptor and redefines an existing entry, so that branch is
covered structurally rather than reported as an execution witness.
The following shared semantic golden passed `2/2` in 676.81 seconds with 683
dumps, adding only this fixture and removing none. All 682 retained dumps
preserve every non-accounting summary after normalizing the main-local and
emitted-size fields; their byte hashes change because the common descriptor and
fresh-error emitters changed.

This is still a foundation, not task closure. Array application paths,
remaining arguments special/named descriptors, several builtin/exotic emitters and lowering shape
facts still consume derived raw words or parallel positional forms. The
ordinary `Object.defineProperty` branches retain their emitted run-time 6.2.6.5
step-9 check before constructing their closed descriptor roles, and the
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
Proxy-Set direct-descriptor fixture is green at `1/1`, and its selected
current-pin invariant cohort is green at `12/12`. The focused
own-descriptor-predicate fixture, strengthened bootstrap-planning checkpoint
and six-file current-pin cohort are green at `1/1`, `1/1` and `12/12`,
respectively. The
`Object.prototype.toLocaleString` Invoke lane is green at `3/3` for its
structure target, `1/1` for its exact CLI fixture and `4/4` for its four
current-pin `onlyStrict` Wasm-AOT leaves, with every failure bucket at zero. A
complete current-pin Wasm-AOT Object/descriptor subtree run has not been
performed.

Array named-property storage now receives a closed
`ArrayNamedStringKeySelection` from the four `Object.getOwnPropertyNames` and
`Object.keys` count/write producers. The two storage consumers project `All`
and `EnumerableOnly` directly and exhaustively, so result sizing and key
writing cannot silently disagree through a raw Boolean. A finite sparse-Array
witness fixes integer-index, `length`, enumerable/non-enumerable named-string
ordering, Symbol exclusion and accessor non-observation. Proxy own-key and
TypedArray integer-indexed paths remain explicitly outside this boundary. The
bounded structure target and exact CLI witness are green at `3/3` and `1/1`.
The following workspace semantic golden passes `2/2` in 704.11 seconds with
666 dumps, adds only this witness, removes none and preserves all 665 retained
non-accounting summaries.

Batch AP makes the raw `ArrayNamedStringKeySelection` and its two exhaustive
consumers private to `builtins/array.rs`. Four fixed count/write operations are
the only sibling-visible boundary, so Object key producers cannot construct or
pass the raw selection. The strengthened structure target passes `4/4`, the
exact Array named-string selection CLI passes `1/1`, and `cargo xc` is green;
this source-equivalent tightening adds no new Array or Object behavior.

The complete `Object.getOwnPropertyDescriptor` compiler now lives in the
private `builtins/object/get_own_property_descriptor.rs` owner. Its 1,431-line
ordinary and exotic descriptor family moved together, leaving one private
module declaration in the Object parent and one fixed standard-dispatch call.
The focused owner contract is
[`object-get-own-property-descriptor-owner.md`](../docs/rust-rewrite/contracts/object-get-own-property-descriptor-owner.md).
The owner, Arguments neighbor, Array neighbor and Proxy ownership structure
targets pass `4/4`, `4/4`, `3/3` and `4/4`; the exact object-descriptor CLI
passes `1/1`; and `cargo xc` is green. This source-equivalent move has
no new descriptor behavior or conformance claim.

The complete `Object.getOwnPropertyDescriptors` compiler now lives in the
private `builtins/object/get_own_property_descriptors.rs` owner. Its 182-line
coercion, own-key enumeration, per-key descriptor lookup, Realm allocation,
key conversion and result-definition family moved together, leaving one
private module declaration and one fixed standard-dispatch call. Normalizing
the child entry visibility reproduces the frozen source SHA-256 exactly. The
focused contract is
[`object-get-own-property-descriptors-owner.md`](../docs/rust-rewrite/contracts/object-get-own-property-descriptors-owner.md).
At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the private-owner
structure target passes `4/4`, and the exact
`built-ins/Object/getOwnPropertyDescriptors/normal-object.js` leaf passes both
sloppy and strict Wasm-AOT executions (`2/2`) with every failure bucket at
zero. This source-equivalent move claims no new Object behavior, broader
Test262 result or published conformance-count change.

The complete `Object.assign` compiler now lives in the private
`builtins/object/assign.rs` owner. Its 262-line target/source coercion, own-key
enumeration, descriptor observation, enumerable filtering, source `Get`, target
`Set`, abrupt-completion and local-release family moved together. Normalizing
only the child entry visibility reproduces the frozen source SHA-256
`65680b329345d9833065718b97a828484f464208d19bc5ff09d7f6ad3a46f6cd`.
The focused contract is
[`object-assign-owner.md`](../docs/rust-rewrite/contracts/object-assign-owner.md).
At the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the owner structure
target passes `4/4`, and the exact `Target-Object.js` Wasm-AOT leaf passes `2/2`
with every failure bucket at zero. This source-equivalent move claims no new
Object behavior, broader Object conformance or published conformance-count
change.

Six Object builtins now enter three closed compiler policy domains:
`EnumerableOwnProperties`, `IntegrityTest` and `PrototypeLookup`. The domains
derive no capability at all, and each compiler borrows its one owned policy
through every decision. Entries and Values independently and exhaustively
select their nullish diagnostic and result-element shape; isSealed and isFrozen
exhaustively select whether descriptor writability is relevant; and the
prototype accessor lookup methods retain their exhaustive getter/setter field
selection. The former `include_keys` and `check_writable` Boolean projections
are gone, so a future variant cannot inherit an existing operation's policy or
be copied into a second path without compile review. The bounded contract is
recorded in `docs/rust-rewrite/contracts/object-builtin-policy-domains.md`. At
the 2026-08-28 Batch W checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact CLI witness passes `1/1`.
The shared 678-dump semantic golden passes `2/2` in 722.99 seconds, adds this
witness plus the independent Array.fromAsync callback-Realm, Promise-mode and
Set-domain witnesses, removes none and leaves all 674 retained dumps equal
after accounting normalization. Broad Test262 verification remains deferred.

The getter/setter half of that policy boundary now has one private
`builtins/object/prototype_lookup.rs` owner. The raw `PrototypeLookup` domain and
its sole 132-line compiler are absent from both the Object parent and standard
dispatcher; two fixed builtins-visible wrappers privately construct the exact
getter and setter variants. The visibility-normalized domain and compiler retain
SHA-256 `7ca467738a2dfd39524325c1fac34084715cbb79a46baf9a862dd54737778a57`
and `f6ba5e5158701597301fc843f203a2e3997665afc54bffefd908fbe9d866876f`.
The 155-line child is
`bf4ec5630203d7a10b6982ac101dcef9192f693f1e819e4c0e2bb79f0f06c2ec`,
and the reduced 8,765-line parent is
`82437af076110c4151c1a82943c93054d89c24ef0caf19b217ad946d77c0fa`.
The exact parent/child/dispatcher guard retains the exhaustive getter/setter
projection and now makes parent-side policy reconstruction a structural
failure. At the Batch AL checkpoint, `cargo xc` is green, the structure target
passes `4/4`, and the policy-domain and full prototype-accessor CLI witnesses
pass `2/2`. No Test262 leaf or semantic golden was required or run.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The integrity-test half of the policy boundary now has one private
`builtins/object/integrity_test.rs` owner. The raw `IntegrityTest` domain and its
sole 198-line compiler are absent from both the Object parent and standard
dispatcher; two fixed builtins-visible wrappers privately construct the exact
Sealed and Frozen variants. The visibility-normalized domain and compiler retain
SHA-256 `0f81ace14c7caea6494f3c6ac21f2b0bba61ba10bb10637eb79e10a22b0f2d64`
and `7263b51a1dcfdcd4eb0bc1a1bcb6569516652347ec3b8bfe773c187bddb7bf79`.
The 221-line child is
`ad029d42fc1fdeb65ae03ac765c7186a7bd7efa8cbe7da51e932f9733ad53d93`,
and the reduced 8,562-line parent is
`67232c7c756062fa9eb24d83506a750e147744b087464ce77deccd4243b27cee`.
The exact parent/two-child/dispatcher guard retains the sole exhaustive
writability decision and makes parent-side policy reconstruction a structural
failure. At the Batch AM checkpoint, `cargo xc` is green, the structure target
passes `4/4`, and the exact policy-domain CLI witness passes `1/1`. No Test262
leaf or semantic golden was required or run.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The enumerable-own-properties half of the policy boundary now has one private
`builtins/object/enumerable_own_properties.rs` owner. The raw
`EnumerableOwnProperties` domain and its sole 309-line compiler are absent from
both the Object parent and standard dispatcher; two fixed builtins-visible
wrappers privately construct the exact Entries and Values variants. The
visibility-normalized domain and compiler retain SHA-256
`791b3ae06c58f2ed8ca870d44a823882e4a7a3262c0eb528d323169116c54dc4`
and `2feb0f6ab4e8fa5c68e75311a45f637fcebc811ad26aad38bd2abb7b5db7ce06`.
The 338-line child is
`8d47fee7765fbcb0691be6b4f1df1de876e662db8552b29f8599a9a2a37d7777`,
and the reduced 8,248-line parent is
`3229401d4da5d26395572f184167c246442d67b2c7121d79adce385b33c7b3b1`.
The exact parent/three-child/dispatcher guard retains both exhaustive policy
decisions and makes parent-side policy reconstruction a structural failure. At
the Batch AN checkpoint, `cargo xc` is green, the policy and Realm structure
targets pass `4/4` and `1/1`, and the exact policy-domain CLI witness passes
`1/1`. No Test262 leaf or semantic golden was required or run. Final formatter,
diff, module-boundary, task-plan and 240-entry shortcut-inventory gates are
green.

The complete `Object.defineProperty` descriptor carrier, Arguments-specialized
helper and compiler family now lives in the private
`builtins/object/define_property.rs` owner. The Object parent retains only the
module declaration, while the standard dispatcher keeps its one fixed builtin
call. The 2,500-line child and reduced 5,751-line parent have SHA-256
`01ea9de92ace5f710bc6fcea6b7b4d64326e8d726e165920a06fdb1d8368b4c6`
and `d8e910a3b8e2edcd7ab4e9fd6ee19507d86de6fbd6721d1da29655ffef817a53`.
At the Batch AO checkpoint, `cargo xc` is green, the strengthened owner target
and five descriptor/proxy neighbors pass `25/25`, and the exact
object-descriptor CLI passes `1/1`. This is a source-equivalent ownership move
and adds no descriptor behavior or conformance claim.

The three own-descriptor predicate builtins now have one private
`builtins/object/own_descriptor_predicate.rs` owner. The Object parent cannot
name, construct, import or project the capability-free
`OwnDescriptorPredicateBuiltin`; its exact
`ObjectHasOwn`, `PrototypeHasOwnProperty` and
`PrototypePropertyIsEnumerable` variants cross three borrowed exhaustive
decisions for receiver/argument acquisition, coercion and nullish-error order,
and result projection. Clone, copy, debug, default, comparison, ordering and
hashing capabilities are absent, so those decisions cannot silently diverge
through a copied policy or equality shortcut. The normalized compiler body and
the three wrapper selections retain SHA-256
`320062e113be88c36172a2f864dae434f563a56fe9c4d663cc7a8571c719be02`
and `54c807ccd77513cc4b2e65e460f7df84d87efd2231d0d37e59593a794c88edba`.
The moved five-line domain and 191-line raw compiler retain SHA-256
`36ed9747dec1c589dd32f763a7bc907fc84d3070988bd0fef7641b08e6138098`
and `05f279033ab151a2c156cdb76ca0da20ad330d953c9ccae8ea055b9d9fbce4a1`;
the resulting 230-line child has SHA-256
`f4db50dd3eb3ba382999dec0dfd9fc578253de1328bbaf41c2a48a0b73b827ba`.
At the 2026-08-28 Batch X checkpoint, `cargo xc` is green, the structure target
passes `4/4`, and the exact CLI witness passes `1/1`. The exact invariant and
nonclaims are recorded in
`docs/rust-rewrite/contracts/object-own-descriptor-predicate-kind.md`.
Batch AK shared `cargo xc` is green, the structure target passes `4/4`, and the
exact CLI witness passes `1/1`; this source-equivalent owner move adds no
Test262 or semantic-golden claim. Final formatter, diff, module-boundary,
task-plan and 240-entry shortcut-inventory gates are green.

Ordinary descriptor rejection, `CreateDataPropertyOrThrow`, non-writable and
non-extensible Set, and Proxy-set failure paths now consume one exhaustive
three-source/two-authority Realm projection when they synthesize TypeError.
Outlined Set helpers receive a compiler-owned trusted environment-or-zero
argument, while ordinary lexical environments cannot be reinterpreted as Realm
metadata. Setter and Proxy-trap throws still propagate unchanged. The bounded
contract is
`docs/rust-rewrite/contracts/array-from-async-result-definition-error-realm.md`;
its two structure targets pass `4/4` each, its two focused CLI fixtures pass
`1/1` each, and the six directly relevant Array.fromAsync files pass `12/12`
sloppy/strict executions.
The following shared semantic golden passes `2/2` in 800.46 seconds with 679
dumps, adds only the Array.fromAsync result-definition Realm witness and
removes none. Of 678 retained dumps, 677 are equal after accounting
normalization; only the independently expanded Promise Realm witness changes
structurally.

The two outlined OrdinarySet helper producers now choose a private, non-derived
`OrdinarySetReceiverFallback` domain. Its sole consumer exhaustively projects
the paired runtime-helper identity and generic receiver-write permission, so a
new helper policy cannot silently inherit either half through equality or a
Boolean shortcut. This is a source-equivalent Rust invariant migration; emitted
Wasm is expected to remain byte-identical. The later helper-body filing table is
independent of the typed value, so the bounded guard pins its two exact insertion
rows in runtime-helper ID order. The bounded contract is
`docs/rust-rewrite/contracts/ordinary-set-receiver-fallback.md`, and its
recursive structure target passes `4/4`. The existing outlined OrdinarySet CLI
witness passes `1/1`, independent dry review is clean, and `cargo xc` plus
repository checks are green. Broad conformance suites were not rerun for this
lane.

The object-read Realm seam now exposes no incidental capability on its two
private projections. `OutlinedObjectReadRealmArgument` still owns the outlined
helper ABI environment-or-zero argument, while the distinct
`ObjectReadRevocationErrorRealm` still owns direct revoked-Proxy TypeError
construction. Both map the three unchanged `ObjectReadErrorRealmSource` rows
exhaustively; their local unit assertions now match the expected rows
exhaustively instead of requiring clone, copy, debug or equality. The production
projections, error calls, instructions, locals and ordering are unchanged.
The dedicated structure target passes `4/4`, the exact projection unit passes
`1/1`, the neighboring object-read Realm structure target passes `3/3`, and the
created-Realm revoked-Proxy CLI witness passes `1/1`. See
[`object-read-realm-projection-capability.md`](../docs/rust-rewrite/contracts/object-read-realm-projection-capability.md).
This invariant does not claim an object-model redesign, broad Test262 result or
conformance-count change. Independent dry review is clean, and the shared
format, `cargo xc`, diff, module-boundary and task-plan checkpoint is green with
the workspace's existing warnings.

Stored descriptor compatibility now has one closed
stored descriptor role relation. `StoredDescriptorLocals::new` accepts distinct data, getter, and
setter wrappers, so its three Array, Arguments, and ordinary named-property
producers cannot transpose data, getter, and setter locals at a boundary where
all three were previously identical `TaggedLocals`. The recursive Rust-lexical
`stored_descriptor_role_relation_structure` guard pins the private wrappers,
typed aggregate constructor, exact producers, and complete role set. This is
source-equivalent type hardening, not wider Object-model or conformance closure.
The dedicated and neighboring Arguments structure targets pass `4/4` each, and
the exact Array descriptor CLI witness passes `1/1`. The mapped-Arguments CLI
witness is blocked by unrelated active callers of the removed
`FunctionArgumentsProtocol::present` method; focused details are recorded in the
contract.

Arguments exotic `[[DefineOwnProperty]]` for `callee` now crosses its builtin
boundary as one private exact-shape `ArgumentsCalleeDescriptorLocals` instead
of fifteen positional payload, tag and presence locals. Each of its six named
fields owns one run-time presence fact and one correctly typed tagged-value or
Boolean carrier. Its sole validated projection feeds the canonical descriptor
classification, and the consumer exhaustively projects the data and accessor
sides before applying the unchanged callee compatibility/storage algorithm.
Static or absent presences cannot enter this exact boundary, tagged values
cannot occupy flag roles, and the former parallel kind fold is gone. The
single producer/consumer census and source invariant are recorded in
`docs/rust-rewrite/contracts/arguments-callee-descriptor-boundary.md`; focused
execution is deferred to the shared T10 verification checkpoint.

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
