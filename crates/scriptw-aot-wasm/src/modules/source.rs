//! A host source object belongs to one actual record in its defining Realm.

use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_source_namespaces(
        &mut self,
        graph: &scriptw_ir::ModuleExecutionGraphIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        for source in graph.source_records() {
            let record = self.module_record_local(source.module(), function)?;
            for mode in [
                scriptw_ir::ModuleNamespaceModeIr::Eager,
                scriptw_ir::ModuleNamespaceModeIr::Deferred,
            ] {
                let namespace = schema.reserve_value_local(function);
                self.emit_empty_module_namespace(mode, &namespace, function)?;
                let cell = self.module_namespace_cell(&record, mode, function);
                self.emit_initialize_environment_cell(&cell, &namespace, function);
                cell.clear(function);
                namespace.clear(function);
            }
            record.clear(function);
        }
        Ok(())
    }

    pub(super) fn module_source_cell(
        &mut self,
        record: &GcLocal<ModuleRecord>,
        module: u32,
        function: &mut Function,
    ) -> Result<GcLocal<BindingCell>, EmitError> {
        let source_tag = self
            .functions
            .module_graph()
            .and_then(|graph| graph.source_record(module))
            .ok_or_else(|| {
                EmitError::unsupported(
                    "a module source cell requires its actual host source record",
                )
            })?
            .source_tag()
            .to_owned();
        let schema = self.runtime_schema();
        let cell = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::SOURCE_CELL)
                .read(record, schema, function)
                .reference(),
            function,
        );
        let initialized = schema.reserve_i32_local(function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::INITIALIZED)
            .read(&cell, schema, function)
            .store(initialized, function);
        initialized.load(function);
        function.instruction(&Instruction::I32Eqz);
        schema.release_i32_local(initialized, function);
        self.open_frame(ControlFrameKind::If, function);
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::REALM)
                .read(record, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ModuleSourcePrototype,
            &prototype,
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        let tag = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(&source_tag, function)?,
            function,
        );
        let object = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ModuleSourceObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(record, schema),
                    GcOperand::reference(&realm, schema),
                    GcOperand::reference(&tag, schema),
                ),
                function,
            ),
            function,
        );
        let value = schema.reserve_value_local(function);
        value.set_reference(&object, schema, function);
        self.emit_initialize_environment_cell(&cell, &value, function);
        value.clear(function);
        object.clear(function);
        tag.clear(function);
        header.clear(function);
        prototype.clear(function);
        realm.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(cell)
    }

    pub(crate) fn emit_module_source_to_string_tag_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        self.completion().initialize(function);
        // HostGetModuleSourceModuleRecord tests the private GC brand. A proxy,
        // a forged prototype, the prototype itself and primitives are not sources.
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ModuleSourceObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let source = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<ModuleSourceObject>(schema, function),
            function,
        );
        let tag = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleSourceObject>()
                .field(ModuleSourceObjectSchema::SOURCE_TAG)
                .read(&source, schema, function)
                .reference(),
            function,
        );
        self.completion()
            .value()
            .set_reference(&tag, schema, function);
        tag.clear(function);
        source.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        receiver.clear(function);
        Ok(())
    }
}
