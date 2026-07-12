use super::*;

fn is_canonical_array_index_name(name: &str) -> bool {
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    if name.len() > 1 && name.starts_with('0') {
        return false;
    }
    name.parse::<u64>()
        .is_ok_and(|index| index <= MAX_ARRAY_LENGTH - 1)
}

fn helper_store_i64_local_at_offset(
    function: &mut Function,
    object_local: u32,
    offset: u64,
    value_local: u32,
) {
    function.instruction(&Instruction::LocalGet(object_local));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(value_local));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
}

fn helper_store_i64_const_at_offset(
    function: &mut Function,
    object_local: u32,
    offset: u64,
    value: i64,
) {
    function.instruction(&Instruction::LocalGet(object_local));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Const(value));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
}

pub(crate) fn emit_array_alloc_helper_function(heap_alloc_function_index: u32) -> Function {
    const LEN_LOCAL: u32 = 0;
    const ARRAY_LOCAL: u32 = 1;
    const BUFFER_LOCAL: u32 = 2;
    const CAP_LOCAL: u32 = 3;
    const SIZE_LOCAL: u32 = 4;
    const SCRATCH_LOCAL: u32 = 5;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 5));

    function.instruction(&Instruction::I64Const(HEAP_ARRAY_RECORD_SIZE as i64));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(ARRAY_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
    function.instruction(&Instruction::I64LeU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::LocalSet(SIZE_LOCAL));
    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));

    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_PTR_OFFSET, BUFFER_LOCAL);
    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_LEN_OFFSET, LEN_LOCAL);
    helper_store_i64_local_at_offset(&mut function, ARRAY_LOCAL, HEAP_CAP_OFFSET, CAP_LOCAL);
    function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        ARRAY_LOCAL,
        HEAP_PROTOTYPE_OFFSET,
        SCRATCH_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        ARRAY_LOCAL,
        HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
        ValueKind::Array.tag() as i64,
    );

    for (offset, value) in [
        (
            HEAP_ARRAY_CONSTRUCTOR_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_CONSTRUCTOR_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET, -1),
        (HEAP_ARRAY_CONSTRUCTOR_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_CONSTRUCTOR_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_CONSTRUCTOR_GETTER_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET, 0),
        (
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET, 0),
        (
            HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as i64,
        ),
        (HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET, 0),
        (HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET, 0),
        (HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET, 0),
    ] {
        helper_store_i64_const_at_offset(&mut function, ARRAY_LOCAL, offset, value);
    }

    function.instruction(&Instruction::LocalGet(ARRAY_LOCAL));
    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::End);
    function
}

pub(crate) fn emit_function_object_alloc_helper_function(
    heap_alloc_function_index: u32,
    object_append_data_property_function_index: u32,
) -> Function {
    const TABLE_INDEX_LOCAL: u32 = 0;
    const ENV_HANDLE_LOCAL: u32 = 1;
    const FLAGS_LOCAL: u32 = 2;
    const TO_STRING_PAYLOAD_LOCAL: u32 = 3;
    const LENGTH_KEY_LOCAL: u32 = 4;
    const LENGTH_PAYLOAD_LOCAL: u32 = 5;
    const NAME_KEY_LOCAL: u32 = 6;
    const NAME_PAYLOAD_LOCAL: u32 = 7;
    const DESCRIPTOR_KIND_LOCAL: u32 = 8;
    const OBJECT_LOCAL: u32 = 9;
    const BUFFER_LOCAL: u32 = 10;
    const SCRATCH_LOCAL: u32 = 11;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 3));

    function.instruction(&Instruction::I64Const(HEAP_FUNCTION_OBJECT_SIZE as i64));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(OBJECT_LOCAL));
    function.instruction(&Instruction::I64Const(
        (MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE) as i64,
    ));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, HEAP_PTR_OFFSET, BUFFER_LOCAL);
    helper_store_i64_const_at_offset(&mut function, OBJECT_LOCAL, HEAP_LEN_OFFSET, 0);
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_CAP_OFFSET,
        MIN_HEAP_CAPACITY as i64,
    );

    function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_PROTOTYPE_OFFSET,
        SCRATCH_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
        ValueKind::Object.tag() as i64,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TABLE_INDEX_OFFSET,
        TABLE_INDEX_LOCAL,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_ENV_HANDLE_OFFSET,
        ENV_HANDLE_LOCAL,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_FLAGS_OFFSET,
        FLAGS_LOCAL,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
        ValueKind::Undefined.tag() as i64,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
        0,
    );
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
        TO_STRING_PAYLOAD_LOCAL,
    );

    function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
    helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_DEFINING_REALM_OFFSET,
        SCRATCH_LOCAL,
    );

    for (global_index, offset) in [
        (
            ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
        ),
        (
            DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
        ),
        (
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
        ),
        (
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
        ),
        (
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
        ),
    ] {
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    for (_, global_index, offset) in error_realm_prototype_entries() {
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    for (constructor_global_index, offset) in [
        (
            FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_FLOAT64_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_FLOAT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT16_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_INT8_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT32_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT16_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            BIGINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BIGINT64_ARRAY_PROTOTYPE_OFFSET,
        ),
        (
            BIGUINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_BIGUINT64_ARRAY_PROTOTYPE_OFFSET,
        ),
    ] {
        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        function.instruction(&Instruction::LocalGet(SCRATCH_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        helper_store_i64_local_at_offset(&mut function, OBJECT_LOCAL, offset, SCRATCH_LOCAL);
    }

    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
        0,
    );
    helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        0,
    );

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::LocalGet(LENGTH_KEY_LOCAL));
    function.instruction(&Instruction::LocalGet(LENGTH_PAYLOAD_LOCAL));
    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::Call(
        object_append_data_property_function_index,
    ));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::LocalGet(NAME_KEY_LOCAL));
    function.instruction(&Instruction::LocalGet(NAME_PAYLOAD_LOCAL));
    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::Call(
        object_append_data_property_function_index,
    ));

    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::I64Const(
        OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
    ));
    function.instruction(&Instruction::I64And);
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    helper_store_i64_const_at_offset(&mut function, OBJECT_LOCAL, HEAP_CAP_OFFSET, 0);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::End);
    function
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_class_definition_payload(
        &mut self,
        class: &ClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_meta = self
            .functions
            .get(&class.constructor_function_id)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unknown class constructor `{}`",
                    class.constructor_function_id
                ))
            })?;
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let heritage_payload_local = self.reserve_temp_local();
        let heritage_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.emit_function_value_payload(constructor_meta, function)?;
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(heritage_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(heritage_tag_local));

        match class.heritage_kind {
            ClassHeritageKind::Constructable => {
                let Some(heritage) = &class.heritage else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing class heritage",
                    ));
                };
                self.compile_expr_to_locals(
                    heritage,
                    heritage_payload_local,
                    heritage_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(flags_local));
                function.instruction(&Instruction::LocalGet(heritage_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_load_function_flags(constructor_local, flags_local, function);
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(
                    FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR as i64,
                ));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(flags_local));
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_FUNCTION_FLAGS_OFFSET,
                    flags_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(heritage_payload_local));
                function.instruction(&Instruction::Else);
                self.emit_is_constructor_i32(heritage_tag_local, heritage_payload_local, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    "TypeError",
                    "class extends value is not a constructor or null",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target) + 2));
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_PROTOTYPE_OFFSET,
                    heritage_payload_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    heritage_tag_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
            ClassHeritageKind::Null | ClassHeritageKind::None => {}
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        if class.heritage_kind == ClassHeritageKind::Constructable {
            function.instruction(&Instruction::LocalGet(heritage_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                heritage_payload_local,
                heritage_tag_local,
                heritage_payload_local,
                heritage_tag_local,
                prototype_key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype(Some(value_payload_local), None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        } else if class.heritage_kind == ClassHeritageKind::Null {
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        } else {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        // Constructors are allocated before their `.prototype` exists.  Now
        // that the exact instance home object has been created, complete the
        // immutable class-function context used by direct constructor `super`.
        self.store_class_function_home_object(
            constructor_local,
            prototype_payload_local,
            ValueKind::Object,
            function,
        );
        self.emit_object_define_data(
            constructor_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            prototype_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        for method in &class.public_methods {
            let compiled_key_local = self.compile_object_key_to_local(&method.key, function)?;
            function.instruction(&Instruction::LocalGet(compiled_key_local));
            function.instruction(&Instruction::LocalSet(key_local));
            self.release_temp_local(compiled_key_local);
            let target_local = match method.placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            match method.kind {
                ClassFunctionKind::Method => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_data(
                        target_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ClassFunctionKind::Getter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        Some((value_payload_local, value_tag_local)),
                        None,
                        function,
                    )?;
                }
                ClassFunctionKind::Setter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        None,
                        Some((value_payload_local, value_tag_local)),
                        function,
                    )?;
                }
                ClassFunctionKind::None | ClassFunctionKind::Constructor => {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: class method kind",
                    ));
                }
            }
        }

        for method in &class.private_methods {
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(&private_data_key(method.private_name_id)),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            let target_local = match method.placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            match method.kind {
                ClassFunctionKind::Method => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_data(
                        target_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ClassFunctionKind::Getter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        Some((value_payload_local, value_tag_local)),
                        None,
                        function,
                    )?;
                }
                ClassFunctionKind::Setter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_class_function_value_payload(meta, target_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        None,
                        Some((value_payload_local, value_tag_local)),
                        function,
                    )?;
                }
                ClassFunctionKind::None | ClassFunctionKind::Constructor => {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: class method kind",
                    ));
                }
            }
        }

        let mut static_private_brands = BTreeSet::new();
        for method in &class.private_methods {
            if method.placement == ClassMethodPlacementIr::Static {
                static_private_brands.insert(method.private_name_id);
            }
        }
        for field in &class.fields {
            if field.is_private && field.placement == ClassMethodPlacementIr::Static {
                if let Some(private_name_id) = field.private_name_id {
                    static_private_brands.insert(private_name_id);
                }
            }
        }
        for private_name_id in static_private_brands {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&private_brand_key(private_name_id)),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_write(
                constructor_local,
                constructor_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        for field in &class.fields {
            if field.placement != ClassMethodPlacementIr::Static {
                continue;
            }
            let key = if let Some(key) = &field.key {
                key.clone()
            } else if let Some(private_name_id) = field.private_name_id {
                private_data_key(private_name_id)
            } else {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: malformed class field",
                ));
            };
            function.instruction(&Instruction::I64Const(self.strings.payload(&key)));
            function.instruction(&Instruction::LocalSet(key_local));
            if let Some(init_function_id) = &field.init_function_id {
                let meta = self.functions.get(init_function_id).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown class field init `{init_function_id}`"
                    ))
                })?;
                self.emit_direct_js_call(
                    meta,
                    Some((constructor_local, Some(constructor_tag_local))),
                    &[],
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
            }
            self.emit_object_write(
                constructor_local,
                constructor_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        for block in &class.static_blocks {
            let meta = self.functions.get(&block.function_id).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unknown class static block `{}`",
                    block.function_id
                ))
            })?;
            self.emit_direct_js_call(
                meta,
                Some((constructor_local, Some(constructor_tag_local))),
                &[],
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(constructor_local));
        self.release_temp_local(flags_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(heritage_tag_local);
        self.release_temp_local(heritage_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn normalize_derived_constructor_result(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Nested arrows carry the same activation metadata so their lexical
        // `this`/`super` reads reach the owner invocation, but they remain
        // ordinary calls. Only the actual derived [[Construct]] body applies
        // the special object/undefined return normalization.
        if !self.is_derived_constructor {
            return Ok(());
        }
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(self.result_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        // An object returned explicitly from a derived constructor wins, even
        // when `super()` was never evaluated.
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_get_derived_this_to_locals(self.result_local, self.result_tag_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            "TypeError",
            "derived constructor may only return object or undefined",
            self.result_local,
            self.result_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_get_derived_this_to_locals(self.result_local, self.result_tag_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_adapt_call_this_arg(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Function.prototype.call/apply thisArg adaptation failed",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_load_bound_function_record(
        &mut self,
        record_local: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        self_payload_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
            self_payload_local,
            function,
        );
    }

    pub(crate) fn emit_concat_argv_payloads(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_len_local = self.reserve_temp_local();
        let rhs_len_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let lhs_index_local = self.reserve_temp_local();
        let rhs_index_local = self.reserve_temp_local();
        let dst_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_LEN_OFFSET,
            lhs_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_LEN_OFFSET,
            rhs_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));

        self.emit_alloc_array_with_len_local(
            total_len_local,
            dst_payload_local,
            dst_buffer_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            lhs_payload_local,
            lhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            rhs_payload_local,
            rhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(dst_index_local);
        self.release_temp_local(rhs_index_local);
        self.release_temp_local(lhs_index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(rhs_len_local);
        self.release_temp_local(lhs_len_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_array_with_len_local(
        &mut self,
        len_local: u32,
        payload_local: u32,
        buffer_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            return Ok(());
        }
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(payload_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            payload_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_constructor_slot(payload_local, function);

        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_bound_function_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Bound function objects dispatch through `[[BoundFunctionInvoke]]`'s
        // funcref-table slot, so its real body must be emitted.
        self.functions
            .record_standard_builtin(StandardBuiltinId::BoundFunctionInvoker);
        let meta = self
            .functions
            .get(&StandardBuiltinId::BoundFunctionInvoker.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `[[BoundFunctionInvoke]]`",
                )
            })?;
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BOUND_FUNCTION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );

        self.emit_load_function_constructable_flag(target_payload_local, flags_local, function);
        self.emit_heap_alloc_const(HEAP_FUNCTION_OBJECT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
            object_local,
            function,
        );
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            meta.table_index as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            flags_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload(meta.to_string_value.as_str()) as u64,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        for (_, _, offset) in error_realm_prototype_entries() {
            self.load_i64_to_local_from_offset(
                target_payload_local,
                offset,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(object_local, offset, self.scratch_local, function);
        }
        self.copy_function_realm_typed_array_prototypes(
            target_payload_local,
            object_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(flags_local);
        self.release_temp_local(record_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_function_or_proxy_construct_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            return self.emit_function_handle_construct_with_argv(
                callee_payload_local,
                callee_tag_local,
                new_target_payload_local,
                new_target_tag_local,
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            );
        }

        if self.outline_proxy_construct {
            if let Some(helper) = self.proxy_construct_helper_function_index() {
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::LocalGet(new_target_payload_local));
                function.instruction(&Instruction::LocalGet(new_target_tag_local));
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::LocalGet(argv_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                return Ok(());
            }
        }

        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_key_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let argv_tag_local = self.reserve_temp_local();
        let trap_args_payload_local = self.reserve_temp_local();
        let proxy_type_error_prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_construct_with_argv(
            current_payload_local,
            current_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET,
            proxy_type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            proxy_type_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("construct")));
        function.instruction(&Instruction::LocalSet(trap_key_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_break_current_completion_if_throw(2, function);

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_array_like_snapshot_payload(
            argv_local,
            argv_tag_local,
            trap_args_payload_local,
            "Reflect.construct argumentsList must be an array",
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_function_handle_call_without_throw_propagation(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (trap_args_payload_local, argv_tag_local),
                (new_target_payload_local, new_target_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_break_current_completion_if_throw(3, function);
        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy construct trap returned non-object",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy construct trap is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_type_error_prototype_local);
        self.release_temp_local(trap_args_payload_local);
        self.release_temp_local(argv_tag_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(trap_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_construct_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let proto_is_object_local = self.reserve_temp_local();
        let prototype_realm_local = self.reserve_temp_local();
        let prototype_realm_revoked_local = self.reserve_temp_local();
        let instance_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let call_completion_local = self.reserve_temp_local();
        let callee_constructable_local = self.reserve_temp_local();
        let callee_flags_local = self.reserve_temp_local();
        let construct_this_payload_local = self.reserve_temp_local();
        let construct_this_tag_local = self.reserve_temp_local();
        let array_buffer_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let object_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ObjectConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let data_view_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::DataViewConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let proxy_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ProxyConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let aggregate_error_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::AggregateErrorConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let suppressed_error_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::SuppressedErrorConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let number_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::NumberConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let string_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::StringConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let boolean_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::BooleanConstructor.function_id())
            .map(|meta| meta.table_index as i64);
        let direct_returning_constructor_table_indices: Vec<i64> = [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
        ]
        .into_iter()
        .filter_map(|builtin| {
            self.functions
                .get(&builtin.function_id())
                .map(|meta| meta.table_index as i64)
        })
        .collect();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_load_function_flags(callee_payload_local, callee_flags_local, function);
        function.instruction(&Instruction::LocalGet(callee_flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(callee_constructable_local));
        function.instruction(&Instruction::LocalGet(callee_constructable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "target is not a constructor",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );

        // Derived constructors provide their receiver through `super()`, so
        // their [[Construct]] path must not inspect newTarget.prototype or
        // allocate a base receiver first. Their function body already
        // normalizes its result according to the derived-constructor rules.
        function.instruction(&Instruction::LocalGet(callee_flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_DERIVED_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        if let Some(array_buffer_constructor_table_index) = array_buffer_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_buffer_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(data_view_constructor_table_index) = data_view_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(data_view_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(proxy_constructor_table_index) = proxy_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(proxy_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(aggregate_error_constructor_table_index) =
            aggregate_error_constructor_table_index
        {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(
                aggregate_error_constructor_table_index,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        if let Some(suppressed_error_constructor_table_index) =
            suppressed_error_constructor_table_index
        {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(
                suppressed_error_constructor_table_index,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        for table_index in direct_returning_constructor_table_indices {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_env_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::CallIndirect {
                type_index: JS_FUNCTION_TYPE_INDEX,
                table_index: 0,
            });
            self.store_call_results_to(
                call_payload_local,
                call_tag_local,
                call_completion_local,
                self.completion_aux_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(call_completion_local));
            function.instruction(&Instruction::LocalSet(self.completion_local));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        // Ordinary [[Construct]] performs exactly one observable Get on the
        // original newTarget. In particular, do this before inspecting an
        // internal Proxy/bound representation: a Proxy's own get trap is not
        // replaceable by a read on its target.
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            proto_key_local,
            proto_payload_local,
            proto_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            proto_payload_local,
            proto_tag_local,
            2,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proto_is_object_local));
        // A primitive result selects the intrinsic from GetFunctionRealm of
        // the original newTarget. Do this only after the observable Get above:
        // the get trap may revoke a Proxy, which GetFunctionRealm must then
        // reject rather than silently using the current realm.
        function.instruction(&Instruction::LocalGet(proto_is_object_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_get_function_realm_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            prototype_realm_local,
            prototype_realm_revoked_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_realm_revoked_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "cannot get function realm from a revoked Proxy",
            payload_local,
            tag_local,
            function,
        )?;
        // revoked if, primitive-prototype if, outer construct block
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_load_realm_intrinsic_prototype_or_global(
            prototype_realm_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            proto_payload_local,
            function,
        );
        if let Some(string_constructor_table_index) = string_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(string_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_realm_intrinsic_prototype_or_global(
                prototype_realm_local,
                HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
                STRING_PROTOTYPE_GLOBAL_INDEX,
                proto_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(array_constructor_table_index) = array_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_realm_intrinsic_prototype_or_global(
                prototype_realm_local,
                HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
                ARRAY_PROTOTYPE_GLOBAL_INDEX,
                proto_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(number_constructor_table_index) = number_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(number_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_realm_intrinsic_prototype_or_global(
                prototype_realm_local,
                HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
                NUMBER_PROTOTYPE_GLOBAL_INDEX,
                proto_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        if let Some(boolean_constructor_table_index) = boolean_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(boolean_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_realm_intrinsic_prototype_or_global(
                prototype_realm_local,
                HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
                BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                proto_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proto_tag_local));
        function.instruction(&Instruction::End);

        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(proto_payload_local),
            Some(proto_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(instance_local));

        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::LocalSet(construct_this_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(construct_this_tag_local));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(construct_this_payload_local));
        function.instruction(&Instruction::LocalGet(construct_this_tag_local));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        for (constructor_table_index, primitive_tag, boxed_kind) in [
            (
                number_constructor_table_index,
                ValueKind::Number,
                BOXED_PRIMITIVE_KIND_NUMBER,
            ),
            (
                string_constructor_table_index,
                ValueKind::String,
                BOXED_PRIMITIVE_KIND_STRING,
            ),
            (
                boolean_constructor_table_index,
                ValueKind::Boolean,
                BOXED_PRIMITIVE_KIND_BOOLEAN,
            ),
        ] {
            if let Some(constructor_table_index) = constructor_table_index {
                function.instruction(&Instruction::LocalGet(table_index_local));
                function.instruction(&Instruction::I64Const(constructor_table_index));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(call_tag_local));
                function.instruction(&Instruction::I64Const(primitive_tag.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_store_boxed_primitive_metadata(
                    instance_local,
                    boxed_kind,
                    call_payload_local,
                    call_tag_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
        }

        if let Some(array_constructor_table_index) = array_constructor_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(array_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_local_at_offset(
                call_payload_local,
                HEAP_PROTOTYPE_OFFSET,
                proto_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if let Some(object_constructor_table_index) = object_constructor_table_index {
            // With a distinct newTarget, Object's construct path must select
            // the pre-created receiver. In particular it must not preserve an
            // object argument returned by Object(value).
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(object_constructor_table_index));
            function.instruction(&Instruction::I64Eq);
            self.emit_tagged_payload_same_value_i32(
                new_target_tag_local,
                new_target_payload_local,
                callee_tag_local,
                callee_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(instance_local));
            function.instruction(&Instruction::LocalSet(call_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_tag_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_heap_object_like_tag_i32(call_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(construct_this_tag_local);
        self.release_temp_local(construct_this_payload_local);
        self.release_temp_local(callee_flags_local);
        self.release_temp_local(callee_constructable_local);
        self.release_temp_local(call_completion_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(instance_local);
        self.release_temp_local(prototype_realm_revoked_local);
        self.release_temp_local(prototype_realm_local);
        self.release_temp_local(proto_is_object_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    pub(crate) fn copy_function_realm_typed_array_prototypes(
        &self,
        source_function_local: u32,
        target_function_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (builtin, _) in typed_array_constructor_bytes_per_element_entries() {
            let offset = typed_array_realm_prototype_offset(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing typed array realm prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.load_i64_to_local_from_offset(
                source_function_local,
                offset,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(
                target_function_local,
                offset,
                self.scratch_local,
                function,
            );
        }
        Ok(())
    }

    pub(crate) fn store_typed_array_realm_prototype_locals(
        &self,
        object_local: u32,
        prototype_locals: &[(StandardBuiltinId, u32)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (builtin, prototype_local) in prototype_locals {
            let offset = typed_array_realm_prototype_offset(*builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing typed array realm prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.store_i64_local_at_offset(object_local, offset, *prototype_local, function);
        }
        Ok(())
    }

    /// Materialize a builtin function value inside a branch that is provably dead
    /// in this module (its guarding heap-shape/kind cannot exist here), without
    /// forcing the builtin's real body through the emission fixpoint. The written
    /// funcref points at the shared stub table slot, which is fine because the
    /// branch can never execute. See `FunctionMetaRegistry::suppress_recording`.
    pub(crate) fn emit_function_value_payload_unrecorded(
        &mut self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.functions.set_recording_suppressed(true);
        let result = self.emit_function_value_payload(meta, function);
        self.functions.set_recording_suppressed(previous);
        result
    }

    /// Emits parameter zero for a standard builtin call or function object.
    /// Created-realm standard builtins carry a self-backed realm record in
    /// their environment slot. A user function's nonzero environment is a
    /// lexical-environment allocation with a different layout and must never
    /// be interpreted as realm metadata by a builtin.
    fn emit_standard_builtin_realm_env_argument(&self, function: &mut Function) {
        if self
            .function_id
            .as_ref()
            .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
            .is_some()
        {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
    }

    pub(crate) fn emit_function_value_payload(
        &mut self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // This is the choke point that makes a builtin's funcref-table slot
        // reachable at runtime (a function object now carries it), so its real
        // body must be emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        let function_object_alloc_function_index =
            self.function_object_alloc_function_index.ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing function object helper",
                )
            })?;
        let flags = (if meta.constructable {
            FUNCTION_FLAG_CONSTRUCTABLE
        } else {
            0
        }) | if meta.class_kind == ClassFunctionKind::Constructor {
            FUNCTION_FLAG_CLASS_CONSTRUCTOR
        } else {
            0
        } | if meta.is_derived_constructor {
            FUNCTION_FLAG_DERIVED_CONSTRUCTOR
        } else {
            0
        } | if meta.is_synthetic_default_derived_constructor {
            FUNCTION_FLAG_SYNTHETIC_DEFAULT_DERIVED_CONSTRUCTOR
        } else {
            0
        } | if meta.class_heritage_kind == ClassHeritageKind::Null {
            FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR
        } else {
            0
        } | if meta.uses_super {
            FUNCTION_FLAG_USES_SUPER
        } else {
            0
        } | if meta.this_before_super {
            FUNCTION_FLAG_THIS_BEFORE_SUPER
        } else {
            0
        } | if meta.strict { FUNCTION_FLAG_STRICT } else { 0 }
            | if meta.name == "__porfIsHTMLDDA" {
                FUNCTION_FLAG_IS_HTMLDDA
            } else {
                0
            };
        let object_local = self.reserve_temp_local();
        let class_context_local =
            (meta.class_kind != ClassFunctionKind::None).then(|| self.reserve_temp_local());
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let proto_value_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();

        if let Some(class_context_local) = class_context_local {
            self.emit_heap_alloc_const(HEAP_CLASS_FUNCTION_CONTEXT_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(class_context_local));
            self.store_i64_local_at_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                self.current_env_local,
                function,
            );
            self.store_i64_const_at_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
        }
        function.instruction(&Instruction::I64Const(meta.table_index as i64));
        if let Some(class_context_local) = class_context_local {
            function.instruction(&Instruction::LocalGet(class_context_local));
        } else if meta.standard_builtin.is_some() {
            self.emit_standard_builtin_realm_env_argument(function);
        } else {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
        }
        function.instruction(&Instruction::I64Const(flags as i64));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(meta.to_string_value.as_str()),
        ));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::F64Const(Ieee64::from(meta.length as f64)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(meta.name.as_str()),
        ));
        function.instruction(&Instruction::I64Const(
            crate::objects::object_data_descriptor_kind(false, false, meta.length_name_configurable)
                as i64,
        ));
        function.instruction(&Instruction::Call(function_object_alloc_function_index));
        function.instruction(&Instruction::LocalSet(object_local));
        if let Some(class_context_local) = class_context_local {
            self.store_i64_local_at_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
                object_local,
                function,
            );
        }

        if !meta.length_name_configurable {
            self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, 0, function);
        }

        if meta.constructable {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                proto_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                true,
                false,
                true,
                function,
            )?;

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.emit_object_append_data_property_with_flags(
                prototype_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_value_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        if let Some(class_context_local) = class_context_local {
            self.release_temp_local(class_context_local);
        }
        self.release_temp_local(object_local);
        Ok(())
    }

    /// Materialize a class member function and attach the exact object on
    /// which the member is being defined as its [[HomeObject]].
    fn emit_class_function_value_payload(
        &mut self,
        meta: &WasmFunctionMeta,
        home_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        debug_assert_ne!(meta.class_kind, ClassFunctionKind::None);
        let function_local = self.reserve_temp_local();
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(function_local));
        let home_object_tag = if meta.is_static_class_member {
            ValueKind::Function
        } else {
            ValueKind::Object
        };
        self.store_class_function_home_object(
            function_local,
            home_object_local,
            home_object_tag,
            function,
        );
        function.instruction(&Instruction::LocalGet(function_local));
        self.release_temp_local(function_local);
        Ok(())
    }

    fn store_class_function_home_object(
        &mut self,
        function_local: u32,
        home_object_local: u32,
        home_object_tag: ValueKind,
        function: &mut Function,
    ) {
        let context_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            context_local,
            function,
        );
        self.store_i64_local_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
            home_object_local,
            function,
        );
        self.store_i64_const_at_offset(
            context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
            home_object_tag.tag() as u64,
            function,
        );
        self.release_temp_local(context_local);
    }

    pub(crate) fn emit_alloc_realm_record(
        &mut self,
        realm_id: u64,
        agent_id: u64,
        realm_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let intrinsics_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_REALM_INTRINSICS_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(intrinsics_local));
        for offset in (0..HEAP_REALM_INTRINSICS_RECORD_SIZE).step_by(8) {
            self.store_i64_const_at_offset(intrinsics_local, offset, 0, function);
        }

        self.emit_heap_alloc_const(HEAP_REALM_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(realm_local));
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_ID_OFFSET, realm_id, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_AGENT_ID_OFFSET, agent_id, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_GLOBAL_OBJECT_OFFSET, 0, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_GLOBAL_THIS_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_HOST_HOOKS_OFFSET, 0, function);
        self.store_i64_const_at_offset(realm_local, HEAP_REALM_MODULE_REGISTRY_OFFSET, 0, function);
        self.release_temp_local(intrinsics_local);
        Ok(())
    }

    pub(crate) fn emit_store_function_defining_realm(
        &self,
        function_object_local: u32,
        realm_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_type_error_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_realm_intrinsic_prototype(
            realm_local,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_intrinsic_prototype(
        &mut self,
        realm_local: u32,
        intrinsic_offset: u64,
        prototype_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_local_at_offset(
            intrinsics_local,
            intrinsic_offset,
            prototype_local,
            function,
        );
        self.release_temp_local(intrinsics_local);
    }

    pub(crate) fn emit_store_current_realm_global_intrinsic(
        &mut self,
        prototype_global_index: u32,
        intrinsic_offset: u64,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_store_realm_intrinsic_prototype(
            realm_local,
            intrinsic_offset,
            prototype_local,
            function,
        );
        self.release_temp_local(prototype_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_store_realm_array_iterator_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_realm_intrinsic_prototype(
            realm_local,
            HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_store_realm_object_prototype(
        &mut self,
        realm_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.emit_store_realm_intrinsic_prototype(
            realm_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
    }

    pub(crate) fn emit_load_realm_intrinsic_prototype_or_global(
        &mut self,
        realm_local: u32,
        intrinsic_offset: u64,
        fallback_global_index: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let fallback_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(fallback_global_index));
        function.instruction(&Instruction::LocalSet(fallback_local));
        self.emit_load_realm_intrinsic_prototype_or_local(
            realm_local,
            intrinsic_offset,
            fallback_local,
            result_local,
            function,
        );
        self.release_temp_local(fallback_local);
    }

    pub(crate) fn emit_load_realm_intrinsic_prototype_or_local(
        &mut self,
        realm_local: u32,
        intrinsic_offset: u64,
        fallback_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        let candidate_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(fallback_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            intrinsic_offset,
            candidate_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_local);
        self.release_temp_local(intrinsics_local);
    }

    /// Implements GetFunctionRealm's recursive bound/proxy traversal without
    /// performing any user-visible property access. Callers invoke this only
    /// after GetPrototypeFromConstructor has already performed its single
    /// observable `Get(newTarget, "prototype")` and found a primitive.
    /// `revoked_local` is set when traversal encounters a revoked Proxy so the
    /// caller can route the TypeError through its current control context.
    pub(crate) fn emit_get_function_realm_to_locals(
        &mut self,
        source_payload_local: u32,
        source_tag_local: u32,
        realm_local: u32,
        revoked_local: u32,
        function: &mut Function,
    ) {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let proxy_handler_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(revoked_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_function_flags(current_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            current_tag_local,
            function,
        );
        // inner if, outer function-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        // function-tag if, loop, exit block
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(revoked_local));
        // revoked if, proxy if, object-tag if, loop, exit block
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        // proxy if, object-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Unknown/non-callable representation: leave realm zero. The caller's
        // global fallback is only a defensive boundary; validated newTarget
        // values should always reach an ordinary function above.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_handler_local);
        self.release_temp_local(record_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
    }

    pub(crate) fn emit_load_function_defining_realm_type_error_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_array_iterator_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_defining_realm_object_prototype(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_load_function_object_fields(
        &mut self,
        function_object_local: u32,
        env_local: u32,
        table_index_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            table_index_local,
            function,
        );
    }

    pub(crate) fn emit_load_function_flags(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            result_local,
            function,
        );
    }

    pub(crate) fn emit_load_function_constructable_flag(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.emit_load_function_flags(function_object_local, result_local, function);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(result_local));
    }

    pub(crate) fn emit_function_handle_call(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_or_proxy_call_leave_throw_completion(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_with_throw_extra_depth(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        throw_extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            Some(throw_extra_depth),
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            Some(1),
            function,
        )
    }

    pub(crate) fn emit_function_handle_call_with_argv_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_handle_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            None,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_without_throw_propagation(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            true,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_leave_throw_completion(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_with_argv_inner(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            false,
            function,
        )
    }

    pub(crate) fn emit_function_or_proxy_call_with_argv_inner(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        return_on_throw: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_function_handle_call_with_argv_without_throw_propagation(
                callee_payload_local,
                callee_tag_local,
                Some((this_payload_local, Some(this_tag_local))),
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            )?;
            if return_on_throw {
                self.emit_return_current_completion_if_throw(function);
            }
            return Ok(());
        }

        if self.outline_proxy_call {
            if let Some(helper) = self.proxy_call_helper_function_index() {
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalGet(argc_local));
                function.instruction(&Instruction::LocalGet(argv_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                if return_on_throw {
                    self.emit_return_current_completion_if_throw(function);
                }
                return Ok(());
            }
        }

        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_key_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let argv_tag_local = self.reserve_temp_local();
        let trap_args_payload_local = self.reserve_temp_local();
        let proxy_type_error_prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            current_payload_local,
            current_tag_local,
            Some((this_payload_local, Some(this_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET,
            proxy_type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(proxy_type_error_prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            proxy_type_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("apply")));
        function.instruction(&Instruction::LocalSet(trap_key_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        } else {
            self.emit_break_current_completion_if_throw(2, function);
        }

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_array_like_snapshot_payload(
            argv_local,
            argv_tag_local,
            trap_args_payload_local,
            "Reflect.construct argumentsList must be an array",
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(argv_tag_local));
        self.emit_function_handle_call_without_throw_propagation(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (this_payload_local, this_tag_local),
                (trap_args_payload_local, argv_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion_if_throw(function);
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy apply trap is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Br(1));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_type_error_prototype_local);
        self.release_temp_local(trap_args_payload_local);
        self.release_temp_local(argv_tag_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(trap_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_function_handle_call_with_argv_inner(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        propagate_throw_extra_depth: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let call_this_payload_local = self.reserve_temp_local();
        let call_this_tag_local = self.reserve_temp_local();
        let proxy_revocable_table_index = self
            .functions
            .get(&StandardBuiltinId::ProxyRevocable.function_id())
            .map(|meta| meta.table_index as i64);

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "value is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            propagate_throw_extra_depth.unwrap_or(0),
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        self.emit_load_function_flags(callee_payload_local, flags_local, function);

        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_CLASS_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            if let Some(this_tag_local) = this_tag_local {
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(FUNCTION_FLAG_STRICT as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(FUNCTION_FLAG_STRICT as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_value_to_object_locals(
                    this_payload_local,
                    this_tag_local,
                    call_this_payload_local,
                    call_this_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(call_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(call_this_tag_local));
            }
        } else {
            function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(call_this_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_this_tag_local));
        }
        if let Some(proxy_revocable_table_index) = proxy_revocable_table_index {
            function.instruction(&Instruction::LocalGet(table_index_local));
            function.instruction(&Instruction::I64Const(proxy_revocable_table_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(callee_payload_local));
            function.instruction(&Instruction::LocalSet(call_this_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(call_this_tag_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(call_this_payload_local));
        function.instruction(&Instruction::LocalGet(call_this_tag_local));
        self.emit_undefined_new_target(function);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results(payload_local, tag_local, function);
        if let Some(extra_depth) = propagate_throw_extra_depth {
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                payload_local,
                tag_local,
                extra_depth,
                function,
            )?;
            self.set_completion_kind(CompletionKind::Normal, function);
        } else {
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "class constructor cannot be invoked without `new`",
            payload_local,
            tag_local,
            function,
        )?;
        if let Some(extra_depth) = propagate_throw_extra_depth {
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                payload_local,
                tag_local,
                extra_depth,
                function,
            )?;
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(call_this_tag_local);
        self.release_temp_local(call_this_payload_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    /// Captures the two values that SuperCall obtains before evaluating its
    /// argument list: the current invocation's `new.target` and the active
    /// constructor's current [[Prototype]].  Keeping this phase separate from
    /// construction lets expression lowering preserve the observable order
    /// when an argument mutates the class heritage.
    pub(crate) fn emit_prepare_super_construct_to_locals(
        &mut self,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        ctor_payload_local: u32,
        ctor_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.lexical_derived_activation.is_none() {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside derived constructor",
            ));
        }

        let active_function_payload_local = self.reserve_temp_local();
        let active_function_tag_local = self.reserve_temp_local();
        self.emit_get_derived_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        self.emit_get_derived_active_function_to_locals(
            active_function_payload_local,
            active_function_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            active_function_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            ctor_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            active_function_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ctor_tag_local,
            function,
        );
        self.release_temp_local(active_function_tag_local);
        self.release_temp_local(active_function_payload_local);
        Ok(())
    }

    pub(crate) fn emit_super_construct_with_prepared_arg_vector(
        &mut self,
        ctor_payload_local: u32,
        ctor_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        self.emit_function_or_proxy_construct_with_argv(
            ctor_payload_local,
            ctor_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            call_payload_local,
            call_tag_local,
            0,
            function,
        )?;
        // Construct consumes the base constructor's completion.  Its produced
        // receiver is an ordinary value for the rest of the derived body.
        self.set_completion_kind(CompletionKind::Normal, function);
        // Binding is intentionally after Construct.  A second `super()` must
        // still perform the base construction before its duplicate-bind
        // ReferenceError, and a failed construction leaves `this` unbound.
        self.emit_bind_derived_this_from_locals(
            call_payload_local,
            call_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        Ok(())
    }

    /// SuperCall entry point for callers whose argument vector already
    /// exists, notably the synthetic default derived constructor.
    pub(crate) fn emit_super_construct_with_arg_vector(
        &mut self,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ctor_payload_local = self.reserve_temp_local();
        let ctor_tag_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        self.emit_prepare_super_construct_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            ctor_payload_local,
            ctor_tag_local,
            function,
        )?;
        self.emit_super_construct_with_prepared_arg_vector(
            ctor_payload_local,
            ctor_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(ctor_tag_local);
        self.release_temp_local(ctor_payload_local);
        Ok(())
    }

    pub(crate) fn store_call_results(
        &self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
    }

    pub(crate) fn store_call_results_to(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalSet(aux_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
    }

    pub(crate) fn emit_arguments_has_index_i32(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            entry_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entry_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_array_has_index_i32(arguments_local, index_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_rest_array_payload(
        &mut self,
        start_index: usize,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rest_len_local = self.reserve_temp_local();
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(rest_len_local));

        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, rest_len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, self.scratch_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_constructor_slot(array_local, function);

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
        ] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        self.release_temp_local(rest_len_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_object_payload(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arguments_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let mapped_count_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        if self.uses_mapped_arguments_object() {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(self.params.len() as i64));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.params.len() as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(mapped_count_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(mapped_count_local));
        }

        self.emit_heap_alloc_const(HEAP_ARGUMENTS_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(arguments_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(arguments_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        if self.uses_mapped_arguments_object() {
            self.store_i64_local_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                self.current_env_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                0,
                function,
            );
        }
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
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

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PAYLOAD_OFFSET] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }
        self.store_i64_const_at_offset(
            dst_entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for (param_index, param) in self.params.iter().enumerate() {
            if param.is_rest {
                continue;
            }
            let Some(storage) = self.lookup_binding(&param.name) else {
                continue;
            };
            function.instruction(&Instruction::I64Const(param_index as i64));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(dst_entry_local));
            self.read_binding_to_locals(
                storage,
                iterator_payload_local,
                iterator_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                dst_entry_local,
                HEAP_ARRAY_TAG_OFFSET,
                iterator_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                dst_entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                iterator_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(arguments_local));
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(mapped_count_local);
        self.release_temp_local(len_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arguments_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_length(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            getter_payload_local,
            getter_tag_local,
            Some((arguments_local, Some(arguments_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_is_concat_spreadable_read(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_arguments_is_concat_spreadable_write(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        function.instruction(&Instruction::Else);
        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_arguments_read(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let arguments_tag_local = self.reserve_temp_local();

        self.emit_array_descriptor_kind_for_index(
            arguments_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_data_read(
            arguments_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(arguments_tag_local));
        self.emit_array_index_get(
            arguments_local,
            index_local,
            arguments_local,
            arguments_tag_local,
            payload_local,
            tag_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(arguments_tag_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    fn emit_arguments_data_read(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mapped_count_local = self.reserve_temp_local();
        let env_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(mapped_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(env_local);
        self.release_temp_local(mapped_count_local);
        Ok(())
    }

    pub(crate) fn emit_arguments_write(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mapped_count_local = self.reserve_temp_local();
        let env_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(mapped_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            arguments_local,
            buffer_local,
            len_local,
            cap_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(env_local);
        self.release_temp_local(mapped_count_local);
        Ok(())
    }

    pub(crate) fn current_function_meta(&self) -> Option<&WasmFunctionMeta> {
        self.function_id
            .as_ref()
            .and_then(|function_id| self.functions.get(function_id))
    }

    pub(crate) fn emit_load_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self
            .current_function_meta()
            .is_some_and(|meta| meta.class_kind != ClassFunctionKind::None)
        {
            let home_object_local = self.reserve_temp_local();
            let home_object_tag_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                home_object_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                home_object_tag_local,
                function,
            );
            self.emit_load_super_base_from_home_object(
                home_object_local,
                home_object_tag_local,
                payload_local,
                tag_local,
                function,
            );
            self.release_temp_local(home_object_tag_local);
            self.release_temp_local(home_object_local);
            return Ok(());
        }
        if self.lexical_derived_activation.is_some() {
            // A constructor's SuperProperty reference is based on its
            // [[HomeObject]], the constructor's `.prototype` object. Arrows
            // lexically enclosed by that constructor share the same base; the
            // arrow call ABI's `this` parameter is not the home object and is
            // deliberately ignored here.
            let active_function_payload_local = self.reserve_temp_local();
            let active_function_tag_local = self.reserve_temp_local();
            let class_context_local = self.reserve_temp_local();
            let home_object_local = self.reserve_temp_local();
            let home_object_tag_local = self.reserve_temp_local();
            self.emit_get_derived_active_function_to_locals(
                active_function_payload_local,
                active_function_tag_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                active_function_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                class_context_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                home_object_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                class_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                home_object_tag_local,
                function,
            );
            self.emit_load_super_base_from_home_object(
                home_object_local,
                home_object_tag_local,
                payload_local,
                tag_local,
                function,
            );
            self.release_temp_local(home_object_tag_local);
            self.release_temp_local(home_object_local);
            self.release_temp_local(class_context_local);
            self.release_temp_local(active_function_tag_local);
            self.release_temp_local(active_function_payload_local);
            return Ok(());
        }

        let Some(home_object) = self.lookup_binding(LEXICAL_HOME_OBJECT_NAME) else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside class method",
            ));
        };
        let home_object_local = self.reserve_temp_local();
        let home_object_tag_local = self.reserve_temp_local();
        self.read_binding_to_locals(
            home_object,
            home_object_local,
            home_object_tag_local,
            function,
        );
        self.emit_load_super_base_from_home_object(
            home_object_local,
            home_object_tag_local,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(home_object_tag_local);
        self.release_temp_local(home_object_local);
        Ok(())
    }

    fn emit_load_super_base_from_home_object(
        &mut self,
        home_object_local: u32,
        home_object_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            home_object_local,
            HEAP_PROTOTYPE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(home_object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            home_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_throw_if_null_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "super property access on null base",
            payload_local,
            tag_local,
            function,
        )?;
        // `emit_throw_runtime_error` has already recorded the throw completion
        // and result locals. Dispatch it through the normal completion path so
        // active `finally` blocks run before the throw reaches its handler (or
        // returns from the function). The dispatch is nested inside this
        // null-test `if`, hence the extra branch depth.
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_call_args_vector(
        &mut self,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(u32, u32), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if let [arg] = args {
            if let ExprIr::SpreadArgument(spread_value) = &arg.expr {
                let spread_payload_local = self.reserve_temp_local();
                let spread_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    spread_value,
                    spread_payload_local,
                    spread_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    spread_payload_local,
                    spread_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(spread_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(spread_payload_local));
                function.instruction(&Instruction::LocalSet(argv_local));
                self.load_i64_to_local_from_offset(
                    spread_payload_local,
                    HEAP_LEN_OFFSET,
                    argc_local,
                    function,
                );
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Spread argument is not an array",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.release_temp_local(spread_tag_local);
                self.release_temp_local(spread_payload_local);
                return Ok((argc_local, argv_local));
            }
        }

        let mut arg_locals = Vec::with_capacity(args.len());
        for arg in args {
            let payload_local = self.reserve_temp_local();
            let tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, payload_local, tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            arg_locals.push((payload_local, tag_local));
        }

        self.emit_pre_evaluated_arg_vector(&arg_locals, argc_local, argv_local, function)?;

        for (payload_local, tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(tag_local);
            self.release_temp_local(payload_local);
        }

        Ok((argc_local, argv_local))
    }

    pub(crate) fn emit_direct_js_call(
        &mut self,
        meta: &WasmFunctionMeta,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if meta.class_kind != ClassFunctionKind::Constructor {
            self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        }
        self.emit_direct_js_call_with_argv(
            meta,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_direct_js_call_with_argv(
        &mut self,
        meta: &WasmFunctionMeta,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // A direct call into a builtin's body requires its real body to be
        // emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        if meta.class_kind == ClassFunctionKind::Constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.throw_handler_stack.last() {
                function.instruction(&Instruction::Br(self.depth_to(*target)));
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            if meta.standard_builtin.is_some() {
                self.emit_standard_builtin_realm_env_argument(function);
            } else {
                function.instruction(&Instruction::LocalGet(self.current_env_local));
            }
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                function.instruction(&Instruction::LocalGet(this_payload_local));
                if let Some(this_tag_local) = this_tag_local {
                    function.instruction(&Instruction::LocalGet(this_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                }
            } else {
                self.emit_default_this_for_known_strictness(meta.strict, function);
            }
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        }

        Ok(())
    }

    pub(crate) fn emit_indirect_call_from_locals(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, u32)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let default_this_payload_local = self.reserve_temp_local();
        let default_this_tag_local = self.reserve_temp_local();

        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;

        let (this_payload_local, this_tag_local) =
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                (this_payload_local, this_tag_local)
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(default_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(default_this_tag_local));
                (default_this_payload_local, default_this_tag_local)
            };

        self.emit_function_or_proxy_call_with_argv_without_throw_propagation(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(default_this_tag_local);
        self.release_temp_local(default_this_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_match_function_id = StandardBuiltinId::StringPrototypeMatch.function_id();
        let string_split_function_id = StandardBuiltinId::StringPrototypeSplit.function_id();
        let string_slice_function_id = StandardBuiltinId::StringPrototypeSlice.function_id();
        if callee.function_targets.len() == 1
            && (callee.function_targets.contains(&string_match_function_id)
                || callee.function_targets.contains(&string_split_function_id)
                || callee.function_targets.contains(&string_slice_function_id))
        {
            if let Some(this_arg) = this_arg {
                if callee.function_targets.contains(&string_match_function_id) {
                    return self.emit_string_match_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                if callee.function_targets.contains(&string_slice_function_id) {
                    return self.emit_string_slice_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if let (
            ExprIr::PropertyRead {
                key: PropertyKeyIr::StaticString(name),
                ..
            },
            Some(this_arg),
        ) = (&callee.expr, this_arg)
        {
            let string_or_undefined = KindSet::from_kind(ValueKind::String)
                .union(KindSet::from_kind(ValueKind::Undefined));
            if name == "split" && this_arg.possible_kinds.is_subset_of(string_or_undefined) {
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        let reflect_define_property_function_id =
            StandardBuiltinId::ReflectDefineProperty.function_id();
        let object_define_property_function_id =
            StandardBuiltinId::ObjectDefineProperty.function_id();
        let is_reflect_define_property_access = matches!(
            &callee.expr,
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(name),
            } if name == "defineProperty"
                && matches!(
                    &target.expr,
                    ExprIr::GlobalPropertyRead { name } | ExprIr::Identifier(name)
                        if name == REFLECT_NAME
                )
        );
        if is_reflect_define_property_access
            && callee.function_targets.len() == 1
            && (callee
                .function_targets
                .contains(&reflect_define_property_function_id)
                || callee
                    .function_targets
                    .contains(&object_define_property_function_id))
        {
            let callee_payload_local = self.reserve_temp_local();
            let callee_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
            let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
            let meta = self
                .functions
                .get(&reflect_define_property_function_id)
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                    )
                })?;
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            self.emit_default_this_for_known_strictness(meta.strict, function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }

        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let default_this_payload_local = self.reserve_temp_local();
        let default_this_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;

        let this_locals = if let Some(this_arg) = this_arg {
            let this_payload_local = self.reserve_temp_local();
            let this_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;
            Some((this_payload_local, this_tag_local))
        } else {
            None
        };
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        let (this_payload_local, this_tag_local) =
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                (this_payload_local, this_tag_local)
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(default_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(default_this_tag_local));
                (default_this_payload_local, default_this_tag_local)
            };

        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            self.release_temp_local(this_tag_local);
            self.release_temp_local(this_payload_local);
        }
        self.release_temp_local(default_this_tag_local);
        self.release_temp_local(default_this_payload_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }

    fn emit_custom_array_named_method_call(
        &mut self,
        receiver: &TypedExpr,
        key: &PropertyKeyIr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        let key_local = self.compile_object_key_to_local(key, function)?;
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            callee_payload_local,
            callee_tag_local,
            function,
        )?;
        self.release_temp_local(key_local);
        self.emit_propagate_throw_from_locals_if_needed(
            callee_payload_local,
            callee_tag_local,
            function,
        )?;

        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_method_call(
        &mut self,
        receiver: &TypedExpr,
        key: &PropertyKeyIr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let custom_array_named_get = matches!(
            receiver.heap_shape.as_deref(),
            Some(HeapShape::Array(shape)) if shape.prototype.is_some()
        ) && matches!(
            key,
            PropertyKeyIr::StaticString(name)
                if name != "length" && !is_canonical_array_index_name(name)
        );
        if custom_array_named_get {
            return self.emit_custom_array_named_method_call(
                receiver,
                key,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "test") {
            return self.emit_regexp_exec_literal_control_method_call(
                receiver,
                args,
                true,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "push")
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
        {
            return self.emit_array_push_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "toLocaleString")
            && receiver.possible_kinds.contains(ValueKind::Array)
        {
            return self.emit_array_direct_builtin_method_call(
                StandardBuiltinId::ArrayPrototypeToLocaleString,
                "Array.prototype.toLocaleString",
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "toString")
            && args.is_empty()
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
        {
            return self.emit_array_join_method_call(
                receiver,
                &[],
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "reverse") {
            return self.emit_array_reverse_method_call(
                receiver,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "split") {
            return self.emit_string_split_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "match") {
            return self.emit_string_match_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "substring") {
            return self.emit_string_substring_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "slice") {
            let receiver_is_string = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String));
            let receiver_has_string_slice = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "slice"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::StringPrototypeSlice.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_string || receiver_has_string_slice {
                return self.emit_string_slice_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "charAt") {
            return self.emit_string_char_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "charCodeAt") {
            return self.emit_string_char_code_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "codePointAt") {
            return self.emit_string_code_point_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "at")
            && receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String))
        {
            return self.emit_string_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "pop") {
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let len_local = self.reserve_temp_local();
            let index_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Array.prototype.pop receiver is not array",
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                len_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_array_read(
                receiver_payload_local,
                index_local,
                payload_local,
                tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                index_local,
                function,
            );
            function.instruction(&Instruction::End);
            self.release_temp_local(index_local);
            self.release_temp_local(len_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "shift") {
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let len_local = self.reserve_temp_local();
            let index_local = self.reserve_temp_local();
            let next_index_local = self.reserve_temp_local();
            let element_payload_local = self.reserve_temp_local();
            let element_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Array.prototype.shift receiver is not array",
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                len_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_array_read(
                receiver_payload_local,
                index_local,
                payload_local,
                tag_local,
                function,
            );
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(next_index_local));
            self.emit_array_read(
                receiver_payload_local,
                next_index_local,
                element_payload_local,
                element_tag_local,
                function,
            );
            self.emit_array_write(
                receiver_payload_local,
                index_local,
                element_payload_local,
                element_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(len_local));
            self.store_i64_local_at_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                len_local,
                function,
            );
            function.instruction(&Instruction::End);
            self.release_temp_local(element_tag_local);
            self.release_temp_local(element_payload_local);
            self.release_temp_local(next_index_local);
            self.release_temp_local(index_local);
            self.release_temp_local(len_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "splice") {
            return self.emit_array_direct_builtin_method_call(
                StandardBuiltinId::ArrayPrototypeSplice,
                "Array.prototype.splice",
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "spliceFromArray") {
            return self.emit_array_splice_from_array_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), "keys" | "entries" | "values") || name == PORFFOR_STATIC_GENERATOR_VALUES_METHOD)
        {
            let kind = match key {
                PropertyKeyIr::StaticString(name) if name == "keys" => ARRAY_ITERATOR_KIND_KEYS,
                PropertyKeyIr::StaticString(name) if name == "entries" => {
                    ARRAY_ITERATOR_KIND_ENTRIES
                }
                _ => ARRAY_ITERATOR_KIND_VALUES,
            };
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            self.emit_array_iterator_create_from_locals(
                receiver_payload_local,
                receiver_tag_local,
                kind,
                payload_local,
                tag_local,
                function,
            )?;
            if matches!(key, PropertyKeyIr::StaticString(name) if name == PORFFOR_STATIC_GENERATOR_VALUES_METHOD)
            {
                self.emit_object_define_bool_data(
                    payload_local,
                    PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT,
                    true,
                    function,
                )?;
            }
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "concat") {
            return self.emit_array_concat_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "flat") {
            return self.emit_array_flat_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "flatMap") {
            let receiver_is_array = receiver.kind == ValueKind::Array
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            if receiver_is_array {
                return self.emit_array_flat_map_method_call(
                    receiver,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "at") {
            return self.emit_array_at_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "includes") {
            return self.emit_array_includes_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "indexOf") {
            return self.emit_array_index_of_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "lastIndexOf") {
            return self.emit_array_last_index_of_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "find") {
            let receiver_is_array = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array))
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "find"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeFind.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFind.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.find`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_find_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findIndex") {
            return self.emit_array_find_index_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "reduce") {
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "reduce"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeReduce.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeReduce.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.reduce`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "take") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "take"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeTake.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeTake.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.take`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "drop") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "drop"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeDrop.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeDrop.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.drop`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findLast") {
            return self.emit_array_find_last_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "findLastIndex") {
            return self.emit_array_find_last_index_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "map") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "map"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeMap.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.map`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_map_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "every") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "every"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeEvery.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeEvery.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.every`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_every_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "some") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "some"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeSome.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeSome.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.some`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_some_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "filter") {
            let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "filter"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeFilter.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFilter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.filter`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_filter_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "flatMap") {
            let receiver_is_array = receiver.kind == ValueKind::Array
                || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
            let receiver_is_iterator = receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "flatMap"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeFlatMap.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                });
            if receiver_is_iterator || !receiver_is_array {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFlatMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.flatMap`",
                        )
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
            return self.emit_array_flat_map_method_call(
                receiver,
                args,
                payload_local,
                tag_local,
                function,
            );
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if name == "forEach")
            && receiver
                .heap_shape
                .as_deref()
                .and_then(|shape| read_static_heap_shape_property(shape, "forEach"))
                .is_some_and(|property| match property {
                    ObjectShapeProperty::Data(info) => info
                        .function_targets
                        .contains(&StandardBuiltinId::IteratorPrototypeForEach.function_id()),
                    ObjectShapeProperty::Accessor { .. } => false,
                })
        {
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let callee_payload_local = self.reserve_temp_local();
            let callee_tag_local = self.reserve_temp_local();
            let key_local = self.reserve_temp_local();
            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload("forEach")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                receiver_payload_local,
                receiver_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                callee_payload_local,
                callee_tag_local,
                function,
            )?;
            let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
            self.emit_function_handle_call_with_argv(
                callee_payload_local,
                callee_tag_local,
                Some((receiver_payload_local, Some(receiver_tag_local))),
                argc_local,
                argv_local,
                payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            self.release_temp_local(key_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), "trim" | "trimStart" | "trimLeft" | "trimEnd" | "trimRight"))
        {
            let trim_start = matches!(
                key,
                PropertyKeyIr::StaticString(name)
                    if matches!(name.as_str(), "trim" | "trimStart" | "trimLeft")
            );
            let trim_end = matches!(
                key,
                PropertyKeyIr::StaticString(name)
                    if matches!(name.as_str(), "trim" | "trimEnd" | "trimRight")
            );
            let receiver_payload_local = self.reserve_temp_local();
            let receiver_tag_local = self.reserve_temp_local();
            let string_local = self.reserve_temp_local();

            self.compile_expr_to_locals(
                receiver,
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "String.prototype method receiver is null or undefined",
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_value_to_string_payload(
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(string_local));
            self.emit_ecmascript_trim_payload_from_locals(
                string_local,
                trim_start,
                trim_end,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::End);

            self.release_temp_local(string_local);
            self.release_temp_local(receiver_tag_local);
            self.release_temp_local(receiver_payload_local);
            return Ok(());
        }
        let string_html_builtin = match key {
            PropertyKeyIr::StaticString(name) => match name.as_str() {
                "anchor" => Some(StandardBuiltinId::StringPrototypeAnchor),
                "big" => Some(StandardBuiltinId::StringPrototypeBig),
                "blink" => Some(StandardBuiltinId::StringPrototypeBlink),
                "bold" => Some(StandardBuiltinId::StringPrototypeBold),
                "fixed" => Some(StandardBuiltinId::StringPrototypeFixed),
                "fontcolor" => Some(StandardBuiltinId::StringPrototypeFontcolor),
                "fontsize" => Some(StandardBuiltinId::StringPrototypeFontsize),
                "italics" => Some(StandardBuiltinId::StringPrototypeItalics),
                "link" => Some(StandardBuiltinId::StringPrototypeLink),
                "small" => Some(StandardBuiltinId::StringPrototypeSmall),
                "strike" => Some(StandardBuiltinId::StringPrototypeStrike),
                "sub" => Some(StandardBuiltinId::StringPrototypeSub),
                "substr" => Some(StandardBuiltinId::StringPrototypeSubstr),
                "substring" => Some(StandardBuiltinId::StringPrototypeSubstring),
                "sup" => Some(StandardBuiltinId::StringPrototypeSup),
                "match" => Some(StandardBuiltinId::StringPrototypeMatch),
                "matchAll" => Some(StandardBuiltinId::StringPrototypeMatchAll),
                "replace" => Some(StandardBuiltinId::StringPrototypeReplace),
                "replaceAll" => Some(StandardBuiltinId::StringPrototypeReplaceAll),
                "search" => Some(StandardBuiltinId::StringPrototypeSearch),
                "indexOf" => Some(StandardBuiltinId::StringPrototypeIndexOf),
                "lastIndexOf" => Some(StandardBuiltinId::StringPrototypeLastIndexOf),
                "at" => Some(StandardBuiltinId::StringPrototypeAt),
                "slice" => Some(StandardBuiltinId::StringPrototypeSlice),
                "split" => Some(StandardBuiltinId::StringPrototypeSplit),
                "padStart" => Some(StandardBuiltinId::StringPrototypePadStart),
                "padEnd" => Some(StandardBuiltinId::StringPrototypePadEnd),
                "repeat" => Some(StandardBuiltinId::StringPrototypeRepeat),
                "endsWith" => Some(StandardBuiltinId::StringPrototypeEndsWith),
                "includes" => Some(StandardBuiltinId::StringPrototypeIncludes),
                "startsWith" => Some(StandardBuiltinId::StringPrototypeStartsWith),
                "toUpperCase" => Some(StandardBuiltinId::StringPrototypeToUpperCase),
                "toString" => Some(StandardBuiltinId::StringPrototypeToString),
                "valueOf" => Some(StandardBuiltinId::StringPrototypeValueOf),
                "isWellFormed" => Some(StandardBuiltinId::StringPrototypeIsWellFormed),
                "toWellFormed" => Some(StandardBuiltinId::StringPrototypeToWellFormed),
                "trim" => Some(StandardBuiltinId::StringPrototypeTrim),
                "trimStart" | "trimLeft" => Some(StandardBuiltinId::StringPrototypeTrimStart),
                "trimEnd" | "trimRight" => Some(StandardBuiltinId::StringPrototypeTrimEnd),
                _ => None,
            },
            _ => None,
        };
        if let Some(builtin) = string_html_builtin {
            let receiver_is_string = receiver
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::String));
            let receiver_has_string_builtin = match key {
                PropertyKeyIr::StaticString(name) => receiver
                    .heap_shape
                    .as_deref()
                    .and_then(|shape| read_static_heap_shape_property(shape, name))
                    .is_some_and(|property| match property {
                        ObjectShapeProperty::Data(info) => {
                            info.function_targets.contains(&builtin.function_id())
                        }
                        ObjectShapeProperty::Accessor { .. } => false,
                    }),
                _ => false,
            };
            if receiver_is_string || receiver_has_string_builtin {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        if receiver.kind == ValueKind::BigInt {
            let builtin = match key {
                PropertyKeyIr::StaticString(name) => match name.as_str() {
                    "toString" => Some(StandardBuiltinId::BigIntPrototypeToString),
                    "toLocaleString" => Some(StandardBuiltinId::BigIntPrototypeToLocaleString),
                    "valueOf" => Some(StandardBuiltinId::BigIntPrototypeValueOf),
                    _ => None,
                },
                _ => None,
            };
            if let Some(builtin) = builtin {
                let receiver_payload_local = self.reserve_temp_local();
                let receiver_tag_local = self.reserve_temp_local();
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    receiver,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(callee_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(callee_tag_local));
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_function_handle_call_with_argv(
                    callee_payload_local,
                    callee_tag_local,
                    Some((receiver_payload_local, Some(receiver_tag_local))),
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Ok(());
            }
        }
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        match receiver.kind {
            ValueKind::Object | ValueKind::Function | ValueKind::Dynamic => {
                let runtime_number_builtin = if receiver.kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::NumberPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "toLocaleString" => {
                            Some(StandardBuiltinId::NumberPrototypeToLocaleString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::NumberPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let runtime_string_builtin = if receiver.kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::StringPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::StringPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let runtime_bigint_builtin = if receiver.kind == ValueKind::Dynamic {
                    match key {
                        PropertyKeyIr::StaticString(name) if name == "toString" => {
                            Some(StandardBuiltinId::BigIntPrototypeToString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "toLocaleString" => {
                            Some(StandardBuiltinId::BigIntPrototypeToLocaleString)
                        }
                        PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                            Some(StandardBuiltinId::BigIntPrototypeValueOf)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(builtin) = runtime_number_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_string_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_bigint_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(callee_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(callee_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                if runtime_number_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
                if runtime_string_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
                if runtime_bigint_builtin.is_some() {
                    function.instruction(&Instruction::End);
                }
            }
            ValueKind::Array => {
                if matches!(
                    key,
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::StringExpr(_)
                ) {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    let own_found_local = self.reserve_temp_local();
                    let prototype_payload_local = self.reserve_temp_local();
                    let prototype_tag_local = self.reserve_temp_local();
                    self.emit_array_named_prop_read(
                        receiver_payload_local,
                        key_local,
                        callee_payload_local,
                        callee_tag_local,
                        Some(own_found_local),
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(own_found_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_PROTOTYPE_OFFSET,
                        prototype_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(prototype_payload_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Else);
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                        prototype_tag_local,
                        function,
                    );
                    self.emit_object_read(
                        prototype_payload_local,
                        prototype_tag_local,
                        receiver_payload_local,
                        receiver_tag_local,
                        key_local,
                        callee_payload_local,
                        callee_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(prototype_tag_local);
                    self.release_temp_local(prototype_payload_local);
                    self.release_temp_local(own_found_local);
                    self.release_temp_local(key_local);
                } else {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_array_read(
                        receiver_payload_local,
                        index_local,
                        callee_payload_local,
                        callee_tag_local,
                        function,
                    );
                    self.release_temp_local(index_local);
                }
            }
            ValueKind::Arguments => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.emit_arguments_read(
                    receiver_payload_local,
                    index_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            ValueKind::String => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Number => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Boolean => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Symbol => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(object_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    object_tag_local,
                    self.scratch_local,
                    object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(object_tag_local);
                self.release_temp_local(key_local);
            }
            _ => {
                self.release_temp_local(flags_local);
                self.release_temp_local(table_index_local);
                self.release_temp_local(callee_env_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: property access on non-object target",
                ));
            }
        }

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_call(
        &mut self,
        name: &str,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.values().find(|meta| meta.name == name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: direct call to unknown top-level function `{name}`"
            ))
        })?;
        // A direct call into a builtin's body requires its real body to be
        // emitted — see `FunctionMetaRegistry`.
        self.functions.record_builtin_meta(meta);
        let wasm_index = meta.wasm_index;
        let is_class_constructor = meta.class_kind == ClassFunctionKind::Constructor;
        let is_strict = meta.strict;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let callee_table_index_local = self.reserve_temp_local();

        if is_class_constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.throw_handler_stack.last() {
                function.instruction(&Instruction::Br(self.depth_to(*target)));
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            if let Some(storage) = self.lookup_binding(name) {
                self.read_binding_to_locals(
                    storage,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                );
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
            } else {
                let key_local = self.reserve_temp_local();
                let global_object_local = self.reserve_temp_local();
                let global_object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(global_object_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(global_object_tag_local));
                self.emit_object_read(
                    global_object_local,
                    global_object_tag_local,
                    global_object_local,
                    global_object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
                self.release_temp_local(global_object_tag_local);
                self.release_temp_local(global_object_local);
                self.release_temp_local(key_local);
            }
            self.emit_default_this_for_known_strictness(is_strict, function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        }
        self.release_temp_local(callee_table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_pre_evaluated_arg_vector(
        &mut self,
        args: &[(u32, u32)],
        argc_local: u32,
        argv_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let capacity = (args.len() as u64).max(MIN_HEAP_CAPACITY);

        function.instruction(&Instruction::I64Const(args.len() as i64));
        function.instruction(&Instruction::LocalSet(argc_local));
        // Argument vectors are built at every call site with pre-evaluated
        // args; go through the shared array-alloc helper (which performs the
        // full ~30-store header/slot init once) instead of inlining that init
        // at each site.
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            function.instruction(&Instruction::I64Const(args.len() as i64));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(argv_local));
        } else {
            self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(argv_local));
            self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(buffer_local));
            self.store_i64_local_at_offset(argv_local, HEAP_PTR_OFFSET, buffer_local, function);
            self.store_i64_const_at_offset(
                argv_local,
                HEAP_LEN_OFFSET,
                args.len() as u64,
                function,
            );
            self.store_i64_const_at_offset(argv_local, HEAP_CAP_OFFSET, capacity, function);
            function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                argv_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(
                argv_local,
                HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                ValueKind::Array.tag() as u64,
                function,
            );
            self.emit_init_array_constructor_slot(argv_local, function);
        }

        for (index, (arg_payload_local, arg_tag_local)) in args.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_TAG_OFFSET,
                *arg_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                *arg_payload_local,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                ARRAY_DESCRIPTOR_NORMAL_DATA,
                function,
            );
        }

        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_builtin_arg_to_locals(
        &mut self,
        index: usize,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let argc_local = self.argc_param_local();
        let argv_local = self.argv_param_local();
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(
            argv_local,
            self.scratch_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }
}
