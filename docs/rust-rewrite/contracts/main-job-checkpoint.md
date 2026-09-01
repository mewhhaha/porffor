# Main completion and the host job checkpoint

## The observable boundary

Running a Script and performing the host's Promise-job checkpoint are two
ordered operations. Abrupt Script completion does not erase jobs that were
already enqueued: the host still drains them before it reports the Script's
completion. The completion remains the primary result of the run. A rejection
created by a drained job may be recorded as unhandled, but it must not replace
an already-pending top-level throw.

The Wasm-AOT backend implements both halves of this rule:
`emit_drain_promise_jobs` saves and restores the complete result tuple, while
`emit_report_unhandled_rejection` reports the complete detached checkpoint
snapshot of the still-unhandled FIFO and promotes its oldest rejection only
while the restored completion is Normal. The control-flow half prevents an
abrupt operation in the main body from returning before either helper can run.

## Closed completion-exit state

`lila-aot-wasm/src/emit/completion_exit.rs` is the sole owner of the closed
state machine. `emit.rs` re-exports only its crate-visible `CompletionExit`
wrapper, whose value owns the exit policy of an emitted body:

- `MainExport` writes the public result globals and returns from `main`;
- `MainJobCheckpoint(target)` unwinds environments and branches to the tracked
  checkpoint block;
- `MultiValue` returns an internal function's four-word completion tuple.

The type is the stored authority from which `ReturnAbi` is derived. Main-body
compilation transitions `MainExport -> MainJobCheckpoint(target) -> MainExport`
through checked methods. A `MultiValue` body cannot enter the checkpoint state,
and a second or mismatched transition panics during emission. The target is a
`ControlTarget` minted by the code sink, so no raw branch depth can be counted or
forwarded by hand.

The constructor and checked enter/leave transitions are visible only to the
parent `emit` module. The ABI and active-checkpoint projections retain their
crate visibility for the parent and the single completion-return consumer.

Neither the wrapper nor its private state derives cloning, copying, formatting,
equality or default capabilities. Each of the ABI projection, active-target
projection, checkpoint-entry check and checkpoint-exit check borrows the one
stored state and matches all three variants explicitly. Adding another state
therefore requires all four decisions to be updated before the backend builds.
Only the active `ControlTarget` is copied out for the unchanged branch API; the
completion-exit authority itself cannot be duplicated or compared outside its
checked transitions.

The checkpoint block begins immediately before source-body emission and ends
immediately before job draining. Realm/global/bootstrap initialization remains
outside it: those steps cannot have run user source or enqueued user jobs, and a
failure there remains a direct main-export failure. Every body emitter shares
the one `emit_return_current_completion` exit, so a newly introduced abrupt
operation cannot silently bypass the checkpoint.

## Completion precedence

The checkpoint performs these steps in order:

1. preserve the Script result, tag, completion kind, auxiliary word and thrown
   error-name/message diagnostic globals;
2. drain all currently reachable Promise jobs, including jobs they enqueue;
3. restore the Script completion and checkpoint realm;
4. capture the unhandled-rejection head and tail, detach both tracker globals,
   and sever the captured tail's intrusive link before any diagnostic
   conversion can run;
5. inspect every candidate in that finite FIFO snapshot, re-reading its strict
   Promise state and `[[IsHandled]]` mark after the drain;
6. when the Script completion is Normal, reserve the oldest still-unhandled
   rejection for the exported Throw and print every later rejection value in
   FIFO order;
7. when an abrupt Script completion is already primary, print every
   still-unhandled rejection value in FIFO order;
8. consume only the detached snapshot, promote the reserved oldest rejection
   only when the Script completion is Normal, and publish the resulting
   main-export tuple.

Thus a queued job can run and reject after `throw primary`, while the public
completion still carries `primary` and every rejection captured after the job
drain remains visible through the line-oriented host output ABI. Symbol values
use the existing non-coercing `SymbolDescriptiveString` path. Other values use
the same `ToString` operation as host `print`; when user conversion throws, the
reporter prints the fixed `unhandled rejection diagnostic ToString threw`
marker, restores the Script/oldest-rejection completion and its name/message
globals, and continues with the next FIFO entry. Thus conversion failure is
visible but cannot silently replace the primary failure. A `print_line_utf8`
host failure is not caught, so the checkpoint cannot report success after its
output failed. The print import is present for every heap-backed module even
when source never names `print`.

`ToString` may itself call `Promise.reject`. Those reentrant rejections append
to the newly empty live tracker: they cannot mutate the severed snapshot tail,
extend the current walk or make a recursively rejecting conversion loop
forever. The reporter never clears that fresh FIFO. It is outside the current
checkpoint snapshot and remains available to later host policy. This lane does
not schedule a second main-export checkpoint for rejections created while the
first checkpoint is formatting diagnostics.

This contract does not claim that the rejection tracker is realm-owned or that
module/finalization jobs share this queue; those remain explicit T14/T06 work.

## Product owner census

- `heap.rs` declares the intrusive Promise-record link, and `module.rs`
  declares the tracker head/tail globals and line-print import index.
- Promise allocation clears the link; `emit_track_unhandled_rejection` is the
  sole FIFO append owner.
- `emit_report_unhandled_rejection` is the sole traversal, diagnostic and
  snapshot-detachment/consumption owner.
- the main-body compiler calls that reporter once after the job drain;
  `emit_script_with_forced_builtins` is the sole import-authority decision that
  makes the host printer reachable from that product call.

No CLI or engine wrapper reconstructs the queue. They observe only the host
lines and the final main-export completion.

## Durable consumer contract

The engine regression queues a Promise reaction that prints and throws a
secondary `RangeError`, then throws a primary top-level `TypeError`. It requires
the printed job side effect, the `RangeError: secondary` rejection diagnostic,
and the primary TypeError/message. Either a premature Wasm return, an omitted
rejection report or an unhandled-rejection overwrite fails the same focused
contract.

The CLI fixtures
`crates/lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections.js` and
`crates/lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections_with_primary_throw.js`
name no host-print global. Together they require handled candidates to remain
silent, all unhandled values to be reported once in FIFO order, the oldest
rejection to supply the Throw when the Script completes normally, and an
existing top-level `TypeError` to remain primary while all rejection values are
printed. They include Symbol, throwing-`toString` and recursively rejecting
`toString` values, pinning the descriptive Symbol output, fixed conversion-
failure marker, finite detached-snapshot policy, continued FIFO walk and
primary-completion restoration. The bounded structure target additionally
rejects the retired first-match loop exit, clearing the fresh reentrant FIFO,
removing the product checkpoint call or its CLI test registrations, and any
restoration of source-reference-only print-import authority.

`crates/lila-aot-wasm/tests/completion_exit_structure.rs` separately fixes the
private file owner, narrow re-export, exact state/method visibility inventory,
closed caller census, no-capability declarations, four exhaustive borrowed
decisions and the checked block-entry/exit and abrupt-return order.

## Verification

The coordinated checkpoint ran `cargo xc` and the focused structure, engine
and CLI targets. The bounded unhandled-rejection structure suite passes `5/5`,
`wasm_backend_drains_promise_jobs_after_top_level_throw_without_replacing_it`
passes `1/1`, and the two `functions::run_wasm_backend_reports_` CLI regressions
pass `2/2`. The shared Wasm-AOT fake suites also pass `187/187` and `191/191`,
with every non-success bucket at zero. These checks do not establish realm-
owned rejection tracking or full Promise/Test262 closure.

The no-capability closure rerun passes the strengthened completion-exit
structure target `3/3`, the exact engine checkpoint witness `1/1` and both
exact rejection-order CLI witnesses `1/1` each. No Wasm golden or broad suite
was rerun for that source-equivalent Rust authority change. Independent dry
review is clean, and the following shared workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, the recursive module-boundary check,
the task-plan check and `git diff --check`.
