#[cfg(any(test, feature = "spec-exec-oracle"))]
use super::super::generated_campaign::{commit_json, write_json, write_new};
use super::*;

#[derive(Debug, Clone)]
pub struct RobustnessCampaignPlan {
    input: RobustnessInput,
    seed: u64,
    count: u8,
}
impl RobustnessCampaignPlan {
    pub fn new(input: RobustnessInput, seed: u64, count: usize) -> Result<Self, DifferentialError> {
        if !(1..=MAX_GENERATED_CAMPAIGN_CASES).contains(&count) {
            return Err(DifferentialError::InvalidGeneration(
                "robustness cases require the existing 1..=128 campaign bound".into(),
            ));
        }
        seed.checked_add(count as u64 - 1).ok_or_else(|| {
            DifferentialError::InvalidGeneration("robustness seed sequence overflows".into())
        })?;
        Ok(Self {
            input,
            seed,
            count: count as u8,
        })
    }
    pub fn count(&self) -> usize {
        self.count as usize
    }
    pub fn input(&self) -> &RobustnessInput {
        &self.input
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(super) fn case(
        &self,
        ordinal: usize,
    ) -> Result<(RobustnessInput, RobustnessMutation), DifferentialError> {
        if ordinal >= self.count() {
            return Err(DifferentialError::GeneratorInvariant(
                "robustness ordinal outside checked plan".into(),
            ));
        }
        let seed = self.seed + ordinal as u64;
        let (bytes, mutation) = if ordinal == 0 {
            (self.input.bytes().to_vec(), RobustnessMutation::Original)
        } else {
            mutate(self.input.bytes(), seed)
        };
        let input = RobustnessInput::new(
            format!("robustness/seed-{seed}-case-{ordinal}"),
            self.input.target(),
            self.input.timeout_ms(),
            bytes,
        )?;
        Ok((input, mutation))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RobustnessMutation {
    Original,
    Delete { start: usize, end: usize },
    Truncate { end: usize },
    Replace { offset: usize, byte: u8 },
    Insert { offset: usize, byte: u8 },
    Duplicate { start: usize, end: usize },
}
#[cfg(any(test, feature = "spec-exec-oracle"))]
fn mutate(original: &[u8], seed: u64) -> (Vec<u8>, RobustnessMutation) {
    enum Choice {
        Delete,
        Truncate,
        Replace,
        Insert,
        Duplicate,
    }
    const CHOICES: [Choice; 5] = [
        Choice::Delete,
        Choice::Truncate,
        Choice::Replace,
        Choice::Insert,
        Choice::Duplicate,
    ];
    let mut random = seed ^ 0x72bc_9457_61d0_09a3;
    let mut next = || {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        random
    };
    let choice = &CHOICES[(next() % CHOICES.len() as u64) as usize];
    let length = original.len();
    let mut bytes = original.to_vec();
    let offset = (next() % (length as u64 + 1)) as usize;
    let byte = next() as u8;
    let mutation = if length == 0 {
        bytes.push(byte);
        RobustnessMutation::Insert { offset: 0, byte }
    } else {
        let start = offset.min(length - 1);
        let end = start + 1 + (next() % (length - start) as u64) as usize;
        match choice {
            Choice::Delete => {
                bytes.drain(start..end);
                RobustnessMutation::Delete { start, end }
            }
            Choice::Truncate => {
                bytes.truncate(offset);
                RobustnessMutation::Truncate { end: offset }
            }
            Choice::Replace => {
                bytes[start] = byte;
                RobustnessMutation::Replace {
                    offset: start,
                    byte,
                }
            }
            Choice::Insert if length < super::input::MAX_INPUT_BYTES => {
                bytes.insert(offset, byte);
                RobustnessMutation::Insert { offset, byte }
            }
            Choice::Insert => {
                bytes[start] = byte;
                RobustnessMutation::Replace {
                    offset: start,
                    byte,
                }
            }
            Choice::Duplicate => {
                let end = end.min(start + super::input::MAX_INPUT_BYTES - length);
                if end > start {
                    bytes.splice(end..end, original[start..end].iter().copied());
                    RobustnessMutation::Duplicate { start, end }
                } else {
                    bytes[start] = byte;
                    RobustnessMutation::Replace {
                        offset: start,
                        byte,
                    }
                }
            }
        }
    };
    (bytes, mutation)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RobustnessCampaignState {
    Running,
    Finished,
    Failed,
    Cancelled,
}
#[derive(Debug, Serialize)]
struct CaseRecord {
    ordinal: usize,
    seed: u64,
    request: Option<String>,
    observation: Option<String>,
    mutation: Option<RobustnessMutation>,
    disposition: Option<Disposition>,
    completed_without_failure: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    Accepted,
    Rejected,
    Executed,
    Unsupported,
    Failure,
    InvalidWasm,
    WorkerFailure,
}
impl From<&RobustnessResult> for Disposition {
    fn from(result: &RobustnessResult) -> Self {
        match result {
            RobustnessResult::Accepted { .. } => Self::Accepted,
            RobustnessResult::Rejected { .. } => Self::Rejected,
            RobustnessResult::Executed { .. } => Self::Executed,
            RobustnessResult::Unsupported { .. } => Self::Unsupported,
            RobustnessResult::Failure { .. } => Self::Failure,
            RobustnessResult::InvalidWasm { .. } => Self::InvalidWasm,
            RobustnessResult::WorkerFailure { .. } => Self::WorkerFailure,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct RobustnessCampaignReport {
    schema_version: u32,
    controller_identity: crate::CompilerProvenance,
    target: RobustnessTarget,
    seed: u64,
    total: usize,
    completed: usize,
    state: RobustnessCampaignState,
    semantic_equivalence: SemanticEquivalence,
    cases: Vec<CaseRecord>,
}
impl RobustnessCampaignReport {
    pub fn state(&self) -> RobustnessCampaignState {
        self.state
    }
    pub fn completed(&self) -> usize {
        self.completed
    }
    pub fn completed_without_failure(&self) -> bool {
        self.state == RobustnessCampaignState::Finished
            && self.total > 0
            && self.completed == self.total
            && self.cases.len() == self.total
            && self
                .cases
                .iter()
                .all(|case| case.completed_without_failure == Some(true))
    }
}
pub fn run_robustness_campaign(
    plan: &RobustnessCampaignPlan,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
    cancellation: &GeneratedCampaignCancellation,
    output: impl AsRef<Path>,
) -> Result<RobustnessCampaignReport, DifferentialError> {
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = (plan, oracle, runner, cancellation, output);
        Err(DifferentialError::OracleNotLinked)
    }
    #[cfg(feature = "spec-exec-oracle")]
    {
        run_with(plan, cancellation, output.as_ref(), &mut |input| {
            replay_robustness(input, oracle, runner)
        })
    }
}
#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn run_with(
    plan: &RobustnessCampaignPlan,
    cancellation: &GeneratedCampaignCancellation,
    output: &Path,
    replay: &mut impl FnMut(&RobustnessInput) -> Result<RobustnessObservation, DifferentialError>,
) -> Result<RobustnessCampaignReport, DifferentialError> {
    let output_error = |path: &Path, error: std::io::Error| DifferentialError::CampaignOutput {
        path: path.into(),
        message: error.to_string(),
    };
    fs::create_dir(output).map_err(|error| output_error(output, error))?;
    let mut report = RobustnessCampaignReport {
        schema_version: 1,
        controller_identity: crate::CompilerProvenance::current()
            .map_err(DifferentialError::WorkerConfiguration)?,
        target: plan.input.target(),
        seed: plan.seed,
        total: plan.count(),
        completed: 0,
        state: RobustnessCampaignState::Running,
        semantic_equivalence: SemanticEquivalence::NotEstablished,
        cases: (0..plan.count())
            .map(|ordinal| CaseRecord {
                ordinal,
                seed: plan.seed + ordinal as u64,
                request: None,
                observation: None,
                mutation: None,
                disposition: None,
                completed_without_failure: None,
            })
            .collect(),
    };
    write_new(&output.join("base.bin"), plan.input.bytes())?;
    write_json(&output.join("base.input.json"), &plan.input)?;
    commit_json(output, "robustness", &report)?;
    for ordinal in 0..plan.count() {
        if cancellation.cancelled() {
            report.state = RobustnessCampaignState::Cancelled;
            commit_json(output, "robustness", &report)?;
            break;
        }
        let (input, mutation) = plan.case(ordinal)?;
        let request = format!("case-{ordinal:03}.input.json");
        let observation = format!("case-{ordinal:03}.observation.json");
        write_new(
            &output.join(format!("case-{ordinal:03}.bin")),
            input.bytes(),
        )?;
        write_json(&output.join(&request), &input)?;
        report.cases[ordinal].request = Some(request);
        report.cases[ordinal].mutation = Some(mutation);
        commit_json(output, "robustness", &report)?;
        let observed = match replay(&input) {
            Ok(observed) => observed,
            Err(error) => {
                write_json(
                    &output.join(format!("case-{ordinal:03}.rejection.json")),
                    &error.to_string(),
                )?;
                report.state = RobustnessCampaignState::Failed;
                commit_json(output, "robustness", &report)?;
                return Err(error);
            }
        };
        if observed.input != input {
            return Err(DifferentialError::GeneratorInvariant(
                "robustness replay returned foreign bytes/target/identity".into(),
            ));
        }
        let completed = observed.completed_without_failure();
        write_json(&output.join(&observation), &observed)?;
        report.cases[ordinal].observation = Some(observation);
        report.cases[ordinal].completed_without_failure = Some(completed);
        report.cases[ordinal].disposition = Some(Disposition::from(observed.result()));
        report.completed += 1;
        if !completed {
            report.state = RobustnessCampaignState::Failed;
        } else if report.completed == report.total {
            report.state = RobustnessCampaignState::Finished;
        }
        commit_json(output, "robustness", &report)?;
        if !completed {
            break;
        }
    }
    Ok(report)
}
