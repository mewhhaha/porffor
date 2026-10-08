//! Scheduling crosses registered calls while the original bodies own queues,
//! whole completions, diagnostic roots and the active execution Realm.

mod async_generator_reactions;

use super::*;
use crate::runtime_helpers::{
    AsyncAwaitReactionsArguments, AsyncAwaitReactionsParameters, AsyncGeneratorDrainQueueArguments,
    AsyncGeneratorDrainQueueParameters, HelperParameters, PromiseDrainJobsArguments,
    PromiseDrainJobsParameters, RuntimeHelperId,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_async_await_reactions(
        &mut self,
        activation: &GcLocal<AsyncActivation>,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let base = self.runtime_helper_base()?;
        self.runtime_schema()
            .call_helper(
                AsyncAwaitReactionsArguments::new(activation, value, self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        Ok(())
    }

    pub(crate) fn compile_async_await_reactions_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::AsyncAwaitReactions);
        let parameters = self.helper_parameters::<AsyncAwaitReactionsParameters>(&mut function);
        self.completion().initialize(&mut function);
        self.emit_async_await_reactions_inner(
            &parameters.activation,
            &parameters.value,
            &mut function,
        )?;
        self.completion().emit(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_drain_async_generator_queue(
        &mut self,
        activation: &GcLocal<AsyncGeneratorActivation>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let base = self.runtime_helper_base()?;
        self.runtime_schema()
            .call_helper(
                AsyncGeneratorDrainQueueArguments::new(activation, self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        Ok(())
    }

    pub(crate) fn compile_async_generator_drain_queue_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::AsyncGeneratorDrainQueue);
        let parameters =
            self.helper_parameters::<AsyncGeneratorDrainQueueParameters>(&mut function);
        self.completion().initialize(&mut function);
        self.emit_drain_async_generator_queue_inner(&parameters.activation, &mut function)?;
        self.completion().emit(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_drain_promise_jobs(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let base = self.runtime_helper_base()?;
        self.runtime_schema()
            .call_helper(
                PromiseDrainJobsArguments::new(self.completion(), self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        Ok(())
    }

    pub(crate) fn compile_promise_drain_jobs_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::PromiseDrainJobs);
        let parameters = self.helper_parameters::<PromiseDrainJobsParameters>(&mut function);
        self.completion()
            .copy_from(&parameters.pending, &mut function);
        self.emit_drain_promise_jobs_inner(&mut function)?;
        self.completion().emit(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
