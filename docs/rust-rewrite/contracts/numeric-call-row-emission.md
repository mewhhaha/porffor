# Compact numeric call emission

Long runs of ordinary source calls with the same live binding read and inert
numeric arguments share one Wasm call body. The program's private static image
holds each row's argument count and exact IEEE-754 bits. The emitted source body
reads that data with ordinary Wasm loads and builds a fresh, rooted `ValueArray`
for the existing `ProxyCall` operation. There are no JavaScript instructions,
operation tags, source evaluator, additional function activation or new ABI
types in the image.

Admission requires at least 64 adjacent `ExpressionStatement` calls in an
ordinary source body. Every callee is the same existing `Identifier`,
`GlobalIdentifierRead`, `GlobalPropertyRead`, or environment-backed bare-call
Reference operation. The emitter performs that operation on every iteration,
before argument construction. Environment-backed calls retain their name,
strictness and resolution start; the existing ResolveBinding/GetValue helpers
capture the callee and the held Reference's receiver anew. This includes
ordinary eval-visible activations without changing their frame. A previous
call may replace the function, mutate captured locals or install a getter; the
next row observes the current binding. Getter and call throws take the existing
handler/finalizer branches and stop subsequent rows.

Only primitive `Number` IR arguments are admitted. Their representation is
copied bit-for-bit without coercion. Empty and varying argument lists remain
fresh private GC lists. Receiver expressions, direct eval, static constructor
annotations, spread, effectful arguments, declaration statements and suspension
do not join a run and retain their existing paths. This is an emission size
choice, not a call-count or execution limit; short runs remain ordinary code.

The loop stays inside the original source function. Parameters, local storage,
lexical environments, hoisted declarations, source Realm, legacy caller and
`Function.prototype.toString` identity retain their existing owners. Each row
publishes the ordinary expression statement's whole completion and final value.
GC references use the existing typed roots, and completed rows clear temporary
roots before the next row. Runtime-operation/builtin and resumable bodies retain
their existing code. Runtime GC types, signatures, imports and native ABI are
unchanged.

Library controls emit general 4,096-call source bodies, validate the actual R/P
Wasm, and bound source-body and whole-program growth. Each Engine control
contains 1,024 eligible numeric calls (or two 512-call runs), checks the actual
validated P body size below 128 KiB, and executes that same compiled artifact
without a second compilation. This whole-program precondition permits fixed
construction, spread and result-check code while staying at one eighth of the
existing 1 MiB size-optimized native selection threshold. The separate AOT
controls keep their tighter 32 KiB source-body and bounded growth assertions.
Positive/empty/varying-arity rows exercise argument freshness and
GC; unary `-0` is a deliberately excluded ordinary-path boundary. Engine controls exercise
live callee replacement, hoisting and captured locals; throwing global and
with-object getters with the original receiver;
catch/finally and GC around call throws; argument bits, freshness, retained GC
roots and source caller identity; Proxy/bound dispatch across an effectful
argument boundary; and entry/prepared Realm Script completion.

The motivating captured program contained a 1,711,700-byte `scriptw::main` body.
Its separately captured cold runtime module phase took 123.729535488 seconds;
JavaScript execution took 45.496840831 seconds under the original 60,000 ms
deadline. Those are observations from the unmodified baseline, not a measured
speedup or a guarantee about this change. Compare a newly built, source-bound
candidate using the unchanged pinned body and actual public materialization.
Keep Wasmtime module construction, JavaScript execution, cache admission and
process/cgroup observations separate. No deadline, cache cap or ECMAScript
completion criterion is changed by this emission rule.

Revision 2's private diagnostic retained six passing controls and two body-size
REDs before either failed source executed. Inspection showed an already compact
67,317-byte Proxy main body, where the former whole-program 64 KiB precondition
rejected fixed code unrelated to call-run growth. The other source had an
820,896-byte ordinary `drive` body containing 1,024 environment-backed call
References, a typed form missing from revision 2 admission. Revision 3 records
the deliberate whole-program test precondition change separately from the
environment-call production extension. Keep those old RED transcripts and
their original disposition; only a fresh source-bound run can verify revision 3.
