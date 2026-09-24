use super::*;

/// The function kinds whose call evaluates `EvaluateGeneratorBody` or
/// `EvaluateAsyncGeneratorBody`.
///
/// Both allocate the instance with `OrdinaryCreateFromConstructor(functionObject,
/// intrinsicDefaultProto)` only after `FunctionDeclarationInstantiation` has
/// completed normally. The exhaustive projection pairs each family with its own
/// realm intrinsic, so a generator can never fall back to the async-generator
/// prototype or to an entry-realm global.
#[derive(Clone, Copy)]
pub(super) enum GeneratorInstanceFamily {
    Generator,
    AsyncGenerator,
}

impl GeneratorInstanceFamily {
    const fn default_prototype(self) -> OrdinaryDefaultPrototype {
        match self {
            Self::Generator => OrdinaryDefaultPrototype::Generator,
            Self::AsyncGenerator => OrdinaryDefaultPrototype::AsyncGenerator,
        }
    }
}

impl FunctionBuilder<'_> {
    /// Install `GetPrototypeFromConstructor(functionObject, default)` as the
    /// `[[Prototype]]` of an already allocated generator instance.
    ///
    /// Callers emit this after parameter initialization returned normally,
    /// which is when the specification reads `functionObject.prototype`. A
    /// parameter initializer that replaces the property therefore selects the
    /// replacement, an abrupt initializer never performs the read, and a
    /// non-Object value selects the intrinsic of the generator function's own
    /// realm. The value comes from an observable `[[Get]]`, never from the
    /// function header's creation-time prototype snapshot.
    pub(super) fn emit_install_generator_instance_prototype(
        &mut self,
        family: GeneratorInstanceFamily,
        function_object_payload_local: u32,
        function_object_tag_local: u32,
        instance_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            function_object_payload_local,
            function_object_tag_local,
            function_object_payload_local,
            function_object_tag_local,
            key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(prototype_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // The generator function is the constructor argument of
        // GetPrototypeFromConstructor, so its realm owns the fallback.
        self.emit_required_new_target_realm_ordinary_prototype(
            function_object_payload_local,
            function_object_tag_local,
            family.default_prototype(),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            instance_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            instance_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );

        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
