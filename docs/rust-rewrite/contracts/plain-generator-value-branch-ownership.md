# Ordinary generator value branch ownership

Ordinary generator conditional and logical values use the existing checked
`OrdinaryGeneratorIfIr` region owner. Their selectors and selected arms may
contain multiple, nested or delegated yields. The same actual source plan
allocates their complete state ranges before lowering publishes the regions.
The experimental Wasmtime capability boundary and ordinary-generator GC
activation ABI remain required. This source successor has no type or runtime
acceptance result.

The private `generator_value_branch_source/branches.rs` owner admits actual
selector and arm expressions through `GeneratorExpressionSourcePlan`. Private
source fields prevent substituting an independently chosen arm. The selector
plan completes before the branch entry. Each arm has a distinct inclusive range,
including eager/skipped arms, followed by a distinct common exit. Checked state
arithmetic also reserves the final exclusive function state count. The outer
expression planner appends into temporary storage, so overflow publishes no
partial points or cursor update.

Lowering stages and retains the selector's actual GetValue before either arm.
Logical `&&`, `||` and `??` preserve that whole original value for the skipped
result. Nullish selection uses strict null/undefined equality and does not
coerce the selector. The complete selected arm stages its whole source in its
own scope, then publishes to the one shared activation result cell only after
Normal completion. Its actual statements and final state must admit against
the original source range before `OrdinaryGeneratorIfIr` can be constructed.
Temporary scope facts are removed before branch facts merge; mutable heap
shape facts are not carried through the result join.

The existing ordinary-generator backend dispatches every nested region and
plain/delegated Yield. Injected Throw or Return leaves through the existing
completion/finalizer owner before arm publication. Normal yields and unfinished
delegation retain live activation values. Surrounding catch and yielding
finally regions retain whole abrupt/result identities. This batch adds no
backend representation or second dispatcher.

Optional Property/Call chains retain their distinct checked source and scalar
step owner. Their base remains eager and each guarded operand retains its
existing one-plain-yield restriction. The old `GeneratorValueBranchStates`,
`CompletedGeneratorValueArm` and flat `GeneratorIf` transport remain consumed
only by that optional-chain producer; general conditional/logical values no
longer depend on its bounded arm grammar. Grouped terminal Property References
and completed Call Values retain their already implemented separate policies.

Complete value branches compose with the admitted ordinary classic loops and
statement branches. Iterator body owners, suspended optional bases, mixed
Await/Yield and async-generator region composition keep their explicit current
source boundaries. General patterns, object evaluation and iterator/resource
regions require their own coherent source successors. No unsupported valid
source is counted as passing conformance.

The retained IR controls compare allocated points with actual complete arm
ranges and optional scalar states, verify distinct result/received/retained
activation cells, and follow later yields after joins. Existing source literals
that now have complete owners move from refusal checks to positive admission
checks. The new finite Engine fixture exercises yielding selectors, two yields
per chosen arm, skipped scheduling and original object identity, nested delegated
yields, GC collection, a catch containing a fresh complete branch, and whole
Return through a yielding finally. Existing three Engine fixtures and test names
remain. These controls are authored and unexecuted; combined type, Wasm/runtime
and pinned acceptance remain mandatory.
