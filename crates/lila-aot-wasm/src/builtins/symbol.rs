use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SymbolBuiltin {
    Constructor,
    For,
    KeyFor,
    PrototypeDescriptionGetter,
    PrototypeToString,
    PrototypeValueOf,
    PrototypeToPrimitive,
}

impl<'a> FunctionBuilder<'a> {
    /// thisSymbolValue(value): resolves the receiver of a `Symbol.prototype`
    /// method to the underlying Symbol payload, accepting both Symbol
    /// primitives and Symbol wrapper objects ([[SymbolData]] boxed
    /// objects); throws a TypeError (and returns the current completion)
    /// for anything else.
    fn emit_this_symbol_value_to_local(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        symbol_local: u32,
        error_message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let boxed_kind_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(symbol_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_SYMBOL as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            symbol_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(boxed_kind_local);
        Ok(())
    }

    /// Reads a Symbol payload's `[[Description]]`: heap `Symbol(desc)`
    /// records (small handle, high 32 bits zero) store it in the record;
    /// well-known/registered symbols carry an interned string payload whose
    /// description is that string itself.
    fn emit_symbol_description_to_locals(
        &mut self,
        symbol_local: u32,
        desc_payload_local: u32,
        desc_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(symbol_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            symbol_local,
            HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
            desc_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            symbol_local,
            HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
            desc_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(symbol_local));
        function.instruction(&Instruction::LocalSet(desc_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(desc_tag_local));
        function.instruction(&Instruction::End);
    }

    /// SymbolDescriptiveString(sym): builds `"Symbol(" + desc + ")"`, where
    /// `desc` is the empty string when `[[Description]]` is undefined.
    pub(super) fn emit_symbol_descriptive_string_to_local(
        &mut self,
        symbol_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let desc_payload_local = self.reserve_temp_local();
        let desc_tag_local = self.reserve_temp_local();
        self.emit_symbol_description_to_locals(
            symbol_local,
            desc_payload_local,
            desc_tag_local,
            function,
        );
        let desc_string_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(desc_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(desc_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(desc_string_local));

        let prefix_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol(")));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_concat_string_payloads_local(prefix_local, desc_string_local, function)?;
        function.instruction(&Instruction::LocalSet(prefix_local));
        let suffix_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(")")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(prefix_local, suffix_local, function)?;
        function.instruction(&Instruction::LocalSet(result_payload_local));

        self.release_temp_local(suffix_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(desc_string_local);
        self.release_temp_local(desc_tag_local);
        self.release_temp_local(desc_payload_local);
        Ok(())
    }

    pub(super) fn emit_symbol(
        &mut self,
        builtin: SymbolBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            SymbolBuiltin::Constructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let handle_local = self.reserve_temp_local();

                // `Symbol` throws a TypeError when invoked via `new`.
                function.instruction(&Instruction::LocalGet(self.new_target_tag_local().unwrap()));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Symbol is not a constructor",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_heap_alloc_const(HEAP_SYMBOL_RECORD_SIZE, function)?;
                function.instruction(&Instruction::LocalSet(handle_local));
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.store_i64_const_at_offset(
                    handle_local,
                    HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                    function,
                );
                function.instruction(&Instruction::Else);
                self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(arg_tag_local));
                self.store_i64_local_at_offset(
                    handle_local,
                    HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                    arg_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    handle_local,
                    HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                    arg_payload_local,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(handle_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(handle_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            SymbolBuiltin::For => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let reg_payload_local = self.reserve_temp_local();
                let reg_tag_local = self.reserve_temp_local();
                let found_payload_local = self.reserve_temp_local();
                let found_tag_local = self.reserve_temp_local();
                let handle_local = self.reserve_temp_local();

                // key = ? ToString(description). Throws for symbol arguments and
                // propagates abrupt completions from user `toString`/`valueOf`.
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(key_local));

                function.instruction(&Instruction::GlobalGet(SYMBOL_REGISTRY_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(reg_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(reg_tag_local));
                self.emit_object_read(
                    reg_payload_local,
                    reg_tag_local,
                    reg_payload_local,
                    reg_tag_local,
                    key_local,
                    found_payload_local,
                    found_tag_local,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(found_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(found_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                // Create a new registered symbol whose [[Description]] and
                // registry key are both the requested string.
                self.emit_heap_alloc_const(HEAP_SYMBOL_RECORD_SIZE, function)?;
                function.instruction(&Instruction::LocalSet(handle_local));
                self.store_i64_const_at_offset(
                    handle_local,
                    HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                    ValueKind::String.tag() as u64,
                    function,
                );
                self.store_i64_local_at_offset(
                    handle_local,
                    HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                    key_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    handle_local,
                    HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
                    key_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(found_tag_local));
                self.emit_object_write(
                    reg_payload_local,
                    reg_tag_local,
                    key_local,
                    handle_local,
                    found_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(handle_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(handle_local);
                self.release_temp_local(found_tag_local);
                self.release_temp_local(found_payload_local);
                self.release_temp_local(reg_tag_local);
                self.release_temp_local(reg_payload_local);
                self.release_temp_local(key_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            SymbolBuiltin::KeyFor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Symbol.keyFor argument must be a symbol",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                // Only heap `Symbol()` records (small handle, high 32 bits zero)
                // carry a registry key. Well-known symbols are interned string
                // payloads (non-zero high bits) and are never registered.
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::I64Const(32));
                function.instruction(&Instruction::I64ShrU);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    arg_payload_local,
                    HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
                    key_local,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                self.release_temp_local(key_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            SymbolBuiltin::PrototypeDescriptionGetter => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.description receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.description receiver",
                    )
                })?;
                let symbol_local = self.reserve_temp_local();
                self.emit_this_symbol_value_to_local(
                    receiver_payload_local,
                    receiver_tag_local,
                    symbol_local,
                    "Symbol.prototype.description requires that 'this' be a Symbol",
                    function,
                )?;
                self.emit_symbol_description_to_locals(
                    symbol_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                );
                self.release_temp_local(symbol_local);
            }
            SymbolBuiltin::PrototypeToString => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.toString receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.toString receiver",
                    )
                })?;
                let symbol_local = self.reserve_temp_local();
                self.emit_this_symbol_value_to_local(
                    receiver_payload_local,
                    receiver_tag_local,
                    symbol_local,
                    "Symbol.prototype.toString requires that 'this' be a Symbol",
                    function,
                )?;
                self.emit_symbol_descriptive_string_to_local(
                    symbol_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(symbol_local);
            }
            SymbolBuiltin::PrototypeValueOf => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.valueOf receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype.valueOf receiver",
                    )
                })?;
                let symbol_local = self.reserve_temp_local();
                self.emit_this_symbol_value_to_local(
                    receiver_payload_local,
                    receiver_tag_local,
                    symbol_local,
                    "Symbol.prototype.valueOf requires that 'this' be a Symbol",
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(symbol_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(symbol_local);
            }
            SymbolBuiltin::PrototypeToPrimitive => {
                // Symbol.prototype[Symbol.toPrimitive](hint) ignores `hint`
                // entirely: 1. If Type(s) is Symbol, return s. 2. If Type(s)
                // is not Object, throw a TypeError. 3. If s does not have a
                // [[SymbolData]] internal slot, throw a TypeError. 4. Return
                // s.[[SymbolData]].
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype[Symbol.toPrimitive] receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Symbol.prototype[Symbol.toPrimitive] receiver",
                    )
                })?;
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Symbol.prototype[Symbol.toPrimitive] requires that 'this' be a Symbol",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::Else);
                let boxed_kind_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_BOXED_KIND_OFFSET,
                    boxed_kind_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(boxed_kind_local));
                function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_SYMBOL as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_current_function_realm_type_error(
                    "Symbol.prototype[Symbol.toPrimitive] requires that 'this' be a Symbol",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.release_temp_local(boxed_kind_local);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }
}
