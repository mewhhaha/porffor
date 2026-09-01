use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MapGetOrInsertValueSource {
    ValueArgument,
    ComputedCallback,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_map_prototype_get_or_insert(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(
            MapCollectionKind::Map,
            MapGetOrInsertValueSource::ValueArgument,
            function,
        )
    }

    pub(crate) fn emit_map_prototype_get_or_insert_computed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(
            MapCollectionKind::Map,
            MapGetOrInsertValueSource::ComputedCallback,
            function,
        )
    }

    pub(crate) fn emit_weak_map_prototype_get_or_insert(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(
            MapCollectionKind::WeakMap,
            MapGetOrInsertValueSource::ValueArgument,
            function,
        )
    }

    pub(crate) fn emit_weak_map_prototype_get_or_insert_computed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_map_prototype_get_or_insert_inner(
            MapCollectionKind::WeakMap,
            MapGetOrInsertValueSource::ComputedCallback,
            function,
        )
    }

    fn emit_map_prototype_get_or_insert_inner(
        &mut self,
        collection_kind: MapCollectionKind,
        value_source: MapGetOrInsertValueSource,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let map_record_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let found_entry_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let live_count_local = self.reserve_temp_local();

        self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        match value_source {
            MapGetOrInsertValueSource::ValueArgument => {
                match collection_kind {
                    MapCollectionKind::Map => {}
                    MapCollectionKind::WeakMap => {
                        self.emit_require_weak_key(
                            key_payload_local,
                            key_tag_local,
                            "WeakMap key must be an object or unregistered symbol",
                            function,
                        )?;
                    }
                }
                self.emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function);
            }
            MapGetOrInsertValueSource::ComputedCallback => {
                self.emit_builtin_arg_to_locals(
                    1,
                    callback_payload_local,
                    callback_tag_local,
                    function,
                );
                self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    match collection_kind {
                        MapCollectionKind::Map => {
                            "Map.prototype.getOrInsertComputed callback must be callable"
                        }
                        MapCollectionKind::WeakMap => {
                            "WeakMap.prototype.getOrInsertComputed callback must be callable"
                        }
                    },
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                match collection_kind {
                    MapCollectionKind::Map => {}
                    MapCollectionKind::WeakMap => {
                        self.emit_require_weak_key(
                            key_payload_local,
                            key_tag_local,
                            "WeakMap key must be an object or unregistered symbol",
                            function,
                        )?;
                    }
                }
            }
        }

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(0.0.into()));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);

        match value_source {
            MapGetOrInsertValueSource::ValueArgument => {}
            MapGetOrInsertValueSource::ComputedCallback => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(undefined_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(undefined_tag_local));
                self.emit_function_or_proxy_call_leave_throw_completion(
                    callback_payload_local,
                    callback_tag_local,
                    undefined_payload_local,
                    undefined_tag_local,
                    &[(key_payload_local, key_tag_local)],
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
            }
        }

        self.emit_find_map_entry(
            map_record_local,
            key_payload_local,
            key_tag_local,
            found_entry_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            found_entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        self.emit_ensure_map_capacity(
            map_record_local,
            entries_ptr_local,
            entries_len_local,
            entries_cap_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(HEAP_MAP_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(entry_local, HEAP_MAP_ENTRY_PRESENT_OFFSET, 1, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(live_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(live_count_local));
        self.store_i64_local_at_offset(
            map_record_local,
            HEAP_MAP_LIVE_COUNT_OFFSET,
            live_count_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(live_count_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(found_entry_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(map_record_local);
        Ok(())
    }
}
