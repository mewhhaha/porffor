# Complete mixed lexical resource scopes

One checked source statement list owns all direct `using` and `await using`
registrations, earlier statements and the remaining suffix. A mixed list selects
one original async DisposeCapability. It does not create a separate capability
for every declarator or split the original null-entry Await behavior.

Each initializer uses the shared mixed expression pipeline and is retained once.
The registration proof consumes this actual Variable and the original
PendingInitialization/InitializedBinding conversion. Runtime GetMethod and
registration precede InitializeBinding. Later initializer rejection disposes
the resources already registered, while unreached entries remain absent.

Registration operations inhabit only their checked resource body. Ordinary mixed
regions reject bare operations. The original disposal machinery owns reverse
order, method receivers, Await, SuppressedError chains and whole completion
dispatch. The source allocator reserves its existing single finalizer plan and
records the implicit resume in actual enclosing-scope metadata.

The current owner covers function-body and Block lexical lists, including nested
whole mixed loops, With and iterator owners. Iterator resource heads own one
per-key capability spanning registration and the complete body. Continue disposes
before advancing; other abrupt completions dispose before IteratorClose. The
original head TDZ and distinct per-key cells remain separate through async
disposer suspension. Classic resource heads own one capability across init,
test, body and update, including Continue; it closes only when that whole loop
exits. A switch clause may contain a resource scope inside an explicit Block;
direct clause-level resource declarations are early errors. Each entered Block
retains its capability across suspension and closes it on fallthrough or abrupt
exit from that Block. Unreached declarations never register. These owners use
the same checked Variable registration and their original lexical finalizer.

Authored IR and Engine artifact controls cover staged multi-declarator
registration, mixed/null entries, original TDZ/captured bindings, earlier live
resources on later rejection, GetMethod failure, nested reverse disposal, selected
References, queued/injected completions and suppression. The 2026-10-07 collection
found invalid direct clause declarations in the resumable fixture family. The
joined fixture correction uses legal Blocks and their actual disposal order;
all nine corrected resource parsing/lowering checks pass with no diagnostics.
The binding-alias repair passes the registration fixture in both modes. The
completion fixture checks resource liveness inside finally, before a queued
request can advance disposal; its whole-completion and cleanup observations pass
in `tasks-resource-realm-followup1` (128.82 seconds). Registration and completion
observations now have independently selectable native tests. Broader task
acceptance remains separate from these scoped passes.
