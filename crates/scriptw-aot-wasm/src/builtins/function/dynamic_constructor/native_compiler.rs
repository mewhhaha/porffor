//! The one native miss after ordered constructor ToString and prepared dispatch.
use super::*;
use crate::gc_types::{
    CompletionLocals, GcHostImport, GcLocal, I32Local, NativeErrorObject, NonNullable, StringValue,
};
use crate::module::ThrowDiagnosticRole;
use crate::runtime_helpers::RuntimeErrorObjectArguments;

impl FunctionBuilder<'_> {
    pub(super) fn emit_native_dynamic_function_compile(
        &mut self,
        kind: I32Local,
        new_target: &ValueLocals,
        sources: &GcLocal<crate::gc_types::ValueArray>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let import = self
            .functions
            .gc_host_import(GcHostImport::CompileDynamicFunction)
            .ok_or_else(|| {
                EmitError::unsupported("native Function compiler import is not declared")
            })?;
        let active_constructor = self.current_function_context().ok_or_else(|| {
            EmitError::unsupported(
                "native Function compiler requires its active constructor context",
            )
        })?;
        let syntax = import.compile_dynamic_function(
            kind,
            new_target,
            sources,
            active_constructor,
            self.current_environment(),
            output,
            schema,
            function,
        )?;
        syntax.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        // A native factory completion retains all five original words. Host,
        // capability and resource errors never become a fake SyntaxError.
        self.completion().copy_from(output, function);
        function.instruction(&Instruction::Else);
        let message = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(
                syntax.load(schema, function).require_non_null(function),
                function,
            );
        self.emit_native_function_syntax_error(&message, function)?;
        message.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        syntax.clear(function);
        Ok(())
    }

    /// Only the typed host syntax result reaches here. Use the active
    /// constructor's actual Realm and the original GC String message directly.
    fn emit_native_function_syntax_error(
        &mut self,
        message: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            crate::functions::NonArrayRealmIntrinsicSlot::SyntaxErrorPrototype,
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
        self.completion().set_throw(&value, function);
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
        realm.clear(function);
        Ok(())
    }
}
