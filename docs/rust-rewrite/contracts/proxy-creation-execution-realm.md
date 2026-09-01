# Proxy creation execution Realm

Status: focused-verified on 2026-08-30.

## Scope

`Proxy` and `Proxy.revocable` create values and errors in the Realm of the
builtin that runs their algorithm. This contract covers four validation
TypeErrors, the ordinary record returned by `Proxy.revocable`, and the revoke
function stored on that record.

This ownership is separate from Proxy `[[Call]]` and `[[Construct]]`. Those
internal methods preserve their caller's execution Realm through the helper
chain described in `proxy-call-construct-execution-realm.md`.

## Execution Realm source

`ProxyCreationExecutionRealm` is a non-copyable, must-use context. Its factory
selects the active builtin function before reading any intrinsic:

- a nonzero standard-builtin environment is a self-backed function object, so
  its `[[Realm]]` field supplies the Realm;
- a zero environment selects the canonical main Proxy constructor and then
  reads that function's `[[Realm]]` field.

The zero-environment branch must not read `CURRENT_REALM_GLOBAL_INDEX`. Promise
jobs can replace that global while they run, so it is not proof of the main
Proxy builtin's defining Realm. Both main Proxy creation builtins share the
canonical main constructor's Realm. Created-Realm bootstrap self-backs its
`Proxy` constructor and `Proxy.revocable` function before publishing either
one, which makes the nonzero branch available for borrowed calls.

After selecting the Realm, the factory loads its intrinsic record and derives
the Object, Function and TypeError prototypes through one closed intrinsic
enum. Null Realm, intrinsic-record or prototype identities trap as compiler
invariant failures. The context keeps the three prototype locals coupled to the
same Realm and releases every retained local through one consuming operation.

## Consumers

The context owns every Realm-dependent product in the two creation algorithms:

- the target and handler validation branches in both builtins construct their
  TypeErrors with the context's TypeError prototype;
- the revocable record is allocated with the context's Object prototype;
- the hidden `[[ProxyRevoke]]` target receives the context's defining Realm,
  Function prototype, TypeError prototype cache and self environment; and
- the Proxy-only bound-function allocation variant borrows the complete context
  and installs its Function prototype on the exposed revoke function.

The ordinary `Function.prototype.bind` allocation branch remains separate and
unchanged. Neither Proxy creation body nor its context module may select
`TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX`, `OBJECT_PROTOTYPE_GLOBAL_INDEX` or
`FUNCTION_PROTOTYPE_GLOBAL_INDEX`.

## Runtime witness

`crates/lila-cli/tests/fixtures/wasm_proxy_creation_execution_realm.js` borrows
`Proxy` and `Proxy.revocable` from a created Realm. It checks the exact created
Realm TypeError prototype for invalid target and invalid handler inputs to both
builtins. It also checks the revocable record's Object prototype, the revoke
function's Function prototype, the revoke function's `length` and `name`, and
two consecutive revoke calls.

The fixture keeps the returned host Realm record only to reach its global
object. It does not mention `evalScript`; string-pool planning for that
host-published property belongs to the createRealm boundary and its focused
`data.rs` unit test.

## Focused verification

The 2026-08-30 implementation checkpoint produced these results:

```text
cargo check -p lila-aot-wasm --lib
PASS, with only the existing Boa trivial-cast warning

cargo test -p lila-aot-wasm --test proxy_creation_execution_realm_structure -- --test-threads=1
PASS: 3 passed, 0 failed

cargo test -p lila-aot-wasm bound_this_capture_has_closed_producers_and_call_time_adaptation --lib -- --test-threads=1
PASS: 1 passed, 0 failed

cargo test -p lila-cli --test cli -- --exact object::proxy_creation_uses_the_builtin_execution_realm --test-threads=1
PASS: 1 passed, 0 failed, 786 filtered

./target/debug/lila --jobs 1 test262 run built-ins/Proxy/revocable --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000
34 Success of 35 executions; the sole non-success is tco-fn-realm.js as the typed $262.evalScript AOT NotImplemented boundary; every failure, Crash and Bug bucket is zero
```

The targeted Rust formatter and scoped diff check also passed. The structure
target pins the closed context and local lifecycle, the main fallback, all
consumers, hidden revoke target fields, created-Realm self-backing, CLI
registration and every load-bearing fixture observation.

## Nonclaims

This contract does not change Proxy `[[Call]]`, `[[Construct]]` or any other
Proxy internal method. It does not implement or invoke dynamic `evalScript`,
remove the remaining T13 boundary or complete T11. No broad Cargo suite,
semantic golden or full Realm matrix ran at this checkpoint.
