//! Each entry fixes its callback kind before emitting the shared Await algorithm.

use super::*;
use crate::runtime_helpers::{
    AsyncGeneratorAwaitReactionsArguments, AsyncGeneratorAwaitReactionsParameters,
    AsyncGeneratorAwaitReturnReactionsArguments, AsyncGeneratorAwaitReturnReactionsParameters,
    AsyncGeneratorYieldReactionsArguments, AsyncGeneratorYieldReactionsParameters,
    AsyncGeneratorYieldReturnReactionsArguments, AsyncGeneratorYieldReturnReactionsParameters,
};

// The typed declaration and static continuation are paired at each invocation.
// There is no caller-supplied numeric callback discriminator.
macro_rules! reaction_helper {
    ($emit:ident, $compile:ident, $id:ident, $arguments:ident, $parameters:ident, $continuation:ident) => {
        impl FunctionBuilder<'_> {
            pub(crate) fn $emit(
                &mut self,
                activation: &GcLocal<AsyncGeneratorActivation>,
                value: &ValueLocals,
                function: &mut Function,
            ) -> Result<(), EmitError> {
                let base = self.runtime_helper_base()?;
                self.runtime_schema()
                    .call_helper(
                        $arguments::new(activation, value, self.current_environment()),
                        base,
                        function,
                    )
                    .store(self.completion(), function);
                Ok(())
            }

            pub(crate) fn $compile(&mut self) -> Result<Function, EmitError> {
                let mut function = self.begin_helper_body(RuntimeHelperId::$id);
                let parameters = self.helper_parameters::<$parameters>(&mut function);
                self.completion().initialize(&mut function);
                self.emit_async_generator_await_with_continuation_inner(
                    &parameters.activation,
                    &parameters.value,
                    AsyncGeneratorAwaitContinuation::$continuation,
                    &mut function,
                )?;
                self.completion().emit(&mut function);
                parameters.release(&mut function);
                function.instruction(&Instruction::End);
                Ok(self.finish_function(function))
            }
        }
    };
}

reaction_helper!(
    emit_async_generator_await_reactions,
    compile_async_generator_await_reactions_helper,
    AsyncGeneratorAwaitReactions,
    AsyncGeneratorAwaitReactionsArguments,
    AsyncGeneratorAwaitReactionsParameters,
    Body
);
reaction_helper!(
    emit_async_generator_yield_reactions,
    compile_async_generator_yield_reactions_helper,
    AsyncGeneratorYieldReactions,
    AsyncGeneratorYieldReactionsArguments,
    AsyncGeneratorYieldReactionsParameters,
    Yield
);
reaction_helper!(
    emit_async_generator_yield_return_reactions,
    compile_async_generator_yield_return_reactions_helper,
    AsyncGeneratorYieldReturnReactions,
    AsyncGeneratorYieldReturnReactionsArguments,
    AsyncGeneratorYieldReturnReactionsParameters,
    YieldReturn
);

impl FunctionBuilder<'_> {
    pub(crate) fn emit_async_generator_await_return_reactions(
        &mut self,
        activation: &GcLocal<AsyncGeneratorActivation>,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let base = self.runtime_helper_base()?;
        self.runtime_schema()
            .call_helper(
                AsyncGeneratorAwaitReturnReactionsArguments::new(
                    activation,
                    value,
                    self.current_environment(),
                ),
                base,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_async_generator_await_return_reactions_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function =
            self.begin_helper_body(RuntimeHelperId::AsyncGeneratorAwaitReturnReactions);
        let parameters =
            self.helper_parameters::<AsyncGeneratorAwaitReturnReactionsParameters>(&mut function);
        self.completion().initialize(&mut function);
        let result = self.runtime_schema().reserve_completion(&mut function);
        result.initialize(&mut function);
        self.emit_async_generator_await_return_reactions_inner(
            &parameters.activation,
            &parameters.value,
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
