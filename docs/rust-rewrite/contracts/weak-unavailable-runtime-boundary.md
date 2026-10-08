# Weak reachability unavailable at the actual builtin boundary

## Whole GC preflight source — 2026-10-05

All sixteen actual native routes consume complete values. Constructor validation
retains its required-NewTarget, first-argument and once-only prototype-Get order,
including the original whole Throw and builtin error Realm. The four prototype
fallbacks select typed Realm intrinsic slots. CanBeHeldWeakly reads the actual
GC Symbol registry-key field, following [CanBeHeldWeakly](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-canbeheldweakly);
unregistered and well-known Symbols are admitted,
while registered Symbols fail before prototype access.

The closed unavailable runtime selection cannot mint a weak instance. Method
receiver checks therefore reject before argument/key/callback observation, with
no raw brand, address, cells or strongly retained substitute. Extending the
runtime capability domain requires handling the new case in the exhaustive
native route. Valid construction still raises the typed host capability failure
outside JavaScript completion handling after preflight.

Existing finite Realm/order and capability controls remain, with additional
well-known Symbol and Proxy object target cases. All controls are unrun
for this atomic source. Compilation, runtime and acceptance remain pending after
the full task code/types/controls/docs phase, under the confirmed aggregate
4096 MiB cap and serial workers. The earlier evidence below remains historical.


The selected product runtime has one shared `WasmWeakReachabilityCapability` authority in `lila-ir`: `PRODUCT_WASM_WEAK_REACHABILITY` is `Unavailable`. Both the Wasmtime policy and the actual Wasm builtin dispatch consume it exhaustively. Enabling strong Wasm GC does not provide JavaScript weak references or ephemerons. This correction does not implement a weak facility or complete the semantic GC cutover.

All sixteen WeakMap, WeakSet, WeakRef and FinalizationRegistry constructor/method routes enter the closed `WeakBuiltin` emitter. That emitter owns ordinary preflight and the selected capability rejection together. Its unavailable arm emits `RuntimeSemanticRejection::UnavailableCapability(RuntimeUnavailableCapability::WeakReachability)` through the existing mandatory out-of-band rejection import, followed by `unreachable`. Wire codes 0–8 retain their meanings; code 9 is the new closed reason. A JavaScript catch cannot intercept this host failure.

Constructor arguments are evaluated by the existing Call/Construct dispatcher before entry. The four weak constructors remain in its direct-returning constructor set, so it does not observe `NewTarget.prototype` or mint a weak instance first. Every constructor checks NewTarget. WeakRef checks CanBeHeldWeakly(target), and FinalizationRegistry checks IsCallable(cleanupCallback), before GetPrototypeFromConstructor. WeakMap and WeakSet observe the prototype before weak-data initialization would begin. Abrupt prototype getters retain their JavaScript completion and defining Realm. Successful preflight rejects before allocating weak records, fetching collection adders/iterators, keeping targets, or creating registry cells; even empty WeakMap/WeakSet construction needs the unavailable facility.

Methods validate their receiver's object/brand domain before arguments that belong inside the method algorithm. These checks retain the intrinsic TypeError Realm and do not invoke property hooks or read boxed weak payloads. The remaining argument checks preserve invalid-key false/undefined policies, weak insertion admission, computed callback callability, and FinalizationRegistry target/holdings/token ordering. There is no producer of a successful private weak brand in this policy; a valid receiver route rejects before any weak storage operation.

The active strong-record weak implementations are deleted. Map/Set shared algorithms and upsert selectors have only their strong variants, so an actual caller cannot select WeakMap or WeakSet storage. Installed constructors, functions, prototypes, descriptors, Symbol.toStringTag and constructor inspection remain independent ordinary intrinsic operations. Existing passive weak layout metadata is an inventory for the pending whole GC switch, not a weak value producer, scanner, collector, or runtime capability.

Engine errors retain the typed unavailable capability independently of dynamic-source reasons and compiler semantic gaps, including worker aggregates. Actual JavaScript exceptions, timeout and trap retain their outcome priority. Test262 classifies a pure unavailable capability as Unsupported/NotImplemented and only a direct JavaScriptException can satisfy a Wasm runtime-negative test.

Primary ordering authorities are ECMA-262 2026 [WeakMap/WeakSet constructors and methods](https://tc39.es/ecma262/2026/multipage/keyed-collections.html) and [WeakRef/FinalizationRegistry](https://tc39.es/ecma262/2026/multipage/managing-memory.html). The pinned Wasmtime 47.0.0 source distinguishes strong GC roots from its internal host-owned-root handle liveness; no usable JavaScript weak-reference/ephemeron facility is selected by this backend.

This packet is source-only. Formatting, hashes and ordinary patch transport do not establish Rust type correctness, emitted Wasm validity, runtime behavior or Test262 PASS. Compilation and focused/broad executable verification remain mandatory at the later authorized checkpoint.
