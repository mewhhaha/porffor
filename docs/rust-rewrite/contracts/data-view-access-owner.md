# DataView access owner — dry source, 2026-10-03

Current source status, 2026-10-05: the atomic Wasm-GC rewrite is authored only. Compilation, emitted Wasm, focused controls, real agents and full pinned conformance remain unverified. No status counts changed.

All 22 native entries consume whole GC values and the closed eleven-element DataView domain. Brand and immutable-write admission precede the offset/value hooks. Number or BigInt conversion and Boolean endian selection complete before one fresh BufferOwner/BufferView observation. Bounds and detachment errors preserve the executing builtin Realm and original whole Throw. Access reads collector ByteArray bytes or the retained shared HostResource; no semantic address or manual header remains. Float16 consumes the sole scalar rounding leaf, and BigInt results publish canonical immutable limbs. DataView construction retains its first cached length check before prototype Get, then performs final detachment/offset/supplied-length checks after that Get.

Four paired strict/sloppy finite Engine cohorts in `aot_gc_binary_data_entries.rs` cover native buffers, DataView, TypedArray construction/statics/species and Atomics/Realm lifecycle. Existing CLI semantic fixtures remain; obsolete raw-spelling guards are retired rather than replaced with mirrors. The maintained `data_view_positive_bounds_realm_structure` and
`created_realm_data_view_publication_structure` targets retain their original
CLI semantic fixture witnesses. Raw allocator/offset and duplicate publication
mirrors are retired. Shared completed GC bootstrap now owns created-Realm
DataView installation. The historical implementation and receipts below do not
certify this batch; final representation/helper/guard composition and later
capped verification remain pending.

## Historical record before the atomic GC rewrite


All 22 DataView getters and setters delegate from the standard builtin dispatch
to the private `builtins::data_view_access` module. Its closed access and
11-element domains determine byte width, numeric conversion, signedness and
method error messages. Uint8Clamped cannot enter this DataView domain.

The private preparation constructor retains already evaluated argument values,
checks the DataView brand, performs ToIndex, converts a setter value through the
existing Number or BigInt authority, converts endian to Boolean, and then
observes the current backing pointer and view length. Detached/out-of-bounds
view TypeErrors precede the positive access RangeError. Only after both checks
does it form the address. This follows
[ECMA-262 GetViewValue](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-getviewvalue)
and [SetViewValue](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-setviewvalue).
The existing immutable-buffer setter preflight remains before offset/value
hooks, as required by the
[immutable ArrayBuffer proposal's SetViewValue](https://tc39.es/proposal-immutable-arraybuffer/#sec-setviewvalue).

The constructor produces separate private, move-only validated read and write
owners. The actual byte-load helpers require the read owner; the byte-store
helper requires the write owner. Those boundaries accept no raw address or
independently selected element width. Dispatch cannot substitute an unchecked
local or a write owner for a read owner. Read/write consumption releases retained
locals after its scratch locals, in reverse reservation order. No user hook
occurs between current-backing validation and raw memory access.

Integer stores still use the shared Number modulo authority, BigInt stores use
the existing low-word conversion, Float16 retains the existing rounding and
decode helpers, and Float32 retains demotion/promotion. Float64 bits pass through
unchanged. BigUint64 results above signed-i64 range retain one-limb heap BigInt
publication. Endian byte-observation order and every existing error-message
alias are preserved. DataView-authored brand, immutable-buffer, view and bounds
errors retain the executing builtin's Realm routes. The shared coercion error
authorities remain unchanged; this does not close their separate Realm policy.

This closes ten source-equivalent duplicated preparation/bounds/address owners;
inspection found no missing freshness check in the previous production paths.
The current binary-data validators, layout, constructor, accessor, Realm
publication and Atomics owners are unchanged. This change establishes no broad
shared-agent, Wasm-GC or T17 conformance claim.

The maintained `data_view_positive_bounds_realm_structure` assertions now follow
the shared preparation owner. Existing `lila-cli --test cli data_view::` targets
already cover all element kinds, offset/value ordering, custom thrown errors,
wrong receivers, immutable setters, detached/resizable views, SAB byte access,
endianness, Float16 and borrowed-Realm errors. No new mirror test or fixture was
added. Only source inspection, formatting and scratch patch checks have run;
compilation, emitted Wasm and focused/broad regression acceptance remain pending.
