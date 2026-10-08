# Checked mixed async-generator classic-loop IR

`AsyncGeneratorLoopIr` and `AsyncGeneratorIfIr` are the complete mixed Await/Yield
owners. Private constructors consume the actual shared source allocator's ranges
and ordered suspension tape. Every initialization, test, body, update and selected
conditional branch has a distinct inclusive range, including eager phases. While
enters Test, DoWhile enters Body and For enters Initialize; Continue selects
Update for For and Test for the other two kinds.

The mixed region constructor uses the same physical statement-state validator as
the ordinary generator owner, with a closed protocol selector. Ordinary generator,
plain async and foreign iterator/resource carriers cannot inhabit mixed regions.
The same physical suspension census retains its original Yield-only sink and a
mixed sink. Nested opaque carriers publish only their validated private tapes.
Lost or reordered Await/Yield operations, overlapping phase ranges, missing or
aliased completion storage and foreign head branches reject before publication.

The original For lexical head validator is one shared authority. It verifies
unique names/slots and the exact mutable per-iteration slots derived from the
actual lowered head declarations. Captured head cells retain their original
environment; storage-only head cells retain their original activation inventory.
Loop completion V is one checked activation cell, retained through body entry,
local Continue and outward completion. Empty statement completion wraps complete
source items, including suspended declaration prefixes.

Each source suspension records a closed resume-environment owner. Actual checked
Loop, If, expression-branch, explicit Block and Try regions reenter from the
original invocation environment so their structured scopes can be rebuilt.
Foreign iterator and resource suspensions retain their original Saved owner.
The opaque function-source certificate separately records their actual enclosing
Block/Try ancestry, including the shared disposal finalizer's implicit Await
resume. Native entry reconstructs these enclosing scopes before the original
foreign continuation enters. A captured for-await iteration finds its original
child record in the saved ancestry; body resume retains the deepest saved edge
until nested blocks reattach. Fresh iterations publish their new record once.

Native environment helpers admit async generators through a token minted from
an actual checked mixed carrier or this opaque source certificate. The exact
entry resume state selects the environment; no function-wide choice is made.
Every emission path restores the prior token, including errors. The retained
scalar branch producer inserts its exit through one checked atomic source-plan
relocation, moving the remaining tape and both certified state lists together.

The source FunctionBody/ForeignIteratorBody domain is separate from execution
kind. Full source planning and lowering consume that same closed domain. Foreign
iterator/resource bodies retain their existing continuation algorithms, and new
unsupported compositions remain explicit refusals. Nested callable bodies are
excluded from the source graph. Yield promise adoption, delegation, implicit
Return Await and async-generator request ordering retain their actual existing
authorities; no extra scalar Await or second runtime representation is invented.

The compiler metadata, early-error/throw analysis, builtin dependencies, storage
and environment collectors traverse all actual regions and the completed
condition value. Meaningful constructor controls corrupt genuine AST-produced
carriers; semantic and emitted-artifact cohorts cover mixed phases, branches,
captured cells, References, queued requests and finalizers. Compilation and
execution remain deferred until the complete coherent source batch is written.
