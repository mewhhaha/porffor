# Generator optional Property/Call yield ownership

Status: dry source implementation. Compilation, Wasm validation and execution
remain pending. Published conformance results remain unchanged.

An ordinary generator outside loop regions owns a finite optional chain with
an eager base, Property and Call links, and yielded computed keys or arguments.
Each source operand is eager or contains one direct nondelegated plain Yield
through the existing checked staging grammar. Several such operands may occur
in source order across a chain. Nested branch operands, more than one Yield
inside one operand, a yielded base within this new checked chain domain, mixed
Await/Yield, async generators, loop regions, direct private/super targets and
private links remain refused before state consumption. Grouped yielded optional
terminal-Property callee/tag References and direct delete retain their separate
boundaries. Terminal Calls now supply a checked completed Value to an ordinary
outer Call/tag through the [grouped yield owner](grouped-optional-call-value-yield-ownership.md).
The full chain completes before the receiver-free outer callee pin, arguments
and original template site. Ordinary outer operands remain unconditional after
inner shorting; the same source plan owns guarded chain and outer operand states.
The extended controls remain uncompiled and unexecuted.
Synchronous chains and independently admitted target-only paths keep their
existing route.

The checked source and state owners retain the actual link shorted flags,
property fields, argument/spread sources and guarded suspension order. Whole
source append checks arithmetic before publishing any suspension point. Every
yielded operand receives one existing scalar GeneratorIf selected resume and a
fresh join. Eager statements between these steps contain no suspension. Nested
GeneratorIf dispatch is unnecessary; the original generator frame, IR variants
and backend remain the sole execution owners.

A retained Boolean live cell becomes false only when an actual shorted link
observes strict null or undefined. Every subsequent key, Get, argument, spread
and Call is guarded by that cell, so shorting skips the complete suffix. An
ordinary link leaves live unchanged even when its receiver is nullish: its
raw key evaluation still precedes the existing RequireObjectCoercible and
ToPropertyKey ordering. Falsy primitives and HTMLDDA do not short the chain.

Property links retain the original GetValue once before later suspension. A
following Call consumes its completed callee and raw receiver together through
the existing optional Reference capture; no Get runs again to recover this.
Arguments retain their evaluated identities in source order, and spread uses
the existing private captured argument vector before a later Yield. Callability
is tested after argument-list evaluation. A completed Call resets a subsequent
Call's receiver to undefined; a subsequent property creates a new Reference.
Optional eval retains the existing indirect route. Constructors consume a
completed optional-chain value with no call receiver, including a chain whose
terminal Call returns the constructor. The purpose-specific source walk retains
prior private/super constructor capture and ordinary argument routes. Outer
ordinary property References over a completed chain value keep their existing
owner; grouping ends optional shorting before that outer Get or invocation.

Generated live, receiver, callee, argument, received and result cells use the
existing activation-owned binding slots. Private staging scopes leave their
facts before selected/skipped flow facts merge. Only Normal completion
publishes an operand or final result. Injected Throw and Return pass through
the existing plain Yield completion owner before residual evaluation;
surrounding try/catch/finally and defining-Realm errors retain their owners.
The skipped chain result remains undefined.

Meaningful source controls compare checked suspension points with actual
scalar entries/resumes/joins and distinct activation cells, retained Get/Call
order, spread snapshots, complete skipping and explicit unsupported boundaries.
Engine controls exercise primitive/Proxy receivers, mutations across next(),
Call-result receiver reset, noncallable argument ordering, injected completion
and arbitrary foreign markers through finally in strict and sloppy wrappers.
These controls are authored sources, not execution evidence.
