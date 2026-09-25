use super::resumable_sync_for_of_iterator::{ResumableSyncForOfOwner, ResumableSyncForOfView};
use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_async_function_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &AsyncFunctionForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_sync_for_of_iterator(
            iterable,
            ResumableSyncForOfView {
                value_storage: plan.value_storage().clone(),
                value_mode: plan.value_mode(),
                record: plan.record(),
                head_environment: plan.head_environment(),
                iteration_environment: plan.iteration_environment(),
                body: plan.body().statements(),
                entry_state: plan.entry_state(),
                body_exit_state: plan.body().exit_state(),
                exit_state: plan.exit_state(),
            },
            ResumableSyncForOfOwner::AsyncFunction,
            function,
        )
    }

    pub(crate) fn compile_generator_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &GeneratorForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_storage = match plan.iteration_environment() {
            ResumableLoopIterationEnvironmentIr::StorageOnly => {
                AsyncFunctionForOfIteratorValueStorageIr::Activation(plan.head().clone())
            }
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(_) => {
                AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(plan.head().clone())
            }
        };
        self.compile_resumable_sync_for_of_iterator(
            iterable,
            ResumableSyncForOfView {
                value_storage,
                value_mode: plan.head().mode,
                record: plan.record(),
                head_environment: plan.head_environment(),
                iteration_environment: plan.iteration_environment(),
                body: plan.body(),
                entry_state: plan.entry_state(),
                body_exit_state: plan.body_exit_state(),
                exit_state: plan.exit_state(),
            },
            ResumableSyncForOfOwner::Generator,
            function,
        )
    }
}
