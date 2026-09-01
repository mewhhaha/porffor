# Destructuring iterator locals ownership

`DestructuringIteratorLocals` is the private, capability-free owner of the 18
temporary locals reserved for one array destructuring iterator walk. It cannot
be cloned or copied into a second release authority.

The shared synchronous iterator projection borrows the bundle and copies only
the eleven numeric local identifiers needed by GetIterator. Each pattern
element and iterator step also borrows the same bundle. The enclosing compiler
retains the sole owner through normal completion, abrupt IteratorClose and the
final reverse-order release of all 18 locals.

This source-equivalent ownership change preserves reservation order, evaluation
order, IteratorClose behavior and emitted instructions. Its four-test recursive
guard pins the capability-free 18-field bundle, borrowed eleven-field protocol
projection, sole construction, borrowed element/step pipeline and one release
walk.

At the shared Batch AF checkpoint, `cargo xc` passed. The four-test recursive
guard and the neighboring `Math.sumPrecise` and synchronous-protocol structure
targets passed `14/14`; the exact array-destructuring iterator and abrupt-close
CLI witnesses passed `2/2`. No Test262 cohort or semantic golden was run because
this source-equivalent ownership invariant claims no new destructuring,
iterator or conformance behavior.
