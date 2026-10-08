# Generated completion and suspension programs

`control-flow-v1` generates real Script source with 1–32 top-level statement
trees, each at depth 1–4. One private checked tree is the only renderer input.
Its closed nodes cover blocks, selected/skipped branches, finite classic loops,
Try/Catch/Finally, ordinary effects, captured lexical cells, Return/Throw and
current/outer labelled Break/Continue. Loop trip counts are 0–3 and the induction
binding is inaccessible to generated effects. A loop also captures its actual
per-iteration binding for observation after execution. No generated input runs
during generation.

The seed selects ordinary, Generator, Async or AsyncGenerator execution and
strictness. Await and Yield require that exact execution kind. Generator drivers
make actual Next requests, pass the fixed entry checkpoint and stop at a second
fixed Yield inside an active Try. Return/Throw injection occurs at that second
checkpoint, before generated Return/Throw leaves can skip it. One fixed yielding
Finally observes and resumes the pending request; later Next requests finish
the original protocol. These fixed harness transitions are separate from the
step/depth budgets on generated statement trees and remain in every reduction.
Promise completion uses the existing root job drain. Backend deadlines retain
responsibility for a compiler/runtime bug that fails to terminate; no driver
silently truncates execution.

The existing schema-v3 observation compares actual primitive completion and
ordered print events, including caught throws, finalizer effects, request results,
retained closure values and final state. The generator contains no interpreter or
expected-trace implementation. Shared failures and unsupported cases stay red.
A nominal matched pair missing both terminal driver transcripts is rejected;
generator pairs also require their fixed finalizer observation and resume.
The real pair remains in the attempt journal. No expected transcript is supplied
to either backend to repair missing work or make a match.

Every reduced tree passes the same constructor. Deleting a loop cannot publish
an orphaned control target; deleting a wrapper cannot introduce Await/Yield into
another protocol. Reductions preserve parse goal, strictness, execution kind and
the original request schedule. Candidates strictly decrease statement count,
node count or finite effect magnitude. The mismatch witness preserves completion
and print difference dimensions, both completion kinds, or the actual one-backend
failure phase together with the other backend's completion kind. Reductions
must fit the original step/depth budgets. A worker failure stops reduction and
cancellation publishes no reduced case. Every changed source retains its
own actual fingerprint in the existing durable campaign journal.

SDK generation and `lila differential campaign --grammar control-flow-v1
--steps 8 --depth 4` use that owner; other generation-budget flags reject. Existing
seed count, worker, timeout, replay and resource limits are unchanged. The eighth
grammar joins the serial PR/nightly campaign tiers. Constructor damage,
actual parser/reducer, mismatch and worker-failure controls, plus a real four-kind
selected-worker campaign and real Return/Throw finalizer controls, are authored
and unrun. Broader AST/builtin/negative
generation and sustained campaign acceptance remain open.
