//! Only an actual global prepared-hook miss reaches native host compilation.
use super::*;
use crate::gc_types::{GcHostImport, NativeErrorObject, NonNullable};
use crate::module::ThrowDiagnosticRole;
use crate::runtime_helpers::RuntimeErrorObjectArguments;

impl FunctionBuilder<'_> {
    pub(super) fn emit_native_global_script_compile(
        &mut self,
        kind: I32Local,
        source: &GcLocal<StringValue>,
        realm: &GcLocal<RealmRecord>,
        output: &CompletionLocals,
        status: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let import = self
            .functions
            .gc_host_import(GcHostImport::CompileGlobalScript)
            .ok_or_else(|| {
                EmitError::unsupported("native global Script compiler import is not declared")
            })?;
        let syntax = import.compile_global_script(
            kind,
            source,
            realm,
            self.current_program(),
            output,
            status,
            schema,
            function,
        )?;
        syntax.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        // The native factory must have entered the witnessed root, including
        // when that Script threw. Unmatched is a compiler/host invariant error.
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
        self.emit_native_global_script_syntax_error(realm, &message, output, function)?;
        message.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        syntax.clear(function);
        Ok(())
    }

    pub(crate) fn emit_require_native_script_status(
        &mut self,
        actual: I32Local,
        expected: PreparedScriptStatus,
        function: &mut Function,
    ) {
        actual.load(function);
        function.instruction(&Instruction::I32Const(expected.code()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Use the passed target Realm directly; neither active Realm nor current P
    /// is changed to construct this genuine parser SyntaxError completion.
    pub(crate) fn emit_native_global_script_syntax_error(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        message: &GcLocal<StringValue>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::SyntaxErrorPrototype,
            &prototype,
            function,
        );
        let error = schema
            .call_helper(
                RuntimeErrorObjectArguments::new(&prototype, message),
                self.runtime_helper_base()?,
                function,
            )
            .bind(
                schema,
                schema.reserve_gc_local::<NativeErrorObject, NonNullable>(function),
                function,
            );
        let value = schema.reserve_value_local(function);
        value.set_reference(&error, schema, function);
        output.set_throw(&value, function);
        self.emit_clear_throw_diagnostic(ThrowDiagnosticRole::Name, function);
        self.emit_clear_throw_diagnostic(ThrowDiagnosticRole::Message, function);
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("SyntaxError", function)?,
            function,
        );
        self.emit_store_throw_diagnostic(ThrowDiagnosticRole::Name, &name, function);
        self.emit_store_throw_diagnostic(ThrowDiagnosticRole::Message, message, function);
        name.clear(function);
        value.clear(function);
        error.clear(function);
        prototype.clear(function);
        Ok(())
    }
}
