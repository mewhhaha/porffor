//! Typed operation calls and sole compiler entries for private object kernels.
use super::*;

impl FunctionBuilder<'_> {
    /// Whole Get preserves the accessor receiver and complete abrupt result.
    pub(crate) fn emit_object_read(
        &mut self,
        target: &ValueLocals,
        receiver: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dynamic_property_read_with_key_locals(target, receiver, key, result, function)
    }

    pub(crate) fn emit_object_read_with_throw_routing(
        &mut self,
        target: &ValueLocals,
        receiver: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        accessor_throw: AccessorThrowRouting,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dynamic_property_read_with_key_locals(target, receiver, key, result, function)?;
        match accessor_throw {
            AccessorThrowRouting::BreakToOrdinaryReadExit => {
                self.completion().copy_from(result, function);
                self.emit_propagate_current_throw_if_needed(function);
            }
            AccessorThrowRouting::LeaveInCompletion => {}
        }
        Ok(())
    }

    pub(crate) fn compile_object_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectRead);
        let parameters =
            self.helper_parameters::<crate::runtime_helpers::ObjectReadParameters>(&mut function);
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_read_kernel(
            &parameters.target,
            &parameters.receiver,
            &parameters.key,
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_object_get_prototype_of(
        &mut self,
        object: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::ObjectGetPrototypeOfArguments::new(
                    object,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_object_get_prototype_of_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectGetPrototypeOf);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::ObjectGetPrototypeOfParameters>(
                &mut function,
            );
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_get_prototype_of_kernel(&parameters.target, &result, &mut function)?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_object_set_prototype_of(
        &mut self,
        object: &ValueLocals,
        prototype: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::ObjectSetPrototypeOfArguments::new(
                    object,
                    prototype,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_object_set_prototype_of_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectSetPrototypeOf);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::ObjectSetPrototypeOfParameters>(
                &mut function,
            );
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_set_prototype_of_kernel(
            &parameters.target,
            &parameters.prototype,
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_object_delete(
        &mut self,
        object: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::ObjectDeleteArguments::new(
                    object,
                    key,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_object_delete_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectDelete);
        let parameters =
            self.helper_parameters::<crate::runtime_helpers::ObjectDeleteParameters>(&mut function);
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_delete_kernel(
            &parameters.target,
            &parameters.key,
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_object_is_extensible(
        &mut self,
        object: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::ObjectIsExtensibleArguments::new(
                    object,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_object_is_extensible_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectIsExtensible);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::ObjectIsExtensibleParameters>(
                &mut function,
            );
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_is_extensible_kernel(&parameters.target, &result, &mut function)?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_object_prevent_extensions(
        &mut self,
        object: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::ObjectPreventExtensionsArguments::new(
                    object,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(crate) fn compile_object_prevent_extensions_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectPreventExtensions);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::ObjectPreventExtensionsParameters>(
                &mut function,
            );
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_object_prevent_extensions_kernel(&parameters.target, &result, &mut function)?;
        result.emit(&mut function);
        result.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    // Preserve the registered Proxy-aware ABI without duplicating the Get
    // kernel. Both hops retain the original caller Environment and receiver.
    pub(crate) fn compile_object_read_proxy_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ObjectReadProxy);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::ObjectReadProxyParameters>(&mut function);
        let result = self.runtime_schema().reserve_completion(&mut function);
        self.emit_dynamic_property_read_with_key_locals(
            &parameters.target,
            &parameters.receiver,
            &parameters.key,
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
