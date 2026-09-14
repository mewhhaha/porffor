//! 10.4.6 namespace internal methods on the canonical object representation.
//!
//! The boxed payload is a private Array of compiled export readers. No reader
//! is installed as a JavaScript property accessor or exposed to user source.

use super::*;
use lila_ir::ModuleNamespaceModeIr;

#[derive(Clone, Copy)]
pub(super) enum NamespaceBindingRead {
    Presence,
    Value,
}

#[derive(Clone, Copy)]
pub(crate) enum NamespaceOwnKeys {
    Strings,
    Symbols,
    All,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_module_namespace(
        &mut self,
        mode: ModuleNamespaceModeIr,
        exports: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ExprIr::ArrayLiteral(elements) = &exports.expr else {
            return Err(EmitError::unsupported(
                "namespace construction requires a private export table",
            ));
        };
        assert!(
            elements.len() % 2 == 1,
            "namespace table contains evaluator and export pairs"
        );
        match mode {
            ModuleNamespaceModeIr::Eager => assert_eq!(elements[0].kind, ValueKind::Undefined),
            ModuleNamespaceModeIr::Deferred => assert_eq!(elements[0].kind, ValueKind::Function),
        }
        let table = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let object = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.compile_expr_to_locals(exports, table.payload, table.tag, function)?;
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(object));
        function.instruction(&Instruction::I64Const(self.strings.payload(match mode {
            ModuleNamespaceModeIr::Eager => "Module",
            ModuleNamespaceModeIr::Deferred => "Deferred Module",
        })));
        function.instruction(&Instruction::LocalSet(value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value.tag));
        self.emit_object_define_local_data_with_flags(
            object,
            "Symbol.toStringTag",
            value.payload,
            value.tag,
            false,
            false,
            false,
            function,
        )?;
        self.store_i64_const_at_offset(
            object,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            OBJECT_KIND_MODULE_NAMESPACE,
            function,
        );
        self.store_i64_local_at_offset(object, HEAP_OBJECT_BOXED_TAG_OFFSET, table.tag, function);
        self.store_i64_local_at_offset(
            object,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            table.payload,
            function,
        );
        self.store_i64_const_at_offset(object, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(object));
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        self.release_temp_local(object);
        self.release_temp_local(table.tag);
        self.release_temp_local(table.payload);
        Ok(())
    }

    pub(crate) fn emit_is_module_namespace_i32(
        &mut self,
        object: u32,
        tag: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(tag));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_from_offset(object, HEAP_OBJECT_BOXED_KIND_OFFSET, function);
        function.instruction(&Instruction::I64Const(OBJECT_KIND_MODULE_NAMESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
    }

    /// Evaluates a deferred namespace for OwnPropertyKeys. Key-specific methods
    /// invoke this only after excluding symbols and the deferred `then` key.
    fn emit_namespace_evaluate(
        &mut self,
        table: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index = self.reserve_temp_local();
        let evaluator = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_array_read(table, index, evaluator.payload, evaluator.tag, function);
        function.instruction(&Instruction::LocalGet(evaluator.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            evaluator.payload,
            evaluator.tag,
            None,
            &[],
            result.payload,
            result.tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(evaluator.tag);
        self.release_temp_local(evaluator.payload);
        self.release_temp_local(index);
        Ok(())
    }

    /// Leaves an abrupt binding/evaluation completion for the caller's normal
    /// throw route. Presence alone does not read an eager module's binding.
    pub(super) fn emit_namespace_property(
        &mut self,
        object: u32,
        key: u32,
        read: NamespaceBindingRead,
        present: u32,
        descriptor: u32,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let table = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let entry = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let key_tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            object,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            table,
            function,
        );
        self.emit_property_key_tag_from_payload(key, key_tag, function);
        for local in [present, descriptor, value.payload] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value.tag));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings
                .static_builtin_property_key_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(entry.payload));
        self.emit_property_key_payload_equality_i32(key, entry.payload, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present));
        function.instruction(&Instruction::I64Const(
            StoredPropertyAttributes::Data {
                writable: false,
                enumerable: false,
                configurable: false,
            }
            .descriptor_word()
            .as_i64(),
        ));
        function.instruction(&Instruction::LocalSet(descriptor));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_array_read(table, index, entry.payload, entry.tag, function);
        function.instruction(&Instruction::LocalGet(entry.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Deferred Module"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("Module")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value.tag));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_array_read(table, index, entry.payload, entry.tag, function);
        function.instruction(&Instruction::LocalGet(entry.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(entry.payload));
        self.emit_string_payload_equality_i32(key, entry.payload, function);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);
        self.emit_namespace_evaluate(table, value, function)?;
        self.emit_break_current_completion_if_throw(1, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value.tag));
        self.load_i64_to_local_from_offset(table, HEAP_LEN_OFFSET, length, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(table, index, entry.payload, entry.tag, function);
        self.emit_string_payload_equality_i32(key, entry.payload, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present));
        function.instruction(&Instruction::I64Const(
            StoredPropertyAttributes::Data {
                writable: true,
                enumerable: true,
                configurable: false,
            }
            .descriptor_word()
            .as_i64(),
        ));
        function.instruction(&Instruction::LocalSet(descriptor));
        if matches!(read, NamespaceBindingRead::Value) {
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index));
            self.emit_array_read(table, index, entry.payload, entry.tag, function);
            self.emit_function_handle_call_without_throw_propagation(
                entry.payload,
                entry.tag,
                None,
                &[],
                value.payload,
                value.tag,
                function,
            )?;
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(key_tag);
        self.release_temp_local(entry.tag);
        self.release_temp_local(entry.payload);
        self.release_temp_local(index);
        self.release_temp_local(length);
        self.release_temp_local(table);
        Ok(())
    }

    pub(crate) fn emit_namespace_own_keys(
        &mut self,
        object: u32,
        keys: NamespaceOwnKeys,
        result: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let table = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output_index = self.reserve_temp_local();
        let key = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(
            object,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            table,
            function,
        );
        self.emit_namespace_evaluate(table, key, function)?;
        self.emit_propagate_throw_from_locals_if_needed(key.payload, key.tag, function)?;
        self.load_i64_to_local_from_offset(table, HEAP_LEN_OFFSET, length, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        match keys {
            NamespaceOwnKeys::Strings => {}
            NamespaceOwnKeys::Symbols => {
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::I64Const(1));
            }
            NamespaceOwnKeys::All => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
            }
        }
        function.instruction(&Instruction::LocalSet(output_index));
        self.emit_alloc_array_payload_with_length(output_index, result, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_index));
        if !matches!(keys, NamespaceOwnKeys::Symbols) {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(index));
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            self.emit_array_read(table, index, key.payload, key.tag, function);
            self.emit_array_write(result, output_index, key.payload, key.tag, function)?;
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index));
            function.instruction(&Instruction::LocalGet(output_index));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(output_index));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if !matches!(keys, NamespaceOwnKeys::Strings) {
            function.instruction(&Instruction::I64Const(
                self.strings
                    .static_builtin_property_key_payload("Symbol.toStringTag"),
            ));
            function.instruction(&Instruction::LocalSet(key.payload));
            self.emit_property_key_payload_to_value_payload(key.payload, function);
            function.instruction(&Instruction::LocalSet(key.payload));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::LocalSet(key.tag));
            self.emit_array_write(result, output_index, key.payload, key.tag, function)?;
        }
        self.release_temp_local(key.tag);
        self.release_temp_local(key.payload);
        self.release_temp_local(output_index);
        self.release_temp_local(index);
        self.release_temp_local(length);
        self.release_temp_local(table);
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_namespace_descriptor_object(
        &mut self,
        object: u32,
        key: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let present = self.reserve_temp_local();
        let descriptor = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let writable = self.reserve_temp_local();
        let enumerable = self.reserve_temp_local();
        let configurable = self.reserve_temp_local();
        self.emit_namespace_property(
            object,
            key,
            NamespaceBindingRead::Value,
            present,
            descriptor,
            value,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(value.payload, value.tag, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result.payload));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(configurable));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(result.tag));
        function.instruction(&Instruction::LocalGet(present));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (flag, mask) in [
            (writable, DescriptorMask::WRITABLE),
            (enumerable, DescriptorMask::ENUMERABLE),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(mask.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag));
        }
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value.payload,
            value.tag,
            writable,
            enumerable,
            configurable,
            result.payload,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result.tag));
        function.instruction(&Instruction::End);
        self.release_temp_local(configurable);
        self.release_temp_local(enumerable);
        self.release_temp_local(writable);
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        self.release_temp_local(descriptor);
        self.release_temp_local(present);
        Ok(())
    }

    pub(crate) fn emit_namespace_define_own_property(
        &mut self,
        object: u32,
        key: u32,
        requested: &WasmPartialDescriptor,
        success: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor = self.reserve_temp_local();
        let current = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_namespace_property(
            object,
            key,
            NamespaceBindingRead::Value,
            success,
            descriptor,
            current,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(current.payload, current.tag, function)?;
        for field in [&requested.get, &requested.set] {
            self.emit_descriptor_compatibility_check(
                DescriptorCompatibilityPredicate::from_presence(field),
                function,
                |_, function| {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(success));
                    Ok(())
                },
            )?;
        }
        for (field, mask) in [
            (&requested.writable, DescriptorMask::WRITABLE),
            (&requested.enumerable, DescriptorMask::ENUMERABLE),
            (&requested.configurable, DescriptorMask::CONFIGURABLE),
        ] {
            if let Some(flag) = field.value().copied() {
                self.emit_descriptor_compatibility_check(
                    DescriptorCompatibilityPredicate::from_presence(field),
                    function,
                    |_, function| {
                        function.instruction(&Instruction::LocalGet(flag));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::I64Ne);
                        function.instruction(&Instruction::LocalGet(descriptor));
                        function.instruction(&Instruction::I64Const(mask.as_i64()));
                        function.instruction(&Instruction::I64And);
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::I64Ne);
                        function.instruction(&Instruction::I32Ne);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(success));
                        function.instruction(&Instruction::End);
                        Ok(())
                    },
                )?;
            }
        }
        if let Some(value) = requested.value.value().copied() {
            self.emit_descriptor_compatibility_check(
                DescriptorCompatibilityPredicate::from_presence(&requested.value),
                function,
                |builder, function| {
                    builder.emit_tagged_payload_same_value_i32(
                        current.tag,
                        current.payload,
                        value.tag,
                        value.payload,
                        function,
                    )?;
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(success));
                    function.instruction(&Instruction::End);
                    Ok(())
                },
            )?;
        }
        self.release_temp_local(current.tag);
        self.release_temp_local(current.payload);
        self.release_temp_local(descriptor);
        Ok(())
    }

    pub(super) fn emit_namespace_set_rejection(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else(
            "Cannot assign to module namespace property",
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    /// Namespace exports are non-configurable and the object is non-extensible.
    /// Proxy OwnPropertyKeys must therefore return exactly this set. Reading all
    /// descriptors precedes validating the trap's set, including export TDZs.
    pub(super) fn emit_namespace_own_keys_invariants(
        &mut self,
        object: u32,
        snapshot: u32,
        snapshot_length: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let candidate_index = self.reserve_temp_local();
        let present = self.reserve_temp_local();
        let descriptor = self.reserve_temp_local();
        let key = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let candidate = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let internal_key = self.reserve_temp_local();
        self.emit_namespace_own_keys(object, NamespaceOwnKeys::All, expected, function)?;
        self.load_i64_to_local_from_offset(expected, HEAP_LEN_OFFSET, length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(expected, index, key.payload, key.tag, function);
        function.instruction(&Instruction::LocalGet(key.payload));
        function.instruction(&Instruction::LocalSet(internal_key));
        self.emit_value_to_property_key_locals(internal_key, key.tag, function)?;
        self.emit_namespace_property(
            object,
            internal_key,
            NamespaceBindingRead::Value,
            present,
            descriptor,
            value,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(value.payload, value.tag, function)?;
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::LocalGet(snapshot_length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy ownKeys trap result does not match non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(expected, index, key.payload, key.tag, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(present));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index));
        function.instruction(&Instruction::LocalGet(snapshot_length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot,
            candidate_index,
            candidate.payload,
            candidate.tag,
            function,
        );
        self.emit_tagged_payload_same_value_i32(
            key.tag,
            key.payload,
            candidate.tag,
            candidate.payload,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(candidate_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(candidate_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(present));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(internal_key);
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        self.release_temp_local(candidate.tag);
        self.release_temp_local(candidate.payload);
        self.release_temp_local(key.tag);
        self.release_temp_local(key.payload);
        self.release_temp_local(descriptor);
        self.release_temp_local(present);
        self.release_temp_local(candidate_index);
        self.release_temp_local(index);
        self.release_temp_local(length);
        self.release_temp_local(expected);
        Ok(())
    }
}
