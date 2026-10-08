# SuperCall through a lexical async arrow

A plain async arrow lexically inside a derived constructor can await in the
argument list of `super()`. It retains the constructor's original activation
cells for `this`, its initialization status, `new.target`, and the active
constructor. Ordinary async functions and methods still fail the frontend
SuperCall early error.

Preparation evaluates the original derived activation's `new.target`, then
GetSuperConstructor, before ArgumentListEvaluation. Two distinct checked owned
bindings retain those values. Each argument uses the existing resumable
argument-list owner; a spread drains into the same private GC argument list
before later awaits. The opaque PreparedSuperConstruct consumes those captured
reads and arguments. A prototype change during await cannot replace the saved
super constructor. The async arrow's own invocation `this` or `new.target`
cannot substitute the original constructor activation.

The native path calls the original prepared construction helper. Construct
precedes BindThisValue, including for a duplicate super call, and the original
active constructor initializes its instance fields only after successful
binding. Argument rejection or construction failure preserves the original
uninitialized status. A constructor may return an explicit object while its
arrow is pending; that return does not replace the retained activation cells.

The constructor, source, and emitted-Wasm controls are authored and unrun. They
cover preparation order, alias refusals, earlier spread evaluation, original
new.target, GC across await, `this` TDZ, duplicate construction, rejected await,
base failure, and selected argument branches. This source change does not claim
runtime or Test262 verification.
