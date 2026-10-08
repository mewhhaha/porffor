# Array.from proxy property-definition failures

The `emit_create_data_property_or_throw` proxy route must retain the completion
of its `Object.defineProperty` call until its caller's supplied close obligation
is discharged. The prior immediate propagation left this route before
`IteratorCloseOnThrowLocals` could be used. Ordinary non-configurable and
non-extensible failure routes already discharge that same obligation.

The proxy call now uses the existing `LeaveInCompletion` call wrapper. Its throw
branch first copies the call's payload and tag into the current completion,
then calls `emit_iterator_close_preserving_current_throw` when close locals were
supplied, then propagates through the current throw target. The successful call
retains the existing `Br(2)` exit; the added throw `If` is closed before that
branch. No new branch-depth convention, object model, runtime bridge or call ABI
is introduced.

The existing close helper saves the original payload, tag, completion kind and
auxiliary state, performs one close attempt in a local completion region, and
restores the saved throw. Errors from reading `return`, a non-callable `return`,
a throwing `return()` or a primitive return result cannot replace the original
property-definition exception. The `return()` receiver remains the iterator.

This follows [Array.from](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.from):
the iterable branch closes for mapper and property-definition failures, while
iterator step/value errors propagate directly. Its array-like branch has no
iterator to close. [IteratorClose](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratorclose)
preserves an incoming throw completion after attempting `GetMethod` and `Call`.
Both primary algorithms were inspected on 2026-10-03; their current draft labels
are 23.1.2.1 and 7.4.11.

The existing caller seam determines the obligation: Array.from's iterable
publication passes `Some(IteratorCloseOnThrowLocals)` and its array-like
publication passes `None`. The mapper path already saves and closes its own
throw. Iterator `next`, `done` and `value` paths are unchanged. Other helper
callers that pass `None` retain throw propagation and gain no close attempt.

`aot_array_from_iterator_close.rs` authors six Engine regressions, each with
ordinary and strict source modes. They observe exact acquisition/construction,
step, mapping, target-definition, return-get/call, outer catch/finally and
successful length-publication traces. They compare the caught object with the
original exception and require one close attempt for the property failure,
including every close-error case. Paired controls cover mapper failure,
next/done/value failure, noniterable array-like failure and successful proxy
publication. These are authored assertions, not executed results.

The complete unchanged pinned `staging/sm/Array/from-iterator-close.js`, `sta.js`
and `assert.js` are retained as witnesses. The frozen 2026-09-30 staging Array
snapshot records both sloppy and strict executions as Runtime/Bug with
`closed` false instead of true. That historical result is not a fresh failure
claim for this packet.

This separate queued batch6 packet is based virtually on current MAIN 2dbb and
the approved future2 composer 496f postimages. It has no materialized Source
binding, compiler result or runtime result. Batch4's two objects.rs signature
comments are separate disjoint replacements; their deterministic composition
must retain this close route. Task16 is supplied as an exact append, and the
README text is a proposal for later integration. Published counts are unchanged.
