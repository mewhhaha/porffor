//! ObjectDefineProperties retains partial descriptors, then applies in order.
use super::*;
use crate::gc_types::{
    CompletionLocals, GcLocal, GcOperand, I32Local, PartialPropertyDescriptor,
    PartialPropertyDescriptorSchema, PropertyDefinition, PropertyDefinitionSchema,
    PropertyDefinitionTable, StoredValue, ValueLocals,
};
use crate::objects::ReservedPropertyDescriptorLocals;
use lila_ir::property_descriptor::DescriptorField;

/// Only the native object check or a fresh native allocation creates a target.
pub(super) struct ObjectDefinitionTarget<'v>(pub(super) &'v ValueLocals);

#[must_use]
struct RetainedPartialDescriptor(GcLocal<PartialPropertyDescriptor>);

impl RetainedPartialDescriptor {
    fn save(
        converted: ReservedPropertyDescriptorLocals,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Self {
        let schema = builder.runtime_schema();
        let value = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(converted.field_locals(DescriptorField::Value).1, function),
            function,
        );
        let getter = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(converted.field_locals(DescriptorField::Get).1, function),
            function,
        );
        let setter = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(converted.field_locals(DescriptorField::Set).1, function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<PartialPropertyDescriptor>().construct(
                (
                    GcOperand::boolean_local(converted.field_locals(DescriptorField::Value).0),
                    GcOperand::boolean_local(converted.field_locals(DescriptorField::Writable).0),
                    GcOperand::boolean_local(converted.field_locals(DescriptorField::Get).0),
                    GcOperand::boolean_local(converted.field_locals(DescriptorField::Set).0),
                    GcOperand::boolean_local(converted.field_locals(DescriptorField::Enumerable).0),
                    GcOperand::boolean_local(
                        converted.field_locals(DescriptorField::Configurable).0,
                    ),
                    GcOperand::reference(&value, schema),
                    GcOperand::reference(&getter, schema),
                    GcOperand::reference(&setter, schema),
                    GcOperand::boolean_local(converted.writable_flag()),
                    GcOperand::boolean_local(converted.enumerable_flag()),
                    GcOperand::boolean_local(converted.configurable_flag()),
                ),
                function,
            ),
            function,
        );
        setter.clear(function);
        getter.clear(function);
        value.clear(function);
        converted.clear(schema, function);
        Self(record)
    }

    fn apply(
        self,
        target: &ObjectDefinitionTarget<'_>,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        let present: [I32Local; 6] = core::array::from_fn(|_| schema.reserve_i32_local(function));
        for (field, local) in [
            (PartialPropertyDescriptorSchema::VALUE_PRESENT, present[0]),
            (
                PartialPropertyDescriptorSchema::WRITABLE_PRESENT,
                present[1],
            ),
            (PartialPropertyDescriptorSchema::GETTER_PRESENT, present[2]),
            (PartialPropertyDescriptorSchema::SETTER_PRESENT, present[3]),
            (
                PartialPropertyDescriptorSchema::ENUMERABLE_PRESENT,
                present[4],
            ),
            (
                PartialPropertyDescriptorSchema::CONFIGURABLE_PRESENT,
                present[5],
            ),
        ] {
            schema
                .struct_type::<PartialPropertyDescriptor>()
                .field(field)
                .read(&self.0, schema, function)
                .store(local, function);
        }
        let writable = schema.reserve_i32_local(function);
        let enumerable = schema.reserve_i32_local(function);
        let configurable = schema.reserve_i32_local(function);
        for (field, local) in [
            (PartialPropertyDescriptorSchema::WRITABLE, writable),
            (PartialPropertyDescriptorSchema::ENUMERABLE, enumerable),
            (PartialPropertyDescriptorSchema::CONFIGURABLE, configurable),
        ] {
            schema
                .struct_type::<PartialPropertyDescriptor>()
                .field(field)
                .read(&self.0, schema, function)
                .store(local, function);
        }
        let value = schema.reserve_value_local(function);
        let getter = schema.reserve_value_local(function);
        let setter = schema.reserve_value_local(function);
        for (field, local) in [
            (PartialPropertyDescriptorSchema::VALUE, &value),
            (PartialPropertyDescriptorSchema::GETTER, &getter),
            (PartialPropertyDescriptorSchema::SETTER, &setter),
        ] {
            let stored = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<PartialPropertyDescriptor>()
                    .field(field)
                    .read(&self.0, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&stored, local, schema, function);
            stored.clear(function);
        }
        let fields = DescriptorObjectFields {
            value: Presence::Runtime {
                present: present[0],
                value: &value,
            },
            writable: Presence::Runtime {
                present: present[1],
                value: DescriptorFlag::BooleanPayload(writable),
            },
            get: Presence::Runtime {
                present: present[2],
                value: &getter,
            },
            set: Presence::Runtime {
                present: present[3],
                value: &setter,
            },
            enumerable: Presence::Runtime {
                present: present[4],
                value: DescriptorFlag::BooleanPayload(enumerable),
            },
            configurable: Presence::Runtime {
                present: present[5],
                value: DescriptorFlag::BooleanPayload(configurable),
            },
        };
        // The immutable record can only be published by save, which consumes
        // the sole successfully converted descriptor owner.
        builder.emit_object_define_entry_validated(
            target.0,
            key,
            &fields.from_runtime_checked(),
            result,
            function,
        )?;
        setter.clear(function);
        getter.clear(function);
        value.clear(function);
        for local in [configurable, enumerable, writable] {
            schema.release_i32_local(local, function);
        }
        for local in present.into_iter().rev() {
            schema.release_i32_local(local, function);
        }
        self.0.clear(function);
        Ok(())
    }
}

#[must_use]
struct CollectingObjectDefinitions {
    table: GcLocal<PropertyDefinitionTable>,
    count: I32Local,
}

#[must_use]
struct CollectedObjectDefinitions {
    table: GcLocal<PropertyDefinitionTable>,
    count: I32Local,
}

impl CollectingObjectDefinitions {
    fn append(
        &self,
        key: &PropertyKeyLocals,
        descriptor: RetainedPartialDescriptor,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        self.count.load(function);
        schema
            .array_type::<PropertyDefinitionTable>()
            .length(&self.table, schema, function);
        function.instruction(&Instruction::I32GeU);
        builder.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let stored_key = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(key.value(), function),
            function,
        );
        let definition = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<PropertyDefinition>().construct(
                (
                    GcOperand::reference(&stored_key, schema),
                    GcOperand::reference(&descriptor.0, schema),
                ),
                function,
            ),
            function,
        );
        // Publish the complete entry before advancing the sole readable count.
        schema.array_type::<PropertyDefinitionTable>().write(
            &self.table,
            self.count,
            GcOperand::nullable_reference(&definition, schema),
            schema,
            function,
        );
        self.count.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        self.count.store(function);
        definition.clear(function);
        stored_key.clear(function);
        descriptor.0.clear(function);
    }

    fn complete(self) -> CollectedObjectDefinitions {
        CollectedObjectDefinitions {
            table: self.table,
            count: self.count,
        }
    }
}

impl CollectedObjectDefinitions {
    fn clear(self, schema: &crate::gc_types::RuntimeSchema, function: &mut Function) {
        self.table.clear(function);
        schema.release_i32_local(self.count, function);
    }
}

impl FunctionBuilder<'_> {
    fn emit_collect_object_definitions(
        &mut self,
        bag: &ValueLocals,
        result: &CompletionLocals,
        cleanup: ControlTarget,
        function: &mut Function,
    ) -> Result<CollectedObjectDefinitions, EmitError> {
        let schema = self.runtime_schema();
        let keys = self.emit_object_own_property_keys(bag, function)?;
        let capacity = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        keys.length(capacity, schema, function);
        let table = schema.reserve_gc_local(function).initialize(
            schema.array_type::<PropertyDefinitionTable>().filled(
                GcOperand::null(schema),
                capacity,
                function,
            ),
            function,
        );
        let list = CollectingObjectDefinitions { table, count };
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        function.instruction(&Instruction::I32Const(0));
        count.store(function);
        let pending = schema.reserve_completion(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let iteration = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        capacity.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, function);
        let key = keys.read_key(index, self, function)?;
        let own = self.emit_proxy_target_own_descriptor(bag, &key, function)?;
        own.emit_found_i32(schema, function);
        own.emit_enumerable_i32(schema, function);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_object_read(bag, bag, &key, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, result, cleanup, function);
        let converted = self.emit_to_property_descriptor(
            pending.value(),
            RuntimeErrorMessage::OBJECT_DEFINEPROPERTY_ATTRIBUTES_MUST_BE_OBJECT,
            function,
        )?;
        let retained = RetainedPartialDescriptor::save(converted, self, function);
        list.append(&key, retained, self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        own.clear(function);
        key.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(iteration, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.clear(function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(capacity, function);
        keys.clear(function);
        Ok(list.complete())
    }

    fn emit_apply_object_definitions(
        &mut self,
        list: CollectedObjectDefinitions,
        target: &ObjectDefinitionTarget<'_>,
        result: &CompletionLocals,
        cleanup: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let index = schema.reserve_i32_local(function);
        let key_value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let iteration = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        list.count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, function);
        let definition = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<PropertyDefinitionTable>()
                .read(&list.table, index, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PropertyDefinition>()
                .field(PropertyDefinitionSchema::KEY)
                .read(&definition, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &key_value, schema, function);
        let key = self.emit_value_to_property_key_locals(&key_value, function)?;
        let descriptor = RetainedPartialDescriptor(
            schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<PropertyDefinition>()
                    .field(PropertyDefinitionSchema::DESCRIPTOR)
                    .read(&definition, schema, function)
                    .reference(),
                function,
            ),
        );
        descriptor.apply(target, &key, &pending, self, function)?;
        key.clear(function);
        stored.clear(function);
        definition.clear(function);
        self.emit_native_object_abrupt_exit(&pending, result, cleanup, function);
        self.compile_truthy_tagged_i32(pending.value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::TYPEERROR,
            result,
            function,
        )?;
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(iteration, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.clear(function);
        key_value.clear(function);
        schema.release_i32_local(index, function);
        list.clear(schema, function);
        Ok(())
    }

    /// All descriptor conversion completes before the first target definition.
    pub(super) fn emit_object_define_properties_from_values(
        &mut self,
        target: ObjectDefinitionTarget<'_>,
        properties: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let bag = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_object_locals(properties, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, result, cleanup, function);
        bag.copy_from(pending.value(), function);
        let list = self.emit_collect_object_definitions(&bag, result, cleanup, function)?;
        self.emit_apply_object_definitions(list, &target, result, cleanup, function)?;
        result.set_normal(target.0, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.clear(function);
        bag.clear(function);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_define_properties_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let properties = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        self.emit_builtin_arg_to_value(0, &target, function);
        self.emit_is_heap_object_like_tag_i32(target.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_builtin_arg_to_value(1, &properties, function);
        self.emit_object_define_properties_from_values(
            ObjectDefinitionTarget(&target),
            &properties,
            &output,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::TYPEERROR,
            &output,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        output.clear(function);
        properties.clear(function);
        target.clear(function);
        Ok(())
    }
}
