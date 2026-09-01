# Proxy call and construct execution Realm

Status: focused-verified on 2026-08-29; dynamic-source materializers retired on
2026-09-01.

## Specification scope

The Proxy object's creation Realm does not select the Realm for work performed
by its `[[Call]]` or `[[Construct]]` internal method. The relevant algorithms
are:

- [Proxy `[[Call]]`](https://tc39.es/ecma262/2026/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist);
- [Proxy `[[Construct]]`](https://tc39.es/ecma262/2026/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget).

Both algorithms can create TypeErrors while they run. Both also expose an
Array created by `CreateArrayFromList(argumentsList)` when they invoke a
present trap. Those algorithm-created values belong to the current execution
Realm.

## Closed Realm source

`ProxyExecutionRealmSource` classifies every function body into one of four
states:

- `MainRealmFallback` covers the main body and ordinary user or host bodies;
- `StandardBuiltinEnvironment` covers a standard builtin whose self-backed
  environment records its defining Realm;
- `ObjectReadHelperArgument` covers the outlined ordinary and Proxy-aware
  object-read helpers, whose parameter 6 preserves the projection while an
  accessor or Proxy trap is invoked;
- `ProxyDispatchHelperArgument` covers the outlined `ProxyCall` and
  `ProxyConstruct` helpers, whose parameter 6 contains the already-projected
  standard-builtin environment or zero.

Initial bodies derive this source from the existing closed body
classification. `for_runtime_helper` exhaustively assigns every
`RuntimeHelperId`; only `ObjectRead` and `ObjectReadProxy` receive
`ObjectReadHelperArgument`, while only `ProxyCall` and `ProxyConstruct` receive
`ProxyDispatchHelperArgument`. A new helper or Realm source therefore requires
an explicit mapping before Rust builds.

The private `ProxyExecutionRealmAccess` projection has only
`TrustedCurrentEnvironment` and `MainRealmFallback`. Standard builtins, the two
object-read helpers and the two Proxy helper bodies select the trusted route.
All other bodies select the fallback. An ordinary lexical-environment pointer
cannot be interpreted as a function object or Realm record.

## Consumers

`emit_proxy_execution_realm_argument` is the sole projection into the
Proxy-sensitive helper ABIs. It emits the trusted current environment or zero.
Direct outlined Call and Construct entry points use it, and nested Proxy calls
use it again before entering the call helper. Ordinary and Proxy-aware
object-read helper entries also restore and forward it before invoking accessor
getters or Proxy traps. The helper chain therefore preserves the original
execution-Realm word through nested targets, handler reads and callable Proxy
traps.

`emit_proxy_execution_realm_type_error` owns every TypeError emitted directly
by the shared Proxy Call and Construct dispatchers. It selects the current
standard-builtin Realm only for a trusted source and otherwise constructs the
entry-Realm TypeError. The former Proxy creation-Realm TypeError prototype
field, allocator write and defining-Realm loader are deleted.

Trap acquisition uses the shared object-read operation. Proxy Call and
Construct helper bodies therefore select
`ObjectReadErrorRealmSource::ProxyDispatchHelperArgument`, while both outlined
read helpers select `ObjectReadErrorRealmSource::ObjectReadHelperArgument`.
Each hop forwards the same trusted word. A revoked Proxy handler or callable
Proxy accessor cannot lose the execution Realm during `GetMethod`.

After each dispatcher snapshots the argument list into a normal Array,
`emit_install_proxy_execution_realm_array_prototype` installs the current
execution Realm's `%Array.prototype%` before the trap can observe the Array.
The trusted route reads the standard builtin's defining Realm. The fallback
route installs the entry `%Array.prototype%`.

## Test262 materialization

`built-ins/Proxy/apply/null-handler-realm.js` uses its unchanged pinned source
and the complete LocalMerged Realm and assertion preludes. The apply and
construct rewrite dispatchers are now deleted. Their four remaining sources
also survive unchanged: both `arguments-realm.js` leaves retain their
indirect-eval source, and both new-target-Realm construct leaves retain their
ordinary-Function source. One exact four-path harness boundary reports them as
T13 Wasm-AOT unsupported dynamic-source cases because the compiler does not yet
type these created-Realm property calls. They are not replaced with handwritten
Proxy successes.

The unchanged apply and construct `null-handler-realm.js` leaves are the exact
current-execution-Realm checks for revoked cross-Realm Proxies. Each passes both
sloppy and strict Wasm-AOT execution (`4/4` together). The unchanged apply and
construct `trap-is-not-callable-realm.js` leaves also pass both executions
(`4/4` together) as neighboring generated-TypeError controls. The full Proxy
apply and construct directory results recorded earlier do not prove this
raw-source follow-up.

`crates/lila-cli/tests/fixtures/wasm_proxy_dispatch_execution_realm.js` is the
bounded runtime witness. It contrasts direct calls with borrowed created-Realm
`Reflect.apply` and `Reflect.construct`, checks generated TypeError prototypes,
checks fresh trap argument Arrays and their prototypes, and covers a callable
Proxy apply trap that re-enters the outlined helper. Revoked Proxy handlers and
revoked callable Proxy accessors pin both object-read hops. A user function
with 13 initialized captured bindings pins the ordinary lexical-environment
fallback.
The fixture is registered as
`object::run_wasm_backend_uses_execution_realms_for_proxy_call_and_construct`;
its exact Wasm-AOT run passes `1/1`.

## Verification

The coordinated checkpoint passed the four focused and neighboring structure
targets `16/16`, the three matching exhaustive Realm-source projection units
`3/3`, the raw-source harness unit `1/1`, the exact CLI witness `1/1`, and the
four raw Test262 leaves `8/8`. The all-target `lila-aot-wasm`, `lila-cli` and
`lila-test262` compile and the formatting check passed with only the existing
Boa trivial-cast warning. Module-boundary, task-plan, shortcut-inventory and
diff checks also passed. At that checkpoint the exact shortcut baseline was
239 entries: 31
legitimate reductions, 47 diagnostic observations and 161 semantic shortcuts;
T11 owns 8.

## Nonclaims

This contract does not add dynamic source evaluation, complete Proxy or
Reflect, or prove the full pinned Proxy tree. It does not change user-thrown
trap or accessor completions. Proxy `[[Get]]` TypeErrors other than revocation,
including a noncallable live `get` trap and Proxy invariant violations, still
use the object-read error authority and are separate T11 work. It does not add
a Realm word to the separate
`IndexedElementRead` helper, which cannot occur during the named `apply` or
`construct` trap lookup covered here. The apply and construct dynamic-source
leaves remain unsupported until T13 can compile their created-Realm source; no
Test262 materializer hides that boundary.
