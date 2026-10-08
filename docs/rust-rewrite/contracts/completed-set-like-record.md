# Completed set-like records

`CompletedSetLikeRecord` is the actual Wasm-AOT `GetSetRecord` result. Its
private fields retain the argument, converted size and once-observed `has` and
`keys` methods. The getter/conversion/callability prelude alone constructs this
non-copyable owner. Set predicates and algebra require the whole record at
each iteration-helper boundary instead of accepting independently supplied
receiver, size and method locals.

Receiver validation still precedes argument observation. The prelude observes
size, performs Number conversion, rejects NaN and negative integer sizes, then
observes and validates `has`, then `keys`. Every abrupt path emits the existing
completion return before later observation. Borrowed records retain each
method's original receiver through user callbacks and iterator closing.

The owner retains seven temporary locals. The constructor releases its two
private scratch locals before returning; iteration helpers release their own
scratch above the record. One consuming release operation restores the seven
retained locals in exact reverse order before the caller releases its earlier
receiver/result locals. This preserves the builder's strict local-stack rule.
It does not prove that arbitrary raw locals elsewhere belong to one builder.

This is a dry implementation change dated 2026-10-03. Existing semantic Set
copy-phase, live-mutation, duplicate/zero and protocol-order fixtures remain
the verification coverage. Compilation and execution are deferred to the
coherent batch checkpoint. Full collection conformance and weak reachability
remain open.
