//! Normal recursive [[GetOwnProperty]] acquisition through its real builtin
//! algorithm, followed by nonobserving reads of the private fresh descriptor.

use super::*;
use crate::gc_types::{I32Local, RuntimeSchema};

#[must_use = "a completed descriptor root must be consumed and cleared"]
pub(crate) struct CompletedProxyTargetDescriptor {
    descriptor: GcLocal<PropertyDescriptor, Nullable>,
}

impl CompletedProxyTargetDescriptor {
    pub(crate) fn emit_found_i32(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.descriptor.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
    }

    fn emit_flag_i32(&self, mask: DescriptorMask, schema: &RuntimeSchema, function: &mut Function) {
        self.emit_found_i32(schema, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let flags = schema.reserve_i64_local(function);
        schema
            .struct_type::<PropertyDescriptor>()
            .field(PropertyDescriptorSchema::FLAGS)
            .read(&self.descriptor, schema, function)
            .store_i64(flags, function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(mask.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        schema.release_i64_local(flags, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_accessor_i32(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.emit_flag_i32(DescriptorMask::ACCESSOR, schema, function);
    }
    pub(crate) fn emit_writable_i32(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.emit_flag_i32(DescriptorMask::WRITABLE, schema, function);
    }
    pub(crate) fn emit_enumerable_i32(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.emit_flag_i32(DescriptorMask::ENUMERABLE, schema, function);
    }
    pub(crate) fn emit_configurable_i32(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.emit_flag_i32(DescriptorMask::CONFIGURABLE, schema, function);
    }

    fn read_field(
        &self,
        field: crate::gc_types::GcField<
            PropertyDescriptor,
            crate::gc_types::GcRef<StoredValue>,
            crate::gc_types::Mutable,
            crate::gc_types::NonNullable,
        >,
        result: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        result.set_undefined(function);
        self.emit_found_i32(schema, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        let stored_slot = schema.reserve_gc_local(function);
        let stored = stored_slot.initialize(
            schema
                .struct_type::<PropertyDescriptor>()
                .field(field)
                .read(&self.descriptor, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, result, schema, function);
        stored.clear(function);
        function.instruction(&Instruction::End);
    }
    pub(crate) fn read_value(
        &self,
        out: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.read_field(PropertyDescriptorSchema::VALUE, out, schema, function);
    }
    pub(crate) fn read_getter(
        &self,
        out: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.read_field(PropertyDescriptorSchema::GETTER, out, schema, function);
    }
    pub(crate) fn read_setter(
        &self,
        out: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.read_field(PropertyDescriptorSchema::SETTER, out, schema, function);
    }
    /// IsCompatiblePropertyDescriptor uses the same stored-descriptor kernel
    /// as DefineOwnProperty, after this proof has completed target acquisition.
    pub(crate) fn emit_compatibility(
        &self,
        incoming: &WasmDescriptor<'_>,
        extensible: I32Local,
        valid: I32Local,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        self.emit_found_i32(schema, function);
        builder.open_frame(ControlFrameKind::If, function);
        let descriptor = schema.reserve_gc_local(function).initialize(
            self.descriptor
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        builder.emit_validate_stored_descriptor(&descriptor, incoming, valid, function)?;
        descriptor.clear(function);
        function.instruction(&Instruction::Else);
        extensible.load(function);
        valid.store(function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn into_descriptor(self) -> GcLocal<PropertyDescriptor, Nullable> {
        self.descriptor
    }
    pub(crate) fn clear(self, function: &mut Function) {
        self.descriptor.clear(function);
    }
}

impl FunctionBuilder<'_> {
    /// The real GPD builtin owns trap/exotic acquisition and descriptor
    /// validation. Its private fresh complete result has only own data fields.
    pub(crate) fn emit_proxy_target_own_descriptor(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        function: &mut Function,
    ) -> Result<CompletedProxyTargetDescriptor, EmitError> {
        let schema = self.runtime_schema();
        let descriptor = schema
            .reserve_gc_local::<PropertyDescriptor, Nullable>(function)
            .initialize_null(schema, function);
        let result = schema.reserve_completion(function);
        self.emit_native_object_algorithm_call(
            crate::functions::NativeObjectAlgorithm::GetOwnPropertyDescriptor,
            &[target, key.value()],
            &result,
            function,
        )?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().copy_from(&result, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.value().tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        let value = schema.reserve_value_local(function);
        let getter = schema.reserve_value_local(function);
        let setter = schema.reserve_value_local(function);
        let writable = schema.reserve_value_local(function);
        let enumerable = schema.reserve_value_local(function);
        let configurable = schema.reserve_value_local(function);
        let accessor = schema.reserve_i32_local(function);
        let present = schema.reserve_i32_local(function);
        let writable_boolean = schema.reserve_i32_local(function);
        let enumerable_boolean = schema.reserve_i32_local(function);
        let configurable_boolean = schema.reserve_i32_local(function);
        for (field, output) in [
            (DescriptorField::Value, &value),
            (DescriptorField::Writable, &writable),
            (DescriptorField::Get, &getter),
            (DescriptorField::Set, &setter),
            (DescriptorField::Enumerable, &enumerable),
            (DescriptorField::Configurable, &configurable),
        ] {
            let name_slot = schema.reserve_gc_local(function);
            let name = name_slot.initialize(
                self.emit_interned_string_reference(field.key(), function)?,
                function,
            );
            let field_key = PropertyKeyLocals::from_string(schema, &name, function);
            name.clear(function);
            self.emit_object_own_data_field_read(
                result.value(),
                &field_key,
                present,
                output,
                function,
            )?;
            if field == DescriptorField::Get {
                present.load(function);
                accessor.store(function);
            }
            field_key.clear(function);
        }
        self.compile_truthy_tagged_i32(&writable, function)?;
        writable_boolean.store(function);
        self.compile_truthy_tagged_i32(&enumerable, function)?;
        enumerable_boolean.store(function);
        self.compile_truthy_tagged_i32(&configurable, function)?;
        configurable_boolean.store(function);
        let value_slot = schema.reserve_gc_local(function);
        let stored_value = value_slot.initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&value, function),
            function,
        );
        let getter_slot = schema.reserve_gc_local(function);
        let stored_getter = getter_slot.initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&getter, function),
            function,
        );
        let setter_slot = schema.reserve_gc_local(function);
        let stored_setter = setter_slot.initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&setter, function),
            function,
        );
        accessor.load(function);
        self.open_frame(ControlFrameKind::If, function);
        descriptor.replace(
            schema
                .struct_type::<PropertyDescriptor>()
                .construct(
                    (
                        GcOperand::accessor_descriptor_flags(
                            enumerable_boolean,
                            configurable_boolean,
                        ),
                        GcOperand::reference(&stored_value, schema),
                        GcOperand::reference(&stored_getter, schema),
                        GcOperand::reference(&stored_setter, schema),
                    ),
                    function,
                )
                .nullable(),
            function,
        );
        function.instruction(&Instruction::Else);
        descriptor.replace(
            schema
                .struct_type::<PropertyDescriptor>()
                .construct(
                    (
                        GcOperand::data_descriptor_flags(
                            writable_boolean,
                            enumerable_boolean,
                            configurable_boolean,
                        ),
                        GcOperand::reference(&stored_value, schema),
                        GcOperand::reference(&stored_getter, schema),
                        GcOperand::reference(&stored_setter, schema),
                    ),
                    function,
                )
                .nullable(),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        stored_setter.clear(function);
        stored_getter.clear(function);
        stored_value.clear(function);
        schema.release_i32_local(configurable_boolean, function);
        schema.release_i32_local(enumerable_boolean, function);
        schema.release_i32_local(writable_boolean, function);
        schema.release_i32_local(present, function);
        schema.release_i32_local(accessor, function);
        configurable.clear(function);
        enumerable.clear(function);
        writable.clear(function);
        setter.clear(function);
        getter.clear(function);
        value.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.clear(function);
        Ok(CompletedProxyTargetDescriptor { descriptor })
    }

    pub(super) fn emit_direct_own_descriptor_for_proxy_get(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        function: &mut Function,
    ) -> Result<CompletedProxyTargetDescriptor, EmitError> {
        self.emit_proxy_target_own_descriptor(target, key, function)
    }
    pub(super) fn emit_direct_own_descriptor_for_proxy_set(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        function: &mut Function,
    ) -> Result<CompletedProxyTargetDescriptor, EmitError> {
        self.emit_proxy_target_own_descriptor(target, key, function)
    }
    pub(super) fn emit_proxy_target_descriptor_fact(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        function: &mut Function,
    ) -> Result<CompletedProxyTargetDescriptor, EmitError> {
        self.emit_proxy_target_own_descriptor(target, key, function)
    }
}
