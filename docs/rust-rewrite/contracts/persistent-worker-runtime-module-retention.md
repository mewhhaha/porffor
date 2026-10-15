# Persistent workers retain immutable runtime code

Persistent Test262 workers retain the program-independent Wasmtime runtime
Module in the existing bounded in-memory module LRU. Unique program Modules
continue to bypass retention. The cache counts actual native compilation image
bytes and keeps the same entry and byte budgets, eviction and oversized-image
rules. No separate unbounded runtime cache is introduced.

Runtime keys contain the exact runtime Wasm digest and the execution Engine's
native compilation mode. The two process-shared Wasmtime Engines own their
respective keys. Agent workers still bypass all retention, since their group's
Engine is not the process-shared Engine. An explicit full bypass keeps its old
meaning for both runtime and program Modules.

A cached Module owns immutable compiled code, not an Instance, Store, heap,
Realm, roots, jobs or agents. Every request instantiates fresh runtime and
program modules in a fresh Store. Script and Module worker entry points use
the same runtime-only policy; agent entry points retain their existing policy.

This removes repeated runtime native-image deserialization after a retained hit.
The earlier focused trace measured about 132 ms for runtime-module loading; it
is a motivation, not a measured improvement for this change. Native controls
and a matched current-source timing sample remain required before acceptance.
