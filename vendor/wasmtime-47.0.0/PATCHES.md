# Lila collector patch

This directory vendors the crates.io Wasmtime 47.0.0 release under its original
Apache-2.0 WITH LLVM-exception license. The workspace selects it through
`[patch.crates-io]`; the version and other Wasmtime dependency pins remain 47.0.0.

The local changes add Store-scoped weak-target and ephemeron handles. Their
generational slots are outside the ordinary strong-root census. The copying
collector processes conditional values to a fixed point after strong tracing,
then updates or clears weak edges before sweeping ExternRef host data. A value
requires both a live owner and a live key. Other collectors reject registration.
Upgrades create a scoped strong root while collection is forbidden; stale or
foreign-Store handles cannot access another registration. Conditional marking
uses sorted owner/key dependencies and first-copy notifications to activate
each edge once, including chains through boxed values. All queue and retirement
storage is reserved before flipping the heap. Store-scoped, epoch-scoped opaque
key tokens permit non-rooting host lookup; every copying cycle invalidates them
before movement, even when no weak edges exist. Other collectors reject the
token capability by default.

The changes are confined to `src/runtime/gc/enabled{.rs,/weakref.rs}`,
`src/runtime/vm/gc{.rs,/weak.rs,/gc_runtime.rs,/enabled/copying.rs,
/enabled/copying/{weak.rs,marking.rs}}` and a cancellation control in
`src/runtime/store/gc.rs`. They do not change the JIT heap-header or VMContext ABI.
The manifest adds an independent unit-test workspace, the existing local cache
patch and matching dependency optimization profiles. Its separate lockfile pins
the upstream test dependencies; it does not replace the root workspace lockfile.

Three upstream test feature guards are corrected so the collector control can
compile with the product's narrow feature set plus `wat`: Linux copy-on-write
tests in `src/runtime/vm/cow.rs` require `pooling-allocator`, and the hash-key
controls in `src/runtime/{memory.rs,externals/global.rs}` require `coredump`,
matching the methods they call. Those controls remain present and compile when
their dependencies are enabled. This is not a full upstream Wasmtime test run.

Collector tests exercise actual reachability and moving references. JavaScript
weak builtins require additional compiler, host and job integration before the
product capability can be enabled. See the
[collector contract](../../docs/rust-rewrite/contracts/collector-weak-edges.md).
