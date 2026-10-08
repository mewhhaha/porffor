//! Dynamic source allocation completes captures before publishing the function.

use super::*;
use crate::gc_types::FunctionObject;

impl FunctionBuilder<'_> {
    pub(super) fn compile_function_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mut empty = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototype.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing planned Function.prototype empty body")
            })?;
        empty.name = "anonymous".to_string();
        empty.to_string_value = "function anonymous(\n) {\n\n}".to_string();
        empty.entry = empty.entry.empty_ordinary_function()?;
        empty.strict = false;
        empty.standard_builtin = None;
        empty.host_builtin = None;
        empty.is_named_expression = false;
        self.emit_dynamic_constructor_body(DynamicFunctionKind::Ordinary, &empty, function)
    }

    /// The constructor's `newTarget`, defaulting to the active function.
    pub(super) fn emit_dynamic_constructor_new_target(
        &mut self,
        new_target: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let callable = self
            .body_entry_locals()
            .and_then(|entry| entry.function_object())
            .ok_or_else(|| {
                EmitError::unsupported("dynamic constructor requires its actual entry callable")
            })?;
        let active = schema.reserve_value_local(function);
        active.set_reference::<FunctionObject>(callable, schema, function);
        self.compile_new_target_to_locals(new_target, function)?;
        new_target.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        new_target.copy_from(&active, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        active.clear(function);
        Ok(())
    }

    /// Allocates the dynamic function for the already defaulted `newTarget`.
    pub(super) fn emit_dynamic_function_allocation(
        &mut self,
        kind: DynamicFunctionKind,
        body: &WasmFunctionMeta,
        new_target: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_completion(function);
        prototype.initialize(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        self.emit_get_prototype_from_constructor(
            new_target,
            crate::functions::OrdinaryDefaultPrototype::Function(kind),
            &prototype,
            function,
        )?;
        self.completion().copy_from(&prototype, function);
        prototype.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let created = schema.reserve_gc_local(function).initialize(
            self.emit_dynamic_source_function_record(body, &realm, prototype.value(), function)?,
            function,
        );
        let result = schema.reserve_value_local(function);
        result.set_reference(&created, schema, function);
        self.completion().set_normal(&result, function);
        result.clear(function);
        created.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        prototype.clear(function);
        realm.clear(function);
        Ok(())
    }
}
