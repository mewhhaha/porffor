# Array.fromAsync internal callback Realm context

Status: implemented on 2026-08-26; focused verification is recorded below.

## Ownership

Each Array.fromAsync execution materializes one fulfillment/rejection callback
pair. Both callbacks are builtin closures owned by the executing method's Realm:
their defining Realm, default Function prototype and Realm TypeError prototype
come from the same typed context as the intrinsic Promise constructor. Their
continuation state is separate execution-specific data stored in the builtin
closure's GC-visible context slot.

The callback object self-backs its environment handle. The environment is not a
raw alias for continuation state, and the continuation state does not cache or
restore a second Realm environment.

## Typed boundary

`ArrayFromAsyncExecutionRealmContext` is private, non-`Copy` and must-use. Its
factory acquires the canonical Promise constructor, defining Realm, default
Function prototype and TypeError prototype as one ownership unit. Missing
catalog state traps before either callback can be materialized.

`emit_array_from_async_internal_callback_pair` borrows that context and is the
only raw producer of the two callback values. A fixed two-member loop installs
the complete Realm-owned function header, the continuation-state context and
the self-backed environment together. The array-like and iterable branches each
call the pair materializer once; neither branch can independently choose a
Realm, prototype or state transport.

## Producer and consumer census

Before this boundary, two branches performed four raw callback allocations and
four raw state-as-environment writes. Both callback bodies recovered state from
raw argument zero, restored a Realm environment cached at byte offset 176 and
therefore required a 184-byte state record.

After this boundary, the pair materializer contains one raw allocation site
covering the fixed fulfillment/rejection domain. It stores two closure-context
links and two self-backed environment links. The two callback bodies are the
only consumers; each loads state once from the builtin-closure context slot.
The Realm-environment field is deleted and the state record is 176 bytes. All
nine await scheduling sites continue to load the same rooted pair from the
state record.

## Observable evidence

The finite CLI fixture covers both callback kinds in both source modes. The
array-like fulfillment control preserves the materialized result, while the
async-iterable fulfillment control forces a TypeError inside the continuation
and observes the created method Realm's TypeError prototype. The two rejection
controls preserve object identity through the same array-like and iterable
continuations. The marker is printed only after all four controls pass.

The bounded structure target pins the complete function header, the two branch
producers, closure-context-only state recovery, the deleted raw Realm/state
convention, all nine scheduling sites, planner roots and fixture registration.
The structure target passes `5/5` and the finite CLI target passes `1/1` on
2026-08-26.

```sh
cargo test -p lila-aot-wasm --test array_from_async_internal_callback_realm_context_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_preserves_array_from_async_internal_callback_realms --quiet
```

The shared semantic golden passes `2/2` in 722.99 seconds with 678 dumps. It
adds this witness plus the independent Object-policy, Promise-mode and Set-domain
witnesses, removes none and leaves all 674 retained dumps equal after accounting
normalization. Broad Test262 verification remains deferred.

Errors created later by Array result property definition and length publication
are owned by the separate
[`array-from-async-result-definition-error-realm.md`](array-from-async-result-definition-error-realm.md)
boundary. They consume this callback's corrected Realm environment but remain
independent evidence from the callback header and Promise-job Realm itself.
