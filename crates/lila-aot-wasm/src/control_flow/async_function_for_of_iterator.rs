use super::resumable_sync_for_of_iterator::ResumableSyncForOfPlan;
use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_async_function_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &AsyncFunctionForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match plan.execution() {
            lila_ir::AsyncFunctionForOfIteratorExecutionIr::Synchronous(plan) => self
                .compile_resumable_sync_for_of_iterator(
                    iterable,
                    ResumableSyncForOfPlan::Async(plan),
                    function,
                ),
            lila_ir::AsyncFunctionForOfIteratorExecutionIr::Awaited(plan) => self
                .compile_async_for_of_iterator(
                    iterable,
                    ForAwaitIteratorPlan::awaited(plan),
                    &[],
                    function,
                ),
        }
    }

    pub(crate) fn compile_generator_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &lila_ir::GeneratorForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_sync_for_of_iterator(
            iterable,
            ResumableSyncForOfPlan::Generator(plan),
            function,
        )
    }
}
