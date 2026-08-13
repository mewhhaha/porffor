# Main completion and the host job checkpoint

## The observable boundary

Running a Script and performing the host's Promise-job checkpoint are two
ordered operations. Abrupt Script completion does not erase jobs that were
already enqueued: the host still drains them before it reports the Script's
completion. The completion remains the primary result of the run. A rejection
created by a drained job may be recorded as unhandled, but it must not replace
an already-pending top-level throw.

The Wasm-AOT backend already implements the second half of this rule:
`emit_drain_promise_jobs` saves and restores the complete result tuple, and
`emit_report_unhandled_rejection` promotes a rejection only while the restored
completion is Normal. The missing first half was control flow. An abrupt
operation in the main body called `emit_return_current_completion`, which
returned from the Wasm export before either helper could run.

## Closed completion-exit state

One `CompletionExit` value owns the exit policy of an emitted body:

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
4. inspect the unhandled-rejection list;
5. promote the oldest rejection only when the Script completion is Normal;
6. publish the resulting main-export tuple.

Thus a queued job can run and reject after `throw primary`, while the public
completion still carries `primary`. This contract does not claim that every
unhandled rejection is reported, that the rejection tracker is realm-owned, or
that module/finalization jobs share this queue; those remain explicit T14/T06
work.

## Durable consumer contract

The engine regression queues a Promise reaction that prints and throws a
secondary `RangeError`, then throws a primary top-level `TypeError`. It requires
both the printed job side effect and the primary TypeError/message. Either a
premature Wasm return or an unhandled-rejection overwrite fails the same focused
contract.
