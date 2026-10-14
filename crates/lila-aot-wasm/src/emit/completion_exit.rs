use super::*;

/// The one exit policy owned by an emitted body.
///
/// `ReturnAbi` describes the public Wasm signature. This state additionally
/// records the host-checkpoint target while main-source statements are emitted
/// and the source epilogue while an admitted callable body is emitted. Its
/// private representation prevents an internal function from acquiring a main
/// checkpoint or a caller from manufacturing an exit target without the
/// checked transition methods below.
pub(crate) struct CompletionExit(CompletionExitState);

enum CompletionExitState {
    MainExport,
    MainJobCheckpoint(ControlTarget),
    MultiValue,
    SourceActivationEpilogue(ControlTarget),
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
            CompletionExitState::MultiValue | CompletionExitState::SourceActivationEpilogue(_) => {
                ReturnAbi::MultiValue
            }
        }
    }

    pub(crate) const fn main_job_checkpoint_target(&self) -> Option<ControlTarget> {
        match &self.0 {
            CompletionExitState::MainJobCheckpoint(target) => Some(*target),
            CompletionExitState::MainExport
            | CompletionExitState::MultiValue
            | CompletionExitState::SourceActivationEpilogue(_) => None,
        }
    }

    pub(super) fn enter_main_job_checkpoint(&mut self, target: ControlTarget) {
        assert!(match &self.0 {
            CompletionExitState::MainExport => true,
            CompletionExitState::MainJobCheckpoint(_)
            | CompletionExitState::MultiValue
            | CompletionExitState::SourceActivationEpilogue(_) => false,
        });
        self.0 = CompletionExitState::MainJobCheckpoint(target);
    }

    pub(super) fn leave_main_job_checkpoint(&mut self, target: ControlTarget) {
        assert!(match &self.0 {
            CompletionExitState::MainJobCheckpoint(active) => *active == target,
            CompletionExitState::MainExport
            | CompletionExitState::MultiValue
            | CompletionExitState::SourceActivationEpilogue(_) => false,
        });
        self.0 = CompletionExitState::MainExport;
    }

    /// Only a successfully admitted source owner can install this epilogue.
    /// Native/helper bodies cannot manufacture a SourceActivation and retain
    /// their direct completion-return ABI.
    pub(super) fn enter_source_activation_epilogue(
        &mut self,
        target: ControlTarget,
        _owner: &crate::functions::SourceActivation,
    ) {
        assert!(matches!(self.0, CompletionExitState::MultiValue));
        self.0 = CompletionExitState::SourceActivationEpilogue(target);
    }

    pub(crate) const fn source_activation_epilogue_target(&self) -> Option<ControlTarget> {
        match &self.0 {
            CompletionExitState::SourceActivationEpilogue(target) => Some(*target),
            CompletionExitState::MainExport
            | CompletionExitState::MainJobCheckpoint(_)
            | CompletionExitState::MultiValue => None,
        }
    }

    pub(super) fn leave_source_activation_epilogue(&mut self, target: ControlTarget) {
        assert!(match &self.0 {
            CompletionExitState::SourceActivationEpilogue(active) => *active == target,
            CompletionExitState::MainExport
            | CompletionExitState::MainJobCheckpoint(_)
            | CompletionExitState::MultiValue => false,
        });
        self.0 = CompletionExitState::MultiValue;
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn open_source_activation_epilogue(
        &mut self,
        function: &mut Function,
    ) -> Option<ControlTarget> {
        // The source-depth guard has already admitted this actual owner.
        if self.source_activation.is_none() {
            return None;
        }
        let target = self.open_frame(ControlFrameKind::Block, function);
        self.completion_exit.enter_source_activation_epilogue(
            target,
            self.source_activation
                .as_ref()
                .expect("admitted source owner"),
        );
        Some(target)
    }

    pub(super) fn finish_source_activation_epilogue(
        &mut self,
        target: Option<ControlTarget>,
        function: &mut Function,
    ) {
        if let Some(target) = target {
            // Return, Throw and suspension retain their whole completion and
            // branch past fallthrough normalization to this one restoration.
            self.completion_exit
                .leave_source_activation_epilogue(target);
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
        }
        self.emit_leave_source_activation(function);
    }
}
