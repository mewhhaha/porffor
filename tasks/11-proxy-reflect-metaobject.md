# T11 — Proxy and Reflect meta-object protocol

**Status:** In progress — Proxy/Reflect paths exist; product semantics and broad verification remain

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T06, T09, T10  
**Blocks:** Proxy-sensitive closure in most other lanes

## Current repository state

Proxy and Reflect builtins are implemented through dedicated backend paths, and
focused tests cover several traps and object-integrity interactions. The
Test262 materializer contains no Proxy-owned shortcut observation: the apply,
construct, revocation and descriptor rewrites are gone. The four former apply
and construct leaves that require dynamic source now retain their pinned source
and report T13's explicit Wasm-AOT unsupported boundary instead of being rewritten
to green Proxy cases. Until the remaining product semantics are implemented and
the complete Proxy/Reflect trees are verified, this lane remains open.

At the coordinated focused T11 checkpoint, the dedicated structure targets are
green: `proxy_set_prototype_of_handler_protocol_structure` passes `4/4`,
`proxy_reflect_set_handler_protocol_structure` passes `4/4`,
`reflect_optional_argument_presence_structure` passes `5/5`,
`reflect_property_key_conversion_structure` passes `4/4`,
`proxy_revocation_route_ownership_structure` passes `4/4`,
`object_write_proxy_realm_structure` passes `5/5`,
`object_read_proxy_realm_structure` passes `3/3`,
`proxy_execution_realm_structure` passes `6/6`,
`proxy_call_throw_routing_structure` passes `3/3`, and
`proxy_get_trap_result_lifecycle_structure` passes `4/4`. Seven exact CLI
regressions are green. The formatting, diff, module-boundary, task-plan and
audit gates are also green. This is focused evidence only; it does not claim
any other Cargo target, broad Test262 aggregate, published result or unrun
command.

The IR standard-builtin catalog now classifies the complete modeled
Object/Reflect proxy-capable surface as synchronous user code. This single
authority reaches direct, spread and mixed-candidate calls; a catalog contract
pins both the trap-capable set and the three Object exclusions whose algorithms
cannot proxy-dispatch. The caller-flow regression proves a `getPrototypeOf`
trap can invalidate a previously created realm shape, and a spread
`Object.assign` regression covers the path that does not execute exact builtin
result analysis.

`Function.prototype.apply` now shares that catalog authority because
CreateListFromArrayLike can synchronously invoke Proxy or accessor `length` and
index reads before the forwarded target runs. The caller-flow fixture uses a
pure target and a Proxy-backed argument list, so it specifically proves those
preprocessing effects rather than relying on target invalidation. The compiled
Wasm-AOT regression passes; safe concrete-array precision remains future work.

The set-path Realm environment argument is now a private, capability-free
two-row authority consumed once to emit exactly one helper ABI argument. The
complete projection, sole consumer, 11-mention census and exhaustive focused
unit are pinned without changing helper stack order or Realm selection. The
focused source contract is
`docs/rust-rewrite/contracts/set-path-realm-environment-argument-ownership.md`;
runtime CLI verification is deferred, and Test262 remains deferred to the
shared checkpoint.

The `Reflect.setPrototypeOf` metadata materializer has been removed. Its three
unchanged pinned sources now execute with the full `propertyHelper.js` harness;
all six sloppy/strict Wasm-AOT executions pass. The five-case `Reflect.set`
materializer has also been removed. Those unchanged pinned sources now use
ordinary materialization and their declared full `propertyHelper.js` harness;
the complete `built-ins/Reflect/set` leaf passes all 36 sloppy/strict Wasm-AOT
executions. The three-case Proxy `defineProperty` materializer is gone as well.
Its unchanged pinned sources now use ordinary materialization and the full
declared `propertyHelper.js`; the complete current-pin leaf passes all 48
sloppy/strict Wasm-AOT executions. At that checkpoint, the generated inventory
retained 19 T11 observations. The four-case Proxy `getOwnPropertyDescriptor`
fallback materializer is now gone too. Those
unchanged sources use the complete embedded LocalMerged `propertyHelper.js` in
ordinary materialization, while separate raw runs with the full upstream helper
pass all four sources in sloppy and strict modes (`8/8`). The full 21-file leaf
passes all 42 sloppy/strict Wasm-AOT executions. Removing the dispatcher and
physical rewrite owner leaves 15 T11 observations.

The four ProxyCreate callable, constructable and revoked-target shape cases now
execute their unchanged pinned sources. Raw runs with the full upstream
`assert.js` and, where declared, `isConstructor.js` pass all eight sloppy and
strict executions. Ordinary materialization uses their LocalMerged equivalents;
the two sameValue-only revoked-target cases retain the shared trimmed SameValue
assertion route and are not claimed as complete-helper executions. A real-source
unit pins the exact sources, includes, helper provenance and that explicit
nonclaim. Removing the dispatcher, rewrite owner and four exact path predicates
left 10 T11 observations at that checkpoint.

The Proxy apply `trap-is-not-callable-realm.js` case now executes its unchanged
vendored source with the complete LocalMerged `sta.js` and `assert.js`
preludes. Both sloppy and strict executions pass (`2/2`), and a real-source
materialization invariant rejects the handwritten rewrite and trimmed assertion
helper. The current-execution-Realm follow-up also removes the
`null-handler-realm.js` branch. Its real-source materialization invariant keeps
the complete Realm and assertion preludes and requires the pinned source to
survive unchanged. `rewrite_proxy_apply_case` remains solely for
`arguments-realm.js`, whose pinned source calls created-Realm `eval` and
therefore remains T13 dynamic-source work. The unchanged apply and construct
`null-handler-realm.js` leaves pass all four sloppy and strict executions. The
neighboring apply and construct `trap-is-not-callable-realm.js` controls also
pass `4/4`. Removing the null-handler branch deletes its two selectors. That
checkpoint assigned 10 observations to T11 in the 409-entry token-aware
inventory, which also records exact rewrite calls, source guards and selector
tables that the earlier line-oriented census omitted.

The complete `Proxy.revocable` rewrite owner and its dispatcher branch are now
gone. A real-source matrix rejects any replacement for all 18 pinned files and
pins every declared complete helper plus its provenance. Seventeen ordinary
physical cases therefore retain their original sources through ordinary
materialization. `tco-fn-realm.js` also retains its raw `other.evalScript`
call and complete Realm prelude. The synthetic created-realm record shape types
that property as `HostBuiltinId::RealmEvalScript`, and current AOT bootstrap
work publishes the corresponding realm-local function identity. Calling it is
still T13's typed `DynamicSourceIntrinsic::RealmEvalScript` unsupported
boundary, not a green Proxy result. At that retirement checkpoint, removing the
rewrite left six observations assigned to T11 in the 405-entry inventory.

The final apply and construct materializers are now deleted too. The unchanged
`arguments-realm.js` leaves retain their indirect-eval source, while the two
new-target-Realm leaves retain their ordinary-Function constructor source.
All four remain explicit Wasm-AOT unsupported dynamic-source cases; none is
counted as a Proxy pass. Removing the two rewrite entry points and four exact
path predicates leaves T11 with zero observations in the current 181-entry
shortcut inventory.

Proxy construction and `Proxy.revocable` now take all creation-owned Realm
identities from one non-copyable `ProxyCreationExecutionRealm`. A self-backed
created-Realm `Proxy` constructor or `revocable` function supplies its defining
Realm. A main builtin with a zero environment loads the canonical main Proxy
constructor's defining Realm instead of reading the dynamic current-Realm
global. The context then couples that Realm's `%TypeError.prototype%`,
`%Object.prototype%` and `%Function.prototype%`. Both target and handler
validation algorithms use its TypeError prototype at all four sites. The
revocable record uses its Object prototype, while the hidden revoke target and
exposed revoke function use its Function prototype. No Proxy-specific branch
selects a direct entry-Realm prototype global. The bounded source contract is
`docs/rust-rewrite/contracts/proxy-creation-execution-realm.md`.

At the 2026-08-30 focused checkpoint, the three structure invariants pass
`3/3`, the neighboring exact-bound-this ownership unit passes `1/1`, and the
exact created-Realm CLI witness passes `1/1` with 786 other CLI tests filtered.
The library check, targeted formatter and scoped diff check also pass with only
the existing Boa trivial-cast warning. No broad suite, semantic golden or
Test262 cohort ran. The fixture observes `realm.evalScript` only as a published
function so the host-owned name enters the static string pool. It does not call
dynamic source evaluation or change the T13 boundary above.

The subsequent unchanged-source `built-ins/Proxy/revocable` sweep reports 34
Success outcomes from 35 executions. `tco-fn-realm.js` is the sole
non-success, classified as the explicit `$262.evalScript` Wasm-AOT
NotImplemented boundary. Every parser, early-error, lowering, runtime,
Wasm-backend and host-harness failure bucket is zero, as are Crash and Bug.
This is raw-source evidence for every supported case without claiming T13
dynamic source evaluation.

Proxy `[[Call]]` and `[[Construct]]` now share the closed
`ProxyExecutionRealmSource` classification. Its four variants are
`MainRealmFallback`, `StandardBuiltinEnvironment`, `ObjectReadHelperArgument`
and `ProxyDispatchHelperArgument`.
Main, user and host bodies use the entry Realm. A standard builtin may use its
self-backed defining Realm, while the outlined object-read, Proxy Call and
Proxy Construct helpers may use only the environment-or-zero projection
received in ABI parameter 6.
Every generated TypeError and every `%Array.prototype%` installed on the
`CreateArrayFromList` trap argument consumes that projection. Nested Proxy
dispatch, ordinary accessor reads and Proxy-aware handler reads forward the
same trusted word. The former Proxy creation-Realm TypeError prototype field
and its defining-Realm loader are deleted. The focused source contract is
`docs/rust-rewrite/contracts/proxy-call-construct-execution-realm.md`;
the affected structure and projection tests pass `19/19`, the harness and CLI
witnesses pass `2/2`, and the four raw leaves pass `8/8`. Non-revocation
TypeErrors created by a nested live Proxy `[[Get]]`, such as a noncallable
`get` trap or invariant violation, remain separate T11 object-read work.
At the current coordinated checkpoint, the exact execution-Realm structure
target passes `6/6` and the Proxy Call throw-routing target passes `3/3`.

The `Object.prototype.toString` builtin-tag decision now enters the recursive
Proxy-aware `IsArray` and `IsCallable` authorities. Direct and nested Proxies to
Arrays therefore report the Array builtin tag, callable Proxies report Function,
and a revoked Proxy throws before `@@toStringTag` lookup. The focused runtime
fixture exercises both the direct Object call and the non-callable-`join` Array
fallback. Revocation uses the current builtin function's Realm through the
shared `IsArray` authority; borrowed other-Realm `Object.prototype.toString`
and `Array.isArray` calls produce that Realm's TypeError prototype. The outlined
Proxy `[[Get]]` helper now receives the same trusted standard-builtin Realm
environment through its formerly unused ABI word; ordinary user and host
lexical environments project to the main-Realm fallback instead. Therefore the
earlier `Array.prototype.toString` gap is also closed: its required
`Get(O, "join")` on a revoked Proxy constructs the null-handler TypeError in the
borrowed builtin's Realm before the Object fallback is selected.
The current object-read Realm structure target passes `3/3`.

Proxy `[[Set]]` now has the matching closed Realm projection. ObjectWrite, both
receiver-side helpers and both OrdinarySet helpers accept a Realm word only
after their callers project the current environment to a trusted standard
builtin Realm record or zero. Those five helper bodies may forward that typed
word or attach it to nested `Reflect.set`; user and host lexical environments
remain excluded. The direct write owner routes revoked handlers, non-callable
traps and strict falsy trap results through the same projection. The shared
post-trap descriptor owner routes both incompatible-result TypeErrors likewise,
and direct `Reflect.set` revoked/non-callable sites remain source-pinned to the
current function Realm. Assignment false results remain strict-guarded, while
both Array push consumers use the unconditional Realm-aware owner required by
their internal `Set(..., Throw=true)`. Exhaustive helper and throw-site censuses pass `5/5`;
a ten-branch borrowed created-Realm fixture passes `1/1`, including Array
and Reflect writes that reach a Proxy through an ordinary target's prototype.
The shared semantic golden passes `2/2` across 658 fixture dumps. It adds only
the created-Realm Atomics and Proxy `[[Set]]` fixtures to the preceding
656-dump checkpoint and removes none. Every retained dump preserves its roots,
builtin/helper counts, locals, imports, exports, globals, memories, data
segments and name count; only emitted-function byte sizes change, with the
largest deltas confined to write-heavy environment fixtures as expected from
the new Realm argument.

`lowering/proxy_traps.rs` now owns one private, closed `ProxyTrap` domain
containing all thirteen ECMA-262 10.5 handler methods. Each trap maps
exhaustively to one of eight semantic argument records rather than to an
untyped arity. When a proven
`new Proxy(target, handler)` path has a statically visible handler method,
pre-lowering and typed lowering both enumerate every trap through that mapping.
Ordinary object-literal lowering deliberately retains its former five-name
heuristic so methods merely named `apply`, `construct`, or `set` are not
misclassified as traps. This removes the former raw-string match and its
catch-all, which silently discarded eight valid trap signatures. The seam is
covered by the green central feature-enabled CLI compile and by the complete
620-test default CLI inventory, including focused apply, construct,
defineProperty and set behavior. It is an inference invariant, not a claim
that the runtime implementations or full Proxy/Reflect trees are complete.

The Wasm-AOT `has` path now shares T10's closed `[[HasProperty]]` dispatcher.
An absent `has` trap re-enters the complete target dispatch rather than a
representation-specific fallback, including through nested Proxy targets, and
any callable value is accepted as the trap, including a callable Proxy. The
positive regression is dry-written but its focused runtime gate has not run
while the shared conformance matrix is active.

The bounded representation now retains both `[[ProxyTarget]]` and
`[[ProxyHandler]]` as typed payload/tag pairs. A single Proxy allocator takes
`ProxySlotLocals` with distinct target and handler newtypes; both constructors
must supply it, while its slot writer is private. Omission or target/handler
transposition is therefore a compile error.
Revocation keeps the existing handler-payload
sentinel and does not change target layout. The `has` consumer then loads the
retained tag and routes lookup through the existing full object-read seam,
with abrupt getter completion leaving the traversal and the exact tagged
handler passed as trap `this`. Other Proxy methods that reconstruct Object
remain separate T11 migrations over the same stored slot. The exact Wasm-AOT
regression for Function, Array, arguments and nested-Proxy handlers, tagged
handler `this`, and an abrupt lookup getter is written but has not run.

The post-trap boolean-result batch now has one direct-target contract. A
private, closed object-representation order is shared by `[[HasProperty]]` and
a value-free own-descriptor fact emitter. The fact is two distinct Wasm locals,
`present` and `descriptor`: zero is a legal descriptor word for a present data
property whose three attributes are false, so it may never double as the
absence sentinel. Its fields are private and consumers can ask only the named
presence/configurability/writability questions, making a raw-local
transposition a compile error. The emitter allocates no JavaScript descriptor
object and invokes no property getter.

That direct emitter owns integer-indexed, Array, arguments, boxed-String,
Function-special and ordinary storage in the same exhaustive representation
match. In particular, Array `length` is an unconditional present,
non-configurable descriptor; arguments `length` carries an explicit own-property
bit while it is live, including the all-false data-descriptor state; and an
invalid canonical integer-index is handled as absent rather than falling
through to ordinary storage. The public descriptor builtin's Proxy target
checks, the `has` false-result check and the `deleteProperty` true-result check
all consume this fact. The two boolean-trap invariants accept an absent
descriptor, reject a present non-configurable descriptor, and only then call
the shared typed `[[IsExtensible]]` emitter for a present configurable
descriptor. This preserves ECMA-262 order and the distinct Array/arguments
extensibility slots. The former Array-only `has` mirror and the raw
Array/ordinary delete scans are deleted.

The exact Wasm-AOT regression is written for Array `length`, both ordinary and
non-configurable Array indices, a configurable named property on a
non-extensible Array, a present ordinary all-false descriptor word,
boxed-String indices, a fixed typed-array index, ordinary arguments indices,
arguments `length` (including delete-and-recreate), both public descriptor-trap
result forms and the absent/non-extensible ordering case. It has not run while
the release matrix owns runtime verification.

Proxy `[[Delete]]` trap acquisition now consumes the typed live-slot reader and
full handler `[[Get]]` seam. Function, Array and arguments handlers retain their
representation tags, Proxy handlers observe their own `get` trap, and lookup
abrupt completions leave the operation before absent/callable classification.
The shared callable test and call operation accept a callable Proxy trap and
invoke every trap with the exact tagged handler as `this` plus the target and
property key. An absent trap preserves the existing recursive target fallback;
a present result continues into the direct descriptor invariant below.

The existing Proxy `deleteProperty` fixture applies that acquisition contract
across Function, Array, arguments and Proxy handlers, a callable Proxy trap,
exact target/key/`this`, abrupt lookup and trap-call sentinels, absent lookup
forwarding to a Proxy target and both direct and Reflect entry points including
a Symbol key. The Function-handler trap is obtained through an accessor whose
exact tagged `this === functionHandler` comparison makes retention of the
Function tag load-bearing; the adjacent `this.prototype` check reinforces
receiver identity but is not itself tag-sensitive. The trap-call sentinel comes
from a callable Proxy over a non-configurable property on a non-extensible
target, proving the original throw wins before `ToBoolean` and invariant
validation. The fixture also retains the post-trap cases for Array named and
symbol properties, boxed-String virtual properties, Function `prototype`,
arguments all-false and non-configurable descriptors, and an absent property on
a non-extensible target. Absent or nullish traps now traverse target Proxies in
one emitted Wasm loop rather than recursively expanding Rust emission to a
fixed depth. The fixture crosses six nullish forwarding handlers in exact
outside-in lookup order, reaches one callable inner trap and requires deletion
from the ultimate ordinary target. Dedicated current-target locals preserve
the caller's operands. This remains a bounded migration because the direct
descriptor fact does not recursively validate a nested Proxy target. Focused
verification on 2026-09-01 is green: `cargo check -p lila-aot-wasm`, both the
new traversal and adjacent revocation-route structure targets at `4/4`, the
exact CLI fixture at `1/1` through emitted Wasm, and the missing/null/undefined
nested-target Test262 cohort at `6/6` with every failure bucket at zero. The
fixture also passes JavaScript syntax and reference-semantics evaluation under
Node. Independent dry review is clean, as are formatting, diff, task-plan,
module-boundary and exact 186-entry shortcut gates. The invariant and evidence
live in `docs/rust-rewrite/contracts/proxy-delete-traversal.md`.

Proxy `[[Set]]` truthy-result validation now joins those direct-target
consumers through a richer projection of the same descriptor authority. One
closed Rust result domain contains the value-free fact and a complete Proxy-Set
record with distinct fact, data-value and setter locals; every projection
consumes the same exhaustive object-representation loop. Target, property key
and incoming value are typed call-site roles. Array, arguments,
boxed-String, Function-special and ordinary values are read from descriptor
storage without invoking getters, and mapped arguments data observes the
current parameter value. Missing setters normalize to tagged `undefined`; a
callable Proxy setter is accepted because ECMA-262 tests only whether
`[[Set]]` is undefined. Ordinary entry storage wins before virtual fallbacks,
so freezing a Function's materialized `prototype` entry changes the invariant
while DataView/intrinsic fallbacks remain available when no entry exists.

The focused Wasm-AOT fixture covers Array length and dense/sparse indices,
named and Symbol keys, boxed-String virtual values, mapped arguments and an
arguments accessor whose getter must not run, callable-Proxy setters,
`SameValue` edge cases, writable and frozen Function `prototype` entries,
integer-indexed no-false-positive cases and both assignment and Reflect entry
points. The module-boundary guard pins the complete projection, typed call-site
roles, unique projection call, `SameValue` check, undefined-setter check and an
active exact CLI registration. On 2026-08-24, that CLI witness passed `1/1`.
At current Test262 pin
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, six selected unrewritten Proxy Set
invariant files passed all `12/12` ordinary Wasm-AOT executions with every
failure bucket at zero. This is only the post-trap, direct-target migration:
Set trap lookup/fallback, recursive nested-Proxy target `[[GetOwnProperty]]`,
module namespaces and the complete 27-file/54-variant Proxy Set subtree remain
T11 work. The integer-indexed fixture cases are false-positive controls, not
complete TypedArray Set evidence.

Direct `Reflect.set` handler acquisition now uses the typed live-Proxy slot
reader with `CurrentFunctionRealm`, the full Proxy-aware handler read and an
immediate abrupt-completion checkpoint before `IsCallable`. A present trap is
called through the Function-or-Proxy owner with the exact tagged handler as
`this` and `(target, key, value, receiver)` as its four arguments. This removes
the direct owner's fabricated Object handler tag, ordinary-only read and
Function-only Call while preserving its Boolean result, post-trap invariant,
nullish nested-target fallback, ordinary fallback and release order.

The focused `wasm_proxy_reflect_set_handler_protocol.js` fixture covers
Function, Array, arguments and Proxy handlers, exact lookup/Call records, a
Symbol key, a tag-sensitive Function target and Array receiver, a callable
Proxy trap, abrupt lookup and call identity and both nullish fallbacks. The
existing borrowed-Realm Proxy Set fixture retains the direct
revoked/non-callable Realm witness. The
`proxy_reflect_set_handler_protocol_structure` target and module-boundary guard
pin this direct owner; the bounded contract is
`docs/rust-rewrite/contracts/proxy-reflect-set-handler-protocol.md`. Focused
verification passes the direct handler-protocol and revocation-route structure
targets `4/4` each and the object-write Realm target `5/5`. No individually
attributed CLI or Cargo-check result is claimed in this paragraph.
Assignment/internal Set acquisition and the complete Proxy Set tree remain
separate T11 work.

Reflect optional-argument defaults now use the shared builtin ABI presence
authority (`argc > index`) instead of comparing the loaded value with
`undefined`. `Reflect.get` and `Reflect.set` preserve an explicitly supplied
`undefined` receiver and default only an omitted receiver after
`ToPropertyKey`; `Reflect.construct` preserves an explicit `undefined`
`newTarget` so the constructor check rejects it. The
`reflect_optional_argument_presence_structure` target and
`wasm_reflect_optional_argument_presence.js` fixture pin the three consumers,
their order and observable Proxy identities. The bounded contract is
`docs/rust-rewrite/contracts/reflect-optional-argument-presence.md`.
The structure target passes `5/5`; no individually attributed CLI or Cargo-
check result is claimed in this paragraph.

The five Reflect property-key boundaries for get, set, has, defineProperty and
deleteProperty now retain both the converted payload and tag from the full
`ToPropertyKey` authority. An Object whose conversion returns a Symbol can no
longer be reclassified from its source Object tag. The
`reflect_property_key_conversion_structure` target and
`wasm_reflect_property_key_conversion.js` fixture pin boxed and Object-to-
Symbol behavior, exact single conversion in the Set trap and abrupt conversion
before the target internal method. The bounded contract is
`docs/rust-rewrite/contracts/reflect-property-key-conversion.md`. Focused
verification passes the structure target `4/4`. No individually attributed CLI
or Cargo-check result, broad Test262 result, or published conformance result is
claimed.

Proxy `[[Get]]` post-trap validation now consumes a second richer projection of
that same direct descriptor authority. The closed projection domain has
distinct Proxy-Get and Proxy-Set records, and a closed getter/setter endpoint
enum makes using the wrong accessor role an exhaustive-match type error. The
Get invariant accepts typed target, property-key and normal trap-result roles.
A trap call initially yields a distinct pending result; the only transition to
the normal-only type emits abrupt-completion routing first, so a trap's thrown
value cannot be replaced by a later frozen-target TypeError.

The pending and normal Proxy-Get trap-result roles are now also capability-
free. Both are must-use, non-copyable one-way lifecycle values with no debug,
default, comparison, ordering or hashing surface. A recursive four-test guard
pins the sole raw-result producer, completion-routing transition, consuming
invariant, borrowed observers and exact existing abrupt-result CLI witness.
At the 2026-08-28 Batch Z checkpoint, `cargo xc` is green and the recursive
structure target passes `4/4`. The existing CLI fixture remains `0/1`: it
throws at its pre-existing mapped-arguments current-value assertion before the
later abrupt-result identity controls. This derive-only closure changes no
instruction body; no Test262 rerun or new behavior/conformance claim is made.

The shared storage-only walk observes Array dense/sparse and named entries,
Array length, mapped and accessor arguments indices, arguments special
`length`/`callee`, boxed-String virtual values, ordinary entries and
Function/DataView special values without invoking a stored getter. Missing
getters normalize both raw zero and tagged `undefined`. The invariant then
requires `SameValue` for a present non-configurable, non-writable data
descriptor and requires an undefined trap result for a present
non-configurable accessor with no getter. The former Object/Function-only raw
entry scan is deleted.

The exact Wasm-AOT fixture covers direct and Reflect Get, all of those direct
representations, callable-Proxy and missing getters without invocation,
`SameValue` edge cases, configurable/integer-indexed/absent false-positive
guards, and preservation of the original thrown trap. It is written but has
not run while the shared verification lane owns Cargo and Test262. This remains
only a direct-target post-trap migration: Get trap lookup/fallback, recursive
nested-Proxy target `[[GetOwnProperty]]`, module namespaces and complete
Proxy/Reflect Get closure remain T11 work.

The retained Proxy slots now also have one typed read authority. The reader
accepts the same `ProxySlotLocals` record as the writer, maps each heap word into
the distinct target/handler newtype, and emits the revoked-handler check before
the loaded slots become usable. Its closed completion route keeps the existing
builtin, internal-helper and HasProperty throw boundaries explicit. Both the
public descriptor path and shared `[[Delete]]` and `[[IsExtensible]]` operations
now join `has` in consuming the exact handler tag and the proxy-aware object-read
seam for `GetMethod`. Function, Array, arguments and nested-Proxy handlers
therefore retain their storage behavior and exact handler-as-`this` identity in
these four methods; an abrupt trap lookup is routed before callable/absent
classification.

The exact Wasm-AOT regression covering those four handler representations,
Object and Reflect entry points, exact `this`, and abrupt lookup is written but
has not run while the release matrix owns runtime verification.

Proxy `[[PreventExtensions]]` now has a closed recursive request boundary. A
private, non-copyable `ObjectPreventExtensionsRequest` owns distinct tagged
traversal and Boolean-result roles, and pending/normal trap-result carriers
make consuming a thrown call result as a Boolean a type error. Its outlined
runtime helper is catalogued by `RuntimeHelperId::ObjectPreventExtensions`;
missing, `undefined`, or `null` traps call that same helper with the retained
target tag instead of decrementing a Rust emission depth. Trap lookup keeps the
typed live handler, abrupt lookup/call completion is routed before
classification or invariants, and a true result performs the complete
proxy-aware `[[IsExtensible]]` check before publication.

The retained source-free fixture now covers Object-versus-Reflect false
results, more than four nested fallbacks, Function/Array/arguments/Proxy
handlers, exact getter/trap receivers, a callable Proxy trap, abrupt lookup and
call identity, non-callable traps, invariants, and revocation. The sole
`rewrite_proxy_prevent_extensions_case` shortcut and its materialization unit
have been removed. Consequently the original Module file
`built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js` now
runs from its vendored self-import source. Verification on `2026-08-21` is
green for that exact raw Module execution (`1/1`), the complete leaf's 12
physical files / 23 executions (`23/23`), the typed structure witness (`3/3`),
and the expanded source-free Wasm fixture (`1/1`, 55.92 s). The adjacent
recursive `built-ins/Proxy/isExtensible` and
`built-ins/Reflect/preventExtensions` leaves are green at `24/24` and `20/20`.
At clean pre-batch commit `22ab459107`, the broader
`built-ins/Object/preventExtensions` regression reported `77/78`; the sole
failure was the strict-script half of `15.2.3.10-3-4.js`. Its expected
array-index PutValue `TypeError` returned from the non-main harness function
instead of entering the catch owned by that same function. The adjacent batch
now uses one canonical route: fresh runtime errors delegate to
`emit_propagate_current_throw`, whose typed `ControlTarget` branch wins whenever
a handler or finalizer is active, and only the no-target case returns the
current completion. The retained fixture now distinguishes an external catch
around a strict call from the load-bearing catch inside the same strict
non-main function, and separately pins two nested finalizers running in order
before the unchanged array-index TypeError reaches the outer catch. Verification
on `2026-08-21` is green for workspace/all-target and `cargo xc`, the bounded
structure witness (`3/3`), the expanded Wasm fixture (`1/1`, 21.08 s), the
exact file (`2/2`), and the complete `built-ins/Object/preventExtensions` leaf
(`78/78`, zero unsupported, crashes, timeouts, or runtime failures). Resumable
throw transport, unrelated throw/catch paths, and
object-literal method `[[HomeObject]]` remain outside this batch. Focused
Object freeze, primitive-integrity, and TypedArray prevention fixtures remain
green at `1/1` each. The older path-counted green leaf included the rewrite and
is not promoted to source-level evidence here.

The complete Proxy `[[PreventExtensions]]` request/completion lifecycle is now
capability-free. `PreventExtensionsTraversalTargetLocals`,
`PreventExtensionsResultLocal`, `ObjectPreventExtensionsRequest`,
`PendingProxyPreventExtensionsTrapResultLocals` and
`NormalProxyPreventExtensionsTrapResultLocals` implement no clone, copy, debug,
default, comparison, ordering or hashing capability; construction and one-way
consumption are their only surface. The normal-completion transition,
normal-result consumer and recursive traversal bodies remain byte-identical at
`08ec7efc44446238a2faa8a34163b212cad3de76427bc5d35dfb9c5429979616`,
`158d5fa2f9ce31871ac1e711310b1167a126671eaa5d095a2470b04261de8c38`
and `ffbac884ee4acaee1567677169776c5ad4417b9b182df24c9b3d4d356e4b5c5a`.
At the 2026-08-28 Batch Y checkpoint, the strengthened structure target passes
`3/3`, the exact existing Proxy CLI control passes `1/1`, and `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. No current-
tree Test262 rerun or behavior/conformance change is claimed.

The shared proxy-aware `[[GetPrototypeOf]]` emitter now consumes that same typed
live-slot reader and full object-read seam. It no longer reconstructs every
handler as an Object, so Function, Array and arguments handlers retain their
tags for both `GetMethod` and trap `this`, while a Proxy handler observes the
complete `[[Get]]` protocol. Abrupt method lookup is routed before the
absent/non-callable split. The existing object-or-null result check,
non-extensible target prototype equality check, nested-target fallback and late
result publication remain unchanged.

The existing `getPrototypeOf` Wasm-AOT fixture now covers both Object and
Reflect entry points across those handler representations, an inherited `get`
trap on a Proxy handler's own handler and an abrupt accessor lookup. It has not
run while the release matrix owns runtime verification. Other Proxy methods
that still reconstruct an Object handler remain separate migrations.

Proxy `[[SetPrototypeOf]]` handler acquisition now uses the same typed
live-slot reader and full GetMethod path. It retains the exact target and
handler tags, performs the Proxy-aware `"setPrototypeOf"` read with the handler
as receiver, routes an abrupt lookup before `IsCallable`, and calls a present
Function-or-Proxy trap with the handler as `this` and the exact target and
prototype arguments. The existing nullish fallback, Boolean result,
non-extensible-target prototype invariant and temporary-local release order are
unchanged. Its dedicated
`ObjectMutationRealmToActiveHandler` revocation route and the two local
post-acquisition TypeError sites now use the established object-mutation Realm
authority while retaining active-handler completion routing.
`Object.setPrototypeOf`,
`Reflect.setPrototypeOf` and the `Object.prototype.__proto__` setter continue
to share the internal method and preserve their distinct result behavior.

The focused `wasm_proxy_set_prototype_of_handler_protocol.js` fixture covers
Function, Array, arguments and Proxy handlers, a callable Proxy trap, exact
receivers and arguments, abrupt lookup identity, nested-Proxy nullish fallback
and created-realm revoked/non-callable/local-invariant errors. Realm forwarding
for errors raised by nested Proxy targets inside the outlined
`[[IsExtensible]]` and `[[GetPrototypeOf]]` helpers remains explicit follow-up
debt. The
`proxy_set_prototype_of_handler_protocol_structure` target pins that observable
surface together with the module-boundary slot-reader census and raw-read ban.
The bounded source contract is
`docs/rust-rewrite/contracts/proxy-set-prototype-of-handler-protocol.md`.
Focused verification passes the handler-protocol and revocation-route structure
targets `4/4` each, and the shared formatting, diff, module-boundary, task-plan
and audit gates are green. No individually attributed CLI or Cargo-check result,
broad Test262 result, or published conformance result is claimed in this
paragraph.

The bounded Proxy `[[DefineOwnProperty]]` acquisition checkpoint now joins the
typed live-slot authority as well. One shared emitter accepts a tagged
traversal object, distinct typed target/handler slots, a typed property key, the
completed descriptor object and tagged trap/result roles. It preserves
Function, Array, arguments and Proxy handler tags through Proxy-aware
`GetMethod` and Call, routes an abrupt lookup before `IsCallable`, invokes the
trap with the exact handler as `this` and retains the complete tagged target on
a nullish fallback. `Object.defineProperty` keeps its throwing false-result
behavior, `Reflect.defineProperty` keeps its Boolean result, and both retain
the existing post-trap descriptor invariant consumer.

This repairs a directly observed product-path defect. On the pre-fix
`d412ca624be8fa3eba974b05274775d8165522eb` checkout, exact Wasm-AOT probes for
Function, Array and arguments handlers all failed their getter/trap receiver
identity checks. The Function control observed the retained handler as an
Object-tagged value unequal to the original Function, proving that the former
payload-only load and fabricated Object tag were observable.

The focused source-free fixture and structural owner cover both public entry
points, all four handler representations, a callable Proxy trap, exact
receiver/`this` and argument order, abrupt lookup identity, nested-Proxy
nullish fallback and created-realm revoked/non-callable errors. At the
2026-08-25 coordinated checkpoint, the structure target passes `4/4` and the
exact CLI registration passes `1/1`. At current vendored content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, five raw unflagged Proxy
defineProperty files pass all ten ordinary sloppy/strict Wasm-AOT executions:
`call-parameters.js`, `return-is-abrupt.js`, `trap-is-not-callable.js`,
`trap-is-not-callable-realm.js` and
`trap-is-undefined-target-is-proxy.js`. Every failure and non-success bucket is
zero.

This is handler acquisition only. Recursive Proxy descriptor compatibility and
module-namespace exotics, removal of the three retained defineProperty
materializer rewrites and raw verification of the complete 24-file/48-variant
leaf, remaining descriptor-lattice obligations, unrelated Proxy Get/Set
acquisition, and created-realm descriptor-object allocation remain explicit
nonclaims. The bounded source contract is
`docs/rust-rewrite/contracts/proxy-define-property-handler-protocol.md`.

Proxy `[[OwnPropertyKeys]]` handler acquisition now consumes the typed live-slot
reader as well. The shared emitter accepts `TaggedLocals` for the traversal
object, prospective trap and trap result plus one `ProxySlotLocals` record whose
target and handler roles are distinct types. Its one live-slot read uses the
current Function Realm revocation route. The four Object/Reflect consumers
therefore preserve Function, Array, arguments and Proxy handler tags through
Proxy-aware `GetMethod` and Call, route an abrupt lookup before `IsCallable`,
and use the exact handler as `this`. Nullish traps still re-enter traversal with
the complete tagged target, and the existing result-list and target-key
invariant validators remain the only post-trap consumers.

The focused source-free Wasm-AOT regression is written across
`Object.getOwnPropertyNames`, `Object.getOwnPropertySymbols`, `Object.keys` and
`Reflect.ownKeys`. It covers the four handler representations, exact getter and
trap receivers, a callable Proxy trap, an abrupt lookup sentinel, nested-Proxy
target fallback and created-realm revoked/non-callable errors. Its structure
target now has five tests: the added source witness requires both exact CLI
registrations to remain active and unignored, pins their Wasm invocation and
success markers, and keeps every load-bearing fixture scenario fail-loud with
only within-scenario order assertions. Comment masking prevents disabled CLI
owners or commented-out fixture markers from satisfying that witness, including
multi-line `cfg` and `cfg_attr` attributes. At the 2026-08-25 coordinated
checkpoint, the structure target passes `5/5` and both exact CLI registrations
pass `2/2`. At vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the current-pin
`built-ins/Proxy/ownKeys` and `built-ins/Reflect/ownKeys` leaves contain 27 + 13
unflagged files and therefore 54 + 26 ordinary sloppy/strict executions. All
`80/80` pass under Wasm-AOT with every failure and non-success bucket at zero.

This is deliberately not the recursive Proxy descriptor-record protocol.
When `[[ProxyTarget]]` is itself a Proxy, `[[GetOwnProperty]]` must run that
Proxy's `GetMethod`, call and full `IsCompatiblePropertyDescriptor` validation;
re-entering the allocating public builtin would violate this seam. Handler-tag
preservation in that descriptor path, nested Proxy targets, full descriptor
compatibility and module-namespace exotics therefore remain explicit T11 work.
The direct-target batch closes the Array/arguments/integer-indexed/boxed-String
and ordinary invariant gap without claiming those cases.

The Proxy trap signature authority now has no incidental `Debug`, `Clone`,
`Copy`, `PartialEq` or `Eq` capability. The private eight-state domain is
created only by the complete thirteen-row `proxy_traps!` table and consumed
once by the exhaustive ordered argument-record projection. A recursive
structure guard pins all twelve source mentions, the sole lexically normalized
method route, every table row and all eight consumer bodies; it passes `3/3`.
The existing Set omitted-formals unit passes `1/1`. This derive-only closure
changes no inferred argument, emitted IR or runtime Proxy behavior and makes no
new Proxy/Reflect conformance claim. Independent dry review is clean, and the
shared format, `cargo xc`, diff, module-boundary and task-plan checkpoint is
green with the workspace's existing warnings.

The `[[OwnPropertyKeys]]` acquisition boundary now distinguishes its trap
scratch from its trap-result scratch with non-copyable
`ProxyOwnKeysTrapLocals` and `ProxyOwnKeysTrapResultLocals` roles. All four
Object/Reflect producers pass the result authority through the sole acquisition
and consume it once in their typed post-trap validator; existing distinct target
and handler roles close the remaining adjacent-pair transpositions. The robust
Rust-lexical census and ownership chain are pinned by
`proxy_own_keys_handler_protocol_structure`; the bounded source contract is
`docs/rust-rewrite/contracts/proxy-own-keys-result-ownership.md`.

## Objective

Implement every Proxy internal method and every Reflect method through the shared object/call protocols, including revocation and all invariant checks. Remove static-shape behavior that bypasses observable traps.

## Proxy scope

Support proxies over ordinary, callable, constructable and exotic targets. Implement:

- creation validation and `Proxy.revocable`;
- revocation behavior for every internal method;
- `getPrototypeOf`, `setPrototypeOf`, `isExtensible`, `preventExtensions`;
- `getOwnPropertyDescriptor`, `defineProperty`, `has`, `get`, `set`, `deleteProperty`, `ownKeys`;
- `apply` and `construct`;
- nested proxies and proxies as handlers/targets;
- realm-correct errors and target/handler lifetime.

Each trap must use `GetMethod`, invoke with the correct handler `this`, preserve argument order, and fall back to the target's real internal method when absent.

## Invariant checks

Implement all post-trap checks, including:

- non-configurable and non-existent property constraints;
- non-writable data/accessor consistency;
- non-extensible target restrictions;
- prototype equality requirements;
- `ownKeys` duplicate/type checks and exact inclusion constraints;
- callable/constructable target requirements;
- object-result requirements for descriptor/prototype/construct traps.

Do not weaken invariants for arrays, typed arrays, module namespaces or other exotic targets.

## Reflect scope

Complete all Reflect methods and route them to shared operations:

- `apply`, `construct`;
- `defineProperty`, `deleteProperty`;
- `get`, `set`, `has`;
- `getOwnPropertyDescriptor`, `getPrototypeOf`, `setPrototypeOf`;
- `isExtensible`, `preventExtensions`, `ownKeys`.

Reflect methods return booleans where specified rather than throwing on ordinary failure, while still propagating abrupt completions from coercion/traps.

## Acceptance criteria

- The full pinned `built-ins/Proxy` and `built-ins/Reflect` trees pass.
- Every trap has unit tests for absent trap, successful trap, thrown trap and invariant violation.
- Revoked proxies fail consistently for every operation.
- Proxy-wrapped functions/classes preserve call/construct/new-target behavior.
- Proxy operations work against arrays, typed arrays and non-extensible targets.
- No property/call fast path skips a possible proxy trap without a proven non-proxy guard.
- Nested proxy and cross-realm handler tests pass without materialization.

## Required tests

```sh
cargo test -p lila-aot-wasm proxy_ --quiet
cargo test -p lila-cli proxy_ --quiet
./target/debug/lila test262 run built-ins/Proxy --execution-backend wasm-aot --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Reflect --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

Re-run adjacent Object, Array, TypedArray and Function filters because proxy invariants are shared across them.
