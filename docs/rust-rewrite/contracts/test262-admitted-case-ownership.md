# Test262 admitted-case ownership

Status: implemented with focused structural verification, 2026-08-27.

## Scope

This contract owns the in-process Test262 runner transition from a planned
case to one journal-authorized execution. It does not own snapshot publication,
strike persistence, result classification, worker process isolation, or the
differential replay protocol.

## Rust invariant

The transition is a non-cloneable
`RunPhase -> QueuedCase -> CaseAdmission -> AdmittedCase` chain:

1. `RunPhase::into_queue(self)` consumes a planned phase and privately creates
   each `QueuedCase`.
2. `AttemptJournal::admit` consumes that queue authority after recording the
   durable in-flight entry. Its exhaustive result is either
   `CaseAdmission::Run(AdmittedCase)` or `CaseAdmission::Quarantined`.
3. The worker exhaustively consumes `CaseAdmission`. The run arm transfers its
   `AdmittedCase` into `run_case_entry`; the quarantine arm creates a crash
   result without a runnable proof.
4. `run_case_entry` consumes the `AdmittedCase`. The proof's private field and
   test-only bypass constructor prevent production code from manufacturing a
   second proof from a retained `TestCase`.

All four authority types retain only `Debug`. They have no clone, copy,
equality or default capability. The worker copies the admitted path before the
proof transfer solely for a possible retirement diagnostic; a path string is
not accepted by the runner and cannot authorize an execution.

This turns “one durable admission authorizes one runner entry” into ownership
rather than convention. Retiring the journal slot remains after the infallible
`TestResult` return, preserving the existing process-death accounting order.

## Verification and non-claims

The Rust-lexical structure guard ignores comments and normal, raw, byte,
C-string and character literals plus raw identifiers. It pins all four exact
declarations, their recursive source census, each consuming transition, the
sole product worker match, the diagnostic-path-before-transfer order and all
four runner entry references. The focused structure target passes `4/4`.

This invariant does not claim process-death recovery, panic isolation, timeout
correctness, complete Test262 accounting, differential equivalence, or broader
T25 completion. It changes no Test262 case selection, journal bytes, execution
backend, result, snapshot, or published conformance count.
