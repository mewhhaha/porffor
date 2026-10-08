//! Bounded native input mutations, executed only in the selected worker.
use super::*;
mod campaign;
#[cfg(any(test, feature = "spec-exec-oracle"))]
mod filesystem;
mod input;
#[cfg(any(test, feature = "spec-exec-oracle"))]
mod prelude;
mod reduce;
#[cfg(any(test, feature = "spec-exec-oracle"))]
mod sandbox;
#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "spec-exec-oracle"))]
mod uri;
#[cfg(feature = "spec-exec-oracle")]
pub(super) mod worker;
pub use campaign::{
    run_robustness_campaign, RobustnessCampaignPlan, RobustnessCampaignReport,
    RobustnessCampaignState, RobustnessMutation,
};
pub use input::{BuiltinParserTarget, RobustnessInput, RobustnessTarget};
pub use reduce::{minimize_robustness, RobustnessReductionReport, RobustnessReductionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobustnessStage {
    Decode,
    IrInput,
    Preparation,
    Lowering,
    IrAdmission,
    Emission,
    RuntimeSetup,
    Validation,
    Frontmatter,
    Snapshot,
    Corpus,
    ModuleGraph,
    ReportObservation,
    BuiltinExecution,
    BuiltinInput,
    PreludeInput,
    PreludeLoad,
    PreludeMaterialization,
    FilesystemInput,
    FilesystemSetup,
    ModuleResolution,
    ModuleLoading,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobustnessCompletionKind {
    Normal,
    Throw,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobustnessRejectionPhase {
    Encoding,
    Parse,
    EarlyError,
    Resolution,
    NativeBoundary,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RobustnessResult {
    Accepted {
        stage: RobustnessStage,
        artifact_bytes: Option<u64>,
    },
    Rejected {
        stage: RobustnessStage,
        phase: RobustnessRejectionPhase,
        message: String,
    },
    Unsupported {
        stage: RobustnessStage,
        message: String,
    },
    Failure {
        stage: RobustnessStage,
        message: String,
    },
    InvalidWasm {
        message: String,
    },
    Executed {
        completion: RobustnessCompletionKind,
        value: String,
    },
    WorkerFailure {
        failure: DifferentialWorkerFailure,
        cleanup_error: Option<String>,
    },
}
impl RobustnessResult {
    /// This says a bounded target returned normally; it is not source acceptance,
    /// semantic equivalence, a fuzz campaign milestone or a conformance verdict.
    pub fn completed_without_failure(&self) -> bool {
        matches!(
            self,
            Self::Accepted { .. } | Self::Rejected { .. } | Self::Executed { .. }
        )
    }
    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(in crate::differential) fn valid_terminal(
        &self,
        target: RobustnessTarget,
        stages: &[RobustnessStage],
    ) -> bool {
        let Some(last) = stages.last() else {
            return false;
        };
        let expected = target.stages();
        if stages.len() > expected.len() || stages != &expected[..stages.len()] {
            return false;
        }
        match self {
            Self::Accepted {
                stage,
                artifact_bytes,
            } => {
                stages == expected
                    && stage == last
                    && match target {
                        RobustnessTarget::Compiler { .. } | RobustnessTarget::IrAdmission {} => {
                            artifact_bytes.is_some_and(|bytes| bytes >= 8)
                        }
                        RobustnessTarget::Builtin { .. } => false,
                        RobustnessTarget::Frontmatter {}
                        | RobustnessTarget::Snapshot {}
                        | RobustnessTarget::Corpus {}
                        | RobustnessTarget::ModuleGraph {}
                        | RobustnessTarget::Prelude {}
                        | RobustnessTarget::FilesystemResolver {}
                        | RobustnessTarget::ReportObservation {} => artifact_bytes.is_none(),
                    }
            }
            Self::Rejected { stage, phase, .. } => {
                stage == last
                    && match phase {
                        RobustnessRejectionPhase::Encoding => *stage == RobustnessStage::Decode,
                        RobustnessRejectionPhase::Parse => *stage == RobustnessStage::Preparation,
                        RobustnessRejectionPhase::EarlyError => matches!(
                            stage,
                            RobustnessStage::Preparation | RobustnessStage::Lowering
                        ),
                        RobustnessRejectionPhase::Resolution => *stage == RobustnessStage::Lowering,
                        RobustnessRejectionPhase::NativeBoundary => matches!(
                            stage,
                            RobustnessStage::BuiltinInput
                                | RobustnessStage::PreludeInput
                                | RobustnessStage::PreludeLoad
                                | RobustnessStage::PreludeMaterialization
                                | RobustnessStage::FilesystemInput
                                | RobustnessStage::ModuleResolution
                                | RobustnessStage::ModuleLoading
                                | RobustnessStage::Frontmatter
                                | RobustnessStage::IrInput
                                | RobustnessStage::IrAdmission
                                | RobustnessStage::Snapshot
                                | RobustnessStage::Corpus
                                | RobustnessStage::ModuleGraph
                                | RobustnessStage::ReportObservation
                        ),
                    }
            }
            Self::Unsupported { stage, .. } | Self::Failure { stage, .. } => stage == last,
            Self::InvalidWasm { .. } => *last == RobustnessStage::Validation,
            Self::Executed { .. } => {
                matches!(target, RobustnessTarget::Builtin { .. }) && stages == expected
            }
            Self::WorkerFailure { .. } => false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RobustnessObservation {
    pub(in crate::differential) input: RobustnessInput,
    pub(in crate::differential) compiler_identity: Option<crate::CompilerProvenance>,
    pub(in crate::differential) stages: Vec<RobustnessStage>,
    pub(in crate::differential) result: RobustnessResult,
    pub(in crate::differential) journal_bytes_hex: String,
    pub(in crate::differential) stderr_bytes_hex: String,
    pub(in crate::differential) journal_error: Option<String>,
    // Only the bound decoder mints this for a fully committed stage prefix
    // without a terminal. Torn, foreign and completed journals cannot qualify.
    pub(in crate::differential) interrupted_stage: Option<RobustnessStage>,
}
impl RobustnessObservation {
    pub fn input(&self) -> &RobustnessInput {
        &self.input
    }
    pub fn compiler_identity(&self) -> Option<&crate::CompilerProvenance> {
        self.compiler_identity.as_ref()
    }
    pub fn stages(&self) -> &[RobustnessStage] {
        &self.stages
    }
    pub fn result(&self) -> &RobustnessResult {
        &self.result
    }
    pub fn completed_without_failure(&self) -> bool {
        self.compiler_identity.is_some()
            && self.journal_error.is_none()
            && self.result.completed_without_failure()
    }
}
pub fn replay_robustness(
    input: &RobustnessInput,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
) -> Result<RobustnessObservation, DifferentialError> {
    #[cfg(feature = "spec-exec-oracle")]
    {
        runner.run_robustness(input, oracle)
    }
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = (input, oracle, runner);
        Err(DifferentialError::OracleNotLinked)
    }
}
