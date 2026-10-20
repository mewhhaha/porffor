//! Only the witnessed direct-eval root receives the actual request records.
use super::*;
use crate::gc_types::{DirectEvalExecutionContext, Nullable};
use crate::runtime_helpers::{HelperParameters, NativeDirectEvalFactoryParameters};

impl FunctionBuilder<'_> {
    pub(crate) fn compile_native_direct_eval_factory(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::NativeDirectEvalFactory);
        // Ordinary P modules do not export this factory. Only the validated
        // NativeProgramFactory witness may publish an executable direct root.
        let root = self.functions.prepared_scripts().first().cloned();
        let Some(root) =
            root.filter(|root| matches!(&root.kind, PreparedScriptKind::DirectEval(_)))
        else {
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            return Ok(self.finish_function(function));
        };
        let PreparedScriptOutcome::Executable(unit) = root.outcome else {
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            return Ok(self.finish_function(function));
        };
        let parameters = self.helper_parameters::<NativeDirectEvalFactoryParameters>(&mut function);
        let entry = self
            .functions
            .get(&unit.id.function_id())
            .ok_or_else(|| {
                EmitError::unsupported("native direct eval factory root is not planned")
            })?
            .entry
            .clone();
        let schema = self.runtime_schema();
        let saved_realm = self.load_current_realm(&mut function);
        self.replace_current_realm(&parameters.realm, &mut function);
        let invocation = schema
            .reserve_gc_local::<DirectEvalExecutionContext, Nullable>(&mut function)
            .initialize(
                parameters.invocation.load(schema, &mut function).nullable(),
                &mut function,
            );
        let output = schema.reserve_completion(&mut function);
        output.store_call(
            entry.prepared_script()?.emit_call(
                crate::function_entry::PreparedScriptInputs::new(
                    &parameters.lexical,
                    &parameters.this_value,
                    &parameters.new_target,
                    &parameters.variable,
                    &parameters.private,
                    &invocation,
                    self.current_program(),
                ),
                schema,
                &mut function,
            ),
            &mut function,
        );
        self.replace_current_realm(&saved_realm, &mut function);
        saved_realm.clear(&mut function);
        invocation.clear(&mut function);
        self.completion().copy_from(&output, &mut function);
        output.clear(&mut function);
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::I32Const(2)); // Entered, including JS Throw.
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
