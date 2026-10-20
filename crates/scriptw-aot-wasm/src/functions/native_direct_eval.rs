//! A prepared intrinsic-eval miss retains the actual invocation across the host.
use super::*;
use crate::gc_types::{
    DirectEvalExecutionContext, Environment, GcHostImport, GcLocal, GcOperand, ImmutableByteArray,
    NonNullable, Nullable, PrivateEnvironment, RealmRecord, StringValue,
};
use crate::prepared_script::PreparedScriptStatus;
use scriptw_ir::DirectEvalContextIr;

impl FunctionBuilder<'_> {
    pub(super) fn emit_native_direct_eval(
        &mut self,
        context: &DirectEvalContextIr,
        realm: &GcLocal<RealmRecord>,
        source: &GcLocal<StringValue>,
        output: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self
            .functions
            .gc_host_imports()
            .source_compilation_profile()
            == crate::RuntimeSourceCompilationProfile::Portable
        {
            self.emit_reject_dynamic_source(
                scriptw_ir::DynamicSourceRuntimeOperation::Eval,
                function,
            );
            return Ok(());
        }
        let schema = self.runtime_schema();
        let descriptor_bytes = context.runtime_descriptor().encode();
        let descriptor = schema
            .reserve_gc_local::<ImmutableByteArray, NonNullable>(function)
            .initialize(
                schema.array_type::<ImmutableByteArray>().fixed(
                    descriptor_bytes
                        .iter()
                        .map(|byte| GcOperand::i32(i32::from(*byte))),
                    function,
                ),
                function,
            );
        let variable = self.emit_eval_variable_environment_to_local(function);
        let lexical = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(self.current_environment().load(schema, function), function);
        let private = schema
            .reserve_gc_local::<PrivateEnvironment, Nullable>(function)
            .initialize(
                self.current_private_environment().load(schema, function),
                function,
            );
        let invocation = self.emit_capture_direct_eval_invocation(function)?;
        let required = schema
            .reserve_gc_local::<DirectEvalExecutionContext, NonNullable>(function)
            .initialize(
                invocation
                    .context()
                    .load(schema, function)
                    .require_non_null(function),
                function,
            );
        let status = schema.reserve_i32_local(function);
        let import = self
            .functions
            .gc_host_import(GcHostImport::CompileDirectEval)
            .ok_or_else(|| EmitError::unsupported("native direct eval import is not declared"))?;
        let syntax = import.compile_direct_eval(
            source,
            &descriptor,
            realm,
            &lexical,
            &variable,
            &private,
            invocation.this_value(),
            invocation.new_target(),
            &required,
            self.current_program(),
            output,
            status,
            schema,
            function,
        )?;
        syntax.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_require_native_script_status(status, PreparedScriptStatus::Entered, function);
        output.kind().load(function);
        function.instruction(&Instruction::I32Const(crate::COMPLETION_KIND_NORMAL as i32));
        function.instruction(&Instruction::I32Ne);
        output.kind().load(function);
        function.instruction(&Instruction::I32Const(crate::COMPLETION_KIND_THROW as i32));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I32And);
        output.target().load(function);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_require_native_script_status(status, PreparedScriptStatus::SyntaxError, function);
        let message = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(
                syntax.load(schema, function).require_non_null(function),
                function,
            );
        // This shared leaf constructs an Error in the passed Realm only; it
        // does not select the global Script compiler or enter a global path.
        self.emit_native_global_script_syntax_error(realm, &message, output, function)?;
        message.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        syntax.clear(function);
        schema.release_i32_local(status, function);
        required.clear(function);
        self.release_direct_eval_invocation(invocation, function);
        private.clear(function);
        lexical.clear(function);
        variable.clear(function);
        descriptor.clear(function);
        Ok(())
    }
}
