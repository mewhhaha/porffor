# Existing memory32 allocation boundary

Status: source-only repair in the 2026-10-03 implementation batch. Compilation,
emitted Wasm validation and runtime acceptance remain pending.

The existing HeapAlloc helper returns an i64 that all current allocation
consumers narrow to a memory32 address. Its arithmetic already traps on size
alignment overflow, addition overflow and failed growth. Those checks alone do
not establish the address-width domain before a page count or pointer narrows.

The helper now requires the allocation start to fit u32 and the one-past end to
fit the memory32 address space, before computing/narrowing the growth count or
committing the heap cursor. An allocation may end exactly at the address-space
boundary; a later zero-sized allocation cannot return that one-past pointer.
Both checks use unsigned comparisons, so negative i64 bit patterns cannot be
accepted as addresses. The bounded end also makes growth rounding arithmetic
and its page-count narrowing representable.

The shared helper is the sole emitted private-heap allocation implementation.
The unreachable inline allocator in `emit_heap_alloc_from_local` is retired.
The real emitter derives its helper index from `uses_heap.then_some(...)`; all
current main, user, builtin and runtime-helper factories preserve that pair.
There is no hand-built or default test builder requiring a separate allocator.
The free object/function/property/Array helpers already take mandatory indices.

`emit_heap_alloc_from_local` retains its existing rejection when heap memory is
absent. A heap-enabled builder missing its shared helper now fails emission with
an explicit internal-invariant diagnostic; it cannot emit another allocator.
The existing LocalGet/Call success instructions and shared helper body are
unchanged. No new allocator, object model, fallback backend or representation
bridge is introduced. Existing successful alignment, growth, publication and
capacity-failure traps retain their shared owner. This does not implement the
required atomic semantic Wasm-GC graph switch or claim reclamation.

The maintained allocation test now inspects the emitted product helper and
requires both guards, in order before page-count narrowing, growth and cursor
commit. It retains the existing alignment/overflow/growth obligations. No new
stress workload, failure reproducer, oracle or executable check was run. This
source boundary does not diagnose or claim to fix any historical URI failure.
The inline deletion requires no new test mirroring an impossible compiler state;
the maintained product allocation test and its guard obligations are unchanged.
Both source changes still require the combined Rust, emitted-Wasm and focused/
broad verification checkpoint before executable acceptance.
