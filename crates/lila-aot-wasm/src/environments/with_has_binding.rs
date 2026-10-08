use super::*;
use crate::gc_types::SymbolValue;
use crate::operations::PropertyKeyLocals;
use crate::runtime_helpers::{
    HelperParameters, ObjectHasPropertyArguments, ObjectReadArguments,
    WithEnvironmentHasBindingArguments, WithEnvironmentHasBindingParameters,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_with_environment_has_binding(
        &mut self,
        object: &ValueLocals,
        name: &GcLocal<StringValue>,
        present: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                WithEnvironmentHasBindingArguments::new(object, name, self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        self.completion().value().scalar().load(function);
        function.instruction(&Instruction::I32WrapI64);
        present.store(function);
        Ok(())
    }

    pub(crate) fn compile_with_environment_has_binding_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::WithEnvironmentHasBinding);
        let parameters =
            self.helper_parameters::<WithEnvironmentHasBindingParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        self.replace_current_environment(
            parameters.caller_environment.load(schema, &mut function),
            &mut function,
        );
        self.completion().initialize(&mut function);
        let present = schema.reserve_i32_local(&mut function);
        self.emit_with_environment_has_binding_inline(
            &parameters.target,
            &parameters.name,
            present,
            &mut function,
        )?;
        let result = schema.reserve_value_local(&mut function);
        result.set_boolean(present, &mut function);
        self.completion().commit_if_normal(&result, &mut function);
        result.clear(&mut function);
        schema.release_i32_local(present, &mut function);
        self.pop_scope();
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_with_environment_has_binding_inline(
        &mut self,
        object: &ValueLocals,
        name: &GcLocal<StringValue>,
        present: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let base = self.runtime_helper_base()?;
        let name_key = PropertyKeyLocals::from_string(schema, name, function);
        schema
            .call_helper(
                ObjectHasPropertyArguments::new(object, &name_key, self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        self.completion().value().scalar().load(function);
        function.instruction(&Instruction::I32WrapI64);
        present.store(function);
        present.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let symbol = schema
            .reserve_gc_local::<SymbolValue, NonNullable>(function)
            .initialize(
                self.emit_well_known_symbol_reference(
                    lila_ir::WellKnownSymbol::Unscopables,
                    function,
                )?,
                function,
            );
        let key = PropertyKeyLocals::from_symbol(schema, &symbol, function);
        schema
            .call_helper(
                ObjectReadArguments::new(object, object, &key, self.current_environment()),
                base,
                function,
            )
            .store(self.completion(), function);
        key.clear(function);
        symbol.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        let exclusions = schema.reserve_value_local(function);
        exclusions.copy_from(self.completion().value(), function);
        self.emit_is_heap_object_like_tag_i32(exclusions.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .call_helper(
                ObjectReadArguments::new(
                    &exclusions,
                    &exclusions,
                    &name_key,
                    self.current_environment(),
                ),
                base,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        self.compile_truthy_tagged_i32(self.completion().value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        present.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        exclusions.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        name_key.clear(function);
        Ok(())
    }
}
