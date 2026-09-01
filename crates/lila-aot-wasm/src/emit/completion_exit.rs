use super::*;

/// The one exit policy owned by an emitted body.
///
/// `ReturnAbi` describes the public Wasm signature. This state additionally
/// records the temporary host-checkpoint target that is live while main-source
/// statements are emitted. Its private representation prevents an internal
/// function from acquiring a main checkpoint or a caller from manufacturing a
/// target without the checked transition methods below.
pub(crate) struct CompletionExit(CompletionExitState);

enum CompletionExitState {
    MainExport,
    MainJobCheckpoint(ControlTarget),
    MultiValue,
}

impl CompletionExit {
    pub(super) fn for_return_abi(return_abi: ReturnAbi) -> Self {
        Self(match return_abi {
            ReturnAbi::MainExport => CompletionExitState::MainExport,
            ReturnAbi::MultiValue => CompletionExitState::MultiValue,
        })
    }

    pub(crate) const fn return_abi(&self) -> ReturnAbi {
        match &self.0 {
            CompletionExitState::MainExport | CompletionExitState::MainJobCheckpoint(_) => {
                ReturnAbi::MainExport
            }
            CompletionExitState::MultiValue => ReturnAbi::MultiValue,
        }
    }

    pub(crate) const fn main_job_checkpoint_target(&self) -> Option<ControlTarget> {
        match &self.0 {
            CompletionExitState::MainJobCheckpoint(target) => Some(*target),
            CompletionExitState::MainExport | CompletionExitState::MultiValue => None,
        }
    }

    pub(super) fn enter_main_job_checkpoint(&mut self, target: ControlTarget) {
        assert!(match &self.0 {
            CompletionExitState::MainExport => true,
            CompletionExitState::MainJobCheckpoint(_) | CompletionExitState::MultiValue => false,
        });
        self.0 = CompletionExitState::MainJobCheckpoint(target);
    }

    pub(super) fn leave_main_job_checkpoint(&mut self, target: ControlTarget) {
        assert!(match &self.0 {
            CompletionExitState::MainJobCheckpoint(active) => *active == target,
            CompletionExitState::MainExport | CompletionExitState::MultiValue => false,
        });
        self.0 = CompletionExitState::MainExport;
    }
}
