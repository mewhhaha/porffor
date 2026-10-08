# Created-Realm iterator-next publication

Status: typed GC source migration; compilation and execution pending.

Created-Realm native functions retain their defining Realm through immutable
`FunctionContext` GC capture before publication. Iterator next allocation and
receiver errors consume that actual execution context, rather than raw function
heap offsets or the current public Object/TypeError globals. Created-Realm
bootstrap owns publication of the concrete native functions and prototypes.

The common `emit_iterator_result_object_in_realm` accepts the supplied typed
Realm, a whole Value and a normalized Boolean done local. It allocates a fresh
ordinary object with that Realm's intrinsic Object prototype, then completes
own writable/enumerable/configurable value and done data properties. It does
not repair a prototype after publication. Native iterator steps and the
AsyncFromSync fulfillment consumer share this producer.

The common native entry convenience wrapper resolves the real executing Realm.
Borrowing local next onto a foreign iterator therefore returns a local result;
borrowing foreign next onto a local iterator returns a foreign result. Receiver
brand checks use concrete GC reference tests without getters or Proxy traps.
Errors use the defining native function Realm even after public globals change.

`aot_gc_iterator_entries.rs` authors both result directions, lone UTF-16
surrogates and astral pairs, private Iterator.from records, zero-argument callable
Proxy forwarding, fresh nullable return acquisition, poisoned public globals,
and trap-free wrong-brand rejection. The earlier CLI receiver/Realm fixtures
remain. The old `created_realm_iterator_next_publication_structure.rs` raw-slot
publication mirror is retired.

The 2026-08-29 focused structure/CLI/format/type results are historical predecessor
evidence, not verification of this GC source. All new controls are unrun. The
common completed GC bootstrap is now the actual created-Realm source owner,
consumed by `builtins/host/create_realm.rs`. Native helper entry/resume sources
are authored independently. Final representation/helper/guard composition,
compilation and execution remain pending after all source lanes finish;
created-Realm, iterator-helper and T22 acceptance stay open.
