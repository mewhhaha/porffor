# Plain async With

This source batch gives actual plain async function With statements a complete
head/body/cleanup owner. Analysis selects `PlainAsyncWhole` only in the current
FunctionBody graph through Block, If, Try, Switch, Labelled and With. Foreign
loop bodies retain their original eager/linear ownership, and AsyncGenerator
remains separate. Strict With remains an ECMAScript early error.

`AsyncWithSource` consumes the actual AST expression planner already used by
async patterns. Its statement planner includes real If branch reservations,
try/catch/finally boundaries, selector-first Switch ranges, labelled exits,
recursive With and complete Array pattern owners. Each source Await identity
must match the actual lowered tape. Known static If folding is suppressed only
in that consumed source scope or around a real complete With phase child.
Known-nullish optional tails keep their checked source ranges while skipping
all runtime Await/call observations. Nested callable bodies own separate tapes.

The head finishes once in the outer environment, including ToObject, and stores
the boxed object in one actual invocation-owned cell. The body enters or
reattaches the original analyzed Object Environment Record and its original
hidden object cell. The native Generator/Async adapters share the physical
With pipeline. Cleanup is rebuilt before resumed rejection delivery; it leaves
and saves the original environment while preserving the entire Completion.
Normal Await retains the record and captured closures.

Awaited Identifier plain, compound and logical assignments capture the original
Reference before RHS evaluation using the existing private native record.
Write-only capture does no GetValue. Compound/logical capture performs GetValue
once before the RHS. Normal Put consumes the record; skipped logical arms
release it; committed Throw/Return uses the existing activation retirement.
With var initializers use the same original Reference while hoisting remains
in the original variable environment. Eager assignments and lexical binding
initialization retain their existing authorities.

IR controls cover original cells and state tapes, complete branches/Array
patterns, nested activation separation, Reference ordering and explicit foreign
scope refusals. Two real JS-to-Wasm fixtures cover Proxy HasBinding/unscopables,
changed objects across Await, captured escaping closures, interleaved async
activations, array defaults, nested/labelled With, and whole rejection/return
through awaited finalizers. They are authored and unrun. No compilation,
runtime, guard, generator or metadata verification was performed in this batch.

Mixed async-generator graphs, complete async classic/ForIn/ForOf loop ownership
and implicit awaited resource-disposal source plans remain separately owned.
The source planner retains explicit boundaries rather than replaying unowned
phase children through an eager loop.
