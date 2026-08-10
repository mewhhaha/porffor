//! Durable pre-attempt state for `report-all --resume`, so that a process
//! death can be attributed to the case that caused it.
//!
//! # Why this module exists
//!
//! Nothing durable was written *before* a case was attempted. The first
//! durable write on the resume path is [`crate::write_resume_case_checkpoint`],
//! which fires only after a case **completes**, and only every
//! [`crate::RESUME_CASE_CHECKPOINT_INTERVAL`]th completion. `execute_cases`
//! reconstructs the completed set from the previous snapshot's failures plus
//! its `completed_paths` and runs `cases - completed`, so a case whose
//! *process* died enters neither list and is re-selected on every resume,
//! forever.
//!
//! That is not a hypothetical. `target/test262-scratch/baseline-sweep.log`
//! records supervisor attempts 4 through 40 as 37 consecutive `exit 134`s
//! (`fatal runtime error: stack overflow`), with **zero** cases completed and
//! **zero** diagnostics naming any case; identifying the poison case afterwards
//! required diffing a node snapshot against the matrix cache by hand. The
//! whole 40-retry budget bought nothing.
//!
//! The in-process `panic::catch_unwind` in [`crate::run_case_entry`] cannot
//! help with this class of death. A Rust stack overflow hits the guard page and
//! `abort()`s; an OOM kill is `SIGKILL`. Neither unwinds, neither runs `Drop`,
//! neither is catchable. **Durable pre-attempt state is the only possible
//! attribution mechanism**, which is what this module is.
//!
//! # The shape
//!
//! * A case is written into the journal *before* it is attempted and retired
//!   after the attempt returns, on every path.
//! * Every entry still present at startup was in flight when the previous
//!   process died, and is charged one strike.
//! * At [`CrashStrikeLimit`] strikes the case is not run at all: it is recorded
//!   as an ordinary `TestStatus::Failed` result carrying
//!   [`crate::OutcomeKind::Crash`], so it lands in `completed_paths`, in
//!   `failures`, and in the outcome counts. It is never a silent skip
//!   (AGENTS.md, "Correctness Rules").
//!
//! # Why the types, and not a `bool` plus a `u32`
//!
//! Shaped after the `FunctionBodySize`/`FunctionBodyBudget` precedent in
//! `crates/porffor-aot-wasm/src/emitted_function.rs`: validate once in a
//! constructor and hand back the only type the rest of the code accepts.
//!
//! * [`CaseStrikes`] wraps a `NonZeroU32`, so "has this case any strikes?" is
//!   `Option::is_some` rather than a separate boolean somebody can forget to
//!   check against a `0` sentinel.
//! * [`QueuedCase`] holds its `TestCase` privately, and [`CaseAdmission`] is
//!   the only value [`AttemptJournal::admit`] returns. A worker therefore
//!   *cannot* obtain a runnable `TestCase` from a queue pop without going
//!   through the admission decision that journals it. "Forgot to check the
//!   quarantine" is a compile error, not a silent re-attempt loop.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::TestCase;

/// On-disk format version. A journal written by a different version is
/// discarded with a message rather than misread; losing attribution for one
/// death is recoverable, misreading a strike count is not.
const ATTEMPT_JOURNAL_VERSION: u32 = 1;

/// `NonZeroU32::new(..).unwrap()` is not usable in a `const` on every toolchain
/// this tree builds with; this is the const-stable spelling.
const fn nonzero(value: u32) -> NonZeroU32 {
    match NonZeroU32::new(value) {
        Some(value) => value,
        None => panic!("attempt-journal strike constants are non-zero by construction"),
    }
}

/// How many process deaths a case has been charged with.
///
/// Non-zero by construction: a case with no strikes has no entry, so there is
/// no `0` that some call site can read as "has strikes" or "is quarantined".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CaseStrikes(NonZeroU32);

impl CaseStrikes {
    /// The count a case reaches the first time a process dies on it.
    pub(crate) const FIRST: Self = Self(nonzero(1));

    /// The only way to read a persisted count back into the type. `None` means
    /// "no strikes", which is exactly what an absent journal entry means.
    pub(crate) fn from_count(count: u32) -> Option<Self> {
        NonZeroU32::new(count).map(Self)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0.get()
    }

    /// The only way to raise a count. Saturating rather than wrapping: a
    /// wrapped count would drop a quarantined case back to zero strikes and
    /// re-arm the exact loop this module exists to break.
    #[must_use]
    pub(crate) const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl core::fmt::Display for CaseStrikes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// The number of process deaths a case is allowed before it is quarantined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CrashStrikeLimit(NonZeroU32);

impl CrashStrikeLimit {
    /// Two, and the number is read off the sweep log rather than chosen by
    /// taste. Attempt 1 died at 03:32:05 and attempt 2 at 03:33:41 — 96 s later,
    /// on the same pending set. That is the signature of a deterministic case.
    /// A limit of 1 would instead blame a case for a container restart or an
    /// unrelated OOM kill, both of which also leave journal entries behind and
    /// neither of which is the case's fault.
    pub(crate) const DEFAULT: Self = Self(nonzero(2));

    pub(crate) const fn get(self) -> u32 {
        self.0.get()
    }

    pub(crate) const fn is_reached_by(self, strikes: CaseStrikes) -> bool {
        strikes.get() >= self.0.get()
    }
}

impl core::fmt::Display for CrashStrikeLimit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// A case that has been selected off the queue but not yet journalled.
///
/// The inner `TestCase` is private, so the only way to get a runnable
/// `TestCase` out of one is [`AttemptJournal::admit`] — which writes the
/// journal entry first and may instead answer
/// [`CaseAdmission::Quarantined`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueuedCase(TestCase);

/// The only value [`AttemptJournal::admit`] returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CaseAdmission {
    /// Journalled and cleared to run.
    Run(TestCase),
    /// At the strike limit. Not run; recorded as a crash result by the caller.
    Quarantined { path: String, strikes: CaseStrikes },
}

/// A case that was in flight when a previous process died, with the strike
/// count it has just reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StruckCase {
    pub(crate) path: String,
    pub(crate) strikes: CaseStrikes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct InFlightAttempt {
    worker_slot: usize,
    case_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AttemptJournalFile {
    #[serde(default)]
    version: u32,
    /// Cases whose attempt started and has not been retired. On a clean exit
    /// this is empty; a non-empty list at startup *is* the record of a process
    /// death, and is the only thing that grants a strike.
    #[serde(default)]
    in_flight: Vec<InFlightAttempt>,
    /// Accumulated strikes per case path.
    #[serde(default)]
    strikes: BTreeMap<String, u32>,
}

impl AttemptJournalFile {
    fn fresh() -> Self {
        Self {
            version: ATTEMPT_JOURNAL_VERSION,
            in_flight: Vec::new(),
            strikes: BTreeMap::new(),
        }
    }
}

/// The journal for one `execute_cases` run.
///
/// One mutex guards the whole state, and every mutation persists the file
/// before returning. One small `fs::write` per case is bounded against the
/// measured ~1.0 s/case budget of a sweep.
#[derive(Debug)]
pub(crate) struct AttemptJournal {
    path: PathBuf,
    limit: CrashStrikeLimit,
    state: Mutex<AttemptJournalFile>,
}

impl AttemptJournal {
    /// Opens the journal for a run.
    ///
    /// A resume run reads whatever the previous process left; a fresh run
    /// starts from an empty journal, because a fresh run is a fresh accounting
    /// and must not inherit a stale node's strikes.
    pub(crate) fn open(
        path: PathBuf,
        resume: bool,
        limit: CrashStrikeLimit,
    ) -> Result<Self, String> {
        let state = if resume {
            read_journal(&path)
        } else {
            AttemptJournalFile::fresh()
        };
        Ok(Self {
            path,
            limit,
            state: Mutex::new(state),
        })
    }

    /// Charges one strike to every entry left in flight by a dead process,
    /// clears the in-flight list, and persists.
    ///
    /// Called exactly once, at run start. Returns the suspects in journal
    /// order. One survivor names the poison case uniquely; more than one (the
    /// sweep runs `--threads 2`, so up to two can be in flight) is narrowed by
    /// [`plan_run_phases`].
    pub(crate) fn charge_strikes_for_survivors(&self) -> Result<Vec<StruckCase>, String> {
        let mut struck = Vec::new();
        {
            let mut state = self.lock()?;
            let survivors = std::mem::take(&mut state.in_flight);
            let mut already_charged = BTreeSet::new();
            for attempt in survivors {
                if !already_charged.insert(attempt.case_path.clone()) {
                    // One death charges a case at most once even if two worker
                    // slots somehow recorded the same path.
                    continue;
                }
                let previous = state
                    .strikes
                    .get(&attempt.case_path)
                    .copied()
                    .and_then(CaseStrikes::from_count);
                let strikes = match previous {
                    Some(previous) => previous.next(),
                    None => CaseStrikes::FIRST,
                };
                state
                    .strikes
                    .insert(attempt.case_path.clone(), strikes.get());
                struck.push(StruckCase {
                    path: attempt.case_path,
                    strikes,
                });
            }
        }
        self.persist()?;
        Ok(struck)
    }

    /// The admission decision, and the only way to turn a queue pop into a
    /// runnable `TestCase`.
    ///
    /// On the `Run` path the in-flight entry is written **before** the case is
    /// handed back, so the durable record always precedes the attempt.
    pub(crate) fn admit(
        &self,
        worker_slot: usize,
        queued: QueuedCase,
    ) -> Result<CaseAdmission, String> {
        let QueuedCase(case) = queued;
        {
            let mut state = self.lock()?;
            let strikes = state
                .strikes
                .get(&case.path)
                .copied()
                .and_then(CaseStrikes::from_count);
            if let Some(strikes) = strikes {
                if self.limit.is_reached_by(strikes) {
                    return Ok(CaseAdmission::Quarantined {
                        path: case.path,
                        strikes,
                    });
                }
            }
            state
                .in_flight
                .retain(|attempt| attempt.worker_slot != worker_slot);
            state.in_flight.push(InFlightAttempt {
                worker_slot,
                case_path: case.path.clone(),
            });
        }
        self.persist()?;
        Ok(CaseAdmission::Run(case))
    }

    /// Retires a worker slot's entry. Must be called after every attempt on
    /// every path — including the panic path, which `run_case_entry` already
    /// converts into an ordinary returned `TestResult`.
    pub(crate) fn retire(&self, worker_slot: usize) -> Result<(), String> {
        {
            let mut state = self.lock()?;
            state
                .in_flight
                .retain(|attempt| attempt.worker_slot != worker_slot);
        }
        self.persist()
    }

    pub(crate) fn limit(&self) -> CrashStrikeLimit {
        self.limit
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, AttemptJournalFile>, String> {
        self.state
            .lock()
            .map_err(|_| format!("attempt journal mutex poisoned for {}", self.path.display()))
    }

    fn persist(&self) -> Result<(), String> {
        let state = self.lock()?;
        write_journal(&self.path, &state)
    }

    /// Reads a persisted strike count back. Test-only: the product path never
    /// asks this question outside [`Self::admit`], which must own the decision.
    #[cfg(test)]
    pub(crate) fn strikes_for(&self, case_path: &str) -> Option<CaseStrikes> {
        self.state
            .lock()
            .expect("attempt journal mutex poisoned")
            .strikes
            .get(case_path)
            .copied()
            .and_then(CaseStrikes::from_count)
    }

    /// The paths currently recorded as in flight, in journal order.
    #[cfg(test)]
    pub(crate) fn in_flight_paths(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("attempt journal mutex poisoned")
            .in_flight
            .iter()
            .map(|attempt| attempt.case_path.clone())
            .collect()
    }
}

/// Leaves the journal in exactly the state an aborted process leaves: every
/// case admitted, none retired.
///
/// This deliberately drives the **product** `admit` path rather than
/// hand-writing a file, so a test that seeds a death cannot pass against a
/// format the running code would not produce.
#[cfg(test)]
pub(crate) fn simulate_process_death_with_cases_in_flight(
    path: PathBuf,
    cases: &[TestCase],
) -> Result<(), String> {
    let journal = AttemptJournal::open(path, true, CrashStrikeLimit::DEFAULT)?;
    for (worker_slot, case) in cases.iter().enumerate() {
        let admission = journal.admit(worker_slot, QueuedCase(case.clone()))?;
        assert!(
            matches!(admission, CaseAdmission::Run(_)),
            "seeding a death requires the case to still be admissible: {}",
            case.path
        );
    }
    Ok(())
}

/// One phase of a run: a case list and the worker count it runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunPhase {
    worker_count: usize,
    cases: Vec<TestCase>,
}

impl RunPhase {
    pub(crate) fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub(crate) fn into_queue(self) -> Vec<QueuedCase> {
        self.cases.into_iter().map(QueuedCase).collect()
    }

    #[cfg(test)]
    pub(crate) fn case_paths(&self) -> Vec<String> {
        self.cases.iter().map(|case| case.path.clone()).collect()
    }
}

/// Splits a run into phases so that a death with several cases in flight
/// converges on the culprit instead of spreading blame.
///
/// With two or more suspects the run executes **exactly the suspects, one at a
/// time, first**. A death during that phase can only be attributed to one case.
/// A 2-worker sweep therefore needs at most two further deaths to name the
/// case, against the 37 it actually burned.
///
/// With zero or one suspect there is nothing to narrow — one survivor already
/// names the case — so the run keeps its configured worker count and its
/// scheduled order, and pays nothing.
pub(crate) fn plan_run_phases(
    suspects: &[StruckCase],
    scheduled_cases: Vec<TestCase>,
    worker_count: usize,
) -> Vec<RunPhase> {
    let suspect_paths = suspects
        .iter()
        .map(|suspect| suspect.path.as_str())
        .collect::<BTreeSet<_>>();
    let suspects_present = scheduled_cases
        .iter()
        .filter(|case| suspect_paths.contains(case.path.as_str()))
        .count();
    if suspects_present < 2 {
        return vec![RunPhase {
            worker_count,
            cases: scheduled_cases,
        }];
    }

    let (suspect_cases, rest): (Vec<TestCase>, Vec<TestCase>) = scheduled_cases
        .into_iter()
        .partition(|case| suspect_paths.contains(case.path.as_str()));
    let mut phases = vec![RunPhase {
        worker_count: 1,
        cases: suspect_cases,
    }];
    if !rest.is_empty() {
        phases.push(RunPhase {
            worker_count,
            cases: rest,
        });
    }
    phases
}

fn read_journal(path: &Path) -> AttemptJournalFile {
    let Ok(raw) = fs::read_to_string(path) else {
        return AttemptJournalFile::fresh();
    };
    match serde_json::from_str::<AttemptJournalFile>(&raw) {
        Ok(file) if file.version == ATTEMPT_JOURNAL_VERSION => file,
        Ok(file) => {
            // Loud, never silent: the run continues (refusing to start would
            // let one stale file cost a whole sweep, which is the failure this
            // module exists to prevent), but the operator is told that this
            // run cannot attribute a previous death.
            eprintln!(
                "test262 attempt journal: {} has version {} (expected {}); starting a fresh journal, so a previous process death cannot be attributed",
                path.display(),
                file.version,
                ATTEMPT_JOURNAL_VERSION
            );
            AttemptJournalFile::fresh()
        }
        Err(err) => {
            eprintln!(
                "test262 attempt journal: {} could not be parsed ({err}); starting a fresh journal, so a previous process death cannot be attributed",
                path.display()
            );
            AttemptJournalFile::fresh()
        }
    }
}

/// Writes the journal atomically.
///
/// A plain `fs::write` can be interrupted by the very abort this module is
/// built to survive, leaving a truncated file — and a truncated journal is
/// indistinguishable from a corrupt one. Writing to a per-process temporary and
/// renaming makes the visible file always a complete journal, so the parse
/// failure above means genuine corruption rather than routine tearing.
fn write_journal(path: &Path, state: &AttemptJournalFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create attempt journal directory {}: {err}",
                parent.display()
            )
        })?;
    }
    let rendered = serde_json::to_string(state)
        .map_err(|err| format!("failed to render attempt journal {}: {err}", path.display()))?;
    let temporary = path.with_extension(format!("attempts-{}.tmp", std::process::id()));
    fs::write(&temporary, rendered).map_err(|err| {
        format!(
            "failed to write attempt journal {}: {err}",
            temporary.display()
        )
    })?;
    fs::rename(&temporary, path).map_err(|err| {
        format!(
            "failed to publish attempt journal {}: {err}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(path: &str) -> TestCase {
        TestCase {
            path: path.to_string(),
            source_path: PathBuf::from(path),
            original_source: "0;".to_string(),
            flags: BTreeSet::new(),
            features: BTreeSet::new(),
            includes: Vec::new(),
            negative: None,
            is_module: false,
        }
    }

    fn journal_path(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "porffor-attempt-journal-{label}-{}-{nanos}",
                std::process::id()
            ))
            .join("node-1234.attempts")
    }

    #[test]
    fn attempt_journal_strike_counts_are_non_zero_and_only_ever_rise() {
        assert_eq!(CaseStrikes::from_count(0), None);
        assert_eq!(CaseStrikes::from_count(1), Some(CaseStrikes::FIRST));
        assert_eq!(CaseStrikes::FIRST.next().get(), 2);
        assert_eq!(CaseStrikes::FIRST.next().next().get(), 3);
    }

    #[test]
    fn attempt_journal_default_limit_quarantines_on_the_second_death_not_the_first() {
        let limit = CrashStrikeLimit::DEFAULT;
        assert_eq!(limit.get(), 2);
        assert!(!limit.is_reached_by(CaseStrikes::FIRST));
        assert!(limit.is_reached_by(CaseStrikes::FIRST.next()));
        assert!(limit.is_reached_by(CaseStrikes::FIRST.next().next()));
    }

    #[test]
    fn attempt_journal_admission_records_before_it_hands_back_the_case() {
        let path = journal_path("admit-records-first");
        let journal = AttemptJournal::open(path.clone(), false, CrashStrikeLimit::DEFAULT)
            .expect("journal should open");
        let admission = journal
            .admit(0, QueuedCase(case("built-ins/Array/prototype/map/poison.js")))
            .expect("admission should work");
        assert!(matches!(admission, CaseAdmission::Run(_)));

        // Read the file back through the product reader: the entry must be on
        // disk already, not merely in memory.
        let reopened = AttemptJournal::open(path, true, CrashStrikeLimit::DEFAULT)
            .expect("journal should reopen");
        assert_eq!(
            reopened.in_flight_paths(),
            vec!["built-ins/Array/prototype/map/poison.js".to_string()]
        );
    }

    #[test]
    fn attempt_journal_retire_clears_the_slot_so_a_clean_exit_charges_nothing() {
        let path = journal_path("retire-clears");
        let journal = AttemptJournal::open(path.clone(), false, CrashStrikeLimit::DEFAULT)
            .expect("journal should open");
        journal
            .admit(3, QueuedCase(case("language/statements/try/clean.js")))
            .expect("admission should work");
        journal.retire(3).expect("retire should work");

        let reopened = AttemptJournal::open(path, true, CrashStrikeLimit::DEFAULT)
            .expect("journal should reopen");
        assert!(reopened.in_flight_paths().is_empty());
        assert!(reopened
            .charge_strikes_for_survivors()
            .expect("charging should work")
            .is_empty());
    }

    #[test]
    fn attempt_journal_charges_one_strike_per_death_and_quarantines_at_the_limit() {
        let path = journal_path("two-deaths");
        let poison = case("built-ins/Array/prototype/map/15.4.4.19-5-21.js");

        simulate_process_death_with_cases_in_flight(path.clone(), std::slice::from_ref(&poison))
            .expect("first death should seed");
        let first_resume = AttemptJournal::open(path.clone(), true, CrashStrikeLimit::DEFAULT)
            .expect("journal should reopen");
        let struck = first_resume
            .charge_strikes_for_survivors()
            .expect("charging should work");
        assert_eq!(struck.len(), 1);
        assert_eq!(struck[0].strikes, CaseStrikes::FIRST);
        assert!(matches!(
            first_resume
                .admit(0, QueuedCase(poison.clone()))
                .expect("admission should work"),
            CaseAdmission::Run(_)
        ));

        // The process dies again with the same case in flight.
        let second_resume = AttemptJournal::open(path, true, CrashStrikeLimit::DEFAULT)
            .expect("journal should reopen");
        let struck = second_resume
            .charge_strikes_for_survivors()
            .expect("charging should work");
        assert_eq!(struck.len(), 1);
        assert_eq!(struck[0].strikes.get(), 2);
        match second_resume
            .admit(0, QueuedCase(poison.clone()))
            .expect("admission should work")
        {
            CaseAdmission::Quarantined { path, strikes } => {
                assert_eq!(path, poison.path);
                assert_eq!(strikes.get(), 2);
            }
            CaseAdmission::Run(_) => panic!("a case at the strike limit must not be admitted"),
        }
    }

    #[test]
    fn attempt_journal_plans_one_phase_when_at_most_one_case_is_suspect() {
        let cases = vec![case("a.js"), case("b.js"), case("c.js")];
        let suspects = vec![StruckCase {
            path: "b.js".to_string(),
            strikes: CaseStrikes::FIRST,
        }];
        let phases = plan_run_phases(&suspects, cases.clone(), 4);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].worker_count(), 4);
        assert_eq!(
            phases[0].case_paths(),
            vec!["a.js".to_string(), "b.js".to_string(), "c.js".to_string()]
        );

        let phases = plan_run_phases(&[], cases, 4);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].worker_count(), 4);
    }

    #[test]
    fn attempt_journal_plan_ignores_a_suspect_that_is_no_longer_in_the_case_list() {
        // A suspect already recorded as completed is not re-run, so it must not
        // drag the whole run into single-worker mode.
        let cases = vec![case("a.js"), case("b.js")];
        let suspects = vec![
            StruckCase {
                path: "b.js".to_string(),
                strikes: CaseStrikes::FIRST,
            },
            StruckCase {
                path: "already-done.js".to_string(),
                strikes: CaseStrikes::FIRST,
            },
        ];
        let phases = plan_run_phases(&suspects, cases, 4);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].worker_count(), 4);
    }
}
