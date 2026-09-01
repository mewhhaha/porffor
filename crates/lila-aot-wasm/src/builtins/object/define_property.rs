use super::*;
use lila_ir::property_descriptor::{classify, DescriptorSide, Presence};
use lila_ir::PropertyDescriptorKind;

struct RuntimeDescriptorField<T> {
    present: u32,
    value: T,
}
enum ObjectDefinePropertyDescriptorLocals {
    Data {
        value: RuntimeDescriptorField<TaggedLocals>,
        writable: RuntimeDescriptorField<u32>,
        enumerable: RuntimeDescriptorField<u32>,
        configurable: RuntimeDescriptorField<u32>,
    },
    Accessor {
        getter: RuntimeDescriptorField<TaggedLocals>,
        setter: RuntimeDescriptorField<TaggedLocals>,
        enumerable: RuntimeDescriptorField<u32>,
        configurable: RuntimeDescriptorField<u32>,
    },
}

impl ObjectDefinePropertyDescriptorLocals {
    fn validated_descriptor(self) -> WasmDescriptor {
        let descriptor = match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => WasmPartialDescriptor {
                value: Presence::Runtime {
                    present: value.present,
                    value: value.value,
                },
                writable: Presence::Runtime {
                    present: writable.present,
                    value: writable.value,
                },
                get: Presence::Absent,
                set: Presence::Absent,
                enumerable: Presence::Runtime {
                    present: enumerable.present,
                    value: enumerable.value,
                },
                configurable: Presence::Runtime {
                    present: configurable.present,
                    value: configurable.value,
                },
            },
            Self::Accessor {
                getter,
                setter,
                enumerable,
                configurable,
            } => WasmPartialDescriptor {
                value: Presence::Absent,
                writable: Presence::Absent,
                get: Presence::Runtime {
                    present: getter.present,
                    value: getter.value,
                },
                set: Presence::Runtime {
                    present: setter.present,
                    value: setter.value,
                },
                enumerable: Presence::Runtime {
                    present: enumerable.present,
                    value: enumerable.value,
                },
                configurable: Presence::Runtime {
                    present: configurable.present,
                    value: configurable.value,
                },
            },
        };
        descriptor
            .validate()
            .expect("an Object.defineProperty branch cannot mix descriptor kinds")
    }
}

struct ArgumentsCalleeDescriptorLocals {
    value: RuntimeDescriptorField<TaggedLocals>,
    writable: RuntimeDescriptorField<u32>,
    get: RuntimeDescriptorField<TaggedLocals>,
    set: RuntimeDescriptorField<TaggedLocals>,
    enumerable: RuntimeDescriptorField<u32>,
    configurable: RuntimeDescriptorField<u32>,
}

impl ArgumentsCalleeDescriptorLocals {
    fn validated_descriptor(&self) -> WasmDescriptor {
        WasmPartialDescriptor {
            value: Presence::Runtime {
                present: self.value.present,
                value: self.value.value,
            },
            writable: Presence::Runtime {
                present: self.writable.present,
                value: self.writable.value,
            },
            get: Presence::Runtime {
                present: self.get.present,
                value: self.get.value,
            },
            set: Presence::Runtime {
                present: self.set.present,
                value: self.set.value,
            },
            enumerable: Presence::Runtime {
                present: self.enumerable.present,
                value: self.enumerable.value,
            },
            configurable: Presence::Runtime {
                present: self.configurable.present,
                value: self.configurable.value,
            },
        }
        .from_runtime_checked()
    }
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
            StoredDescriptorLocals::new(
                StoredDescriptorDataLocals::new(existing_value),
                StoredDescriptorGetterLocals::new(existing_value),
                StoredDescriptorSetterLocals::new(existing_setter),
            ),
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
        descriptor: ArgumentsCalleeDescriptorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let validated_descriptor = descriptor.validated_descriptor();
        let classification = classify(&validated_descriptor);
        let data_terms = classification.terms(DescriptorSide::Data);
        let accessor_terms = classification.terms(DescriptorSide::Accessor);
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_payload_local = self.reserve_temp_local();
        let stored_tag_local = self.reserve_temp_local();
        let stored_setter_payload_local = self.reserve_temp_local();
        let stored_setter_tag_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let data_local = self.reserve_temp_local();
        let accessor_local = self.reserve_temp_local();
        let flag_payload_local = self.reserve_temp_local();

        let value_present_local = descriptor.value.present;
        let value_payload_local = descriptor.value.value.payload;
        let value_tag_local = descriptor.value.value.tag;
        let getter_present_local = descriptor.get.present;
        let getter_payload_local = descriptor.get.value.payload;
        let getter_tag_local = descriptor.get.value.tag;
        let setter_present_local = descriptor.set.present;
        let setter_payload_local = descriptor.set.value.payload;
        let setter_tag_local = descriptor.set.value.tag;
        let writable_present_local = descriptor.writable.present;
        let writable_payload_local = descriptor.writable.value;
        let enumerable_present_local = descriptor.enumerable.present;
        let enumerable_payload_local = descriptor.enumerable.value;
        let configurable_present_local = descriptor.configurable.present;
        let configurable_payload_local = descriptor.configurable.value;

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

        Self::emit_array_descriptor_side_present_to_local(&data_terms, data_local, function);
        Self::emit_array_descriptor_side_present_to_local(
            &accessor_terms,
            accessor_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(accessor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(data_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
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
        self.release_temp_local(data_local);
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

    pub(in crate::builtins) fn compile_object_define_property_builtin(
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
        let original_target_payload_local = self.reserve_temp_local();
        let original_target_tag_local = self.reserve_temp_local();
        let proxy_traversal_payload_local = self.reserve_temp_local();
        let proxy_traversal_tag_local = self.reserve_temp_local();
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
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(original_target_payload_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_traversal_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(original_target_tag_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_traversal_tag_local));
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
        let descriptor = self.emit_to_property_descriptor(
            TaggedLocals::new(descriptor_payload_local, descriptor_tag_local),
            "Object.defineProperty attributes must be object",
            function,
        )?;
        self.emit_from_present_property_descriptor(
            descriptor,
            TaggedLocals::new(
                converted_descriptor_payload_local,
                converted_descriptor_tag_local,
            ),
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

        self.emit_proxy_define_property_trap_result(
            TaggedLocals::new(proxy_traversal_payload_local, proxy_traversal_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            PropertyKeyLocals::new(key_string_local, proxy_key_tag_local),
            TaggedLocals::new(descriptor_payload_local, descriptor_tag_local),
            TaggedLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            TaggedLocals::new(proxy_trap_result_payload_local, proxy_trap_result_tag_local),
            function,
        )?;

        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::LocalGet(original_target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(original_target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_traversal_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_traversal_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
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
        self.emit_function_handle_call(
            proxy_target_value_payload_local,
            proxy_target_value_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
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
        function.instruction(&Instruction::LocalGet(original_target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(original_target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
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
        let descriptor = ArgumentsCalleeDescriptorLocals {
            value: RuntimeDescriptorField {
                present: value_present_local,
                value: TaggedLocals::new(value_payload_local, value_tag_local),
            },
            writable: RuntimeDescriptorField {
                present: writable_present_local,
                value: writable_payload_local,
            },
            get: RuntimeDescriptorField {
                present: getter_present_local,
                value: TaggedLocals::new(getter_payload_local, getter_tag_local),
            },
            set: RuntimeDescriptorField {
                present: setter_present_local,
                value: TaggedLocals::new(setter_payload_local, setter_tag_local),
            },
            enumerable: RuntimeDescriptorField {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: RuntimeDescriptorField {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        };
        self.emit_arguments_define_callee(target_payload_local, descriptor, function)?;
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
        let descriptor = ObjectDefinePropertyDescriptorLocals::Accessor {
            getter: RuntimeDescriptorField {
                present: getter_present_local,
                value: TaggedLocals::new(getter_payload_local, getter_tag_local),
            },
            setter: RuntimeDescriptorField {
                present: setter_present_local,
                value: TaggedLocals::new(setter_payload_local, setter_tag_local),
            },
            enumerable: RuntimeDescriptorField {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: RuntimeDescriptorField {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validated_descriptor();
        self.emit_object_define_entry_validated(
            target_payload_local,
            Some(target_tag_local),
            key_string_local,
            descriptor,
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
        let descriptor = ObjectDefinePropertyDescriptorLocals::Data {
            value: RuntimeDescriptorField {
                present: value_present_local,
                value: TaggedLocals::new(value_payload_local, value_tag_local),
            },
            writable: RuntimeDescriptorField {
                present: writable_present_local,
                value: writable_payload_local,
            },
            enumerable: RuntimeDescriptorField {
                present: enumerable_present_local,
                value: enumerable_payload_local,
            },
            configurable: RuntimeDescriptorField {
                present: configurable_present_local,
                value: configurable_payload_local,
            },
        }
        .validated_descriptor();
        self.emit_object_define_entry_validated(
            target_payload_local,
            Some(target_tag_local),
            key_string_local,
            descriptor,
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
        function.instruction(&Instruction::LocalGet(original_target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(original_target_tag_local));
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
        self.release_temp_local(proxy_traversal_tag_local);
        self.release_temp_local(proxy_traversal_payload_local);
        self.release_temp_local(original_target_tag_local);
        self.release_temp_local(original_target_payload_local);
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
}
