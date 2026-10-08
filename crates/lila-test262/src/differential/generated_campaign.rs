//! Deterministic serial campaigns over the existing generator and reducer.
use super::*;
use crate::CompilerProvenance;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::num::NonZeroU16;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub const MAX_GENERATED_CAMPAIGN_CASES: usize = 128;

/// A bounded, nonwrapping sequence of actual generator plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedCampaignPlan {
    first: GenerationPlan,
    count: NonZeroU16,
    reduction_limit: ArithmeticReductionLimit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GenerationPlan {
    Arithmetic(ArithmeticGenerationPlan),
    Objects(ObjectGenerationPlan),
    Modules(ModuleGenerationPlan),
    ControlFlow(ControlFlowGenerationPlan),
    Negative(NegativeGenerationPlan),
    Scenarios(ScenarioGenerationPlan),
}
impl GenerationPlan {
    fn seed(self) -> u64 {
        match self {
            Self::Arithmetic(plan) => plan.seed().get(),
            Self::Objects(plan) => plan.seed(),
            Self::Modules(plan) => plan.seed(),
            Self::ControlFlow(plan) => plan.seed(),
            Self::Negative(plan) => plan.seed(),
            Self::Scenarios(plan) => plan.seed(),
        }
    }
    fn grammar(self) -> &'static str {
        match self {
            Self::Arithmetic(plan) => plan.grammar().name(),
            Self::Objects(plan) => plan.grammar().name(),
            Self::Modules(plan) => plan.grammar().name(),
            Self::ControlFlow(_) => CONTROL_FLOW_GRAMMAR,
            Self::Negative(_) => NEGATIVE_SOURCE_GRAMMAR,
            Self::Scenarios(plan) => plan.grammar().name(),
        }
    }
    fn with_seed(self, seed: u64) -> Self {
        match self {
            Self::Arithmetic(plan) => Self::Arithmetic(ArithmeticGenerationPlan::new(
                plan.grammar(),
                ArithmeticGenerationSeed::new(seed),
                plan.checks(),
                plan.depth(),
            )),
            Self::Objects(plan) => Self::Objects(plan.with_seed(seed)),
            Self::Modules(plan) => Self::Modules(plan.with_seed(seed)),
            Self::ControlFlow(plan) => Self::ControlFlow(plan.with_seed(seed)),
            Self::Negative(plan) => Self::Negative(plan.with_seed(seed)),
            Self::Scenarios(plan) => Self::Scenarios(plan.with_seed(seed)),
        }
    }
}
impl GeneratedCampaignPlan {
    pub fn new(
        first: ArithmeticGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::Arithmetic(first), count, reduction_limit)
    }
    pub fn for_objects(
        first: ObjectGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::Objects(first), count, reduction_limit)
    }
    pub fn for_modules(
        first: ModuleGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::Modules(first), count, reduction_limit)
    }
    pub fn for_control_flow(
        first: ControlFlowGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::ControlFlow(first), count, reduction_limit)
    }
    pub fn for_negative_source(
        first: NegativeGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::Negative(first), count, reduction_limit)
    }
    pub fn for_scenarios(
        first: ScenarioGenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        Self::checked(GenerationPlan::Scenarios(first), count, reduction_limit)
    }
    fn checked(
        first: GenerationPlan,
        count: usize,
        reduction_limit: ArithmeticReductionLimit,
    ) -> Result<Self, DifferentialError> {
        let count = u16::try_from(count)
            .ok()
            .and_then(NonZeroU16::new)
            .filter(|value| usize::from(value.get()) <= MAX_GENERATED_CAMPAIGN_CASES)
            .ok_or_else(|| {
                DifferentialError::InvalidGeneration(
                    "generated campaign case count must be in 1..=128".into(),
                )
            })?;
        first
            .seed()
            .checked_add(u64::from(count.get()) - 1)
            .ok_or_else(|| {
                DifferentialError::InvalidGeneration(
                    "generated campaign seed sequence overflows u64".into(),
                )
            })?;
        Ok(Self {
            first,
            count,
            reduction_limit,
        })
    }
    pub fn count(self) -> usize {
        usize::from(self.count.get())
    }
    fn case(self, ordinal: usize) -> GenerationPlan {
        self.first.with_seed(self.first.seed() + ordinal as u64)
    }
}

/// Cancellation is observed before every initial or reduced replay. An active
/// bounded worker pair finishes its own existing retirement before returning.
#[derive(Debug, Clone, Default)]
pub struct GeneratedCampaignCancellation(Arc<AtomicBool>);
impl GeneratedCampaignCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub(super) fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedCampaignVerdict {
    Incomplete,
    AllMatched,
    ContainsFailures,
    Cancelled,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum CaseState {
    Pending,
    Finished { record: String, matched: bool },
    Cancelled { record: String },
}

#[derive(Debug, Serialize)]
struct CaseSummary {
    ordinal: usize,
    seed: u64,
    #[serde(flatten)]
    state: CaseState,
}

/// Only the serial journal owner publishes this aggregate.
#[derive(Debug, Serialize)]
pub struct GeneratedCampaignReport {
    schema_version: u32,
    controller_identity: CompilerProvenance,
    grammar: &'static str,
    first_seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    checks: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nodes: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    steps: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modules: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edges: Option<u8>,
    max_reduction_replays: u16,
    total: usize,
    completed: usize,
    matched: usize,
    failed: usize,
    verdict: GeneratedCampaignVerdict,
    semantic_equivalence: SemanticEquivalence,
    cases: Vec<CaseSummary>,
}
impl GeneratedCampaignReport {
    pub const fn total(&self) -> usize {
        self.total
    }
    pub const fn completed(&self) -> usize {
        self.completed
    }
    pub const fn failed(&self) -> usize {
        self.failed
    }
    pub const fn verdict(&self) -> GeneratedCampaignVerdict {
        self.verdict
    }
    pub fn is_green(&self) -> bool {
        self.verdict == GeneratedCampaignVerdict::AllMatched
            && self.total > 0
            && self.completed == self.total
            && self.matched == self.total
            && self.failed == 0
            && self.cases.len() == self.total
            && self
                .cases
                .iter()
                .all(|case| matches!(&case.state, CaseState::Finished { matched: true, .. }))
    }
}

#[derive(Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
enum AttemptOutcome<'a> {
    Observed { report: &'a DifferentialReport },
    Rejected { message: String },
}
#[derive(Serialize)]
struct AttemptRecord<'a> {
    request: String,
    #[serde(flatten)]
    outcome: AttemptOutcome<'a>,
}

#[derive(Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
enum CaseOutcome {
    Verified {
        case: String,
        report: DifferentialReport,
    },
    ReducedMismatch {
        case: String,
        report: DifferentialReport,
        reduction: ReductionSummary,
    },
    Rejected {
        report: DifferentialReport,
    },
    ExecutionRejected {
        message: String,
    },
    FrontendNegative {
        observation: NegativeSourceObservation,
        reduction: Option<NegativeReductionSummary>,
    },
    Scenarios {
        observations: ScenarioObservations,
        reduction: Option<ScenarioReductionSummary>,
    },
    Cancelled,
}
#[derive(Serialize)]
#[serde(tag = "grammar_family", content = "summary", rename_all = "snake_case")]
enum ReductionSummary {
    Arithmetic(ArithmeticReductionSummary),
    Objects(ObjectReductionSummary),
    Modules(ModuleReductionSummary),
    ControlFlow(ControlFlowReductionSummary),
}
enum GeneratedOutcome {
    Verified {
        case: DifferentialReplayInput,
        report: DifferentialReport,
    },
    ReducedMismatch {
        case: DifferentialReplayInput,
        report: DifferentialReport,
        reduction: ReductionSummary,
    },
    Rejected {
        report: DifferentialReport,
    },
    Negative(generated_negative::Outcome),
    Scenarios(generated_scenarios::Outcome),
}
fn generate_with_replay(
    plan: GenerationPlan,
    limit: ArithmeticReductionLimit,
    replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<GeneratedOutcome, DifferentialError> {
    match plan {
        GenerationPlan::Arithmetic(plan) => generated_arithmetic::run_with_replay(
            plan, limit, replay,
        )
        .map(|outcome| match outcome {
            GeneratedArithmeticCampaignOutcome::Verified { case, report } => {
                GeneratedOutcome::Verified { case, report }
            }
            GeneratedArithmeticCampaignOutcome::ReducedMismatch {
                case,
                report,
                reduction,
            } => GeneratedOutcome::ReducedMismatch {
                case,
                report,
                reduction: ReductionSummary::Arithmetic(reduction),
            },
            GeneratedArithmeticCampaignOutcome::Rejected { report, .. } => {
                GeneratedOutcome::Rejected { report }
            }
        }),
        GenerationPlan::Objects(plan) => generated_objects::run_with_replay(plan, limit, replay)
            .map(|outcome| match outcome {
                generated_objects::Outcome::Verified { case, report } => {
                    GeneratedOutcome::Verified { case, report }
                }
                generated_objects::Outcome::ReducedMismatch {
                    case,
                    report,
                    reduction,
                } => GeneratedOutcome::ReducedMismatch {
                    case,
                    report,
                    reduction: ReductionSummary::Objects(reduction),
                },
                generated_objects::Outcome::Rejected { report } => {
                    GeneratedOutcome::Rejected { report }
                }
            }),
        GenerationPlan::Modules(plan) => generated_modules::run_with_replay(plan, limit, replay)
            .map(|outcome| match outcome {
                generated_modules::Outcome::Verified { case, report } => {
                    GeneratedOutcome::Verified { case, report }
                }
                generated_modules::Outcome::ReducedMismatch {
                    case,
                    report,
                    reduction,
                } => GeneratedOutcome::ReducedMismatch {
                    case,
                    report,
                    reduction: ReductionSummary::Modules(reduction),
                },
                generated_modules::Outcome::Rejected { report } => {
                    GeneratedOutcome::Rejected { report }
                }
            }),
        GenerationPlan::ControlFlow(plan) => generated_control_flow::run_with_replay(
            plan, limit, replay,
        )
        .map(|outcome| match outcome {
            generated_control_flow::Outcome::Verified { case, report } => {
                GeneratedOutcome::Verified { case, report }
            }
            generated_control_flow::Outcome::ReducedMismatch {
                case,
                report,
                reduction,
            } => GeneratedOutcome::ReducedMismatch {
                case,
                report,
                reduction: ReductionSummary::ControlFlow(reduction),
            },
            generated_control_flow::Outcome::Rejected { report } => {
                GeneratedOutcome::Rejected { report }
            }
        }),
        GenerationPlan::Negative(plan) => {
            generated_negative::run_with_replay(plan, limit, replay).map(GeneratedOutcome::Negative)
        }
        GenerationPlan::Scenarios(plan) => {
            generated_scenarios::run_with_replay(plan, limit, replay)
                .map(GeneratedOutcome::Scenarios)
        }
    }
}
#[derive(Serialize)]
struct CaseRecord {
    ordinal: usize,
    seed: u64,
    attempts: usize,
    #[serde(flatten)]
    outcome: CaseOutcome,
}

fn output_error(path: &Path, error: impl std::fmt::Display) -> DifferentialError {
    DifferentialError::CampaignOutput {
        path: path.into(),
        message: error.to_string(),
    }
}
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<(), DifferentialError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| output_error(path, error))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| output_error(path, error))
}
pub(super) fn write_json(path: &Path, value: &impl Serialize) -> Result<(), DifferentialError> {
    write_new(
        path,
        &serde_json::to_vec_pretty(value).map_err(|error| output_error(path, error))?,
    )
}
fn commit(output: &Path, report: &GeneratedCampaignReport) -> Result<(), DifferentialError> {
    commit_json(output, "aggregate", report)
}
pub(super) fn commit_json(
    output: &Path,
    stem: &str,
    report: &impl Serialize,
) -> Result<(), DifferentialError> {
    let temporary = output.join(format!("{stem}.tmp"));
    write_json(&temporary, report)?;
    let destination = output.join(format!("{stem}.json"));
    fs::rename(&temporary, &destination).map_err(|error| output_error(&destination, error))?;
    #[cfg(unix)]
    fs::File::open(output)
        .and_then(|file| file.sync_all())
        .map_err(|error| output_error(output, error))?;
    Ok(())
}

pub fn run_generated_campaign(
    plan: GeneratedCampaignPlan,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
    cancellation: &GeneratedCampaignCancellation,
    output: &Path,
) -> Result<GeneratedCampaignReport, DifferentialError> {
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = (plan, oracle, runner, cancellation, output);
        return Err(DifferentialError::OracleNotLinked);
    }
    #[cfg(feature = "spec-exec-oracle")]
    {
        let controller_identity =
            CompilerProvenance::current().map_err(DifferentialError::WorkerConfiguration)?;
        // A previously used directory is never resumed or silently truncated.
        fs::create_dir(output).map_err(|error| output_error(output, error))?;
        for child in ["evidence", "corpus"] {
            let path = output.join(child);
            fs::create_dir(&path).map_err(|error| output_error(&path, error))?;
        }
        let mut aggregate = GeneratedCampaignReport {
            schema_version: 1,
            controller_identity,
            grammar: plan.first.grammar(),
            first_seed: plan.first.seed(),
            checks: match plan.first {
                GenerationPlan::Arithmetic(plan) => Some(plan.checks().get()),
                GenerationPlan::Objects(_)
                | GenerationPlan::Modules(_)
                | GenerationPlan::ControlFlow(_)
                | GenerationPlan::Negative(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            depth: match plan.first {
                GenerationPlan::Arithmetic(plan) => Some(plan.depth().get()),
                GenerationPlan::ControlFlow(plan) => Some(plan.depth()),
                GenerationPlan::Negative(plan) => Some(plan.depth()),
                GenerationPlan::Objects(_)
                | GenerationPlan::Modules(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            nodes: match plan.first {
                GenerationPlan::Objects(plan) => Some(plan.nodes()),
                GenerationPlan::Arithmetic(_)
                | GenerationPlan::Modules(_)
                | GenerationPlan::ControlFlow(_)
                | GenerationPlan::Negative(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            properties: match plan.first {
                GenerationPlan::Objects(plan) => Some(plan.properties()),
                GenerationPlan::Arithmetic(_)
                | GenerationPlan::Modules(_)
                | GenerationPlan::ControlFlow(_)
                | GenerationPlan::Negative(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            steps: match plan.first {
                GenerationPlan::Objects(plan) => {
                    (plan.grammar() == ObjectGrammar::MutationsV2).then_some(plan.steps())
                }
                GenerationPlan::ControlFlow(plan) => Some(plan.steps()),
                GenerationPlan::Negative(plan) => Some(plan.steps()),
                GenerationPlan::Scenarios(plan) => Some(plan.steps()),
                GenerationPlan::Arithmetic(_) | GenerationPlan::Modules(_) => None,
            },
            modules: match plan.first {
                GenerationPlan::Modules(plan) => Some(plan.modules()),
                GenerationPlan::Arithmetic(_)
                | GenerationPlan::Objects(_)
                | GenerationPlan::ControlFlow(_)
                | GenerationPlan::Negative(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            edges: match plan.first {
                GenerationPlan::Modules(plan) => Some(plan.edges()),
                GenerationPlan::Arithmetic(_)
                | GenerationPlan::Objects(_)
                | GenerationPlan::ControlFlow(_)
                | GenerationPlan::Negative(_)
                | GenerationPlan::Scenarios(_) => None,
            },
            max_reduction_replays: plan.reduction_limit.get(),
            total: plan.count(),
            completed: 0,
            matched: 0,
            failed: 0,
            verdict: GeneratedCampaignVerdict::Incomplete,
            semantic_equivalence: SemanticEquivalence::NotEstablished,
            cases: (0..plan.count())
                .map(|ordinal| CaseSummary {
                    ordinal,
                    seed: plan.case(ordinal).seed(),
                    state: CaseState::Pending,
                })
                .collect(),
        };
        commit(output, &aggregate)?;
        for ordinal in 0..plan.count() {
            let mut attempts = 0;
            let outcome = generate_with_replay(plan.case(ordinal), plan.reduction_limit, |input| {
                if cancellation.cancelled() {
                    return Err(DifferentialError::CampaignCancelled);
                }
                let stem = format!("case-{ordinal:03}.attempt-{attempts:03}");
                let request = format!("evidence/{stem}.request.json");
                write_new(&output.join(&request), input.to_pretty_json()?.as_bytes())?;
                attempts += 1;
                let result = replay_case(input, oracle, runner);
                // Retain the exact source-dependent fingerprint for every
                // initial/reduced replay, even when reduction later rejects it.
                let observed = match &result {
                    Ok(report) => AttemptOutcome::Observed { report },
                    Err(error) => AttemptOutcome::Rejected {
                        message: error.to_string(),
                    },
                };
                write_json(
                    &output.join(format!("evidence/{stem}.report.json")),
                    &AttemptRecord {
                        request,
                        outcome: observed,
                    },
                )?;
                result
            });
            let mut worker_failed = false;
            let mut cancelled = false;
            let (outcome, matched) = match outcome {
                Ok(GeneratedOutcome::Verified { case, report }) => {
                    let artifact = format!("corpus/case-{ordinal:03}.json");
                    write_new(&output.join(&artifact), case.to_pretty_json()?.as_bytes())?;
                    (
                        CaseOutcome::Verified {
                            case: artifact,
                            report,
                        },
                        true,
                    )
                }
                Ok(GeneratedOutcome::ReducedMismatch {
                    case,
                    report,
                    reduction,
                }) => {
                    let artifact = format!("corpus/case-{ordinal:03}.json");
                    write_new(&output.join(&artifact), case.to_pretty_json()?.as_bytes())?;
                    (
                        CaseOutcome::ReducedMismatch {
                            case: artifact,
                            report,
                            reduction,
                        },
                        false,
                    )
                }
                Ok(GeneratedOutcome::Rejected { report }) => {
                    worker_failed = report.verdict() == DifferentialVerdict::WorkerFailure;
                    (CaseOutcome::Rejected { report }, false)
                }
                Ok(GeneratedOutcome::Negative(outcome)) => {
                    let (observation, reduction) = match outcome {
                        generated_negative::Outcome::Verified { observation }
                        | generated_negative::Outcome::Rejected { observation } => {
                            (observation, None)
                        }
                        generated_negative::Outcome::ReducedMismatch {
                            observation,
                            reduction,
                        } => (observation, Some(reduction)),
                    };
                    worker_failed = observation.worker_failed();
                    let matched = observation.is_green();
                    // The raw report truthfully retains BothFailed. Expected
                    // source rejection belongs in its typed campaign record,
                    // separate from the green executable regression corpus.
                    write_new(
                        &output.join(format!("evidence/case-{ordinal:03}.negative.json")),
                        observation.input().to_pretty_json()?.as_bytes(),
                    )?;
                    (
                        CaseOutcome::FrontendNegative {
                            observation,
                            reduction,
                        },
                        matched,
                    )
                }
                Ok(GeneratedOutcome::Scenarios(outcome)) => {
                    let (observations, reduction, retain) = match outcome {
                        generated_scenarios::Outcome::Verified { observations } => {
                            (observations, None, true)
                        }
                        generated_scenarios::Outcome::ReducedMismatch {
                            observations,
                            reduction,
                        } => (observations, Some(reduction), true),
                        generated_scenarios::Outcome::Rejected { observations } => {
                            (observations, None, false)
                        }
                    };
                    worker_failed = observations.baseline().report().verdict()
                        == DifferentialVerdict::WorkerFailure
                        || observations.transformed().is_some_and(|observed| {
                            observed.report().verdict() == DifferentialVerdict::WorkerFailure
                        });
                    let matched = observations.is_green();
                    if retain {
                        write_new(
                            &output.join(format!("corpus/case-{ordinal:03}.baseline.json")),
                            observations.baseline().case().to_pretty_json()?.as_bytes(),
                        )?;
                        if let Some(transformed) = observations.transformed() {
                            write_new(
                                &output.join(format!("corpus/case-{ordinal:03}.transformed.json")),
                                transformed.case().to_pretty_json()?.as_bytes(),
                            )?;
                        }
                    }
                    if let Some(pair) = observations.replay_pair() {
                        write_new(
                            &output.join(format!("evidence/case-{ordinal:03}.pair.json")),
                            pair.to_pretty_json()?.as_bytes(),
                        )?;
                    }
                    // This record retains both exact inputs, both original
                    // reports and the separate within-backend relation.
                    (
                        CaseOutcome::Scenarios {
                            observations,
                            reduction,
                        },
                        matched,
                    )
                }
                Err(DifferentialError::CampaignCancelled) => {
                    cancelled = true;
                    (CaseOutcome::Cancelled, false)
                }
                Err(error @ DifferentialError::CampaignOutput { .. }) => return Err(error),
                Err(error) => (
                    CaseOutcome::ExecutionRejected {
                        message: error.to_string(),
                    },
                    false,
                ),
            };
            let record = format!("evidence/case-{ordinal:03}.json");
            write_json(
                &output.join(&record),
                &CaseRecord {
                    ordinal,
                    seed: plan.case(ordinal).seed(),
                    attempts,
                    outcome,
                },
            )?;
            if cancelled {
                aggregate.cases[ordinal].state = CaseState::Cancelled { record };
                aggregate.verdict = GeneratedCampaignVerdict::Cancelled;
            } else {
                aggregate.cases[ordinal].state = CaseState::Finished { record, matched };
                aggregate.completed += 1;
                if matched {
                    aggregate.matched += 1;
                } else {
                    aggregate.failed += 1;
                }
                if aggregate.completed == aggregate.total {
                    aggregate.verdict = if aggregate.failed == 0 {
                        GeneratedCampaignVerdict::AllMatched
                    } else {
                        GeneratedCampaignVerdict::ContainsFailures
                    };
                }
            }
            commit(output, &aggregate)?;
            if cancelled || worker_failed {
                break;
            }
        }
        Ok(aggregate)
    }
}
