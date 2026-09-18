# Plain async conditional continuations

## Demonstrated failure

The completed-baseline follow-up reproduced two plain-async `if`/`else` errors
on frozen compiler checkpoints 6 and 7. A false branch containing `await`
fulfilled with `undefined` instead of the expected `5`. In a captured-cell
probe on checkpoint 7, the true branch read its pre-await value `1` instead of
its updated value `2`, while the false branch again returned `undefined`.

Ordinary `If` had no continuation owner: the surrounding async sequence did
not know its entry and join states, and an untaken branch could be scheduled
from the wrong state. Materialized block environments were also allocated
again on resume, disconnecting direct writes from closures created earlier.

## State ownership

`AsyncFunctionIfPlanIr` has private state fields and one checked constructor.
The condition owns the entry state. Each branch owns a disjoint, nonempty
range, and normal completion advances to one shared exit state. The constructor
rejects reversed branch boundaries and state overflow. Branches that do not
consume continuation states retain ordinary `If` and return their reserved
states to the containing sequence.

Only the plain-async lowerer constructs `AsyncFunctionIf`. It stages supported
condition awaits before allocating branch ranges. The backend evaluates the
condition once, records the chosen range and resumes directly within that
range. It uses the existing tracked control frames and completion dispatch for
return, throw, rejection and finalization. Its join is the next sequential
statement's entry, including a following await or another conditional.

AOT planning, early-error traversal, suspension summaries and throw inference
visit both branches. The existing constant-Number condition optimization
remains attached to ordinary `If`; it does not bypass this continuation owner.

## Environment lifecycle

The plain-async activation has separate pointers for its stable invocation
Environment Record and its saved active lexical chain. Both are registered as
heap pointer fields. Initial invocation creates the former once; every plain
`AsyncAwait` saves the latter before returning to the job queue.

A materialized block enters only while its continuation range is active. Fresh
entry allocates its Environment Record. Resumption walks the saved chain to the
existing child of the already restored enclosing record, then attaches the
compiler's binding view. The chain must contain that child; a missing record is
an internal invariant failure, not a request to allocate replacement cells.
The range guard prevents an inactive sibling block from looking for an
unallocated or already exited record.

The function-body environment continues to use its existing parameter/body
record relationship. Existing loop owners restore their saved iteration
record. Plain-async catch entry allocates its parameter record after a new
throw and before choosing binding storage; catch resumption restores that same
record before entering the catch body. Thus parameter defaults, body variables,
block cells, catch cells and admitted loop cells retain their distinct owners.
The async activation grows from 144 to 152 bytes; its existing offsets remain
unchanged.

## Regression coverage and verification

`lila-ir` includes three constructor tests and five lowering tests for checked
state ranges, empty branches, condition awaits, nested branches, sequence joins
and rollback of unused reservations. `aot_async_if` includes 15 Wasm-AOT
behavior tests covering both branch directions, one condition evaluation,
constant Number conditions, empty and non-suspending branches, condition awaits,
effect order, captured branch and nested/sibling block cells, default parameter
versus body bindings, return/throw/rejection/finally, labels wholly within a
resumed branch, independent pending invocations, recursive async calls,
plain-async synchronous for-of iteration cells and resumed catch parameters.

This stage has only non-compiling review and formatting checks. Compilation,
native execution and Test262 verification are delegated to the integrating
checkpoint and are not claimed here. Published real-suite counts are unchanged.

## Remaining boundaries

The patch does not admit new generator or async-generator suspension shapes.
The four separately reproduced unsupported generator probes remain separate
work. Existing loop admission rules, branch-sensitive awaits inside expressions
and an outer labelled block containing a suspension retain their independent
continuation boundaries. In particular, the label tests here exercise labels
inside the resumed branch; they do not establish suspension support through an
enclosing labelled block. This is not complete async or Test262 conformance.
