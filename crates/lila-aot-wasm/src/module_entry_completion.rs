//! Host status beside, rather than inside, the ECMAScript completion ABI.

/// Present only for an artifact with a compiler-owned Module entry operation.
/// The mutable i64 export is sampled after the main job/host-work checkpoint.
pub const MODULE_EVALUATION_STATUS_EXPORT: &str = "module_evaluation_status";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmModuleEvaluationStatus {
    /// The artifact has not reached its entry checkpoint. Earlier source
    /// throws become Settled there even when Promise adoption was not reached.
    NotStarted,
    /// Supported host work is quiescent, but the entry capability is pending.
    Pending,
    /// The ECMAScript result exports own the terminal Normal or Throw.
    Settled,
}

impl WasmModuleEvaluationStatus {
    pub const fn word(self) -> i64 {
        match self {
            Self::NotStarted => 0,
            Self::Pending => 1,
            Self::Settled => 2,
        }
    }

    pub const fn from_word(word: i64) -> Option<Self> {
        match word {
            0 => Some(Self::NotStarted),
            1 => Some(Self::Pending),
            2 => Some(Self::Settled),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_status_has_a_closed_wire_domain_outside_js_completion_kinds() {
        for status in [
            WasmModuleEvaluationStatus::NotStarted,
            WasmModuleEvaluationStatus::Pending,
            WasmModuleEvaluationStatus::Settled,
        ] {
            assert_eq!(
                WasmModuleEvaluationStatus::from_word(status.word()),
                Some(status)
            );
        }
        for word in [-1, 3, i64::MIN, i64::MAX] {
            assert_eq!(WasmModuleEvaluationStatus::from_word(word), None);
        }
    }
}
