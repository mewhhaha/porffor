# Standard generator and Array iterator implementation owners

The 2026-10-06 T02 source batch gives the consumed helpers formerly above
`compile_standard_builtin` three private owners. The standard builtin dispatcher
remains the same flat exhaustive match, with the same builtin IDs, typed kinds,
entry method names and complete dispatch body.

`builtins/standard/generator.rs` owns synchronous generator method admission and
its private resume-call body. Only the method entry is visible to the standard
parent. The resume helper remains private to this family and has its original
single caller. Receiver and argument evaluation, incompatible/executing receiver
errors, suspended-start abrupt entry, saved-body resumption, delegated result
identity, defining/caller Realm allocation and whole normal/return/throw
completions retain their original order. All activation, value, completion and
Realm roots retain their original reservation and cleanup operations.

`builtins/standard/async_generator.rs` owns the async request method body. Its
parent-only entry preserves intrinsic Promise capability allocation before
receiver validation, the request's whole value and capability roots, head/tail
queue publication, execution/body state transitions, yield-return reactions,
saved-body entry, draining and complete settlement behavior. No request state
or Promise implementation is duplicated.

`builtins/standard/array_iterator_creation.rs` owns both Array and TypedArray
iterator method creation and the closed receiver-policy enum. The enum and
compiler entry are visible only to the standard parent. Generic Array methods
retain ToObject and the converted receiver; TypedArray methods retain their
brand and view admission before iterator allocation. Iteration kind, abrupt
completion propagation, defining Realm behavior and GC-local lifetime remain
the original implementation. The six existing dispatch arms consume this one
owner directly.

The extraction changes source ownership and the narrow child-to-parent method
visibility only. It adds no public API, wrapper, alternate dispatch or new
behavioral control. Beforeimages and source receipts include the unchanged
inventory of 86 existing semantic controls in 17 relevant Engine/CLI sources.
The two generator families retain three dispatch consumers apiece; synchronous
resume retains one internal consumer; Array iterator creation retains six.

All four moved bodies and the complete dispatcher are compared with their
beforeimages as source text. The children and standard parent receive isolated
formatting after code and documentation are written. This does not establish
emitted-artifact equivalence or full T02 acceptance. Compilation, focused
semantic regressions, representative artifact comparison and the shared broad
checkpoint remain pending. No Cargo, tests, runtime, guards or data generators
run during this source batch.
