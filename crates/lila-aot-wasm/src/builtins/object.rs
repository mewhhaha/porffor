use super::super::*;
use super::array::ArrayConcatSpreadableSlotValue;
use crate::objects::{
    ProxyHandlerLocals, ProxyRevocationRoute, ProxySlotLocals, ProxyTargetLocals,
    StoredDescriptorLocals, TaggedLocals, WasmDescriptor, WasmPartialDescriptor,
};
use lila_ir::property_descriptor::{classify, DescriptorSide, Presence};
use lila_ir::PropertyDescriptorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnumerableOwnProperties {
    Entries,
    Values,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IntegrityTest {
    Sealed,
    Frozen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PrototypeLookup {
    Getter,
    Setter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OwnDescriptorPredicateBuiltin {
    ObjectHasOwn,
    PrototypeHasOwnProperty,
    PrototypePropertyIsEnumerable,
}

/// The exact receiver, its temporary GetV lookup object and the result slot.
///
/// This value is deliberately private and non-`Copy`. GetV borrows it, then
/// callability validation consumes it so the lookup object cannot escape into
/// the eventual Call receiver.
#[must_use = "Object.prototype.toLocaleString receiver roles must reach validation"]
struct ObjectToLocaleStringGetVLocals {
    original_receiver: TaggedLocals,
    boxed_lookup: TaggedLocals,
    method: TaggedLocals,
}

/// A callable `toString` method paired with its exact Invoke receiver.
///
/// This token is deliberately private and non-`Copy`. Its sole consumer takes
/// ownership before emitting Proxy-aware Call with no arguments.
#[must_use = "a validated Object.prototype.toLocaleString invocation must be called"]
struct ValidatedObjectToLocaleStringInvocationLocals {
    method: TaggedLocals,
    receiver: TaggedLocals,
}

impl<'a> FunctionBuilder<'a> {
    /// Arguments exotic `[[DefineOwnProperty]]` for an indexed property.
    ///
    /// The validated descriptor is the only semantic input. The complete
    /// ParameterMap fact is captured before validation/application can replace
    /// its descriptor word, and every indexed or environment mutation follows
    /// the shared stored-descriptor compatibility check.
    fn emit_arguments_define_index_descriptor(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        descriptor: WasmDescriptor,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let classification = classify(&descriptor);
        let data_terms = classification.terms(DescriptorSide::Data);
        let accessor_terms = classification.terms(DescriptorSide::Accessor);
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let non_extensible_local = self.reserve_temp_local();
        let requested_data_local = self.reserve_temp_local();
        let requested_accessor_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let enumerable_local = self.reserve_temp_local();
        let configurable_local = self.reserve_temp_local();
        let retain_mapping_local = self.reserve_temp_local();
        let existing_value =
            TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let existing_setter =
            TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let stored_value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let stored_setter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        self.emit_arguments_descriptor_kind_for_index(
            arguments_local,
            index_local,
            existing_descriptor_kind_local,
            function,
        );
        let mapping = self.emit_arguments_index_mapping_from_descriptor_word(
            existing_descriptor_kind_local,
            function,
        );
        // Raw indexed storage carries either data or a getter. A mapped data
        // property's observable current value comes from the captured
        // ParameterMap slot and overwrites only that data projection.
        self.emit_array_read(
            arguments_local,
            index_local,
            existing_value.payload,
            existing_value.tag,
            function,
        );
        self.emit_arguments_parameter_map_read(
            arguments_local,
            &mapping,
            existing_value.payload,
            existing_value.tag,
            function,
        );
        self.emit_array_accessor_setter_for_index(
            arguments_local,
            index_local,
            existing_setter.payload,
            existing_setter.tag,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET,
            non_extensible_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(non_extensible_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot define arguments index on a non-extensible object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Self::emit_array_descriptor_side_present_to_local(
            &data_terms,
            requested_data_local,
            function,
        );
        Self::emit_array_descriptor_side_present_to_local(
            &accessor_terms,
            requested_accessor_local,
            function,
        );

        let validation_success_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(validation_success_local));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_validate_stored_descriptor(
            existing_descriptor_kind_local,
            StoredDescriptorLocals::new(existing_value, existing_value, existing_setter),
            &descriptor,
            validation_success_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(validation_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot redefine non-configurable arguments property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(validation_success_local);

        let descriptor = descriptor.into_partial();
        self.emit_array_index_effective_flag(
            descriptor.enumerable,
            existing_descriptor_kind_local,
            DescriptorMask::ENUMERABLE,
            enumerable_local,
            function,
        );
        self.emit_array_index_effective_flag(
            descriptor.configurable,
            existing_descriptor_kind_local,
            DescriptorMask::CONFIGURABLE,
            configurable_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(requested_accessor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_value.tag));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_setter.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_setter.tag));
        function.instruction(&Instruction::End);
        Self::emit_array_index_effective_value(
            descriptor.get,
            existing_value,
            stored_value,
            function,
        );
        Self::emit_array_index_effective_value(
            descriptor.set,
            existing_setter,
            stored_setter,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );
        self.emit_arguments_store_index_entry(
            arguments_local,
            index_local,
            stored_value.payload,
            stored_value.tag,
            stored_setter.payload,
            stored_setter.tag,
            descriptor_kind_local,
            function,
        )?;
        function.instruction(&Instruction::Else);

        // A run-time generic descriptor preserves an existing accessor.
        function.instruction(&Instruction::LocalGet(requested_data_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_flags_to_local(
            OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );
        self.emit_arguments_store_index_entry(
            arguments_local,
            index_local,
            existing_value.payload,
            existing_value.tag,
            existing_setter.payload,
            existing_setter.tag,
            descriptor_kind_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_value.tag));
        function.instruction(&Instruction::End);
        Self::emit_array_index_effective_value(
            descriptor.value,
            existing_value,
            stored_value,
            function,
        );
        self.emit_array_index_effective_flag(
            descriptor.writable,
            existing_descriptor_kind_local,
            DescriptorMask::WRITABLE,
            writable_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            OBJECT_DESCRIPTOR_DATA,
            Some(writable_local),
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(retain_mapping_local));
        match descriptor.writable {
            Presence::Absent => {}
            Presence::Present(writable) => {
                function.instruction(&Instruction::LocalGet(writable));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(retain_mapping_local));
                function.instruction(&Instruction::End);
            }
            Presence::Runtime {
                present,
                value: writable,
            } => {
                function.instruction(&Instruction::LocalGet(present));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(writable));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(retain_mapping_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(retain_mapping_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_mapping_restore_on_data_descriptor(
            &mapping,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_arguments_store_index_entry(
            arguments_local,
            index_local,
            stored_value.payload,
            stored_value.tag,
            existing_setter.payload,
            existing_setter.tag,
            descriptor_kind_local,
            function,
        )?;
        match descriptor.value {
            Presence::Absent => {}
            Presence::Present(value) => {
                self.emit_arguments_parameter_map_write(
                    arguments_local,
                    &mapping,
                    value.payload,
                    value.tag,
                    function,
                );
            }
            Presence::Runtime { present, value } => {
                function.instruction(&Instruction::LocalGet(present));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_arguments_parameter_map_write(
                    arguments_local,
                    &mapping,
                    value.payload,
                    value.tag,
                    function,
                );
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_arguments_index_mapping(mapping);
        for local in [
            stored_setter.tag,
            stored_setter.payload,
            stored_value.tag,
            stored_value.payload,
            existing_setter.tag,
            existing_setter.payload,
            existing_value.tag,
            existing_value.payload,
            retain_mapping_local,
            configurable_local,
            enumerable_local,
            writable_local,
            descriptor_kind_local,
            requested_accessor_local,
            requested_data_local,
            non_extensible_local,
            existing_descriptor_kind_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_arguments_define_callee(
        &mut self,
        arguments_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        value_present_local: u32,
        getter_present_local: u32,
        setter_present_local: u32,
        writable_present_local: u32,
        enumerable_present_local: u32,
        configurable_present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_payload_local = self.reserve_temp_local();
        let stored_tag_local = self.reserve_temp_local();
        let stored_setter_payload_local = self.reserve_temp_local();
        let stored_setter_tag_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let accessor_local = self.reserve_temp_local();
        let flag_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            stored_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            stored_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(accessor_local));

        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot redefine non-configurable arguments.callee",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot change enumerable flag of non-configurable arguments.callee",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(accessor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot change kind of non-configurable arguments.callee",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(accessor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (present_local, payload_local, tag_local, stored_payload, stored_tag) in [
            (
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
                stored_payload_local,
                stored_tag_local,
            ),
            (
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
                stored_setter_payload_local,
                stored_setter_tag_local,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_tagged_payload_same_value_i32(
                stored_payload,
                stored_tag,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Cannot change non-configurable arguments.callee accessor",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot make non-configurable arguments.callee writable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_tagged_payload_same_value_i32(
            stored_payload_local,
            stored_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot change non-writable arguments.callee",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(accessor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stored_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(stored_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(stored_payload_local));
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::LocalSet(stored_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(setter_payload_local));
        function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR) as i64,
        ));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stored_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(stored_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(stored_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(stored_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
        function.instruction(&Instruction::I64Const(
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as i64,
        ));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);

        for (present_local, payload_local, flag) in [
            (
                writable_present_local,
                writable_payload_local,
                DescriptorMask::WRITABLE,
            ),
            (
                enumerable_present_local,
                enumerable_payload_local,
                DescriptorMask::ENUMERABLE,
            ),
            (
                configurable_present_local,
                configurable_payload_local,
                DescriptorMask::CONFIGURABLE,
            ),
        ] {
            if flag == DescriptorMask::WRITABLE {
                function.instruction(&Instruction::LocalGet(accessor_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(flag_payload_local));
                function.instruction(&Instruction::Else);
            }
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            if flag == DescriptorMask::WRITABLE {
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(flag_payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
        }

        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            stored_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            stored_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );

        self.release_temp_local(flag_payload_local);
        self.release_temp_local(accessor_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(stored_setter_tag_local);
        self.release_temp_local(stored_setter_payload_local);
        self.release_temp_local(stored_tag_local);
        self.release_temp_local(stored_payload_local);
        self.release_temp_local(existing_descriptor_kind_local);
        Ok(())
    }

    fn emit_store_arguments_length_descriptor_kind(
        &mut self,
        arguments_local: u32,
        writable_payload_local: u32,
        writable_present_local: u32,
        enumerable_payload_local: u32,
        enumerable_present_local: u32,
        configurable_payload_local: u32,
        configurable_present_local: u32,
        requested_kind: PropertyDescriptorKind,
        function: &mut Function,
    ) {
        // Arguments `length` is configurable and may become an accessor.
        // A later generic descriptor must preserve that stored kind; spelling
        // the request as `accessor: bool` made Generic indistinguishable from
        // Data and silently converted it back.
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let writable_flag_local = self.reserve_temp_local();
        let enumerable_flag_local = self.reserve_temp_local();
        let configurable_flag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        for (payload_local, present_local, flag, flag_local) in [
            (
                writable_payload_local,
                writable_present_local,
                DescriptorMask::WRITABLE,
                writable_flag_local,
            ),
            (
                enumerable_payload_local,
                enumerable_present_local,
                DescriptorMask::ENUMERABLE,
                enumerable_flag_local,
            ),
            (
                configurable_payload_local,
                configurable_present_local,
                DescriptorMask::CONFIGURABLE,
                configurable_flag_local,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(flag_local));
            function.instruction(&Instruction::End);
        }

        match requested_kind {
            PropertyDescriptorKind::Data => {
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_DATA as i64));
                function.instruction(&Instruction::LocalGet(writable_flag_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Or);
            }
            PropertyDescriptorKind::Accessor => {
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            }
            PropertyDescriptorKind::Generic => {
                function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
                function.instruction(&Instruction::I64Const(
                    DescriptorMask::PRESERVED_BY_GENERIC_UPDATE.as_i64(),
                ));
                function.instruction(&Instruction::I64And);
            }
        }
        for (flag_local, flag) in [
            (enumerable_flag_local, DescriptorMask::ENUMERABLE),
            (configurable_flag_local, DescriptorMask::CONFIGURABLE),
        ] {
            function.instruction(&Instruction::LocalGet(flag_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Or);
        }
        // Zero is the deletion sentinel. Keep presence orthogonal to the four
        // descriptor attributes so a live all-false data descriptor remains an
        // own property and can be observed by Proxy invariant checks.
        function.instruction(&Instruction::I64Const(ARRAY_DESCRIPTOR_OWN_PROPERTY as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(existing_descriptor_kind_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );

        self.release_temp_local(configurable_flag_local);
        self.release_temp_local(enumerable_flag_local);
        self.release_temp_local(writable_flag_local);
        self.release_temp_local(existing_descriptor_kind_local);
    }

    fn emit_store_arguments_length_accessors(
        &mut self,
        arguments_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        getter_present_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        setter_present_local: u32,
        function: &mut Function,
    ) {
        let existing_descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );

        for (payload_local, tag_local, present_local, payload_offset, tag_offset) in [
            (
                getter_payload_local,
                getter_tag_local,
                getter_present_local,
                HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            ),
            (
                setter_payload_local,
                setter_tag_local,
                setter_present_local,
                HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_local_at_offset(
                arguments_local,
                payload_offset,
                payload_local,
                function,
            );
            self.store_i64_local_at_offset(arguments_local, tag_offset, tag_local, function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_const_at_offset(arguments_local, payload_offset, 0, function);
            self.store_i64_const_at_offset(
                arguments_local,
                tag_offset,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(existing_descriptor_kind_local);
    }

    fn emit_store_arguments_length_data_value(
        &mut self,
        arguments_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        value_present_local: u32,
        function: &mut Function,
    ) {
        let existing_descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(existing_descriptor_kind_local);
    }

    pub(super) fn compile_object_assign_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_arg_payload_local = self.reserve_temp_local();
        let target_arg_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let source_arg_payload_local = self.reserve_temp_local();
        let source_arg_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_key_internal_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let set_result_payload_local = self.reserve_temp_local();
        let set_result_tag_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.set builtin"))?;

        self.emit_builtin_arg_to_locals(
            0,
            target_arg_payload_local,
            target_arg_tag_local,
            function,
        );
        self.compile_nullish_tagged_i32(target_arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.assign called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            target_arg_payload_local,
            target_arg_tag_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            self.argv_param_local(),
            source_index_local,
            source_arg_payload_local,
            source_arg_tag_local,
            function,
        );
        self.compile_nullish_tagged_i32(source_arg_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            source_arg_payload_local,
            source_arg_tag_local,
            source_payload_local,
            source_tag_local,
            function,
        )?;
        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(source_payload_local, source_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (source_payload_local, source_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            enumerable_key_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `own_key_payload_local` holds the key as a JS *value*; the internal
        // [[Get]] path keys on the marked property-key payload.
        self.emit_property_key_payload_from_value_local(
            own_key_payload_local,
            own_key_tag_local,
            own_key_internal_local,
            function,
        );
        self.emit_object_read_with_key_tag(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            own_key_internal_local,
            Some(own_key_tag_local),
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_direct_js_call(
            &set_meta,
            None,
            &[
                (target_payload_local, target_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (value_payload_local, value_tag_local),
            ],
            set_result_payload_local,
            set_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(set_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot assign to read only property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(set_result_tag_local);
        self.release_temp_local(set_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_internal_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_arg_tag_local);
        self.release_temp_local(source_arg_payload_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(target_arg_tag_local);
        self.release_temp_local(target_arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_descriptors_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_key_internal_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.compile_nullish_tagged_i32(arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.getOwnPropertyDescriptors called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            arg_payload_local,
            arg_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(object_payload_local, object_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            function_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            object_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(result_payload_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (object_payload_local, object_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            descriptor_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            descriptor_payload_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        // Keys arrive as JS values from `Reflect.ownKeys`; storing them needs
        // the internal property-key encoding back.
        self.emit_property_key_payload_from_value_local(
            own_key_payload_local,
            own_key_tag_local,
            own_key_internal_local,
            function,
        );
        self.emit_object_define_enumerable_data(
            result_payload_local,
            own_key_internal_local,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(result_payload_local);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_internal_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_enumerable_own_properties_builtin(
        &mut self,
        mode: EnumerableOwnProperties,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_field_present_local = self.reserve_temp_local();
        let descriptor_field_payload_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let entry_payload_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        let include_keys = match mode {
            EnumerableOwnProperties::Entries => true,
            EnumerableOwnProperties::Values => false,
        };

        let nullish_message = if include_keys {
            "Object.entries called on null or undefined"
        } else {
            "Object.values called on null or undefined"
        };

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.compile_nullish_tagged_i32(arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            nullish_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            arg_payload_local,
            arg_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            function_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            array_prototype_local,
            function,
        );

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(object_payload_local, object_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(
            own_keys_length_local,
            result_payload_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            array_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            result_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (object_payload_local, object_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_field_present_local,
            descriptor_field_payload_local,
            descriptor_field_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_field_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_field_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_object_read_with_key_tag(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            own_key_payload_local,
            Some(own_key_tag_local),
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        if include_keys {
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::LocalSet(entry_index_local));
            self.emit_alloc_array_payload_with_length(
                entry_index_local,
                entry_payload_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                entry_payload_local,
                HEAP_PROTOTYPE_OFFSET,
                array_prototype_local,
                function,
            );
            self.store_i64_const_at_offset(
                entry_payload_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                ValueKind::Array.tag() as u64,
                function,
            );
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(entry_index_local));
            self.emit_array_write(
                entry_payload_local,
                entry_index_local,
                own_key_payload_local,
                own_key_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(entry_index_local));
            self.emit_array_write(
                entry_payload_local,
                entry_index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::LocalSet(entry_tag_local));
            self.emit_array_write(
                result_payload_local,
                write_index_local,
                entry_payload_local,
                entry_tag_local,
                function,
            )?;
        } else {
            self.emit_array_write(
                result_payload_local,
                write_index_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(array_prototype_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(entry_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(descriptor_field_tag_local);
        self.release_temp_local(descriptor_field_payload_local);
        self.release_temp_local(descriptor_field_present_local);
        self.release_temp_local(descriptor_field_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            function.instruction(&Instruction::LocalGet(arg_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(arg_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(arg_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            SYMBOL_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_SYMBOL,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bigint_constructor_local = self.reserve_temp_local();
        let bigint_fallback_prototype_local = self.reserve_temp_local();
        let bigint_prototype_local = self.reserve_temp_local();
        let bigint_realm_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(bigint_constructor_local));
        self.load_i64_to_local_from_offset(
            bigint_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            bigint_fallback_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(bigint_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            bigint_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_local(
            bigint_realm_local,
            HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
            bigint_fallback_prototype_local,
            bigint_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(bigint_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_store_boxed_primitive_metadata(
            self.result_local,
            BOXED_PRIMITIVE_KIND_BIGINT,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(bigint_realm_local);
        self.release_temp_local(bigint_prototype_local);
        self.release_temp_local(bigint_fallback_prototype_local);
        self.release_temp_local(bigint_constructor_local);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        if let (Some(new_target_payload_local), Some(new_target_tag_local)) =
            (self.new_target_payload_local(), self.new_target_tag_local())
        {
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_function_defining_realm_object_prototype(
                new_target_payload_local,
                object_prototype_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_create_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let properties_payload_local = self.reserve_temp_local();
        let properties_tag_local = self.reserve_temp_local();
        let define_properties_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperties.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.defineProperties`",
                )
            })?;
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            properties_payload_local,
            properties_tag_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.create prototype must be object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(arg_payload_local),
            Some(arg_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(properties_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_direct_js_call(
            &define_properties_meta,
            None,
            &[
                (self.result_local, self.result_tag_local),
                (properties_payload_local, properties_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(properties_tag_local);
        self.release_temp_local(properties_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.compile_nullish_tagged_i32(argument_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot convert undefined or null to object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            argument_payload_local,
            argument_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                object_payload_local,
                object_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_object_get_prototype_of_without_proxy(
                object_payload_local,
                object_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_set_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, proto_payload_local, proto_tag_local, function);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf prototype must be object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_set_prototype_of_i32(
            target_payload_local,
            target_tag_local,
            proto_payload_local,
            proto_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(set_result_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_define_property_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let get_key_local = self.reserve_temp_local();
        let set_key_local = self.reserve_temp_local();
        let value_key_local = self.reserve_temp_local();
        let writable_key_local = self.reserve_temp_local();
        let configurable_key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let writable_tag_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let configurable_tag_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let array_buffer_local = self.reserve_temp_local();
        let array_entry_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let array_length_success_local = self.reserve_temp_local();
        let array_named_define_success_local = self.reserve_temp_local();
        let getter_present_local = self.reserve_temp_local();
        let setter_present_local = self.reserve_temp_local();
        let value_present_local = self.reserve_temp_local();
        let writable_present_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let configurable_present_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let proxy_key_value_payload_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_trap_truthy_local = self.reserve_temp_local();
        let proxy_target_desc_found_local = self.reserve_temp_local();
        let proxy_target_desc_configurable_local = self.reserve_temp_local();
        let proxy_target_desc_writable_local = self.reserve_temp_local();
        let proxy_target_desc_accessor_local = self.reserve_temp_local();
        let proxy_target_value_payload_local = self.reserve_temp_local();
        let proxy_target_value_tag_local = self.reserve_temp_local();
        let proxy_target_cap_local = self.reserve_temp_local();
        let array_index_found_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let boxed_string_len_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_array_numeric_index_payload_local = self.reserve_temp_local();
        let typed_array_canonical_numeric_index_local = self.reserve_temp_local();
        let typed_array_valid_index_local = self.reserve_temp_local();
        let converted_descriptor_payload_local = self.reserve_temp_local();
        let converted_descriptor_tag_local = self.reserve_temp_local();

        let object_define_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                )
            })?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(
            2,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_named_define_success_local));
        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));
        // The converted key may be a freshly allocated string (for
        // example, `"len" + "gth"`), so downstream builtin-property
        // dispatch must not rely on packed-payload identity.
        self.emit_property_key_tag_from_payload(key_string_local, proxy_key_tag_local, function);
        // Proxy traps observe the key as a JS value, so they must not
        // see `PROPERTY_KEY_SYMBOL_MARKER`.
        self.emit_property_key_value_payload_to_local(
            key_string_local,
            proxy_key_value_payload_local,
            function,
        );

        // ToPropertyDescriptor has to observe inherited fields and own
        // accessors, and has to observe every present field exactly
        // once.  The own-data reads below therefore run against the
        // normalized descriptor, never against the caller's object.
        // Converting up front also means the Proxy `defineProperty`
        // trap receives a completed descriptor.
        self.emit_to_property_descriptor_object(
            descriptor_payload_local,
            descriptor_tag_local,
            "Object.defineProperty attributes must be object",
            converted_descriptor_payload_local,
            converted_descriptor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(converted_descriptor_payload_local));
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::LocalGet(converted_descriptor_tag_local));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            get_key_local,
            getter_present_local,
            getter_payload_local,
            getter_tag_local,
            function,
        );

        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(set_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            set_key_local,
            setter_present_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(writable_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            writable_key_local,
            writable_present_local,
            writable_payload_local,
            writable_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            enumerable_key_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(configurable_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            configurable_key_local,
            configurable_present_local,
            configurable_payload_local,
            configurable_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(value_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            value_key_local,
            value_present_local,
            value_payload_local,
            value_tag_local,
            function,
        );

        // ToPropertyDescriptor applies ToBoolean to these tagged
        // values.  Raw payload comparisons mis-handle -0, NaN and
        // non-Boolean primitives.
        self.emit_to_boolean_payload_from_tagged_locals(
            writable_tag_local,
            writable_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_to_boolean_payload_from_tagged_locals(
            enumerable_tag_local,
            enumerable_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        self.emit_to_boolean_payload_from_tagged_locals(
            configurable_tag_local,
            configurable_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(configurable_payload_local));

        for (present_local, payload_local, tag_local) in [
            (getter_present_local, getter_payload_local, getter_tag_local),
            (setter_present_local, setter_payload_local, setter_tag_local),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            self.emit_is_callable_i32(tag_local, payload_local, function)?;
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Property descriptor getter/setter must be callable or undefined",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("defineProperty"),
        ));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_object_read_ordinary(
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            get_key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(proxy_trap_tag_local, proxy_trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_property_key_tag_from_payload(key_string_local, proxy_key_tag_local, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (proxy_key_value_payload_local, proxy_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(
            proxy_trap_result_tag_local,
            proxy_trap_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proxy_trap_truthy_local));
        function.instruction(&Instruction::LocalGet(proxy_trap_truthy_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy defineProperty trap returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_proxy_define_property_trap_invariants(
            proxy_target_payload_local,
            proxy_target_tag_local,
            key_string_local,
            proxy_key_tag_local,
            value_present_local,
            value_payload_local,
            value_tag_local,
            writable_present_local,
            writable_payload_local,
            enumerable_present_local,
            enumerable_payload_local,
            configurable_present_local,
            configurable_payload_local,
            getter_present_local,
            getter_payload_local,
            getter_tag_local,
            setter_present_local,
            setter_payload_local,
            setter_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_target_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_target_cap_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&object_define_meta, function)?;
        function.instruction(&Instruction::LocalSet(proxy_target_value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_target_value_tag_local));
        self.emit_property_key_tag_from_payload(key_string_local, proxy_key_tag_local, function);
        self.emit_function_handle_call(
            proxy_target_value_payload_local,
            proxy_target_value_tag_local,
            None,
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (proxy_key_value_payload_local, proxy_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy defineProperty trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_canonical_numeric_index_string(
            key_string_local,
            typed_array_numeric_index_payload_local,
            typed_array_canonical_numeric_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_index_found_local));
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_string_local,
            index_local,
            array_index_found_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            target_payload_local,
            target_tag_local,
            typed_array_numeric_index_payload_local,
            index_local,
            typed_array_valid_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_valid_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot define invalid TypedArray index",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot define incompatible TypedArray index descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            target_payload_local,
            target_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_define_callee(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            value_present_local,
            getter_present_local,
            setter_present_local,
            writable_present_local,
            enumerable_present_local,
            configurable_present_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_string_local,
            index_local,
            array_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            boxed_string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            TYPE_ERROR_NAME,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            TYPE_ERROR_NAME,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_string_payload_equality_i32(key_string_local, get_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_arguments_length_accessors(
            target_payload_local,
            getter_payload_local,
            getter_tag_local,
            getter_present_local,
            setter_payload_local,
            setter_tag_local,
            setter_present_local,
            function,
        );
        self.emit_store_arguments_length_descriptor_kind(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            enumerable_payload_local,
            enumerable_present_local,
            configurable_payload_local,
            configurable_present_local,
            PropertyDescriptorKind::Accessor,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            key_string_local,
            index_local,
            array_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let descriptor = WasmPartialDescriptor {
            value: Presence::Absent,
            writable: Presence::Absent,
            get: Presence::Runtime {
                present: getter_present_local,
                value: TaggedLocals::new(getter_payload_local, getter_tag_local),
            },
            set: Presence::Runtime {
                present: setter_present_local,
                value: TaggedLocals::new(setter_payload_local, setter_tag_local),
            },
            enumerable: Presence::Runtime {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: Presence::Runtime {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validate()
        .expect("the accessor branch excludes data descriptor fields");
        self.emit_arguments_define_index_descriptor(
            target_payload_local,
            index_local,
            descriptor,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let descriptor = WasmPartialDescriptor {
            value: Presence::Absent,
            writable: Presence::Absent,
            get: Presence::Runtime {
                present: getter_present_local,
                value: TaggedLocals::new(getter_payload_local, getter_tag_local),
            },
            set: Presence::Runtime {
                present: setter_present_local,
                value: TaggedLocals::new(setter_payload_local, setter_tag_local),
            },
            enumerable: Presence::Runtime {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: Presence::Runtime {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validate()
        .expect("the accessor branch excludes data descriptor fields");
        self.emit_array_define_index_descriptor(
            target_payload_local,
            index_local,
            descriptor,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            TYPE_ERROR_NAME,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_is_concat_spreadable_slot_write(
            target_payload_local,
            ArrayConcatSpreadableSlotValue::Getter(TaggedLocals::new(
                getter_payload_local,
                getter_tag_local,
            )),
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prop")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_define_named_accessor_descriptor(
            target_payload_local,
            key_string_local,
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(getter_present_local),
            Some(setter_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_define_named_accessor_descriptor(
            target_payload_local,
            key_string_local,
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(getter_present_local),
            Some(setter_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_define_named_accessor_descriptor(
            target_payload_local,
            key_string_local,
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(getter_present_local),
            Some(setter_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_define_entry(
            target_payload_local,
            Some(target_tag_local),
            key_string_local,
            None,
            Some((getter_payload_local, getter_tag_local)),
            Some((setter_payload_local, setter_tag_local)),
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(getter_present_local),
            Some(setter_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_string_payload_equality_i32(key_string_local, get_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Unlike Array length, Arguments length is configurable.  Keep
        // an accessor redefinition distinct from the backing element
        // count so generic LengthOfArrayLike operations can observe it.
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_arguments_length_accessors(
            target_payload_local,
            getter_payload_local,
            getter_tag_local,
            getter_present_local,
            setter_payload_local,
            setter_tag_local,
            setter_present_local,
            function,
        );
        self.emit_store_arguments_length_descriptor_kind(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            enumerable_payload_local,
            enumerable_present_local,
            configurable_payload_local,
            configurable_present_local,
            PropertyDescriptorKind::Accessor,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_arguments_length_data_value(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            value_present_local,
            function,
        );
        self.emit_store_arguments_length_descriptor_kind(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            enumerable_payload_local,
            enumerable_present_local,
            configurable_payload_local,
            configurable_present_local,
            PropertyDescriptorKind::Data,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_arguments_length_descriptor_kind(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            enumerable_payload_local,
            enumerable_present_local,
            configurable_payload_local,
            configurable_present_local,
            PropertyDescriptorKind::Generic,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_string_payload_equality_i32(key_string_local, get_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Value-present descriptors must finish ArraySetLength's two
        // coercions before an otherwise-invalid descriptor is rejected.
        // Reuse this scratch local as the helper's post-coercion gate.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_length_from_value(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            writable_present_local,
            descriptor_kind_local,
            array_length_success_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot define array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot define array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_array_set_length_without_value(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            array_length_success_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot define array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_is_concat_spreadable_write(
            target_payload_local,
            TaggedLocals::new(value_payload_local, value_tag_local),
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            key_string_local,
            index_local,
            array_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let descriptor = WasmPartialDescriptor {
            value: Presence::Runtime {
                present: value_present_local,
                value: TaggedLocals::new(value_payload_local, value_tag_local),
            },
            writable: Presence::Runtime {
                present: writable_present_local,
                value: writable_payload_local,
            },
            get: Presence::Absent,
            set: Presence::Absent,
            enumerable: Presence::Runtime {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: Presence::Runtime {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validate()
        .expect("the data branch excludes accessor descriptor fields");
        self.emit_arguments_define_index_descriptor(
            target_payload_local,
            index_local,
            descriptor,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let descriptor = WasmPartialDescriptor {
            value: Presence::Runtime {
                present: value_present_local,
                value: TaggedLocals::new(value_payload_local, value_tag_local),
            },
            writable: Presence::Runtime {
                present: writable_present_local,
                value: writable_payload_local,
            },
            get: Presence::Absent,
            set: Presence::Absent,
            enumerable: Presence::Runtime {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: Presence::Runtime {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validate()
        .expect("the data branch excludes accessor descriptor fields");
        self.emit_array_define_index_descriptor(
            target_payload_local,
            index_local,
            descriptor,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prop")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_define_named_data_descriptor(
            target_payload_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_define_named_data_descriptor(
            target_payload_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_define_named_data_descriptor(
            target_payload_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_define_success_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_define_entry(
            target_payload_local,
            Some(target_tag_local),
            key_string_local,
            Some((value_payload_local, value_tag_local)),
            None,
            None,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(getter_present_local),
            Some(setter_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(array_named_define_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            TYPE_ERROR_NAME,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(converted_descriptor_tag_local);
        self.release_temp_local(converted_descriptor_payload_local);
        self.release_temp_local(typed_array_valid_index_local);
        self.release_temp_local(typed_array_canonical_numeric_index_local);
        self.release_temp_local(typed_array_numeric_index_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(boxed_string_len_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(array_index_found_local);
        self.release_temp_local(proxy_target_cap_local);
        self.release_temp_local(proxy_target_value_tag_local);
        self.release_temp_local(proxy_target_value_payload_local);
        self.release_temp_local(proxy_target_desc_accessor_local);
        self.release_temp_local(proxy_target_desc_writable_local);
        self.release_temp_local(proxy_target_desc_configurable_local);
        self.release_temp_local(proxy_target_desc_found_local);
        self.release_temp_local(proxy_trap_truthy_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_key_value_payload_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(configurable_present_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(writable_present_local);
        self.release_temp_local(value_present_local);
        self.release_temp_local(setter_present_local);
        self.release_temp_local(getter_present_local);
        self.release_temp_local(array_named_define_success_local);
        self.release_temp_local(array_length_success_local);
        self.release_temp_local(descriptor_field_tag_local);
        self.release_temp_local(array_entry_local);
        self.release_temp_local(array_buffer_local);
        self.release_temp_local(index_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(configurable_tag_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(writable_tag_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(configurable_key_local);
        self.release_temp_local(writable_key_local);
        self.release_temp_local(value_key_local);
        self.release_temp_local(set_key_local);
        self.release_temp_local(get_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_define_properties_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let properties_arg_payload_local = self.reserve_temp_local();
        let properties_arg_tag_local = self.reserve_temp_local();
        let properties_payload_local = self.reserve_temp_local();
        let properties_tag_local = self.reserve_temp_local();
        let converted_descriptors_payload_local = self.reserve_temp_local();
        let converted_descriptors_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_property_key_local = self.reserve_temp_local();
        let own_descriptor_payload_local = self.reserve_temp_local();
        let own_descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let converted_descriptor_payload_local = self.reserve_temp_local();
        let converted_descriptor_tag_local = self.reserve_temp_local();
        let converted_descriptor_present_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let object_define_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Object.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            properties_arg_payload_local,
            properties_arg_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.defineProperties target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(properties_arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.defineProperties properties must not be null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            properties_arg_payload_local,
            properties_arg_tag_local,
            properties_payload_local,
            properties_tag_local,
            function,
        )?;

        // Convert every enumerable descriptor before defining anything
        // on the target. A fresh ordinary object stores the completed
        // descriptors without introducing observable operations.
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(converted_descriptors_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(converted_descriptors_tag_local));
        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(properties_payload_local, properties_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (properties_payload_local, properties_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            own_descriptor_payload_local,
            own_descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(own_descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_own_data_field_read(
            own_descriptor_payload_local,
            own_descriptor_tag_local,
            enumerable_key_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_payload_local));
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_property_key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::End);
        self.emit_object_read_with_key_tag(
            properties_payload_local,
            properties_tag_local,
            properties_payload_local,
            properties_tag_local,
            own_property_key_local,
            Some(own_key_tag_local),
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        // Store the completed descriptor itself, not a property whose
        // attributes were taken from it: a heap property entry keeps
        // only the four attribute bits, so defining the descriptor as
        // a real property would turn every absent field into an
        // explicit `false` for the second pass.
        self.emit_to_property_descriptor_object(
            descriptor_payload_local,
            descriptor_tag_local,
            "Object.defineProperties descriptor must be object",
            converted_descriptor_payload_local,
            converted_descriptor_tag_local,
            function,
        )?;
        self.emit_object_define_enumerable_data(
            converted_descriptors_payload_local,
            own_property_key_local,
            converted_descriptor_payload_local,
            converted_descriptor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(own_key_payload_local));
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_property_key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::End);
        // The scratch object is ours, so an own data read returns the
        // stored descriptor without observable operations, and its
        // presence bit marks the keys the first pass skipped.
        self.emit_object_own_data_field_read(
            converted_descriptors_payload_local,
            converted_descriptors_tag_local,
            own_property_key_local,
            converted_descriptor_present_local,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(converted_descriptor_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &object_define_meta,
            None,
            &[
                (target_payload_local, target_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(converted_descriptor_present_local);
        self.release_temp_local(converted_descriptor_tag_local);
        self.release_temp_local(converted_descriptor_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(own_descriptor_tag_local);
        self.release_temp_local(own_descriptor_payload_local);
        self.release_temp_local(own_property_key_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(converted_descriptors_tag_local);
        self.release_temp_local(converted_descriptors_payload_local);
        self.release_temp_local(properties_tag_local);
        self.release_temp_local(properties_payload_local);
        self.release_temp_local(properties_arg_tag_local);
        self.release_temp_local(properties_arg_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_descriptor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let proxy_key_payload_local = self.reserve_temp_local();
        let entry_buffer_local = self.reserve_temp_local();
        let entry_len_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let function_like_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let proxy_target_desc_found_local = self.reserve_temp_local();
        let proxy_result_configurable_present_local = self.reserve_temp_local();
        let proxy_result_configurable_payload_local = self.reserve_temp_local();
        let proxy_result_writable_present_local = self.reserve_temp_local();
        let proxy_result_writable_payload_local = self.reserve_temp_local();
        let proxy_result_field_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_array_numeric_index_payload_local = self.reserve_temp_local();
        let typed_array_canonical_numeric_index_local = self.reserve_temp_local();
        let typed_array_valid_index_local = self.reserve_temp_local();
        let proxy_target_extensible_local = self.reserve_temp_local();
        let proxy_target_descriptor_fact = self.reserve_own_descriptor_fact_locals();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.getOwnPropertyDescriptor called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_property_key_payload_to_value_payload(key_string_local, function);
        function.instruction(&Instruction::LocalSet(proxy_key_payload_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_live_proxy_slots(
            target_payload_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(value_payload_local, value_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            ProxyRevocationRoute::CurrentFunctionRealm,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload("getOwnPropertyDescriptor"),
        ));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_read_without_throw_propagation(
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            entry_key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(proxy_trap_tag_local, proxy_trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_key_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (proxy_key_payload_local, proxy_key_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_own_descriptor_fact(
            value_payload_local,
            value_tag_local,
            key_string_local,
            key_tag_local,
            proxy_target_descriptor_fact,
            function,
        )?;
        proxy_target_descriptor_fact.emit_present_i32(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        proxy_target_descriptor_fact.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_object_is_extensible_i32(
            value_payload_local,
            value_tag_local,
            proxy_target_extensible_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_own_descriptor_fact(
            value_payload_local,
            value_tag_local,
            key_string_local,
            key_tag_local,
            proxy_target_descriptor_fact,
            function,
        )?;
        proxy_target_descriptor_fact.emit_present_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proxy_target_desc_found_local));
        proxy_target_descriptor_fact.emit_configurable_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        proxy_target_descriptor_fact.emit_writable_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_own_data_field_read(
            self.result_local,
            self.result_tag_local,
            entry_key_local,
            proxy_result_configurable_present_local,
            proxy_result_configurable_payload_local,
            proxy_result_field_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_own_data_field_read(
            self.result_local,
            self.result_tag_local,
            entry_key_local,
            proxy_result_writable_present_local,
            proxy_result_writable_payload_local,
            proxy_result_field_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_is_extensible_i32(
            value_payload_local,
            value_tag_local,
            proxy_target_extensible_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result incompatible with non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_present_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_payload_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report configurable for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_present_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_payload_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_result_writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(proxy_result_writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-writable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_canonical_numeric_index_string(
            key_string_local,
            typed_array_numeric_index_payload_local,
            typed_array_canonical_numeric_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            target_payload_local,
            target_tag_local,
            typed_array_numeric_index_payload_local,
            entry_index_local,
            typed_array_valid_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_valid_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            target_payload_local,
            target_tag_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_like_local));
        self.emit_known_array_index_from_property_key(
            key_string_local,
            entry_index_local,
            function_like_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(function_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            entry_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entry_len_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_kind_for_index(
            target_payload_local,
            entry_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            target_payload_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let mapping =
            self.emit_arguments_index_mapping_from_descriptor_word(descriptor_kind_local, function);
        self.emit_arguments_parameter_map_read(
            target_payload_local,
            &mapping,
            value_payload_local,
            value_tag_local,
            function,
        );
        self.release_arguments_index_mapping(mapping);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_array_accessor_setter_for_index(
            target_payload_local,
            entry_index_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        self.emit_array_length_writable_i64(target_payload_local, writable_payload_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_descriptor_read(
            target_payload_local,
            key_string_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(entry_buffer_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(entry_len_local));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_read(
            entry_buffer_local,
            entry_len_local,
            entry_buffer_local,
            entry_len_local,
            entry_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        for (flag, flag_local) in [
            (DescriptorMask::WRITABLE, writable_payload_local),
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        for (flag, flag_local) in [
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (flag, flag_local) in [
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_boxed_string_length_number_payload(
            value_payload_local,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_string_local, entry_index_local, function);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_read(
            value_payload_local,
            entry_index_local,
            getter_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            true,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(5));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_boxed_string_length_number_payload(
            target_payload_local,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_string_local, entry_index_local, function);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_read(
            target_payload_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            true,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(function_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("ArrayBuffer")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("DataView")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            entry_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            entry_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::LocalGet(entry_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entry_buffer_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_string_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        function.instruction(&Instruction::End);
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_own_descriptor_fact_locals(proxy_target_descriptor_fact);
        self.release_temp_local(proxy_target_extensible_local);
        self.release_temp_local(typed_array_valid_index_local);
        self.release_temp_local(typed_array_canonical_numeric_index_local);
        self.release_temp_local(typed_array_numeric_index_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(proxy_result_field_tag_local);
        self.release_temp_local(proxy_result_writable_payload_local);
        self.release_temp_local(proxy_result_writable_present_local);
        self.release_temp_local(proxy_result_configurable_payload_local);
        self.release_temp_local(proxy_result_configurable_present_local);
        self.release_temp_local(proxy_target_desc_found_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(function_like_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(entry_len_local);
        self.release_temp_local(entry_buffer_local);
        self.release_temp_local(proxy_key_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_names_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_index_local = self.reserve_temp_local();
        let key_index_found_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let own_property_names_string_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_array_buffer_payload_local = self.reserve_temp_local();
        let typed_array_byte_offset_local = self.reserve_temp_local();
        let typed_array_byte_length_local = self.reserve_temp_local();
        let typed_array_bytes_per_element_local = self.reserve_temp_local();
        let typed_array_length_local = self.reserve_temp_local();
        let ordinary_string_count_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(own_property_names_string_tag_local));

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.getOwnPropertyNames called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            TaggedLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            TaggedLocals::new(proxy_trap_result_payload_local, proxy_trap_result_tag_local),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_own_keys_filtered_result(
            proxy_target_payload_local,
            proxy_target_tag_local,
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            ValueKind::String,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_alloc_array_payload_with_length(entry_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            len_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.load_i64_to_local_from_offset(
            write_index_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Else);
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(entry_local, result_payload_local, function)?;
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            typed_array_buffer_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            typed_array_byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            typed_array_byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            typed_array_bytes_per_element_local,
            function,
        );
        self.emit_typed_array_current_byte_length(
            arg_payload_local,
            arg_tag_local,
            typed_array_buffer_payload_local,
            typed_array_byte_offset_local,
            typed_array_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_array_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(typed_array_length_local));

        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        self.emit_alloc_array_payload_with_length(
            ordinary_string_count_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_named_string_props_count(
            arg_payload_local,
            write_index_local,
            false,
            function,
        );
        self.emit_alloc_array_payload_with_length(
            write_index_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_named_string_props_write_keys(
            arg_payload_local,
            result_payload_local,
            write_index_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(
            write_index_local,
            result_payload_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index_found_local));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_array_read(
            result_payload_local,
            len_local,
            buffer_local,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        self.emit_known_array_index_from_property_key(
            buffer_local,
            index_number_payload_local,
            ordinary_string_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_write(
            result_payload_local,
            entry_local,
            buffer_local,
            boxed_kind_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_write(
            result_payload_local,
            entry_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(ordinary_string_count_local);
        self.release_temp_local(typed_array_length_local);
        self.release_temp_local(typed_array_bytes_per_element_local);
        self.release_temp_local(typed_array_byte_length_local);
        self.release_temp_local(typed_array_byte_offset_local);
        self.release_temp_local(typed_array_buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(own_property_names_string_tag_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(key_index_found_local);
        self.release_temp_local(key_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_symbols_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.getOwnPropertySymbols called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            TaggedLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            TaggedLocals::new(proxy_trap_result_payload_local, proxy_trap_result_tag_local),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_own_keys_filtered_result(
            proxy_target_payload_local,
            proxy_target_tag_local,
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            ValueKind::Symbol,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(len_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_property_key_payload_to_value_payload(key_payload_local, function);
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_property_key_payload_to_value_payload(key_payload_local, function);
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_keys_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_key_payload_local = self.reserve_temp_local();
        let found_index_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.keys requires object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            TaggedLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            TaggedLocals::new(proxy_trap_result_payload_local, proxy_trap_result_tag_local),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_object_keys_from_own_keys_result(
            proxy_target_payload_local,
            proxy_target_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(count_local));
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            scan_index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_descriptor_kind_for_index(
            arg_payload_local,
            scan_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_string_props_count(arg_payload_local, count_local, true, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            scan_index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_descriptor_kind_for_index(
            arg_payload_local,
            scan_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_string_props_write_keys(
            arg_payload_local,
            result_payload_local,
            write_index_local,
            true,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_unpack_string_payload(key_payload_local, buffer_local, len_local, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            len_local,
            count_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(next_key_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            index_number_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::LocalSet(next_key_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            next_key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            index_number_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(found_index_local);
        self.release_temp_local(next_key_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(count_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    fn compile_object_own_descriptor_predicate_builtin(
        &mut self,
        builtin: OwnDescriptorPredicateBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();

        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Object.getOwnPropertyDescriptor builtin")
            })?;

        match builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn => {
                self.emit_builtin_arg_to_locals(
                    0,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
            }
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty
            | OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                let this_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "missing Object.prototype own-descriptor predicate receiver",
                    )
                })?;
                let this_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "missing Object.prototype own-descriptor predicate receiver",
                    )
                })?;
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(receiver_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalSet(receiver_tag_local));
                self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
            }
        }

        match builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn => {
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.hasOwn called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
            }
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty => {
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.prototype.hasOwnProperty called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
            }
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.prototype.propertyIsEnumerable called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
            }
        }

        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (object_payload_local, object_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        match builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn
            | OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty => {
                function.instruction(&Instruction::LocalGet(descriptor_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
            }
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                let enumerable_key_local = self.reserve_temp_local();
                let enumerable_present_local = self.reserve_temp_local();
                let enumerable_payload_local = self.reserve_temp_local();
                let enumerable_tag_local = self.reserve_temp_local();

                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(descriptor_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
                function.instruction(&Instruction::LocalSet(enumerable_key_local));
                self.emit_object_own_data_field_read(
                    descriptor_payload_local,
                    descriptor_tag_local,
                    enumerable_key_local,
                    enumerable_present_local,
                    enumerable_payload_local,
                    enumerable_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(enumerable_present_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(enumerable_payload_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(enumerable_tag_local);
                self.release_temp_local(enumerable_payload_local);
                self.release_temp_local(enumerable_present_local);
                self.release_temp_local(enumerable_key_local);
            }
        }

        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_has_own_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::ObjectHasOwn,
            function,
        )
    }

    pub(super) fn compile_object_is_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, lhs_payload_local, lhs_tag_local, function);
        self.emit_builtin_arg_to_locals(1, rhs_payload_local, rhs_tag_local, function);

        self.emit_tagged_payload_same_value_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_integrity_test_builtin(
        &mut self,
        mode: IntegrityTest,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let extensible_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_field_present_local = self.reserve_temp_local();
        let descriptor_field_payload_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let reject_descriptor_local = self.reserve_temp_local();
        let check_writable = match mode {
            IntegrityTest::Sealed => false,
            IntegrityTest::Frozen => true,
        };

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_is_extensible_i32(
            arg_payload_local,
            arg_tag_local,
            extensible_result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(extensible_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(reject_descriptor_local));
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_field_present_local,
            descriptor_field_payload_local,
            descriptor_field_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_field_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_field_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(reject_descriptor_local));
        function.instruction(&Instruction::End);

        if check_writable {
            function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
            function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
            self.emit_object_own_data_field_read(
                descriptor_payload_local,
                descriptor_tag_local,
                descriptor_field_key_local,
                descriptor_field_present_local,
                descriptor_field_payload_local,
                descriptor_field_tag_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor_field_present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(descriptor_field_payload_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(reject_descriptor_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(reject_descriptor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(reject_descriptor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(reject_descriptor_local);
        self.release_temp_local(descriptor_field_tag_local);
        self.release_temp_local(descriptor_field_payload_local);
        self.release_temp_local(descriptor_field_present_local);
        self.release_temp_local(descriptor_field_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(extensible_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_is_extensible_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_object_is_extensible_i32(
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_seal_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let prevent_extensions_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let boxed_string_len_local = self.reserve_temp_local();
        let boxed_string_index_local = self.reserve_temp_local();
        let boxed_string_index_found_local = self.reserve_temp_local();
        let boxed_string_exotic_key_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_object_prevent_extensions_i32(
            arg_payload_local,
            arg_tag_local,
            prevent_extensions_result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(prevent_extensions_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.seal could not prevent extensions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(own_key_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            own_key_payload_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        // Direct String exotic indices and `length` are already
        // permanently non-configurable. Proxies still take the
        // observable Reflect.defineProperty path below.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(boxed_kind_local));
        self.emit_string_payload_equality_i32(own_key_payload_local, boxed_kind_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            own_key_payload_local,
            boxed_string_index_local,
            boxed_string_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            boxed_string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_local));
        function.instruction(&Instruction::LocalGet(boxed_string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(define_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.seal could not make an own property non-configurable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(boxed_string_exotic_key_local);
        self.release_temp_local(boxed_string_index_found_local);
        self.release_temp_local(boxed_string_index_local);
        self.release_temp_local(boxed_string_len_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(prevent_extensions_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_freeze_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let prevent_extensions_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let current_descriptor_payload_local = self.reserve_temp_local();
        let current_descriptor_tag_local = self.reserve_temp_local();
        let configurable_descriptor_payload_local = self.reserve_temp_local();
        let frozen_data_descriptor_payload_local = self.reserve_temp_local();
        let selected_descriptor_payload_local = self.reserve_temp_local();
        let descriptor_object_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_has_get_local = self.reserve_temp_local();
        let descriptor_has_set_local = self.reserve_temp_local();
        let accessor_descriptor_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let boxed_string_len_local = self.reserve_temp_local();
        let boxed_string_index_local = self.reserve_temp_local();
        let boxed_string_index_found_local = self.reserve_temp_local();
        let boxed_string_exotic_key_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_object_prevent_extensions_i32(
            arg_payload_local,
            arg_tag_local,
            prevent_extensions_result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(prevent_extensions_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.freeze could not prevent extensions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_object_tag_local));

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(
            configurable_descriptor_payload_local,
        ));
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_define_data(
            configurable_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(frozen_data_descriptor_payload_local));
        self.emit_object_define_data(
            frozen_data_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_define_data(
            frozen_data_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(current_descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_property_present(
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_has_get_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_property_present(
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_has_set_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_has_get_local));
        function.instruction(&Instruction::LocalGet(descriptor_has_set_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(accessor_descriptor_local));
        function.instruction(&Instruction::LocalGet(accessor_descriptor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(
            configurable_descriptor_payload_local,
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frozen_data_descriptor_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(selected_descriptor_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(boxed_kind_local));
        self.emit_string_payload_equality_i32(own_key_payload_local, boxed_kind_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            own_key_payload_local,
            boxed_string_index_local,
            boxed_string_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            boxed_string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_local));
        function.instruction(&Instruction::LocalGet(boxed_string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (
                    selected_descriptor_payload_local,
                    descriptor_object_tag_local,
                ),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(define_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.freeze could not make an own property non-configurable and non-writable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(boxed_string_exotic_key_local);
        self.release_temp_local(boxed_string_index_found_local);
        self.release_temp_local(boxed_string_index_local);
        self.release_temp_local(boxed_string_len_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(accessor_descriptor_local);
        self.release_temp_local(descriptor_has_set_local);
        self.release_temp_local(descriptor_has_get_local);
        self.release_temp_local(descriptor_field_key_local);
        self.release_temp_local(descriptor_object_tag_local);
        self.release_temp_local(selected_descriptor_payload_local);
        self.release_temp_local(frozen_data_descriptor_payload_local);
        self.release_temp_local(configurable_descriptor_payload_local);
        self.release_temp_local(current_descriptor_tag_local);
        self.release_temp_local(current_descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(prevent_extensions_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prevent_extensions_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_trap_truthy_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let target_cap_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_prevent_extensions_i32(
            arg_payload_local,
            arg_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("preventExtensions"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            Some((proxy_handler_payload_local, Some(proxy_handler_tag_local))),
            &[(proxy_target_payload_local, proxy_target_tag_local)],
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(
            proxy_trap_result_tag_local,
            proxy_trap_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proxy_trap_truthy_local));
        function.instruction(&Instruction::LocalGet(proxy_trap_truthy_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(proxy_target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_CAP_OFFSET,
            target_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap returned true for extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(proxy_target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(proxy_target_payload_local, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(arg_payload_local, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(arg_payload_local, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(result_local);
        self.release_temp_local(target_cap_local);
        self.release_temp_local(key_local);
        self.release_temp_local(proxy_trap_truthy_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_proto_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ getter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ getter receiver",
            )
        })?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                receiver_payload_local,
                receiver_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_object_get_prototype_of_without_proxy(
                receiver_payload_local,
                receiver_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        Ok(())
    }

    pub(super) fn compile_object_prototype_proto_setter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ setter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ setter receiver",
            )
        })?;
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, proto_payload_local, proto_tag_local, function);

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.prototype.__proto__ setter called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_set_prototype_of_i32(
            receiver_payload_local,
            receiver_tag_local,
            proto_payload_local,
            proto_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.prototype.__proto__ setter could not set prototype",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(set_result_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_has_own_property_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty,
            function,
        )
    }

    pub(super) fn compile_object_prototype_lookup_builtin(
        &mut self,
        mode: PrototypeLookup,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype accessor lookup receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype accessor lookup receiver",
            )
        })?;
        let object_get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                )
            })?;
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let accessor_key_local = self.reserve_temp_local();
        let accessor_present_local = self.reserve_temp_local();
        let accessor_payload_local = self.reserve_temp_local();
        let accessor_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_direct_js_call(
            &object_get_own_property_descriptor_meta,
            None,
            &[
                (object_payload_local, object_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        let accessor_name = match mode {
            PrototypeLookup::Getter => "get",
            PrototypeLookup::Setter => "set",
        };
        function.instruction(&Instruction::I64Const(self.strings.payload(accessor_name)));
        function.instruction(&Instruction::LocalSet(accessor_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            accessor_key_local,
            accessor_present_local,
            accessor_payload_local,
            accessor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(accessor_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(accessor_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_object_get_prototype_of(
            object_payload_local,
            object_tag_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(accessor_tag_local);
        self.release_temp_local(accessor_payload_local);
        self.release_temp_local(accessor_present_local);
        self.release_temp_local(accessor_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_property_is_enumerable_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable,
            function,
        )
    }

    pub(super) fn compile_object_prototype_is_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.isPrototypeOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.isPrototypeOf receiver",
            )
        })?;
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let current_proto_local = self.reserve_temp_local();
        let current_proto_tag_local = self.reserve_temp_local();
        let next_proto_local = self.reserve_temp_local();
        let next_proto_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(current_proto_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(current_proto_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_proto_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_object_get_prototype_of(
            current_proto_local,
            current_proto_tag_local,
            next_proto_local,
            next_proto_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_tagged_payload_same_value_i32(
            object_tag_local,
            object_payload_local,
            next_proto_tag_local,
            next_proto_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalSet(current_proto_local));
        function.instruction(&Instruction::LocalGet(next_proto_tag_local));
        function.instruction(&Instruction::LocalSet(current_proto_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_proto_tag_local);
        self.release_temp_local(next_proto_local);
        self.release_temp_local(current_proto_tag_local);
        self.release_temp_local(current_proto_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toString receiver",
            )
        })?;
        let tag_payload_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let to_string_tag_key_local = self.reserve_temp_local();
        let custom_tag_payload_local = self.reserve_temp_local();
        let custom_tag_tag_local = self.reserve_temp_local();
        let prefix_local = self.reserve_temp_local();
        let suffix_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));

        for (kind, tag) in [
            (ValueKind::Undefined, "[object Undefined]"),
            (ValueKind::Null, "[object Null]"),
            (ValueKind::Boolean, "[object Boolean]"),
            (ValueKind::Number, "[object Number]"),
            (ValueKind::String, "[object String]"),
            (ValueKind::Symbol, "[object Symbol]"),
            (ValueKind::Object, "[object Object]"),
            (ValueKind::Array, "[object Array]"),
            (ValueKind::Function, "[object Function]"),
            (ValueKind::Arguments, "[object Arguments]"),
            (ValueKind::BigInt, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            brand_local,
            function,
        );
        for (boxed_kind, tag) in [
            (BOXED_PRIMITIVE_KIND_BOOLEAN, "[object Boolean]"),
            (BOXED_PRIMITIVE_KIND_NUMBER, "[object Number]"),
            (BOXED_PRIMITIVE_KIND_STRING, "[object String]"),
            (BOXED_PRIMITIVE_KIND_SYMBOL, "[object Symbol]"),
            (BOXED_PRIMITIVE_KIND_BIGINT, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(boxed_kind as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Error]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_DATE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Date]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(to_string_tag_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            to_string_tag_key_local,
            custom_tag_payload_local,
            custom_tag_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(custom_tag_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("[object ")));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_concat_string_payloads_local(prefix_local, custom_tag_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(prefix_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(prefix_local, suffix_local, function)?;
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(tag_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(suffix_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(custom_tag_tag_local);
        self.release_temp_local(custom_tag_payload_local);
        self.release_temp_local(to_string_tag_key_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(tag_payload_local);
        Ok(())
    }

    fn emit_object_to_locale_string_get_v(
        &mut self,
        get_v: &ObjectToLocaleStringGetVLocals,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read(
            get_v.boxed_lookup.payload,
            get_v.boxed_lookup.tag,
            get_v.original_receiver.payload,
            get_v.original_receiver.tag,
            key_local,
            get_v.method.payload,
            get_v.method.tag,
            function,
        )
    }

    fn emit_validate_object_to_locale_string_invocation(
        &mut self,
        get_v: ObjectToLocaleStringGetVLocals,
        function: &mut Function,
    ) -> Result<ValidatedObjectToLocaleStringInvocationLocals, EmitError> {
        let ObjectToLocaleStringGetVLocals {
            original_receiver,
            method,
            ..
        } = get_v;
        self.emit_is_callable_i32(method.tag, method.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.prototype.toLocaleString target is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedObjectToLocaleStringInvocationLocals {
            method,
            receiver: original_receiver,
        })
    }

    fn emit_call_validated_object_to_locale_string_invocation(
        &mut self,
        invocation: ValidatedObjectToLocaleStringInvocationLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedObjectToLocaleStringInvocationLocals { method, receiver } = invocation;

        self.emit_function_or_proxy_call_leave_throw_completion(
            method.payload,
            method.tag,
            receiver.payload,
            receiver.tag,
            &[],
            result.payload,
            result.tag,
            function,
        )
    }

    pub(super) fn compile_object_prototype_to_locale_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toLocaleString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toLocaleString receiver",
            )
        })?;
        let lookup_payload_local = self.reserve_temp_local();
        let lookup_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.prototype.toLocaleString called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(lookup_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(lookup_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            lookup_payload_local,
            lookup_tag_local,
            lookup_payload_local,
            lookup_tag_local,
            function,
        )?;

        let get_v = ObjectToLocaleStringGetVLocals {
            original_receiver: TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            boxed_lookup: TaggedLocals::new(lookup_payload_local, lookup_tag_local),
            method: TaggedLocals::new(method_payload_local, method_tag_local),
        };
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_to_locale_string_get_v(&get_v, key_local, function)?;
        self.emit_return_current_completion_if_throw(function);
        let invocation = self.emit_validate_object_to_locale_string_invocation(get_v, function)?;
        self.emit_call_validated_object_to_locale_string_invocation(
            invocation,
            TaggedLocals::new(self.result_local, self.result_tag_local),
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(lookup_tag_local);
        self.release_temp_local(lookup_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_value_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.valueOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.valueOf receiver",
            )
        })?;
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        Ok(())
    }
}
