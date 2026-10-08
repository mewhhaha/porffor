//! A distinct crash/timeout reduction domain, never a semantic mismatch reducer.
#[cfg(any(test, feature = "spec-exec-oracle"))]
use super::super::generated_campaign::{commit_json, write_json, write_new};
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RobustnessReductionState {
    Running,
    NotReducible,
    CandidateSetExhausted,
    ReplayBudgetExhausted,
    Stopped,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CrashDisposition {
    Timeout {
        timeout_ms: u64,
    },
    Exit {
        code: Option<i32>,
        signal: Option<i32>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CrashSignature {
    stage: RobustnessStage,
    disposition: CrashDisposition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ByteComplexity {
    length: usize,
    byte_sum: u64,
}
#[cfg(any(test, feature = "spec-exec-oracle"))]
impl ByteComplexity {
    fn of(bytes: &[u8]) -> Self {
        // The input constructor bounds this at MAX_INPUT_BYTES * 255.
        Self {
            length: bytes.len(),
            byte_sum: bytes.iter().map(|byte| u64::from(*byte)).sum(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ReductionEdit {
    Delete { start: usize, end: usize },
    LowerByte { offset: usize, byte: u8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Pending,
    Retained,
    DifferentOutcome,
    Stopped,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StopReason {
    ForeignInput,
    ProvenanceMismatch,
    IncompleteJournal,
    OutsideCrashDomain,
    UnavailableTarget,
    ReplayError,
}
#[derive(Debug, Serialize)]
struct Attempt {
    ordinal: usize,
    baseline: bool,
    request: String,
    bytes: String,
    edit: Option<ReductionEdit>,
    complexity: ByteComplexity,
    observation: Option<String>,
    replay_error: Option<String>,
    decision: Decision,
}
#[derive(Debug, Serialize)]
pub struct RobustnessReductionReport {
    schema_version: u32,
    controller_identity: crate::CompilerProvenance,
    original: RobustnessInput,
    replay_limit: u16,
    candidate_replays: u16,
    retained_candidates: u16,
    state: RobustnessReductionState,
    stop_reason: Option<StopReason>,
    worker_identity: Option<crate::CompilerProvenance>,
    witness: Option<CrashSignature>,
    retained_input: Option<RobustnessInput>,
    replay_artifact: Option<String>,
    semantic_equivalence: SemanticEquivalence,
    attempts: Vec<Attempt>,
}
impl RobustnessReductionReport {
    pub fn state(&self) -> RobustnessReductionState {
        self.state
    }
    pub fn candidate_replays(&self) -> u16 {
        self.candidate_replays
    }
    pub fn retained_candidates(&self) -> u16 {
        self.retained_candidates
    }
    pub fn reproducing_input(&self) -> Option<&RobustnessInput> {
        self.retained_input.as_ref()
    }
}

/// Replay the baseline once, then spend the existing checked reduction budget
/// only on strictly smaller raw-byte candidates in the same target and policy.
/// A successful reduction retains a failure; it establishes no green corpus.
pub fn minimize_robustness(
    input: &RobustnessInput,
    limit: ArithmeticReductionLimit,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
    cancellation: &GeneratedCampaignCancellation,
    output: impl AsRef<Path>,
) -> Result<RobustnessReductionReport, DifferentialError> {
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = (input, limit, oracle, runner, cancellation, output);
        Err(DifferentialError::OracleNotLinked)
    }
    #[cfg(feature = "spec-exec-oracle")]
    {
        run_with(input, limit, cancellation, output.as_ref(), &mut |input| {
            replay_robustness(input, oracle, runner)
        })
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
fn signature(
    observation: &RobustnessObservation,
    requested: &RobustnessInput,
    expected_identity: Option<&crate::CompilerProvenance>,
) -> Result<Option<CrashSignature>, StopReason> {
    if &observation.input != requested {
        return Err(StopReason::ForeignInput);
    }
    let identity = observation
        .compiler_identity
        .as_ref()
        .ok_or(StopReason::ProvenanceMismatch)?;
    if expected_identity.is_some_and(|expected| expected != identity) {
        return Err(StopReason::ProvenanceMismatch);
    }
    if observation.journal_error.is_some() {
        return Err(StopReason::IncompleteJournal);
    }
    let expected = requested.target().stages();
    let stages = observation.stages.as_slice();
    if stages.is_empty() || stages.len() > expected.len() || stages != &expected[..stages.len()] {
        return Err(StopReason::IncompleteJournal);
    }
    match &observation.result {
        RobustnessResult::WorkerFailure {
            failure,
            cleanup_error,
        } => {
            if cleanup_error.is_some() {
                return Err(StopReason::OutsideCrashDomain);
            }
            let stage = observation
                .interrupted_stage
                .ok_or(StopReason::IncompleteJournal)?;
            if stages.last() != Some(&stage) {
                return Err(StopReason::IncompleteJournal);
            }
            let disposition = match failure {
                DifferentialWorkerFailure::Timeout { timeout_ms }
                    if *timeout_ms == requested.timeout_ms() =>
                {
                    CrashDisposition::Timeout {
                        timeout_ms: *timeout_ms,
                    }
                }
                DifferentialWorkerFailure::Timeout { .. } => {
                    return Err(StopReason::OutsideCrashDomain)
                }
                DifferentialWorkerFailure::Exit {
                    code: Some(code),
                    signal: None,
                } if *code != 0 => CrashDisposition::Exit {
                    code: Some(*code),
                    signal: None,
                },
                DifferentialWorkerFailure::Exit {
                    code: None,
                    signal: Some(signal),
                } if *signal > 0 => CrashDisposition::Exit {
                    code: None,
                    signal: Some(*signal),
                },
                DifferentialWorkerFailure::Exit { .. }
                | DifferentialWorkerFailure::ObservationLimit { .. }
                | DifferentialWorkerFailure::Protocol { .. }
                | DifferentialWorkerFailure::Process { .. }
                | DifferentialWorkerFailure::UnsupportedPlatform => {
                    return Err(StopReason::OutsideCrashDomain)
                }
            };
            Ok(Some(CrashSignature { stage, disposition }))
        }
        RobustnessResult::Accepted { .. }
        | RobustnessResult::Rejected { .. }
        | RobustnessResult::Executed { .. } => {
            if observation.interrupted_stage.is_some()
                || !observation
                    .result
                    .valid_terminal(requested.target(), stages)
            {
                return Err(StopReason::IncompleteJournal);
            }
            Ok(None)
        }
        RobustnessResult::Unsupported { .. }
        | RobustnessResult::Failure { .. }
        | RobustnessResult::InvalidWasm { .. } => Err(StopReason::UnavailableTarget),
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
enum Cursor {
    Delete { width: usize, start: usize },
    LowerByte { offset: usize, half: bool },
    Finished,
}
#[cfg(any(test, feature = "spec-exec-oracle"))]
impl Cursor {
    fn new(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            Self::Finished
        } else {
            Self::Delete {
                width: bytes.len(),
                start: 0,
            }
        }
    }
    fn next(&mut self, original: &[u8]) -> Option<(Vec<u8>, ReductionEdit)> {
        loop {
            match self {
                Self::Delete { width, start } => {
                    if *start >= original.len() {
                        if *width == 1 {
                            *self = Self::LowerByte {
                                offset: 0,
                                half: false,
                            };
                        } else {
                            *width = (*width / 2).max(1);
                            *start = 0;
                        }
                        continue;
                    }
                    let first = *start;
                    let end = (first + *width).min(original.len());
                    *start = end;
                    let mut bytes = original.to_vec();
                    bytes.drain(first..end);
                    return Some((bytes, ReductionEdit::Delete { start: first, end }));
                }
                Self::LowerByte { offset, half } => {
                    if *offset >= original.len() {
                        *self = Self::Finished;
                        continue;
                    }
                    let first = *offset;
                    let byte = if *half { original[first] / 2 } else { 0 };
                    if *half {
                        *offset += 1;
                        *half = false;
                    } else {
                        *half = true;
                    }
                    if original[first] == 0 || (byte == 0 && !*half) {
                        continue;
                    }
                    let mut bytes = original.to_vec();
                    bytes[first] = byte;
                    return Some((
                        bytes,
                        ReductionEdit::LowerByte {
                            offset: first,
                            byte,
                        },
                    ));
                }
                Self::Finished => return None,
            }
        }
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
fn attempt(
    report: &mut RobustnessReductionReport,
    input: &RobustnessInput,
    edit: Option<ReductionEdit>,
    output: &Path,
    replay: &mut impl FnMut(&RobustnessInput) -> Result<RobustnessObservation, DifferentialError>,
) -> Result<Option<RobustnessObservation>, DifferentialError> {
    let ordinal = report.attempts.len();
    let baseline = ordinal == 0;
    let stem = format!("attempt-{ordinal:03}");
    let request = format!("{stem}.input.json");
    let bytes = format!("{stem}.bin");
    write_new(&output.join(&bytes), input.bytes())?;
    write_json(&output.join(&request), input)?;
    report.attempts.push(Attempt {
        ordinal,
        baseline,
        request,
        bytes,
        edit,
        complexity: ByteComplexity::of(input.bytes()),
        observation: None,
        replay_error: None,
        decision: Decision::Pending,
    });
    commit_json(output, "reduction", report)?;
    let observed = replay(input);
    // Count only a callback that was actually invoked, including an error.
    if !baseline {
        report.candidate_replays += 1;
    }
    match observed {
        Ok(observation) => {
            let path = format!("{stem}.observation.json");
            write_json(&output.join(&path), &observation)?;
            report.attempts[ordinal].observation = Some(path);
            // Persist the actual journal before making any retention decision,
            // including a foreign callback or a failed provenance check.
            commit_json(output, "reduction", report)?;
            Ok(Some(observation))
        }
        Err(error) => {
            let path = format!("{stem}.error.json");
            write_json(&output.join(&path), &error.to_string())?;
            report.attempts[ordinal].replay_error = Some(path);
            report.attempts[ordinal].decision = Decision::Stopped;
            report.state = RobustnessReductionState::Stopped;
            report.stop_reason = Some(StopReason::ReplayError);
            commit_json(output, "reduction", report)?;
            Ok(None)
        }
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
fn finish(
    mut report: RobustnessReductionReport,
    output: &Path,
) -> Result<RobustnessReductionReport, DifferentialError> {
    if let Some(input) = &report.retained_input {
        write_new(&output.join("minimized.bin"), input.bytes())?;
        write_json(&output.join("minimized.input.json"), input)?;
        report.replay_artifact = Some("minimized.input.json".into());
    }
    commit_json(output, "reduction", &report)?;
    Ok(report)
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn run_with(
    input: &RobustnessInput,
    limit: ArithmeticReductionLimit,
    cancellation: &GeneratedCampaignCancellation,
    output: &Path,
    replay: &mut impl FnMut(&RobustnessInput) -> Result<RobustnessObservation, DifferentialError>,
) -> Result<RobustnessReductionReport, DifferentialError> {
    fs::create_dir(output).map_err(|error| DifferentialError::CampaignOutput {
        path: output.into(),
        message: error.to_string(),
    })?;
    let mut report = RobustnessReductionReport {
        schema_version: 1,
        controller_identity: crate::CompilerProvenance::current()
            .map_err(DifferentialError::WorkerConfiguration)?,
        original: input.clone(),
        replay_limit: limit.get(),
        candidate_replays: 0,
        retained_candidates: 0,
        state: RobustnessReductionState::Running,
        stop_reason: None,
        worker_identity: None,
        witness: None,
        retained_input: None,
        replay_artifact: None,
        semantic_equivalence: SemanticEquivalence::NotEstablished,
        attempts: Vec::new(),
    };
    commit_json(output, "reduction", &report)?;
    if cancellation.cancelled() {
        report.state = RobustnessReductionState::Cancelled;
        return finish(report, output);
    }
    let Some(baseline) = attempt(&mut report, input, None, output, replay)? else {
        return finish(report, output);
    };
    let witness = match signature(&baseline, input, None) {
        Ok(Some(witness)) => witness,
        Ok(None) => {
            report.state = RobustnessReductionState::NotReducible;
            report.attempts[0].decision = Decision::DifferentOutcome;
            return finish(report, output);
        }
        Err(reason) => {
            report.state = RobustnessReductionState::Stopped;
            report.stop_reason = Some(reason);
            report.attempts[0].decision = Decision::Stopped;
            return finish(report, output);
        }
    };
    report.worker_identity = baseline.compiler_identity;
    report.witness = Some(witness.clone());
    report.retained_input = Some(input.clone());
    report.attempts[0].decision = Decision::Retained;
    commit_json(output, "reduction", &report)?;
    let mut current = input.clone();
    let mut cursor = Cursor::new(current.bytes());
    loop {
        if cancellation.cancelled() {
            report.state = RobustnessReductionState::Cancelled;
            break;
        }
        if report.candidate_replays == report.replay_limit {
            report.state = RobustnessReductionState::ReplayBudgetExhausted;
            break;
        }
        let Some((bytes, edit)) = cursor.next(current.bytes()) else {
            report.state = RobustnessReductionState::CandidateSetExhausted;
            break;
        };
        if ByteComplexity::of(&bytes) >= ByteComplexity::of(current.bytes()) {
            return Err(DifferentialError::GeneratorInvariant(
                "robustness reduction candidate did not strictly decrease byte complexity".into(),
            ));
        }
        let candidate = RobustnessInput::new(
            input.id().as_str(),
            input.target(),
            input.timeout_ms(),
            bytes,
        )?;
        let Some(observed) = attempt(&mut report, &candidate, Some(edit), output, replay)? else {
            break;
        };
        let ordinal = report.attempts.len() - 1;
        match signature(&observed, &candidate, report.worker_identity.as_ref()) {
            Ok(Some(actual)) if actual == witness => {
                report.attempts[ordinal].decision = Decision::Retained;
                report.retained_candidates += 1;
                current = candidate;
                report.retained_input = Some(current.clone());
                cursor = Cursor::new(current.bytes());
            }
            Ok(Some(_)) | Ok(None) => {
                report.attempts[ordinal].decision = Decision::DifferentOutcome
            }
            Err(reason) => {
                report.attempts[ordinal].decision = Decision::Stopped;
                report.state = RobustnessReductionState::Stopped;
                report.stop_reason = Some(reason);
            }
        }
        commit_json(output, "reduction", &report)?;
        if report.state == RobustnessReductionState::Stopped {
            break;
        }
    }
    finish(report, output)
}

#[cfg(test)]
mod tests;
