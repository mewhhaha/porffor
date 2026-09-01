# Retired async Array walk form

Status: retired and replacement verified on 2026-08-29.

`AsyncForOfArrayWalkForm` once classified whether a plain async loop with one
direct body `await` could use a resumable Array index walk. That model is now
deleted together with `lower_async_for_of_array_with_body_await` and
`ARRAY_INDEX_WALK_RESUMABLE`.

The replacement is `StatementIr::AsyncFunctionForOfIterator` with a required
`AsyncFunctionForOfIteratorPlanIr`. The plan persists a real synchronous
Iterator Record across the body await. It owns the body split, state order, and
environment lifecycle instead of proving that Array protocol operations may be
skipped. No Array classifier remains, so synchronous String and custom iterable
sources use the same path.

The old four-mention classifier structure test is retired with the type. The
replacement bounded target is
`crates/lila-aot-wasm/tests/plain_async_sync_for_of_iterator_record_structure.rs`.
It pins the closed plan, typed slot allocation, dynamic yielded value,
emission-site join, additive temporary-local budget, and absence of the old
index synthesis and witness.

See `synchronous-array-for-of-iterator-protocol.md` for the runtime semantics,
focused fixtures, exact verification results, and explicit source boundary.
