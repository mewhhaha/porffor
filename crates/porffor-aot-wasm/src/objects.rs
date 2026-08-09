use super::*;

fn static_array_index_name(name: &str) -> Option<u64> {
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if name.len() > 1 && name.starts_with('0') {
        return None;
    }
    let index = name.parse::<u64>().ok()?;
    if index <= 4_294_967_294 {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn object_data_descriptor_kind(
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> u64 {
    let mut descriptor = OBJECT_DESCRIPTOR_DATA;
    if writable {
        descriptor |= OBJECT_DESCRIPTOR_WRITABLE;
    }
    if enumerable {
        descriptor |= OBJECT_DESCRIPTOR_ENUMERABLE;
    }
    if configurable {
        descriptor |= OBJECT_DESCRIPTOR_CONFIGURABLE;
    }
    descriptor
}

pub(crate) fn object_accessor_descriptor_kind(enumerable: bool, configurable: bool) -> u64 {
    let mut descriptor = OBJECT_DESCRIPTOR_ACCESSOR;
    if enumerable {
        descriptor |= OBJECT_DESCRIPTOR_ENUMERABLE;
    }
    if configurable {
        descriptor |= OBJECT_DESCRIPTOR_CONFIGURABLE;
    }
    descriptor
}

fn object_helper_store_i64_local_at_offset(
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

fn object_helper_store_i64_const_at_offset(
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

pub(crate) fn emit_plain_object_alloc_helper_function(heap_alloc_function_index: u32) -> Function {
    const PROTOTYPE_PAYLOAD_LOCAL: u32 = 0;
    const PROTOTYPE_TAG_LOCAL: u32 = 1;
    const OBJECT_LOCAL: u32 = 2;
    const BUFFER_LOCAL: u32 = 3;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 2));

    function.instruction(&Instruction::I64Const(HEAP_HEADER_SIZE as i64));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(OBJECT_LOCAL));
    function.instruction(&Instruction::I64Const(
        (MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE) as i64,
    ));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));

    object_helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_PTR_OFFSET,
        BUFFER_LOCAL,
    );
    object_helper_store_i64_const_at_offset(&mut function, OBJECT_LOCAL, HEAP_LEN_OFFSET, 0);
    object_helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_CAP_OFFSET,
        MIN_HEAP_CAPACITY as i64,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
        0,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_OBJECT_BOXED_KIND_OFFSET,
        BOXED_PRIMITIVE_KIND_NONE as i64,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_OBJECT_BOXED_TAG_OFFSET,
        0,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
        0,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_PROTOTYPE_OFFSET,
        PROTOTYPE_PAYLOAD_LOCAL,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        OBJECT_LOCAL,
        HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
        PROTOTYPE_TAG_LOCAL,
    );

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::End);
    function
}

pub(crate) fn emit_object_append_data_property_helper_function(
    heap_alloc_function_index: u32,
) -> Function {
    const OBJECT_LOCAL: u32 = 0;
    const KEY_LOCAL: u32 = 1;
    const PAYLOAD_LOCAL: u32 = 2;
    const TAG_LOCAL: u32 = 3;
    const DESCRIPTOR_KIND_LOCAL: u32 = 4;
    const BUFFER_LOCAL: u32 = 5;
    const LEN_LOCAL: u32 = 6;
    const CAP_LOCAL: u32 = 7;
    const ENTRY_LOCAL: u32 = 8;
    const NEW_CAP_LOCAL: u32 = 9;
    const SIZE_LOCAL: u32 = 10;
    const NEW_BUFFER_LOCAL: u32 = 11;
    const INDEX_LOCAL: u32 = 12;
    const OLD_ENTRY_LOCAL: u32 = 13;
    const NEW_ENTRY_LOCAL: u32 = 14;
    const SCRATCH_LOCAL: u32 = 15;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 11));

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_PTR_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_LEN_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_CAP_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));

    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Const(2));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalSet(NEW_CAP_LOCAL));

    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::LocalSet(SIZE_LOCAL));
    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(NEW_BUFFER_LOCAL));

    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(INDEX_LOCAL));
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));

    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(OLD_ENTRY_LOCAL));
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(NEW_ENTRY_LOCAL));

    for offset in [
        HEAP_OBJECT_KEY_OFFSET,
        HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
        HEAP_OBJECT_DATA_TAG_OFFSET,
        HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
        HEAP_OBJECT_GETTER_TAG_OFFSET,
        HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
        HEAP_OBJECT_SETTER_TAG_OFFSET,
        HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
    ] {
        function.instruction(&Instruction::LocalGet(OLD_ENTRY_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(offset)));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        function.instruction(&Instruction::LocalGet(NEW_ENTRY_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(SCRATCH_LOCAL));
        function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
    }

    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(INDEX_LOCAL));
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_PTR_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_CAP_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(ENTRY_LOCAL));
    function.instruction(&Instruction::LocalGet(ENTRY_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(KEY_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_OBJECT_KEY_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(ENTRY_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(DESCRIPTOR_KIND_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(ENTRY_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(TAG_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_OBJECT_DATA_TAG_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(ENTRY_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(PAYLOAD_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
    )));
    for offset in [
        HEAP_OBJECT_GETTER_TAG_OFFSET,
        HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
        HEAP_OBJECT_SETTER_TAG_OFFSET,
        HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
    ] {
        function.instruction(&Instruction::LocalGet(ENTRY_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
    }

    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_LEN_OFFSET,
    )));
    function.instruction(&Instruction::End);
    function
}

pub(crate) fn emit_object_append_accessor_property_helper_function(
    heap_alloc_function_index: u32,
) -> Function {
    const OBJECT_LOCAL: u32 = 0;
    const KEY_LOCAL: u32 = 1;
    const GETTER_PAYLOAD_LOCAL: u32 = 2;
    const GETTER_TAG_LOCAL: u32 = 3;
    const SETTER_PAYLOAD_LOCAL: u32 = 4;
    const SETTER_TAG_LOCAL: u32 = 5;
    const DESCRIPTOR_KIND_LOCAL: u32 = 6;
    const BUFFER_LOCAL: u32 = 7;
    const LEN_LOCAL: u32 = 8;
    const CAP_LOCAL: u32 = 9;
    const ENTRY_LOCAL: u32 = 10;
    const NEW_CAP_LOCAL: u32 = 11;
    const SIZE_LOCAL: u32 = 12;
    const NEW_BUFFER_LOCAL: u32 = 13;
    const INDEX_LOCAL: u32 = 14;
    const OLD_ENTRY_LOCAL: u32 = 15;
    const NEW_ENTRY_LOCAL: u32 = 16;
    const SCRATCH_LOCAL: u32 = 17;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 11));

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_PTR_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_LEN_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
        HEAP_CAP_OFFSET,
    )));
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));

    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::LocalGet(CAP_LOCAL));
    function.instruction(&Instruction::I64Const(2));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalSet(NEW_CAP_LOCAL));

    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::LocalSet(SIZE_LOCAL));
    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::Call(heap_alloc_function_index));
    function.instruction(&Instruction::LocalSet(NEW_BUFFER_LOCAL));

    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(INDEX_LOCAL));
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));

    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(OLD_ENTRY_LOCAL));
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(NEW_ENTRY_LOCAL));

    for offset in [
        HEAP_OBJECT_KEY_OFFSET,
        HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
        HEAP_OBJECT_DATA_TAG_OFFSET,
        HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
        HEAP_OBJECT_GETTER_TAG_OFFSET,
        HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
        HEAP_OBJECT_SETTER_TAG_OFFSET,
        HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
    ] {
        function.instruction(&Instruction::LocalGet(OLD_ENTRY_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(offset)));
        function.instruction(&Instruction::LocalSet(SCRATCH_LOCAL));
        function.instruction(&Instruction::LocalGet(NEW_ENTRY_LOCAL));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(SCRATCH_LOCAL));
        function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(offset)));
    }

    function.instruction(&Instruction::LocalGet(INDEX_LOCAL));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(INDEX_LOCAL));
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_PTR_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_CAP_OFFSET,
    )));
    function.instruction(&Instruction::LocalGet(NEW_BUFFER_LOCAL));
    function.instruction(&Instruction::LocalSet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(NEW_CAP_LOCAL));
    function.instruction(&Instruction::LocalSet(CAP_LOCAL));
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(BUFFER_LOCAL));
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(ENTRY_LOCAL));
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_KEY_OFFSET,
        KEY_LOCAL,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
        DESCRIPTOR_KIND_LOCAL,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_DATA_TAG_OFFSET,
        0,
    );
    object_helper_store_i64_const_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
        0,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_GETTER_TAG_OFFSET,
        GETTER_TAG_LOCAL,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
        GETTER_PAYLOAD_LOCAL,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_SETTER_TAG_OFFSET,
        SETTER_TAG_LOCAL,
    );
    object_helper_store_i64_local_at_offset(
        &mut function,
        ENTRY_LOCAL,
        HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        SETTER_PAYLOAD_LOCAL,
    );

    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(LEN_LOCAL));
    function.instruction(&Instruction::LocalGet(OBJECT_LOCAL));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(LEN_LOCAL));
    function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(
        HEAP_LEN_OFFSET,
    )));
    function.instruction(&Instruction::End);
    function
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_is_array_i64(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
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
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_plain_object_with_prototype(
        &mut self,
        prototype_local: Option<u32>,
        prototype_global: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_plain_object_with_prototype_and_tag(
            prototype_local,
            None,
            prototype_global,
            function,
        )
    }

    pub(crate) fn emit_alloc_plain_object_with_prototype_and_tag(
        &mut self,
        prototype_local: Option<u32>,
        prototype_tag_local: Option<u32>,
        prototype_global: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(plain_object_alloc_function_index) = self.plain_object_alloc_function_index {
            if let Some(prototype_local) = prototype_local {
                function.instruction(&Instruction::LocalGet(prototype_local));
                if let Some(prototype_tag_local) = prototype_tag_local {
                    function.instruction(&Instruction::LocalGet(prototype_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                }
            } else if let Some(prototype_global) = prototype_global {
                function.instruction(&Instruction::GlobalGet(prototype_global));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            }
            function.instruction(&Instruction::Call(plain_object_alloc_function_index));
            return Ok(());
        }
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_PAYLOAD_OFFSET, 0, function);
        if let Some(prototype_local) = prototype_local {
            self.store_i64_local_at_offset(
                object_local,
                HEAP_PROTOTYPE_OFFSET,
                prototype_local,
                function,
            );
            if let Some(prototype_tag_local) = prototype_tag_local {
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                    prototype_tag_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Object.tag() as u64,
                    function,
                );
            }
        } else if let Some(prototype_global) = prototype_global {
            function.instruction(&Instruction::GlobalGet(prototype_global));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(
                object_local,
                HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                ValueKind::Object.tag() as u64,
                function,
            );
        } else {
            self.store_i64_const_at_offset(object_local, HEAP_PROTOTYPE_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                object_local,
                HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                ValueKind::Null.tag() as u64,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_store_boxed_primitive_metadata(
        &mut self,
        object_local: u32,
        boxed_kind: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
    }

    pub(crate) fn emit_object_boxed_kind_for_tag(
        &self,
        object_local: u32,
        object_tag_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_from_offset(object_local, HEAP_OBJECT_BOXED_KIND_OFFSET, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    pub(crate) fn emit_load_prototype_to_current_locals(
        &self,
        current_local: u32,
        current_tag_local: u32,
        prototype_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            current_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            current_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            current_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::LocalSet(current_local));
    }

    pub(crate) fn emit_alloc_boxed_wrapper_from_locals(
        &mut self,
        prototype_global_index: u32,
        boxed_kind: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_boxed_wrapper_with_prototype_from_locals(
            prototype_local,
            boxed_kind,
            value_payload_local,
            value_tag_local,
            result_payload_local,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_boxed_wrapper_with_prototype_from_locals(
        &mut self,
        prototype_local: u32,
        boxed_kind: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_store_boxed_primitive_metadata(
            object_local,
            boxed_kind,
            value_payload_local,
            value_tag_local,
            function,
        );
        if boxed_kind == BOXED_PRIMITIVE_KIND_STRING {
            let key_local = self.reserve_temp_local();
            let length_payload_local = self.reserve_temp_local();
            let length_tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload("length")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_boxed_string_length_number_payload(
                value_payload_local,
                length_payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(length_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                length_payload_local,
                length_tag_local,
                function,
            )?;
            self.release_temp_local(length_tag_local);
            self.release_temp_local(length_payload_local);
            self.release_temp_local(key_local);
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_string_data(
        &mut self,
        object_local: u32,
        key: &str,
        value: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(value)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        if self.is_main() {
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                false,
                true,
                function,
            )?;
        } else {
            self.emit_object_define_data(
                object_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_function_data(
        &mut self,
        object_local: u32,
        key: &str,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // No per-method plan gate here: once a bootstrap arm runs, every
        // property it defines must actually exist (the spec makes them all
        // observable, e.g. via Object.getOwnPropertyDescriptor, without any
        // call ever happening). The materialization below records the builtin
        // in the FunctionMetaRegistry, which guarantees its real body is
        // emitted, so installing unconditionally is safe.
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        if self.is_main() {
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                false,
                true,
                function,
            )?;
        } else {
            self.emit_object_define_data(
                object_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    /// Define a canonical builtin property and any Annex B aliases with one
    /// function object. Alias properties must not independently materialize
    /// their function value: ECMAScript observes their identity and canonical
    /// `name` through ordinary property reads.
    pub(crate) fn emit_object_define_function_data_with_aliases(
        &mut self,
        object_local: u32,
        key: &str,
        aliases: &[&str],
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        for property in std::iter::once(key).chain(aliases.iter().copied()) {
            function.instruction(&Instruction::I64Const(
                self.strings.static_builtin_property_key_payload(property),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            if self.is_main() {
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
            } else {
                self.emit_object_define_data(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_function_global_data(
        &mut self,
        object_local: u32,
        key: &str,
        global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        if self.is_main() {
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                false,
                true,
                function,
            )?;
        } else {
            self.emit_object_define_data(
                object_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    /// Get-or-create the canonical function object held in `global_index`,
    /// allocating it from `meta` the first time. Used so a realm's global
    /// `parseInt`/`parseFloat` and its `Number.parseInt`/`Number.parseFloat`
    /// resolve to a single function-object identity (spec: they are the same
    /// object). No realm-error-prototype wiring here — the main realm relies on
    /// the default-realm throw fallback.
    pub(crate) fn emit_ensure_canonical_host_function(
        &mut self,
        meta: &WasmFunctionMeta,
        global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::GlobalSet(global_index));
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Realm-aware variant: get-or-create the canonical function object in
    /// `global_index`, wiring the defining realm and realm TypeError prototype
    /// on first allocation, then define it as an own data property of
    /// `object_local`. Both the global-object and `Number` install sites call
    /// this within one realm build so they share the same object.
    pub(crate) fn emit_define_canonical_realm_host_function(
        &mut self,
        object_local: u32,
        name: &str,
        meta: &WasmFunctionMeta,
        global_index: u32,
        realm_record_local: u32,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_store_function_defining_realm(payload_local, realm_record_local, function);
        self.store_i64_local_at_offset(
            payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::GlobalSet(global_index));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(global_index));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(object_local, name, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_append_data_property_with_flags(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(append_function_index) = self.object_append_data_property_function_index {
            if self.is_main() {
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                let writable_local = self.reserve_temp_local();
                let enumerable_local = self.reserve_temp_local();
                let configurable_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(i64::from(writable)));
                function.instruction(&Instruction::LocalSet(writable_local));
                function.instruction(&Instruction::I64Const(i64::from(enumerable)));
                function.instruction(&Instruction::LocalSet(enumerable_local));
                function.instruction(&Instruction::I64Const(i64::from(configurable)));
                function.instruction(&Instruction::LocalSet(configurable_local));
                self.emit_array_define_named_data_descriptor(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    writable_local,
                    enumerable_local,
                    configurable_local,
                    None,
                    None,
                    None,
                    None,
                    None,
                    function,
                )?;
                self.release_temp_local(configurable_local);
                self.release_temp_local(enumerable_local);
                self.release_temp_local(writable_local);
                function.instruction(&Instruction::Else);
            }
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalGet(key_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(object_data_descriptor_kind(
                writable,
                enumerable,
                configurable,
            ) as i64));
            function.instruction(&Instruction::Call(append_function_index));
            if self.is_main() {
                function.instruction(&Instruction::End);
            }
            return Ok(());
        }
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            object_data_descriptor_kind(writable, enumerable, configurable),
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_object_append_accessor_property_with_flags(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let missing_getter_payload_local = getter.is_none().then(|| self.reserve_temp_local());
        let missing_getter_tag_local = getter.is_none().then(|| self.reserve_temp_local());
        if let (Some(payload_local), Some(tag_local)) =
            (missing_getter_payload_local, missing_getter_tag_local)
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        let missing_setter_payload_local = setter.is_none().then(|| self.reserve_temp_local());
        let missing_setter_tag_local = setter.is_none().then(|| self.reserve_temp_local());
        if let (Some(payload_local), Some(tag_local)) =
            (missing_setter_payload_local, missing_setter_tag_local)
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        let getter = getter.or_else(|| {
            Some((
                missing_getter_payload_local.expect("missing getter payload local"),
                missing_getter_tag_local.expect("missing getter tag local"),
            ))
        });
        let setter = setter.or_else(|| {
            Some((
                missing_setter_payload_local.expect("missing setter payload local"),
                missing_setter_tag_local.expect("missing setter tag local"),
            ))
        });

        if let Some(append_function_index) = self.object_append_accessor_property_function_index {
            let (getter_payload_local, getter_tag_local) =
                getter.expect("getter locals must be materialized");
            let (setter_payload_local, setter_tag_local) =
                setter.expect("setter locals must be materialized");
            if self.is_main() {
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                let enumerable_local = self.reserve_temp_local();
                let configurable_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(i64::from(enumerable)));
                function.instruction(&Instruction::LocalSet(enumerable_local));
                function.instruction(&Instruction::I64Const(i64::from(configurable)));
                function.instruction(&Instruction::LocalSet(configurable_local));
                self.emit_array_define_named_accessor_descriptor(
                    object_local,
                    key_local,
                    getter_payload_local,
                    getter_tag_local,
                    setter_payload_local,
                    setter_tag_local,
                    enumerable_local,
                    configurable_local,
                    None,
                    None,
                    None,
                    None,
                    None,
                    function,
                )?;
                self.release_temp_local(configurable_local);
                self.release_temp_local(enumerable_local);
                function.instruction(&Instruction::Else);
            }
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalGet(key_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::I64Const(object_accessor_descriptor_kind(
                enumerable,
                configurable,
            ) as i64));
            function.instruction(&Instruction::Call(append_function_index));
            if self.is_main() {
                function.instruction(&Instruction::End);
            }
        } else {
            let configurable_payload_local = self.reserve_temp_local();
            let enumerable_payload_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(i64::from(enumerable)));
            function.instruction(&Instruction::LocalSet(enumerable_payload_local));
            function.instruction(&Instruction::I64Const(i64::from(configurable)));
            function.instruction(&Instruction::LocalSet(configurable_payload_local));
            self.emit_object_define_accessor_with_flag_local(
                object_local,
                key_local,
                getter,
                setter,
                enumerable_payload_local,
                configurable_payload_local,
                function,
            )?;
            self.release_temp_local(enumerable_payload_local);
            self.release_temp_local(configurable_payload_local);
        }

        if let Some(tag_local) = missing_setter_tag_local {
            self.release_temp_local(tag_local);
        }
        if let Some(payload_local) = missing_setter_payload_local {
            self.release_temp_local(payload_local);
        }
        if let Some(tag_local) = missing_getter_tag_local {
            self.release_temp_local(tag_local);
        }
        if let Some(payload_local) = missing_getter_payload_local {
            self.release_temp_local(payload_local);
        }
        Ok(())
    }

    pub(crate) fn emit_validate_array_named_descriptor(
        &mut self,
        entry_local: u32,
        existing_descriptor_kind_local: u32,
        requested_data_descriptor: bool,
        value: Option<(u32, u32, Option<u32>)>,
        writable: Option<(u32, Option<u32>)>,
        getter: Option<(u32, u32, Option<u32>)>,
        setter: Option<(u32, u32, Option<u32>)>,
        enumerable: (u32, Option<u32>),
        configurable: (u32, Option<u32>),
        validation_success_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stored_tag_local = self.reserve_temp_local();
        let stored_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        if let Some(configurable_present_local) = configurable.1 {
            function.instruction(&Instruction::LocalGet(configurable_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::LocalGet(configurable.0));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(validation_success_local));
            function.instruction(&Instruction::End);
        }
        if let Some(enumerable_present_local) = enumerable.1 {
            function.instruction(&Instruction::LocalGet(enumerable_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::LocalGet(enumerable.0));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(validation_success_local));
            function.instruction(&Instruction::End);
        }

        let kind_present_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(kind_present_local));
        for present_local in [
            value.and_then(|(_, _, present_local)| present_local),
            writable.and_then(|(_, present_local)| present_local),
            getter.and_then(|(_, _, present_local)| present_local),
            setter.and_then(|(_, _, present_local)| present_local),
        ] {
            if let Some(present_local) = present_local {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(kind_present_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(kind_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Const(i32::from(requested_data_descriptor)));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(validation_success_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if requested_data_descriptor {
            if let Some((writable_payload_local, Some(writable_present_local))) = writable {
                function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(writable_present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::LocalGet(writable_payload_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(validation_success_local));
                function.instruction(&Instruction::End);
            }
            if let Some((value_payload_local, value_tag_local, Some(value_present_local))) = value {
                function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
                function.instruction(&Instruction::I64Const(
                    (OBJECT_DESCRIPTOR_ACCESSOR | OBJECT_DESCRIPTOR_WRITABLE) as i64,
                ));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(value_present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_DATA_TAG_OFFSET,
                    stored_tag_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                    stored_payload_local,
                    function,
                );
                self.emit_tagged_payload_same_value_i32(
                    value_tag_local,
                    value_payload_local,
                    stored_tag_local,
                    stored_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(validation_success_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        } else {
            for (
                requested_payload_local,
                requested_tag_local,
                present_local,
                tag_offset,
                payload_offset,
            ) in [
                getter.map(|(payload, tag, present)| {
                    (
                        payload,
                        tag,
                        present,
                        HEAP_OBJECT_GETTER_TAG_OFFSET,
                        HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    )
                }),
                setter.map(|(payload, tag, present)| {
                    (
                        payload,
                        tag,
                        present,
                        HEAP_OBJECT_SETTER_TAG_OFFSET,
                        HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    )
                }),
            ]
            .into_iter()
            .flatten()
            {
                if let Some(present_local) = present_local {
                    function.instruction(&Instruction::LocalGet(present_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        entry_local,
                        tag_offset,
                        stored_tag_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        entry_local,
                        payload_offset,
                        stored_payload_local,
                        function,
                    );
                    self.emit_tagged_payload_same_value_i32(
                        requested_tag_local,
                        requested_payload_local,
                        stored_tag_local,
                        stored_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(validation_success_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
            }
        }

        self.release_temp_local(kind_present_local);
        function.instruction(&Instruction::End);
        self.release_temp_local(stored_payload_local);
        self.release_temp_local(stored_tag_local);
        Ok(())
    }

    pub(crate) fn emit_object_append_local_data_property_with_flags(
        &mut self,
        object_local: u32,
        key: &str,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            writable,
            enumerable,
            configurable,
            function,
        )?;
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_overwrite_own_data_or_define(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_set_function_prototype_data(
        &mut self,
        function_object_local: u32,
        prototype_local: u32,
        define_prototype_constructor: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_set_function_prototype_data_with_flags(
            function_object_local,
            prototype_local,
            true,
            false,
            true,
            define_prototype_constructor,
            function,
        )
    }

    pub(crate) fn emit_set_function_prototype_data_with_flags(
        &mut self,
        function_object_local: u32,
        prototype_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        define_prototype_constructor: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        if writable && !enumerable && configurable {
            self.emit_object_overwrite_own_data_or_define(
                function_object_local,
                key_local,
                prototype_local,
                tag_local,
                function,
            )?;
        } else {
            self.emit_object_define_data_with_configurable(
                function_object_local,
                key_local,
                prototype_local,
                tag_local,
                writable,
                enumerable,
                configurable,
                function,
            )?;
        }
        if define_prototype_constructor {
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data(
                prototype_local,
                "constructor",
                function_object_local,
                tag_local,
                function,
            )?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_number_data_from_i64_local(
        &mut self,
        object_local: u32,
        key: &str,
        value_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_number_data_from_i64_const(
        &mut self,
        object_local: u32,
        key: &str,
        value: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(value as i64));
        function.instruction(&Instruction::LocalSet(value_local));
        self.emit_object_define_number_data_from_i64_local(
            object_local,
            key,
            value_local,
            function,
        )?;
        self.release_temp_local(value_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_number_data_from_f64_const_with_flags(
        &mut self,
        object_local: u32,
        key: &str,
        value: f64,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(value)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        if self.is_main() {
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                writable,
                enumerable,
                configurable,
                function,
            )?;
        } else {
            self.emit_object_define_data_with_configurable(
                object_local,
                key_local,
                payload_local,
                tag_local,
                writable,
                enumerable,
                configurable,
                function,
            )?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_bool_data(
        &mut self,
        object_local: u32,
        key: &str,
        value: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(i64::from(value)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_bool_data_from_local(
        &mut self,
        object_local: u32,
        key: &str,
        value_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            object_local,
            key,
            value_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(tag_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_local_data(
        &mut self,
        object_local: u32,
        key: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_local_data_with_flags(
        &mut self,
        object_local: u32,
        key: &str,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            writable,
            enumerable,
            configurable,
            function,
        )?;
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_read_number_slot_to_i64_local(
        &mut self,
        object_local: u32,
        key: &str,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(dest_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_data_descriptor_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_local));
        self.emit_object_define_local_data(
            descriptor_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data(descriptor_local, "writable", writable, function)?;
        self.emit_object_define_bool_data(descriptor_local, "enumerable", enumerable, function)?;
        self.emit_object_define_bool_data(
            descriptor_local,
            "configurable",
            configurable,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(descriptor_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_value_descriptor_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_local));
        self.emit_object_define_local_data(
            descriptor_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(descriptor_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_data_property_descriptor_object_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_local));
        self.emit_object_define_local_data(
            descriptor_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data(descriptor_local, "writable", writable, function)?;
        self.emit_object_define_bool_data(descriptor_local, "enumerable", enumerable, function)?;
        self.emit_object_define_bool_data(
            descriptor_local,
            "configurable",
            configurable,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(descriptor_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_data_descriptor_from_locals_with_flag_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_local));
        self.emit_object_define_local_data(
            descriptor_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            descriptor_local,
            "writable",
            writable_payload_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            descriptor_local,
            "enumerable",
            enumerable_payload_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            descriptor_local,
            "configurable",
            configurable_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(descriptor_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_accessor_descriptor_from_locals_with_flag_local(
        &mut self,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_local));
        self.emit_object_define_local_data(
            descriptor_local,
            "get",
            getter_payload_local,
            getter_tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            descriptor_local,
            "set",
            setter_payload_local,
            setter_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            descriptor_local,
            "enumerable",
            enumerable_payload_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            descriptor_local,
            "configurable",
            configurable_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(descriptor_local);
        Ok(())
    }

    pub(crate) fn compile_object_literal_payload(
        &mut self,
        properties: &[ObjectPropertyIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let capacity = (properties
            .iter()
            .filter(|property| !matches!(property, ObjectPropertyIr::PrototypeSetter { .. }))
            .count() as u64)
            .max(MIN_HEAP_CAPACITY);
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(capacity * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, capacity, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );

        let mut property_index = 0;
        while property_index < properties.len() {
            let property = &properties[property_index];
            if let ObjectPropertyIr::PrototypeSetter { value } = property {
                let value_payload = self.reserve_temp_local();
                let value_tag = self.reserve_temp_local();
                self.compile_expr_to_locals(value, value_payload, value_tag, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    value_payload,
                    value_tag,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    value_payload,
                    function,
                );
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                    value_tag,
                    function,
                );
                function.instruction(&Instruction::Else);
                self.emit_is_heap_object_like_tag_i32(value_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    value_payload,
                    function,
                );
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
                    value_tag,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(value_tag);
                self.release_temp_local(value_payload);
                property_index += 1;
                continue;
            }
            if let ObjectPropertyIr::Spread { source } = property {
                let source_payload_local = self.reserve_temp_local();
                let source_tag_local = self.reserve_temp_local();
                let source_object_payload_local = self.reserve_temp_local();
                let source_object_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    source,
                    source_payload_local,
                    source_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    source_payload_local,
                    source_tag_local,
                    function,
                )?;
                self.compile_nullish_tagged_i32(source_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                function.instruction(&Instruction::Else);
                self.emit_value_to_object_locals(
                    source_payload_local,
                    source_tag_local,
                    source_object_payload_local,
                    source_object_tag_local,
                    function,
                )?;
                self.emit_copy_data_properties_into(
                    source_object_payload_local,
                    source_object_tag_local,
                    &[],
                    object_local,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.release_temp_local(source_object_tag_local);
                self.release_temp_local(source_object_payload_local);
                self.release_temp_local(source_tag_local);
                self.release_temp_local(source_payload_local);
                property_index += 1;
                continue;
            }
            let key_local = self.reserve_temp_local();
            match property {
                ObjectPropertyIr::PrototypeSetter { .. } | ObjectPropertyIr::Spread { .. } => {
                    unreachable!()
                }
                ObjectPropertyIr::Data { key, .. }
                | ObjectPropertyIr::NonEnumerableData { key, .. }
                | ObjectPropertyIr::Method { key, .. }
                | ObjectPropertyIr::Getter { key, .. }
                | ObjectPropertyIr::Setter { key, .. } => {
                    function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                    function.instruction(&Instruction::LocalSet(key_local));
                }
                ObjectPropertyIr::ComputedData { key, .. }
                | ObjectPropertyIr::ComputedMethod { key, .. }
                | ObjectPropertyIr::ComputedGetter { key, .. }
                | ObjectPropertyIr::ComputedSetter { key, .. } => {
                    let key_payload = self.reserve_temp_local();
                    let key_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(key, key_payload, key_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        key_payload,
                        key_tag,
                        function,
                    )?;
                    self.emit_value_to_property_key_payload(key_payload, key_tag, function)?;
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.release_temp_local(key_tag);
                    self.release_temp_local(key_payload);
                }
            }
            match property {
                ObjectPropertyIr::PrototypeSetter { .. } | ObjectPropertyIr::Spread { .. } => {
                    unreachable!()
                }
                ObjectPropertyIr::NonEnumerableData { key, value } => {
                    let value_payload = self.reserve_temp_local();
                    let value_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(value, value_payload, value_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        value_payload,
                        value_tag,
                        function,
                    )?;
                    if key == "lastIndex" {
                        self.emit_object_define_data_with_configurable(
                            object_local,
                            key_local,
                            value_payload,
                            value_tag,
                            true,
                            false,
                            false,
                            function,
                        )?;
                    } else if matches!(
                        key.as_str(),
                        "source"
                            | "flags"
                            | "hasIndices"
                            | "global"
                            | "ignoreCase"
                            | "multiline"
                            | "dotAll"
                            | "unicode"
                            | "sticky"
                    ) {
                        self.emit_object_define_data_with_configurable(
                            object_local,
                            key_local,
                            value_payload,
                            value_tag,
                            false,
                            false,
                            true,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_data(
                            object_local,
                            key_local,
                            value_payload,
                            value_tag,
                            function,
                        )?;
                    }
                    self.release_temp_local(value_tag);
                    self.release_temp_local(value_payload);
                }
                ObjectPropertyIr::Data { value, .. }
                | ObjectPropertyIr::ComputedData { value, .. }
                | ObjectPropertyIr::Method {
                    function: value, ..
                }
                | ObjectPropertyIr::ComputedMethod {
                    function: value, ..
                } => {
                    let value_payload = self.reserve_temp_local();
                    let value_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(value, value_payload, value_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        value_payload,
                        value_tag,
                        function,
                    )?;
                    self.emit_object_define_enumerable_data(
                        object_local,
                        key_local,
                        value_payload,
                        value_tag,
                        function,
                    )?;
                    self.release_temp_local(value_tag);
                    self.release_temp_local(value_payload);
                }
                ObjectPropertyIr::Getter {
                    key: getter_key,
                    function: getter,
                } => {
                    let getter_payload = self.reserve_temp_local();
                    let getter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(getter, getter_payload, getter_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        getter_payload,
                        getter_tag,
                        function,
                    )?;
                    let paired_setter =
                        properties
                            .get(property_index + 1)
                            .and_then(|next| match next {
                                ObjectPropertyIr::Setter {
                                    key: setter_key,
                                    function,
                                } if setter_key == getter_key => Some(function),
                                _ => None,
                            });
                    let setter_locals = if let Some(setter) = paired_setter {
                        let setter_payload = self.reserve_temp_local();
                        let setter_tag = self.reserve_temp_local();
                        self.compile_expr_to_locals(setter, setter_payload, setter_tag, function)?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            setter_payload,
                            setter_tag,
                            function,
                        )?;
                        Some((setter_payload, setter_tag))
                    } else {
                        None
                    };
                    self.emit_object_define_enumerable_accessor(
                        object_local,
                        key_local,
                        Some((getter_payload, getter_tag)),
                        setter_locals,
                        function,
                    )?;
                    if let Some((setter_payload, setter_tag)) = setter_locals {
                        self.release_temp_local(setter_tag);
                        self.release_temp_local(setter_payload);
                        property_index += 1;
                    }
                    self.release_temp_local(getter_tag);
                    self.release_temp_local(getter_payload);
                }
                ObjectPropertyIr::ComputedGetter {
                    function: getter, ..
                } => {
                    let getter_payload = self.reserve_temp_local();
                    let getter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(getter, getter_payload, getter_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        getter_payload,
                        getter_tag,
                        function,
                    )?;
                    self.emit_object_define_enumerable_accessor(
                        object_local,
                        key_local,
                        Some((getter_payload, getter_tag)),
                        None,
                        function,
                    )?;
                    self.release_temp_local(getter_tag);
                    self.release_temp_local(getter_payload);
                }
                ObjectPropertyIr::Setter {
                    key: setter_key,
                    function: setter,
                } => {
                    if property_index > 0
                        && matches!(
                            &properties[property_index - 1],
                            ObjectPropertyIr::Getter { key, .. } if key == setter_key
                        )
                    {
                        self.release_temp_local(key_local);
                        property_index += 1;
                        continue;
                    }
                    let setter_payload = self.reserve_temp_local();
                    let setter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(setter, setter_payload, setter_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        setter_payload,
                        setter_tag,
                        function,
                    )?;
                    self.emit_object_define_enumerable_accessor(
                        object_local,
                        key_local,
                        None,
                        Some((setter_payload, setter_tag)),
                        function,
                    )?;
                    self.release_temp_local(setter_tag);
                    self.release_temp_local(setter_payload);
                }
                ObjectPropertyIr::ComputedSetter {
                    function: setter, ..
                } => {
                    let setter_payload = self.reserve_temp_local();
                    let setter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(setter, setter_payload, setter_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        setter_payload,
                        setter_tag,
                        function,
                    )?;
                    self.emit_object_define_enumerable_accessor(
                        object_local,
                        key_local,
                        None,
                        Some((setter_payload, setter_tag)),
                        function,
                    )?;
                    self.release_temp_local(setter_tag);
                    self.release_temp_local(setter_payload);
                }
            }
            self.release_temp_local(key_local);
            property_index += 1;
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn compile_property_read_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_local, target_tag_local, function)?;

        let target_may_be_nullish = target.possible_kinds.contains(ValueKind::Undefined)
            || target.possible_kinds.contains(ValueKind::Null);
        let computed_key_precedes_nullish_check = matches!(
            key,
            PropertyKeyIr::StringExpr(_) | PropertyKeyIr::ArrayIndex(_)
        ) && target_may_be_nullish;
        if !computed_key_precedes_nullish_check {
            // Optional-chain lowering bypasses this wrapper and performs its
            // own per-operation nullish check.
            self.compile_nullish_tagged_i32(target_tag_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Cannot read properties of null or undefined",
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                payload_local,
                tag_local,
                1,
                function,
            )?;
            function.instruction(&Instruction::End);

            if target.kind != ValueKind::Dynamic
                && target.possible_kinds.is_subset_of(KindSet::NULLISH)
            {
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Ok(());
            }
        }

        let checked_dynamic_target = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: if computed_key_precedes_nullish_check {
                    target.possible_kinds
                } else {
                    target
                        .possible_kinds
                        .without(ValueKind::Undefined)
                        .without(ValueKind::Null)
                },
                heap_shape: target.heap_shape.clone(),
                function_targets: target.function_targets.clone(),
            },
            ExprIr::Undefined,
        );

        let result = self.compile_property_read_from_locals(
            if matches!(
                target.kind,
                ValueKind::Dynamic | ValueKind::Undefined | ValueKind::Null
            ) {
                &checked_dynamic_target
            } else {
                target
            },
            key,
            target_local,
            target_tag_local,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        result
    }

    /// Emits a property read for an already-evaluated receiver. The caller owns
    /// the receiver locals, which permits compound expressions to retain and
    /// reuse them across multiple property accesses.
    pub(crate) fn compile_property_read_from_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        target_local: u32,
        target_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Array element and length reads use the Array exotic representation.
        // Every other property is an ordinary, observable lookup: Array.prototype
        // is mutable and may be reached through arbitrary aliases, so a static
        // method shortcut is unsound without a runtime prototype-version guard.
        let array_named_read_needs_ordinary_get = target.kind == ValueKind::Array
            && matches!(
                key,
                PropertyKeyIr::StaticString(name)
                    if name != "length"
                        && static_array_index_name(name).is_none()
                        && !matches!(
                            name.as_str(),
                            "index" | "input" | "Symbol.isConcatSpreadable"
                        )
            );

        if target.kind == ValueKind::Dynamic {
            return self.compile_dynamic_property_read_from_locals(
                target.possible_kinds,
                key,
                target_local,
                target_tag_local,
                payload_local,
                tag_local,
                function,
            );
        }

        match target.kind {
            ValueKind::Object | ValueKind::Function | ValueKind::Dynamic => {
                if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    let key_payload_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(
                        key,
                        key_payload_local,
                        key_tag_local,
                        function,
                    )?;
                    self.emit_dynamic_property_read_with_key_locals(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_payload_local,
                        key_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_payload_local);
                    return Ok(());
                }
                let dynamic_symbol_description = target.kind == ValueKind::Dynamic
                    && matches!(key, PropertyKeyIr::StaticString(name) if name == "description");
                let runtime_number_builtin = match key {
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
                };
                let runtime_string_builtin = match key {
                    PropertyKeyIr::StaticString(name) if name == "match" => {
                        Some(StandardBuiltinId::StringPrototypeMatch)
                    }
                    PropertyKeyIr::StaticString(name) if name == "matchAll" => {
                        Some(StandardBuiltinId::StringPrototypeMatchAll)
                    }
                    PropertyKeyIr::StaticString(name) if name == "replace" => {
                        Some(StandardBuiltinId::StringPrototypeReplace)
                    }
                    PropertyKeyIr::StaticString(name) if name == "replaceAll" => {
                        Some(StandardBuiltinId::StringPrototypeReplaceAll)
                    }
                    PropertyKeyIr::StaticString(name) if name == "search" => {
                        Some(StandardBuiltinId::StringPrototypeSearch)
                    }
                    PropertyKeyIr::StaticString(name) if name == "charAt" => {
                        Some(StandardBuiltinId::StringPrototypeCharAt)
                    }
                    PropertyKeyIr::StaticString(name) if name == "concat" => {
                        Some(StandardBuiltinId::StringPrototypeConcat)
                    }
                    PropertyKeyIr::StaticString(name) if name == "charCodeAt" => {
                        Some(StandardBuiltinId::StringPrototypeCharCodeAt)
                    }
                    PropertyKeyIr::StaticString(name) if name == "codePointAt" => {
                        Some(StandardBuiltinId::StringPrototypeCodePointAt)
                    }
                    PropertyKeyIr::StaticString(name) if name == "at" => {
                        Some(StandardBuiltinId::StringPrototypeAt)
                    }
                    PropertyKeyIr::StaticString(name) if name == "slice" => {
                        Some(StandardBuiltinId::StringPrototypeSlice)
                    }
                    PropertyKeyIr::StaticString(name) if name == "split" => {
                        Some(StandardBuiltinId::StringPrototypeSplit)
                    }
                    PropertyKeyIr::StaticString(name) if name == "padStart" => {
                        Some(StandardBuiltinId::StringPrototypePadStart)
                    }
                    PropertyKeyIr::StaticString(name) if name == "padEnd" => {
                        Some(StandardBuiltinId::StringPrototypePadEnd)
                    }
                    PropertyKeyIr::StaticString(name) if name == "repeat" => {
                        Some(StandardBuiltinId::StringPrototypeRepeat)
                    }
                    PropertyKeyIr::StaticString(name) if name == "isWellFormed" => {
                        Some(StandardBuiltinId::StringPrototypeIsWellFormed)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toWellFormed" => {
                        Some(StandardBuiltinId::StringPrototypeToWellFormed)
                    }
                    PropertyKeyIr::StaticString(name) if name == "indexOf" => {
                        Some(StandardBuiltinId::StringPrototypeIndexOf)
                    }
                    PropertyKeyIr::StaticString(name) if name == "lastIndexOf" => {
                        Some(StandardBuiltinId::StringPrototypeLastIndexOf)
                    }
                    PropertyKeyIr::StaticString(name) if name == "endsWith" => {
                        Some(StandardBuiltinId::StringPrototypeEndsWith)
                    }
                    PropertyKeyIr::StaticString(name) if name == "includes" => {
                        Some(StandardBuiltinId::StringPrototypeIncludes)
                    }
                    PropertyKeyIr::StaticString(name) if name == "startsWith" => {
                        Some(StandardBuiltinId::StringPrototypeStartsWith)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toLocaleLowerCase" => {
                        Some(StandardBuiltinId::StringPrototypeToLocaleLowerCase)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toLocaleUpperCase" => {
                        Some(StandardBuiltinId::StringPrototypeToLocaleUpperCase)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toLowerCase" => {
                        Some(StandardBuiltinId::StringPrototypeToLowerCase)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toUpperCase" => {
                        Some(StandardBuiltinId::StringPrototypeToUpperCase)
                    }
                    PropertyKeyIr::StaticString(name) if name == "toString" => {
                        Some(StandardBuiltinId::StringPrototypeToString)
                    }
                    PropertyKeyIr::StaticString(name) if name == "valueOf" => {
                        Some(StandardBuiltinId::StringPrototypeValueOf)
                    }
                    _ => None,
                };
                let runtime_bigint_builtin = match key {
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
                };
                let close_runtime_shortcut_blocks = |function: &mut Function| {
                    if dynamic_symbol_description {
                        function.instruction(&Instruction::End);
                    }
                    if runtime_number_builtin.is_some() {
                        function.instruction(&Instruction::End);
                    }
                    if runtime_string_builtin.is_some() {
                        function.instruction(&Instruction::End);
                    }
                    if runtime_bigint_builtin.is_some() {
                        function.instruction(&Instruction::End);
                    }
                };
                if dynamic_symbol_description {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    // Heap `Symbol(desc)` records (small handle, high 32 bits
                    // zero) store their `[[Description]]` in the record;
                    // well-known symbols carry an interned string payload
                    // whose description is that string.
                    function.instruction(&Instruction::LocalGet(target_local));
                    function.instruction(&Instruction::I64Const(32));
                    function.instruction(&Instruction::I64ShrU);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_from_offset(
                        target_local,
                        HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                        function,
                    );
                    function.instruction(&Instruction::LocalSet(payload_local));
                    self.load_i64_from_offset(
                        target_local,
                        HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                        function,
                    );
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_number_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(target_tag_local));
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
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_string_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(target_tag_local));
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
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let Some(builtin) = runtime_bigint_builtin {
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(target_tag_local));
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
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                if let PropertyKeyIr::StaticString(name) = key {
                    if let Some(index) = static_array_index_name(name) {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(index as i64));
                        function.instruction(&Instruction::LocalSet(index_local));
                        self.emit_typed_array_or_object_index_read_from_locals(
                            target_local,
                            target_tag_local,
                            index_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(index_local);
                        close_runtime_shortcut_blocks(function);
                        return Ok(());
                    }
                }
                if matches!(key, PropertyKeyIr::ArrayIndex(_)) {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_typed_array_or_object_index_read_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                    close_runtime_shortcut_blocks(function);
                    return Ok(());
                }
                if let PropertyKeyIr::StaticString(name) = key {
                    if let Some(index) = match name.as_str() {
                        "0" => Some(0),
                        "1" => Some(1),
                        "2" => Some(2),
                        "3" => Some(3),
                        "4" => Some(4),
                        _ => None,
                    } {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(index));
                        function.instruction(&Instruction::LocalSet(index_local));
                        self.emit_typed_array_or_object_index_read_from_locals(
                            target_local,
                            target_tag_local,
                            index_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(index_local);
                        close_runtime_shortcut_blocks(function);
                        return Ok(());
                    }
                    if let Some(HeapShape::Object(shape)) = target.heap_shape.as_deref() {
                        if let Some(ObjectShapeProperty::Data(info)) = shape.properties.get(name) {
                            if info.kind == ValueKind::Function {
                                if let Some(function_id) = info.function_targets.iter().next() {
                                    if info.function_targets.len() == 1 {
                                        let key_local = self.reserve_temp_local();
                                        function.instruction(&Instruction::I64Const(
                                            self.strings.payload(name),
                                        ));
                                        function.instruction(&Instruction::LocalSet(key_local));
                                        self.emit_object_read(
                                            target_local,
                                            target_tag_local,
                                            target_local,
                                            target_tag_local,
                                            key_local,
                                            payload_local,
                                            tag_local,
                                            function,
                                        )?;
                                        function.instruction(&Instruction::LocalGet(tag_local));
                                        function.instruction(&Instruction::I64Const(
                                            ValueKind::Undefined.tag() as i64,
                                        ));
                                        function.instruction(&Instruction::I64Eq);
                                        function.instruction(&Instruction::If(BlockType::Empty));
                                        if let Some(global_index) =
                                            StandardBuiltinId::from_function_id(function_id)
                                                .and_then(standard_builtin_function_global_index)
                                        {
                                            function
                                                .instruction(&Instruction::GlobalGet(global_index));
                                        } else {
                                            let meta =
                                                self.functions.get(function_id).ok_or_else(|| {
                                                    EmitError::unsupported(format!(
                                                        "unsupported in porffor wasm-aot first slice: unknown function value `{function_id}`"
                                                    ))
                                                })?;
                                            self.emit_function_value_payload(meta, function)?;
                                        }
                                        function.instruction(&Instruction::LocalSet(payload_local));
                                        function.instruction(&Instruction::I64Const(
                                            ValueKind::Function.tag() as i64,
                                        ));
                                        function.instruction(&Instruction::LocalSet(tag_local));
                                        function.instruction(&Instruction::End);
                                        self.release_temp_local(key_local);
                                        close_runtime_shortcut_blocks(function);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                if matches!(key, PropertyKeyIr::ArrayLength)
                    || matches!(key, PropertyKeyIr::StaticString(name) if name == "length")
                {
                    let key_local = self.reserve_temp_local();
                    let buffer_payload_local = self.reserve_temp_local();
                    let byte_offset_local = self.reserve_temp_local();
                    let typed_byte_length_local = self.reserve_temp_local();
                    let typed_bytes_per_element_local = self.reserve_temp_local();
                    let string_offset_local = self.reserve_temp_local();
                    let string_byte_len_local = self.reserve_temp_local();

                    if target.kind == ValueKind::Dynamic {
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.emit_unpack_string_payload(
                            target_local,
                            string_offset_local,
                            string_byte_len_local,
                            function,
                        );
                        self.emit_utf16_code_unit_len_from_utf8_locals(
                            string_offset_local,
                            string_byte_len_local,
                            payload_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalGet(payload_local));
                        function.instruction(&Instruction::F64ConvertI64U);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.emit_array_length(target_local, payload_local, tag_local, function);
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.emit_arguments_length(
                            target_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::Else);
                    }
                    self.emit_is_typed_array_i32(target_local, target_tag_local, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        target_local,
                        HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
                        buffer_payload_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        target_local,
                        HEAP_TYPED_ARRAY_BYTE_OFFSET,
                        byte_offset_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        target_local,
                        HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
                        typed_byte_length_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        target_local,
                        HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
                        typed_bytes_per_element_local,
                        function,
                    );
                    self.emit_typed_array_current_byte_length(
                        target_local,
                        target_tag_local,
                        buffer_payload_local,
                        byte_offset_local,
                        typed_byte_length_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(typed_byte_length_local));
                    function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
                    function.instruction(&Instruction::I64DivU);
                    function.instruction(&Instruction::F64ConvertI64U);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    if target.kind == ValueKind::Dynamic {
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }

                    self.release_temp_local(string_byte_len_local);
                    self.release_temp_local(string_offset_local);
                    self.release_temp_local(typed_bytes_per_element_local);
                    self.release_temp_local(typed_byte_length_local);
                    self.release_temp_local(byte_offset_local);
                    self.release_temp_local(buffer_payload_local);
                    self.release_temp_local(key_local);
                    close_runtime_shortcut_blocks(function);
                    return Ok(());
                }
                let key_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    let array_index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_string_index_0_to_4_or_minus_one(
                        key_local,
                        array_index_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_index_get_with_prototype(
                        target_local,
                        array_index_local,
                        target_local,
                        target_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    self.emit_object_read_with_key_tag(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                } else {
                    self.emit_object_read_with_key_tag(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                }
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_local);
                close_runtime_shortcut_blocks(function);
            }
            ValueKind::Array if array_named_read_needs_ordinary_get => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                let own_found_local = self.reserve_temp_local();
                let prototype_local = self.reserve_temp_local();
                let prototype_tag_local = self.reserve_temp_local();
                self.emit_array_own_named_property_read(
                    target_local,
                    target_local,
                    target_tag_local,
                    key_local,
                    own_found_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(own_found_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    target_local,
                    HEAP_PROTOTYPE_OFFSET,
                    prototype_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(prototype_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                self.load_i64_to_local_from_offset(
                    target_local,
                    HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                    prototype_tag_local,
                    function,
                );
                self.emit_object_read(
                    prototype_local,
                    prototype_tag_local,
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(prototype_tag_local);
                self.release_temp_local(prototype_local);
                self.release_temp_local(own_found_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Array => match key {
                PropertyKeyIr::ArrayLength => {
                    self.emit_array_or_object_length_read(
                        target_local,
                        target_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                }
                PropertyKeyIr::StaticString(name) if static_array_index_name(name).is_some() => {
                    let index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(
                        static_array_index_name(name).expect("array index name") as i64,
                    ));
                    function.instruction(&Instruction::LocalSet(index_local));
                    self.emit_typed_array_or_object_index_read_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::StaticString(name) => match name.as_str() {
                    "reduce" | "reduceRight" => {
                        let key_local = self.reserve_temp_local();
                        let own_found_local = self.reserve_temp_local();
                        let prototype_payload_local = self.reserve_temp_local();
                        let prototype_tag_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        self.emit_array_named_prop_read(
                            target_local,
                            key_local,
                            payload_local,
                            tag_local,
                            Some(own_found_local),
                            function,
                        );
                        function.instruction(&Instruction::LocalGet(own_found_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.load_i64_to_local_from_offset(
                            target_local,
                            HEAP_PROTOTYPE_OFFSET,
                            prototype_payload_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalGet(prototype_payload_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                        function.instruction(&Instruction::LocalSet(prototype_tag_local));
                        self.emit_object_read(
                            prototype_payload_local,
                            prototype_tag_local,
                            target_local,
                            target_tag_local,
                            key_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        self.release_temp_local(prototype_tag_local);
                        self.release_temp_local(prototype_payload_local);
                        self.release_temp_local(own_found_local);
                        self.release_temp_local(key_local);
                    }
                    "length" => {
                        self.emit_array_or_object_length_read(
                            target_local,
                            target_tag_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                    }
                    "Symbol.isConcatSpreadable" => {
                        self.emit_array_is_concat_spreadable_read(
                            target_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                    }
                    "fill" => {
                        if let Some(array_fill_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFill.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_fill_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "push" => {
                        if let Some(array_push_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypePush.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_push_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "shift" => {
                        if let Some(array_shift_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeShift.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_shift_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "unshift" => {
                        if let Some(array_unshift_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeUnshift.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_unshift_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "flat" => {
                        if let Some(array_flat_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFlat.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_flat_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "flatMap" => {
                        if let Some(array_flat_map_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFlatMap.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_flat_map_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "at" => {
                        if let Some(array_at_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeAt.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_at_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "includes" => {
                        if let Some(array_includes_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeIncludes.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_includes_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "indexOf" => {
                        if let Some(array_index_of_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeIndexOf.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_index_of_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "lastIndexOf" => {
                        if let Some(array_last_index_of_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_last_index_of_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "find" => {
                        if let Some(array_find_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFind.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_find_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "findIndex" => {
                        if let Some(array_find_index_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFindIndex.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_find_index_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "findLast" => {
                        if let Some(array_find_last_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFindLast.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_find_last_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "findLastIndex" => {
                        if let Some(array_find_last_index_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_find_last_index_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "map" => {
                        if let Some(array_map_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeMap.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_map_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "pop" => {
                        if let Some(array_pop_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypePop.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_pop_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "values" => {
                        if let Some(array_values_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(array_values_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "hasOwnProperty" => {
                        if let Some(has_own_property_meta) = self
                            .functions
                            .get(&StandardBuiltinId::ObjectPrototypeHasOwnProperty.function_id())
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(has_own_property_meta, function)?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "propertyIsEnumerable" => {
                        if let Some(property_is_enumerable_meta) = self
                            .functions
                            .get(
                                &StandardBuiltinId::ObjectPrototypePropertyIsEnumerable
                                    .function_id(),
                            )
                            .cloned()
                            .as_ref()
                        {
                            self.emit_function_value_payload(
                                property_is_enumerable_meta,
                                function,
                            )?;
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Function.tag() as i64
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(payload_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                    }
                    "constructor" => {
                        self.emit_array_constructor_read(
                            target_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                    }
                    "index" => {
                        self.emit_array_read_builtin_named_data_property(
                            target_local,
                            HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                            HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                            HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
                            payload_local,
                            tag_local,
                            function,
                        );
                    }
                    "input" => {
                        self.emit_array_read_builtin_named_data_property(
                            target_local,
                            HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                            HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                            HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
                            payload_local,
                            tag_local,
                            function,
                        );
                    }
                    "0" | "1" => {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(if name == "0" {
                            0
                        } else {
                            1
                        }));
                        function.instruction(&Instruction::LocalSet(index_local));
                        self.emit_typed_array_or_object_index_read_from_locals(
                            target_local,
                            target_tag_local,
                            index_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(index_local);
                    }
                    _ => {
                        let key_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        self.emit_object_read(
                            target_local,
                            target_tag_local,
                            target_local,
                            target_tag_local,
                            key_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(key_local);
                    }
                },
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                    let array_index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_string_index_0_to_4_or_minus_one(
                        key_local,
                        array_index_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(-1));
                    function.instruction(&Instruction::LocalSet(array_index_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_index_get_with_prototype(
                        target_local,
                        array_index_local,
                        target_local,
                        target_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_read_with_key_tag(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_object_read_with_key_tag(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_local);
                }
                _ => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_typed_array_or_object_index_read_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
            },
            ValueKind::Number | ValueKind::Boolean => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                let prototype_local = self.reserve_temp_local();
                let prototype_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(match target.kind {
                    ValueKind::Number => NUMBER_PROTOTYPE_GLOBAL_INDEX,
                    ValueKind::Boolean => BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                    _ => unreachable!(),
                }));
                function.instruction(&Instruction::LocalSet(prototype_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(prototype_tag_local));
                self.emit_object_read(
                    prototype_local,
                    prototype_tag_local,
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(prototype_tag_local);
                self.release_temp_local(prototype_local);
                self.release_temp_local(key_local);
            }
            ValueKind::Symbol => match key {
                PropertyKeyIr::StaticString(name) if name == "description" => {
                    // Heap `Symbol(desc)` records (small handle, high 32 bits
                    // zero) store their `[[Description]]` in the record;
                    // well-known symbols carry an interned string payload
                    // whose description is that string.
                    function.instruction(&Instruction::LocalGet(target_local));
                    function.instruction(&Instruction::I64Const(32));
                    function.instruction(&Instruction::I64ShrU);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_from_offset(
                        target_local,
                        HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                        function,
                    );
                    function.instruction(&Instruction::LocalSet(payload_local));
                    self.load_i64_from_offset(
                        target_local,
                        HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                        function,
                    );
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                }
                _ => {
                    // Everything else (`constructor`, `toString`, `valueOf`,
                    // `[Symbol.toPrimitive]`, well-known-symbol-keyed
                    // properties, and any user extensions) is resolved by an
                    // ordinary lookup against the real `Symbol.prototype`
                    // heap object, mirroring the actual [[Prototype]] chain.
                    let key_local = if matches!(
                        key,
                        PropertyKeyIr::StaticString(name) if name == "Symbol.toPrimitive"
                    ) {
                        let key_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(
                            self.strings
                                .property_key_symbol_payload("Symbol.toPrimitive"),
                        ));
                        function.instruction(&Instruction::LocalSet(key_local));
                        key_local
                    } else {
                        self.compile_object_key_to_local(key, function)?
                    };
                    let proto_payload_local = self.reserve_temp_local();
                    let proto_tag_local = self.reserve_temp_local();
                    function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(proto_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(proto_tag_local));
                    self.emit_object_read(
                        proto_payload_local,
                        proto_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(proto_tag_local);
                    self.release_temp_local(proto_payload_local);
                    self.release_temp_local(key_local);
                }
            },
            ValueKind::String => match key {
                PropertyKeyIr::ArrayLength => {
                    let offset_local = self.reserve_temp_local();
                    let byte_len_local = self.reserve_temp_local();
                    self.emit_unpack_string_payload(
                        target_local,
                        offset_local,
                        byte_len_local,
                        function,
                    );
                    self.emit_utf16_code_unit_len_from_utf8_locals(
                        offset_local,
                        byte_len_local,
                        payload_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::F64ConvertI64U);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.release_temp_local(byte_len_local);
                    self.release_temp_local(offset_local);
                }
                PropertyKeyIr::StaticString(name) if name == "length" => {
                    let offset_local = self.reserve_temp_local();
                    let byte_len_local = self.reserve_temp_local();
                    self.emit_unpack_string_payload(
                        target_local,
                        offset_local,
                        byte_len_local,
                        function,
                    );
                    self.emit_utf16_code_unit_len_from_utf8_locals(
                        offset_local,
                        byte_len_local,
                        payload_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::F64ConvertI64U);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.release_temp_local(byte_len_local);
                    self.release_temp_local(offset_local);
                }
                PropertyKeyIr::ArrayIndex(_) => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    let unit_len_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(unit_len_local));
                    self.emit_utf16_code_unit_range_payload_from_locals(
                        target_local,
                        index_local,
                        unit_len_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.release_temp_local(unit_len_local);
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    let index_local = self.reserve_temp_local();
                    let found_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(found_local));

                    // String exotic own properties are considered before the
                    // boxed String.prototype fallback. Only string keys can be
                    // `length` or canonical integer indices; symbol keys flow
                    // directly to the ordinary prototype lookup below.
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let offset_local = self.reserve_temp_local();
                    let byte_len_local = self.reserve_temp_local();
                    self.emit_unpack_string_payload(
                        target_local,
                        offset_local,
                        byte_len_local,
                        function,
                    );
                    self.emit_utf16_code_unit_len_from_utf8_locals(
                        offset_local,
                        byte_len_local,
                        payload_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::F64ConvertI64U);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(found_local));
                    self.release_temp_local(byte_len_local);
                    self.release_temp_local(offset_local);
                    function.instruction(&Instruction::Else);
                    self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let string_offset_local = self.reserve_temp_local();
                    let string_byte_len_local = self.reserve_temp_local();
                    let string_unit_len_local = self.reserve_temp_local();
                    self.emit_unpack_string_payload(
                        target_local,
                        string_offset_local,
                        string_byte_len_local,
                        function,
                    );
                    self.emit_utf16_code_unit_len_from_utf8_locals(
                        string_offset_local,
                        string_byte_len_local,
                        string_unit_len_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::LocalGet(string_unit_len_local));
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_string_index_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(found_local));
                    function.instruction(&Instruction::End);
                    self.release_temp_local(string_unit_len_local);
                    self.release_temp_local(string_byte_len_local);
                    self.release_temp_local(string_offset_local);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(found_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    let prototype_local = self.reserve_temp_local();
                    let prototype_tag_local = self.reserve_temp_local();
                    function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(prototype_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(prototype_tag_local));
                    self.emit_object_read_with_key_tag(
                        prototype_local,
                        prototype_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(prototype_tag_local);
                    self.release_temp_local(prototype_local);
                    function.instruction(&Instruction::End);

                    self.release_temp_local(found_local);
                    self.release_temp_local(index_local);
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_local);
                }
                _ => {
                    let key_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    let prototype_local = self.reserve_temp_local();
                    let prototype_tag_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                    function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(prototype_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(prototype_tag_local));
                    self.emit_object_read_with_key_tag(
                        prototype_local,
                        prototype_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        Some(key_tag_local),
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(prototype_tag_local);
                    self.release_temp_local(prototype_local);
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_local);
                }
            },
            ValueKind::Arguments => match key {
                PropertyKeyIr::ArrayLength => {
                    self.emit_arguments_length(target_local, payload_local, tag_local, function)?;
                }
                PropertyKeyIr::StaticString(name) if name == "length" => {
                    self.emit_arguments_length(target_local, payload_local, tag_local, function)?;
                }
                PropertyKeyIr::StaticString(name) if name == "callee" => {
                    self.emit_arguments_callee_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                }
                PropertyKeyIr::StaticString(name) if name == "Symbol.iterator" => {
                    let array_prototype_local = self.reserve_temp_local();
                    let array_prototype_tag_local = self.reserve_temp_local();
                    let key_local = self.reserve_temp_local();
                    function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(array_prototype_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::LocalSet(array_prototype_tag_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings.property_key_symbol_payload("Symbol.iterator"),
                    ));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read(
                        array_prototype_local,
                        array_prototype_tag_local,
                        array_prototype_local,
                        array_prototype_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(key_local);
                    self.release_temp_local(array_prototype_tag_local);
                    self.release_temp_local(array_prototype_local);
                }
                PropertyKeyIr::StaticString(name) if name == "Symbol.isConcatSpreadable" => {
                    self.emit_arguments_is_concat_spreadable_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                PropertyKeyIr::StaticString(name) if static_array_index_name(name).is_some() => {
                    let index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(
                        static_array_index_name(name).expect("arguments index") as i64,
                    ));
                    function.instruction(&Instruction::LocalSet(index_local));
                    self.emit_arguments_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::StaticString(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(key_local);
                }
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                    self.emit_dynamic_property_read_with_key_locals(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        key_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_local);
                }
                _ => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_arguments_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
            },
            ValueKind::Dynamic => match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    let array_push_meta = self
                        .functions
                        .get(&StandardBuiltinId::ArrayPrototypePush.function_id())
                        .cloned();
                    let array_shift_meta = self
                        .functions
                        .get(&StandardBuiltinId::ArrayPrototypeShift.function_id())
                        .cloned();
                    let array_unshift_meta = self
                        .functions
                        .get(&StandardBuiltinId::ArrayPrototypeUnshift.function_id())
                        .cloned();
                    let array_index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_length(target_local, payload_local, tag_local, function);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function
                        .instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_constructor_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("push")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    if let Some(array_push_meta) = array_push_meta.as_ref() {
                        self.emit_function_value_payload(array_push_meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    }
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("shift")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    if let Some(array_shift_meta) = array_shift_meta.as_ref() {
                        self.emit_function_value_payload(array_shift_meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    }
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("unshift")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    if let Some(array_unshift_meta) = array_unshift_meta.as_ref() {
                        self.emit_function_value_payload(array_unshift_meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                    }
                    function.instruction(&Instruction::Else);
                    self.emit_string_index_0_to_4_or_minus_one(
                        key_local,
                        array_index_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_read(
                        target_local,
                        array_index_local,
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
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_length(target_local, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::Else);
                    self.emit_string_index_0_to_4_or_minus_one(
                        key_local,
                        array_index_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_read(
                        target_local,
                        array_index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                    self.release_temp_local(key_local);
                }
                PropertyKeyIr::ArrayLength => {
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_length(target_local, payload_local, tag_local, function);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_length(target_local, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                _ => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_read(
                        target_local,
                        index_local,
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
                    self.release_temp_local(index_local);
                }
            },
            ValueKind::String => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                let prototype_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(prototype_tag_local));
                self.emit_object_read(
                    self.scratch_local,
                    prototype_tag_local,
                    self.scratch_local,
                    prototype_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(prototype_tag_local);
                self.release_temp_local(key_local);
            }
            ValueKind::BigInt => {
                let key_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let constructor_local = self.reserve_temp_local();
                let prototype_local = self.reserve_temp_local();
                let prototype_tag_local = self.reserve_temp_local();
                self.compile_object_key_to_locals(key, key_local, key_tag_local, function)?;
                function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(constructor_local));
                self.load_i64_to_local_from_offset(
                    constructor_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    prototype_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(prototype_tag_local));
                self.emit_object_read_with_key_tag(
                    prototype_local,
                    prototype_tag_local,
                    target_local,
                    target_tag_local,
                    key_local,
                    Some(key_tag_local),
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(prototype_tag_local);
                self.release_temp_local(prototype_local);
                self.release_temp_local(constructor_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_local);
            }
            // `ValueKind` has exactly twelve variants and every other one is
            // handled above, so this arm is precisely `Undefined` and `Null`.
            // 7.2.1 RequireObjectCoercible makes `null.x` a runtime TypeError,
            // not a compile-time gap. Callers that already performed the check
            // narrow the target away from the nullish kinds; reaching here means
            // the check is still owed, so pay it the same way the runtime
            // dispatch in `compile_dynamic_property_read_from_locals` does.
            _ => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot read properties of null or undefined",
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
        }

        Ok(())
    }

    fn compile_dynamic_property_read_from_locals(
        &mut self,
        target_possible_kinds: KindSet,
        key: &PropertyKeyIr,
        target_local: u32,
        target_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_may_be_nullish = target_possible_kinds.contains(ValueKind::Undefined)
            || target_possible_kinds.contains(ValueKind::Null);
        match key {
            PropertyKeyIr::StringExpr(key_expr) | PropertyKeyIr::ArrayIndex(key_expr)
                if target_may_be_nullish =>
            {
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(key_expr, key_payload_local, key_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;

                self.compile_nullish_tagged_i32(target_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot read properties of null or undefined",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    payload_local,
                    tag_local,
                    1,
                    function,
                )?;
                function.instruction(&Instruction::End);

                if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    let property_key_kinds = KindSet::from_kind(ValueKind::String)
                        .union(KindSet::from_kind(ValueKind::Symbol));
                    if key_expr.possible_kinds.is_subset_of(property_key_kinds) {
                        function.instruction(&Instruction::LocalGet(key_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::LocalGet(key_payload_local));
                        function
                            .instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
                        function.instruction(&Instruction::I64Or);
                        function.instruction(&Instruction::LocalSet(key_payload_local));
                        function.instruction(&Instruction::End);
                    } else {
                        self.emit_value_to_property_key_locals(
                            key_payload_local,
                            key_tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            key_payload_local,
                            key_tag_local,
                            function,
                        )?;
                    }
                }

                let read_result =
                    if let Some(helper) = self.dynamic_property_read_helper_function_index() {
                        function.instruction(&Instruction::LocalGet(target_local));
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function.instruction(&Instruction::LocalGet(target_local));
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function.instruction(&Instruction::LocalGet(key_payload_local));
                        function.instruction(&Instruction::LocalGet(key_tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::Call(helper));
                        self.store_call_results(payload_local, tag_local, function);
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )
                    } else {
                        self.emit_dynamic_property_read_with_key_locals(
                            target_local,
                            target_tag_local,
                            target_local,
                            target_tag_local,
                            key_payload_local,
                            key_tag_local,
                            payload_local,
                            tag_local,
                            function,
                        )
                    };
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                return read_result;
            }
            _ => {}
        }
        if !matches!(key, PropertyKeyIr::StringExpr(_)) || !target_may_be_nullish {
            if let Some(helper) = self.dynamic_property_read_helper_function_index() {
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let compile_key = match key {
                    PropertyKeyIr::ArrayLength => {
                        function
                            .instruction(&Instruction::I64Const(self.strings.payload("length")));
                        function.instruction(&Instruction::LocalSet(key_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(key_tag_local));
                        Ok(())
                    }
                    _ => self.compile_object_key_to_locals(
                        key,
                        key_payload_local,
                        key_tag_local,
                        function,
                    ),
                };
                if let Err(error) = compile_key {
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_payload_local);
                    return Err(error);
                }

                function.instruction(&Instruction::LocalGet(target_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalGet(target_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalGet(key_payload_local));
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                let read_result = self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                );
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                return read_result;
            }
        }

        // Dispatch before compiling the key. A computed key therefore appears
        // in each runtime arm but executes in exactly one selected arm, after
        // the optional-chain nullish check performed by the caller.
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.push_control(ControlFrameKind::Block);
        for kind in [
            ValueKind::Object,
            ValueKind::Function,
            ValueKind::Array,
            ValueKind::Arguments,
            ValueKind::Number,
            ValueKind::Boolean,
            ValueKind::String,
            ValueKind::Symbol,
            ValueKind::BigInt,
        ] {
            function.instruction(&Instruction::LocalGet(target_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.push_control(ControlFrameKind::If);
            let runtime_target = TypedExpr::from_info(ValueInfo::new(kind), ExprIr::Undefined);
            self.compile_property_read_from_locals(
                &runtime_target,
                key,
                target_local,
                target_tag_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Br(1));
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        // Undefined and null normally reach the explicit caller-side
        // RequireObjectCoercible check. Keep the dynamic emitter sound when it
        // is reused from another path by producing the same abrupt completion.
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot read properties of null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Performs a dynamic `[[Get]]` from an already-normalized property key.
    /// The original receiver is kept separate from the boxed lookup target so
    /// primitive accessors and Proxy traps observe the ECMAScript receiver.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_dynamic_property_read_with_key_locals(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let typed_array_numeric_index_local = self.reserve_temp_local();
        let typed_array_index_valid_local = self.reserve_temp_local();
        let typed_array_index_read_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(target_payload_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_callee_read(target_payload_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayPrototypeValues)
        {
            function.instruction(&Instruction::LocalGet(key_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(key_payload_local));
            function.instruction(&Instruction::I64Const(
                self.strings.property_key_symbol_payload("Symbol.iterator"),
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(object_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::LocalSet(object_tag_local));
            function.instruction(&Instruction::I64Const(
                self.strings.property_key_symbol_payload("Symbol.iterator"),
            ));
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_object_read(
                object_payload_local,
                object_tag_local,
                object_payload_local,
                object_tag_local,
                index_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_is_concat_spreadable_read(
            target_payload_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_string_index_0_to_4_or_minus_one(key_payload_local, index_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            target_payload_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_array_own_named_property_read(
            target_payload_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            object_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_read_with_key_tag(
            object_payload_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            Some(key_tag_local),
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_index_read_local));
        self.emit_typed_array_canonical_numeric_index_i32(
            target_payload_local,
            target_tag_local,
            key_payload_local,
            key_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_read_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_read_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            target_payload_local,
            target_tag_local,
            typed_array_numeric_index_local,
            index_local,
            typed_array_index_valid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            target_payload_local,
            target_tag_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(typed_array_index_read_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_object_locals(
            target_payload_local,
            target_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        self.emit_object_read_with_key_tag(
            object_payload_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            Some(key_tag_local),
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_read_local);
        self.release_temp_local(typed_array_index_valid_local);
        self.release_temp_local(typed_array_numeric_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// Reads `target[index]` for an integer index, dispatching over the target's
    /// runtime kind.
    ///
    /// This is the seam. The composite it guards measures 72,635 bytes per
    /// inline copy and has 51 call sites, so by default it emits a `call` into
    /// the `IndexedElementRead` runtime helper and only falls back to the inline
    /// body when the helper is unavailable (no heap) or when the helper's own
    /// body is what is being compiled.
    ///
    /// The public name is deliberately unchanged: all 51 call sites keep
    /// calling it and see only the seam (counted:
    /// `grep -rn 'self\.emit_typed_array_or_object_index_read_from_locals('`
    /// over `crates/` — 7 in this file, 11 in `builtins/standard.rs`, 32 in
    /// `builtins/array.rs`, 1 in `builtins/iterators.rs`).
    ///
    /// **Constraint on new call sites.** The throw propagation below is
    /// computed at `extra_depth = 0`, i.e. it assumes the tracked control stack
    /// is the whole story at the point of call. That is true for every call
    /// site today only because they are all inside hand-emitted builtin bodies,
    /// where `active_throw_target()` is always `None` — `throw_handler_stack`
    /// and `finally_stack` are pushed only from user `try` lowering — so
    /// `emit_propagate_throw_from_locals_if_needed` degrades to
    /// `emit_return_current_completion` and the depth is never used. Several of
    /// those sites sit inside raw `If`/`Block` frames the control stack does not
    /// track and compensate for it in their *own* follow-up propagation (see
    /// `emit_propagate_throw_from_locals_if_needed_with_extra_depth` in
    /// `builtins/array.rs`), which this seam does not do. A user-code call site
    /// added inside untracked frames within a `try` would therefore branch
    /// several frames too shallow and still validate, because every frame is
    /// `BlockType::Empty`. Adding one means giving this seam an `extra_depth`
    /// parameter first — the codebase already has that shape in
    /// `emit_object_read_without_throw_propagation`. Do not guess the number
    /// from a neighbouring call: the existing compensations are not internally
    /// consistent (`builtins/array.rs` passes 0 and 3 for two arms of one
    /// `if`/`else` chain), which is itself unresolved.
    pub(crate) fn emit_typed_array_or_object_index_read_from_locals(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_indexed_element_read {
            if let Some(helper) = self.indexed_element_read_helper_function_index() {
                function.instruction(&Instruction::LocalGet(target_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                // Same discipline as `emit_object_read_ordinary_via_helper`: the
                // value (or, on a throw, the thrown value) lands in the caller's
                // value locals and the completion tuple is adopted; `result_local`
                // is only touched on a throw, so nothing the caller is holding
                // across the read is clobbered.
                self.store_call_results(payload_local, tag_local, function);
                // Inside the helper a getter/proxy throw has no active handler and
                // came back as a throw completion. The inline body would have
                // propagated it from within its own nesting, so propagate here to
                // keep the seam's contract identical for callers that do not
                // separately check the read's completion.
                return self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        self.emit_typed_array_or_object_index_read_from_locals_inner(
            target_local,
            target_tag_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )
    }

    /// Compiles the shared `expr[index]` read composite. The Arguments / Array
    /// (with prototype walk) / TypedArray-element / ordinary-object dispatch is
    /// emitted once here and reached with a plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=target payload,
    /// 1=target tag, 2=integer index. Params 3-6 are unused. Results are the
    /// standard `(result, result_tag, completion, completion_aux)` tuple: on a
    /// normal completion the read value is in the first two slots, and a getter
    /// or proxy-trap throw is surfaced through the completion slots for the
    /// seam to re-raise.
    ///
    /// No realm-environment parameter, matching
    /// [`FunctionBuilder::compile_object_read_helper`]: the ordinary read this
    /// delegates to already passes zero for the object-read helper's params
    /// 5/6, so a read has no realm-dependent behavior to thread.
    ///
    /// This lives here rather than in `emit.rs` with the other 22 helper
    /// compilers so that
    /// [`Self::emit_typed_array_or_object_index_read_from_locals_inner`] — the
    /// 72,635-byte body — can stay private to this module. Its two callers are
    /// the seam's fallback arm and this function; both are in this file, so
    /// "there is no third caller" is a fact the compiler checks rather than a
    /// comment asking readers not to add one.
    pub(crate) fn compile_indexed_element_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::IndexedElementRead);
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_typed_array_or_object_index_read_from_locals_inner(
            0,
            1,
            2,
            self.result_local,
            self.result_tag_local,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_typed_array_or_object_index_read_from_locals_inner(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let buffer_flags_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            target_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            target_local,
            index_local,
            target_local,
            target_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_typed_array_i32(target_local, target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        self.emit_load_array_buffer_flags(buffer_payload_local, buffer_flags_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            bytes_per_element_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );
        self.emit_typed_array_current_byte_length(
            target_local,
            target_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::F64,
            Instruction::F64Load(Self::memarg64(0)),
            Instruction::F64Load(Self::shared_memarg64(0)),
            function,
        );
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::F32,
            Instruction::F32Load(Self::memarg32(0)),
            Instruction::F32Load(Self::shared_memarg32(0)),
            function,
        );
        function.instruction(&Instruction::F64PromoteF32);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load8S(Self::memarg8(0)),
            Instruction::I32Load8S(Self::shared_memarg8(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load16S(Self::memarg16(0)),
            Instruction::I32Load16S(Self::shared_memarg16(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load8U(Self::memarg8(0)),
            Instruction::I32Load8U(Self::shared_memarg8(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load16U(Self::memarg16(0)),
            Instruction::I32Load16U(Self::shared_memarg16(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load(Self::memarg32(0)),
            Instruction::I32Load(Self::shared_memarg32(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I64,
            Instruction::I64Load(Self::memarg64(0)),
            Instruction::I64Load(Self::shared_memarg64(0)),
            function,
        );
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_one_limb_bigint(1, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_array_buffer_memory_load(
            buffer_flags_local,
            ValType::I32,
            Instruction::I32Load(Self::memarg32(0)),
            Instruction::I32Load(Self::shared_memarg32(0)),
            function,
        );
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_index_read_from_locals(
            target_local,
            target_tag_local,
            index_local,
            key_local,
            index_number_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(buffer_flags_local);
        self.release_temp_local(address_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_object_index_read_from_locals(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        key_local: u32,
        index_number_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_property_write_payload(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        value: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_property_write_to_locals(
            target,
            key,
            value,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(payload_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile_property_update_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: ValueKind,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let old_value_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_local, target_tag_local, function)?;
        if target.kind == ValueKind::Array && matches!(key, PropertyKeyIr::ArrayIndex(_)) {
            let index_local = self.compile_array_index_to_local(key, function)?;
            self.emit_array_index_get_with_prototype(
                target_local,
                index_local,
                target_local,
                target_tag_local,
                value_local,
                value_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);

            if value_kind == ValueKind::Number {
                self.emit_value_to_number_payload(value_tag_local, value_local, function)?;
                function.instruction(&Instruction::LocalSet(value_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_return_current_completion_if_throw(function);
            }

            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::LocalSet(old_value_local));
            function.instruction(&Instruction::LocalGet(value_local));
            self.emit_update_delta(op, value_kind, function);
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_array_assignment_write(
                target_local,
                index_local,
                value_local,
                value_tag_local,
                function,
            )?;

            match return_mode {
                UpdateReturnMode::Prefix => {
                    function.instruction(&Instruction::LocalGet(value_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                }
                UpdateReturnMode::Postfix => {
                    function.instruction(&Instruction::LocalGet(old_value_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                }
            }
            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));

            self.release_temp_local(index_local);
            self.release_temp_local(old_value_local);
            self.release_temp_local(value_tag_local);
            self.release_temp_local(value_local);
            self.release_temp_local(target_tag_local);
            self.release_temp_local(target_local);
            return Ok(());
        }
        let key_local = self.compile_object_key_to_local(key, function)?;
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            value_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        if value_kind == ValueKind::Number {
            self.emit_value_to_number_payload(value_tag_local, value_local, function)?;
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_return_current_completion_if_throw(function);
        }

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(old_value_local));
        function.instruction(&Instruction::LocalGet(value_local));
        self.emit_update_delta(op, value_kind, function);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_write(
            target_local,
            target_tag_local,
            key_local,
            value_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        match return_mode {
            UpdateReturnMode::Prefix => {
                function.instruction(&Instruction::LocalGet(value_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            UpdateReturnMode::Postfix => {
                function.instruction(&Instruction::LocalGet(old_value_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
        }
        function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(key_local);
        self.release_temp_local(old_value_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    pub(crate) fn compile_property_compound_assign_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        op: ArithmeticBinaryOp,
        rhs: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if op != ArithmeticBinaryOp::Add {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: property compound assignment operator",
            ));
        }

        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_local, target_tag_local, function)?;

        let key_local = self.compile_object_key_to_local(key, function)?;
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            value_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(value_local, value_tag_local, function)?;

        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            rhs_payload_local,
            rhs_tag_local,
            function,
        )?;
        let lhs_string_local = self.reserve_temp_local();
        let rhs_string_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(lhs_string_local));
        self.emit_value_to_string_payload(rhs_payload_local, rhs_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(rhs_string_local));
        self.emit_concat_string_payloads_local(lhs_string_local, rhs_string_local, function)?;
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_local, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        self.emit_value_to_number_payload(rhs_tag_local, rhs_payload_local, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(rhs_string_local);
        self.release_temp_local(lhs_string_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.emit_propagate_throw_from_locals_if_needed(value_local, value_tag_local, function)?;

        self.emit_object_write(
            target_local,
            target_tag_local,
            key_local,
            value_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.release_temp_local(key_local);

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    /// Writes `target[index] = value`, dispatching between a TypedArray element
    /// store and an ordinary `[[Set]]`.
    ///
    /// This is the seam; the composite it guards measures 174,558 bytes per
    /// inline copy, across 5 call sites, all in this file. Contract on both
    /// arms matches [`Self::emit_object_write`]: on a throw the thrown value is
    /// left in the result locals with `completion == Throw` for the caller's
    /// own check, and on success the caller's result locals are preserved.
    ///
    /// Both arms *additionally* co-home the thrown value in the caller's value
    /// locals — see [`Self::emit_cohome_thrown_value_into_locals`] for why that
    /// is required rather than tidy, and note that the co-homing is emitted
    /// here, after the dispatch, so it is structurally impossible for one arm
    /// to have it and the other not.
    pub(crate) fn emit_typed_array_or_object_index_write_from_locals(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_indexed_element_write_dispatch(
            target_local,
            target_tag_local,
            index_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_cohome_thrown_value_into_locals(value_payload_local, value_tag_local, function);
        Ok(())
    }

    /// On an adopted throw completion, copies the thrown value out of the
    /// result locals into the caller's value locals.
    ///
    /// Both `[[Set]]` arms of the write composite report a throw the same way a
    /// helper call does: the thrown value lands in `result_local`/
    /// `result_tag_local` with `completion == Throw`, and control keeps going.
    /// The caller's value locals then still hold the *assigned value*, which is
    /// a problem because that is what the enclosing expression propagates from.
    /// `compile_property_write_to_locals` leaves the assignment's value in those
    /// locals by contract, and `compile_expr_to_locals`'s `ExprIr::PropertyWrite`
    /// arm hands them straight to the consumer's
    /// [`Self::emit_propagate_throw_from_locals_if_needed`] — so without this,
    /// `var x = (ta[0] = { valueOf() { throw new TypeError(); } });` rethrows
    /// the *object literal* instead of the `TypeError`. (At statement position
    /// the throw is propagated from `result_local` instead and was always
    /// correct; only nested assignment positions relabel.)
    ///
    /// Clobbering the value locals on the throw path is safe: a throw completion
    /// means the assignment produced no value, and every caller either
    /// propagates or returns immediately after.
    fn emit_cohome_thrown_value_into_locals(
        &self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }

    /// The outlined-versus-inline choice itself. Private, and deliberately not
    /// the function the 5 call sites see: everything a caller is promised that
    /// is *not* arm-specific belongs in
    /// [`Self::emit_typed_array_or_object_index_write_from_locals`] around this.
    #[allow(clippy::too_many_arguments)]
    fn emit_indexed_element_write_dispatch(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The helper body is compiled with `function_id: None`, so the nested
        // object-write call inside it passes zero for the object-write helper's
        // realm-environment parameter (param 6). That is exactly what the inline
        // expansion does for a non-builtin caller — see
        // `emit_object_write_via_helper` — but *not* what it does for a standard
        // builtin, which forwards its self-backed realm environment so that e.g.
        // ArraySetLength raises its RangeError in the right Realm.
        //
        // No standard builtin can reach this composite: all 5 call sites are in
        // `compile_property_write_to_locals`, which runs only from IR lowering
        // (`FunctionBuilder::compile`, used for `script.functions`), while
        // standard builtin bodies are hand-emitted by `compile_standard_builtin`
        // under `compile_builtin`. An earlier revision silently declined to
        // outline for such a caller. That branch was unreachable, and worse, if
        // it ever had fired it would have chosen between two arms rather than
        // preserved one behaviour. So the unreachable case is now loud: a
        // builtin that reaches here needs the helper to grow a
        // realm-environment parameter, which is a change to make deliberately
        // and not to discover from a RangeError raised in the wrong Realm.
        if self.outline_indexed_element_write {
            if let Some(helper) = self.indexed_element_write_helper_function_index() {
                if let Some(builtin) = self
                    .function_id
                    .as_ref()
                    .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
                {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: standard builtin `{}` reached the indexed-element write composite, which has no realm-environment parameter",
                        builtin.debug_name()
                    )));
                }
                let saved_result_local = self.reserve_temp_local();
                let saved_result_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalSet(saved_result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalSet(saved_result_tag_local));

                function.instruction(&Instruction::LocalGet(target_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalGet(value_tag_local));
                // Parameter 6: the calling function's strictness, selected the
                // same way `emit_object_write_via_helper` selects its parameter 5,
                // because the helper body is mode-less and the sloppy/strict
                // `[[Set]]`-failure split has to be decided at run time.
                match self.object_write_strict_flag_local {
                    Some(strict_override) => {
                        function.instruction(&Instruction::LocalGet(strict_override));
                    }
                    None => {
                        function.instruction(&Instruction::I64Const(i64::from(
                            self.is_current_function_strict(),
                        )));
                    }
                }
                function.instruction(&Instruction::Call(helper));
                self.store_call_results_to(
                    self.result_local,
                    self.result_tag_local,
                    self.completion_local,
                    self.completion_aux_local,
                    function,
                );

                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(saved_result_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(saved_result_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(saved_result_tag_local);
                self.release_temp_local(saved_result_local);
                return Ok(());
            }
        }
        self.emit_typed_array_or_object_index_write_from_locals_inner(
            target_local,
            target_tag_local,
            index_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )
    }

    /// Compiles the shared `expr[index] = value` write composite (TypedArray
    /// element store versus ordinary `[[Set]]`).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=target payload,
    /// 1=target tag, 2=integer index, 3=key payload (the string form of the
    /// index, used by the ordinary-object arm), 4=value payload, 5=value tag,
    /// 6=calling function's strictness (0 sloppy, nonzero strict).
    ///
    /// Param 6 exists for the same reason as the object-write helper's param 5:
    /// this is a single mode-less body, so the sloppy/strict `[[Set]]` failure
    /// split — silent no-op versus `TypeError` on a non-writable property or a
    /// non-extensible target — has to be decided from a runtime flag rather
    /// than from the compile-time strictness of the helper itself. Dropping it
    /// would turn every strict-mode `a[i] = v` failure into a silent no-op.
    ///
    /// Counterpart of [`Self::compile_indexed_element_read_helper`], including
    /// the reason it lives in this file: the 174,558-byte body it emits stays
    /// module-private with exactly two callers.
    pub(crate) fn compile_indexed_element_write_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::IndexedElementWrite);
        self.object_write_strict_flag_local = Some(6);
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_typed_array_or_object_index_write_from_locals_inner(
            0, 1, 2, 3, 4, 5, &mut function,
        )?;
        self.pop_scope();
        self.object_write_strict_flag_local = None;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_typed_array_or_object_index_write_from_locals_inner(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_is_typed_array_i32(target_local, target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            target_local,
            target_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_write(
            target_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        Ok(())
    }

    pub(crate) fn compile_property_write_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        value: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_local, target_tag_local, function)?;

        match target.kind {
            ValueKind::Object | ValueKind::Function => {
                if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    let key_payload_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    let index_local = self.reserve_temp_local();
                    let index_found_local = self.reserve_temp_local();
                    self.compile_object_key_to_locals(
                        key,
                        key_payload_local,
                        key_tag_local,
                        function,
                    )?;
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(index_found_local));
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_known_array_index_from_property_key(
                        key_payload_local,
                        index_local,
                        index_found_local,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(index_found_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_payload_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_typed_array_or_object_index_write_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        key_payload_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(index_found_local);
                    self.release_temp_local(index_local);
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(key_payload_local);
                    self.release_temp_local(target_tag_local);
                    self.release_temp_local(target_local);
                    return Ok(());
                }
                let static_index_name = match key {
                    PropertyKeyIr::StaticString(name) => {
                        static_array_index_name(name).map(|index| (name.as_str(), index))
                    }
                    _ => None,
                };
                if matches!(key, PropertyKeyIr::ArrayIndex(_)) || static_index_name.is_some() {
                    let index_local = if let Some((_, index)) = static_index_name {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(index as i64));
                        function.instruction(&Instruction::LocalSet(index_local));
                        index_local
                    } else {
                        self.compile_array_index_to_local(key, function)?
                    };
                    let key_local = if let Some((name, _)) = static_index_name {
                        let key_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        key_local
                    } else {
                        self.compile_object_key_to_local(key, function)?
                    };
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    self.emit_typed_array_or_object_index_write_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(key_local);
                    self.release_temp_local(index_local);
                    self.release_temp_local(target_tag_local);
                    self.release_temp_local(target_local);
                    return Ok(());
                }
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_object_write(
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => {
                let computed_key_local = if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    Some(self.compile_object_key_to_local(key, function)?)
                } else {
                    None
                };
                let computed_index_local = if matches!(key, PropertyKeyIr::ArrayIndex(_)) {
                    Some(self.compile_array_index_to_local(key, function)?)
                } else {
                    None
                };
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                if matches!(key, PropertyKeyIr::ArrayLength)
                    || matches!(key, PropertyKeyIr::StaticString(name) if name == "length")
                {
                    let length_success_local = self.reserve_temp_local();
                    let length_writable_present_local = self.reserve_temp_local();
                    let length_allow_define_local = self.reserve_temp_local();
                    let length_initial_writable_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_writable_present_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(length_allow_define_local));
                    self.emit_array_length_writable_i64(
                        target_local,
                        length_initial_writable_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(length_initial_writable_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_set_length_from_value(
                        target_local,
                        payload_local,
                        tag_local,
                        length_writable_present_local,
                        length_writable_present_local,
                        length_allow_define_local,
                        length_success_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_success_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(length_success_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write_set_failure_else(
                        "Cannot assign to array length",
                        0,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(length_initial_writable_local);
                    self.release_temp_local(length_allow_define_local);
                    self.release_temp_local(length_writable_present_local);
                    self.release_temp_local(length_success_local);
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "index") {
                    self.emit_array_define_builtin_named_data_property(
                        target_local,
                        HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                        HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                        HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
                        payload_local,
                        tag_local,
                        function,
                    );
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "input") {
                    self.emit_array_define_builtin_named_data_property(
                        target_local,
                        HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                        HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                        HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
                        payload_local,
                        tag_local,
                        function,
                    );
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "Symbol.isConcatSpreadable")
                {
                    self.emit_array_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                } else if matches!(key, PropertyKeyIr::StringExpr(_)) {
                    let key_local = computed_key_local.expect("computed array property key local");
                    let index_local = self.reserve_temp_local();
                    let length_success_local = self.reserve_temp_local();
                    let length_writable_present_local = self.reserve_temp_local();
                    let length_allow_define_local = self.reserve_temp_local();
                    let length_initial_writable_local = self.reserve_temp_local();
                    self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_assignment_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_writable_present_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(length_allow_define_local));
                    self.emit_array_length_writable_i64(
                        target_local,
                        length_initial_writable_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(length_initial_writable_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_set_length_from_value(
                        target_local,
                        payload_local,
                        tag_local,
                        length_writable_present_local,
                        length_writable_present_local,
                        length_allow_define_local,
                        length_success_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_success_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(length_success_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write_set_failure_else(
                        "Cannot assign to array length",
                        0,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings
                            .property_key_symbol_payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(length_initial_writable_local);
                    self.release_temp_local(length_allow_define_local);
                    self.release_temp_local(length_writable_present_local);
                    self.release_temp_local(length_success_local);
                    self.release_temp_local(index_local);
                    self.release_temp_local(key_local);
                } else if let PropertyKeyIr::StaticString(name) = key {
                    if let Some(index) = static_array_index_name(name) {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(index as i64));
                        function.instruction(&Instruction::LocalSet(index_local));
                        self.emit_array_assignment_write(
                            target_local,
                            index_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(index_local);
                    } else {
                        let key_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        self.emit_object_write(
                            target_local,
                            target_tag_local,
                            key_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(key_local);
                    }
                } else {
                    let index_local = computed_index_local.expect("computed array index local");
                    self.emit_array_assignment_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
            }
            ValueKind::Arguments => {
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "length") {
                    let len_local = self.reserve_temp_local();
                    self.emit_value_to_number_payload(tag_local, payload_local, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::I64TruncSatF64U);
                    function.instruction(&Instruction::LocalSet(len_local));
                    self.store_i64_local_at_offset(
                        target_local,
                        HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
                        len_local,
                        function,
                    );
                    self.release_temp_local(len_local);
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "callee") {
                    self.emit_arguments_callee_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                } else if matches!(key, PropertyKeyIr::StaticString(name) if static_array_index_name(name).is_some())
                {
                    let PropertyKeyIr::StaticString(name) = key else {
                        unreachable!("static arguments index")
                    };
                    let index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(
                        static_array_index_name(name).expect("arguments index") as i64,
                    ));
                    function.instruction(&Instruction::LocalSet(index_local));
                    self.emit_arguments_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "Symbol.isConcatSpreadable")
                {
                    self.emit_arguments_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                } else if matches!(
                    key,
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::StringExpr(_)
                ) {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    let is_spreadable_key_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings
                            .property_key_symbol_payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(is_spreadable_key_local));
                    function.instruction(&Instruction::LocalGet(is_spreadable_key_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_callee_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_array_define_named_data_property(
                        target_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(is_spreadable_key_local);
                    self.release_temp_local(key_local);
                } else {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_arguments_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
            }
            ValueKind::Dynamic => {
                let static_index_name = match key {
                    PropertyKeyIr::StaticString(name) => {
                        static_array_index_name(name).map(|index| (name.as_str(), index))
                    }
                    _ => None,
                };
                if let Some((name, index)) = static_index_name {
                    let index_local = self.reserve_temp_local();
                    let key_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(index as i64));
                    function.instruction(&Instruction::LocalSet(index_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_assignment_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_typed_array_or_object_index_write_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(key_local);
                    self.release_temp_local(index_local);
                } else if matches!(key, PropertyKeyIr::ArrayIndex(_)) {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_assignment_write(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_typed_array_or_object_index_write_from_locals(
                        target_local,
                        target_tag_local,
                        index_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(key_local);
                    self.release_temp_local(index_local);
                } else {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    let set_result_local = self.reserve_temp_local();
                    let key_tag_local = self.reserve_temp_local();
                    let array_index_local = self.reserve_temp_local();
                    let array_index_found_local = self.reserve_temp_local();
                    let length_key_local = self.reserve_temp_local();
                    let length_success_local = self.reserve_temp_local();
                    let length_writable_present_local = self.reserve_temp_local();
                    let length_allow_define_local = self.reserve_temp_local();
                    let length_initial_writable_local = self.reserve_temp_local();
                    self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_known_array_index_from_property_key(
                        key_local,
                        array_index_local,
                        array_index_found_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(array_index_found_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_assignment_write(
                        target_local,
                        array_index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(length_key_local));
                    self.emit_string_payload_equality_i32(key_local, length_key_local, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_writable_present_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(length_allow_define_local));
                    self.emit_array_length_writable_i64(
                        target_local,
                        length_initial_writable_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(length_initial_writable_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_set_length_from_value(
                        target_local,
                        payload_local,
                        tag_local,
                        length_writable_present_local,
                        length_writable_present_local,
                        length_allow_define_local,
                        length_success_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(length_success_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(length_success_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write_set_failure_else(
                        "Cannot assign to array length",
                        0,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_known_array_index_from_property_key(
                        key_local,
                        array_index_local,
                        array_index_found_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(array_index_found_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_typed_array_or_object_index_write_from_locals(
                        target_local,
                        target_tag_local,
                        array_index_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(set_result_local));
                    self.emit_array_define_named_data_descriptor(
                        target_local,
                        key_local,
                        payload_local,
                        tag_local,
                        set_result_local,
                        set_result_local,
                        set_result_local,
                        None,
                        None,
                        None,
                        None,
                        None,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(length_initial_writable_local);
                    self.release_temp_local(length_allow_define_local);
                    self.release_temp_local(length_writable_present_local);
                    self.release_temp_local(length_success_local);
                    self.release_temp_local(length_key_local);
                    self.release_temp_local(array_index_found_local);
                    self.release_temp_local(array_index_local);
                    self.release_temp_local(key_tag_local);
                    self.release_temp_local(set_result_local);
                    self.release_temp_local(key_local);
                }
            }
            _ => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_object_write(
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
            }
        }

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    pub(crate) fn emit_validate_typed_array_from_constructed_target(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        requested_length_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let requested_length_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(brand_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Constructed target is not a typed array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            bytes_per_element_local,
            function,
        );
        self.emit_validate_typed_array_current_byte_length(
            target_payload_local,
            target_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(requested_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(requested_length_local));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(capacity_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::LocalGet(requested_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Constructed typed array is too small",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(capacity_local);
        self.release_temp_local(requested_length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(brand_local);
        Ok(())
    }

    /// Applies the integer typed-array conversion modulo 2^32. Stores for
    /// narrower element kinds consume the corresponding low bits.
    pub(crate) fn emit_integer_typed_array_value_i64(
        &mut self,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        let truncated_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
            function.instruction(&Instruction::LocalGet(number_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(truncated_local));
        function.instruction(&Instruction::LocalGet(truncated_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(truncated_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4_294_967_296.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(4_294_967_296.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::End);

        self.release_temp_local(remainder_local);
        self.release_temp_local(truncated_local);
    }

    pub(crate) fn emit_store_number_payload_to_typed_array_address_by_kind(
        &mut self,
        bytes_per_element_local: u32,
        element_kind_local: u32,
        address_local: u32,
        number_payload_local: u32,
        memory_index: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Store(Self::memarg64_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F32DemoteF64);
        function.instruction(&Instruction::F32Store(Self::memarg32_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(255.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(255));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I32TruncSatF64U);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Store8(Self::memarg8_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        self.emit_integer_typed_array_value_i64(number_payload_local, function);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store16(Self::memarg16_in(memory_index, 0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store(Self::memarg32_in(memory_index, 0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_to_bigint_u64_word_from_value_locals(
        &mut self,
        value_tag_local: u32,
        value_payload_local: u32,
        word_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bigint_payload_local = self.reserve_temp_local();
        let bigint_tag_local = self.reserve_temp_local();

        self.emit_to_bigint_value_and_u64_word_from_value_locals(
            value_tag_local,
            value_payload_local,
            bigint_payload_local,
            bigint_tag_local,
            word_payload_local,
            function,
        )?;

        self.release_temp_local(bigint_tag_local);
        self.release_temp_local(bigint_payload_local);
        Ok(())
    }

    pub(crate) fn emit_to_bigint_value_and_u64_word_from_value_locals(
        &mut self,
        value_tag_local: u32,
        value_payload_local: u32,
        bigint_payload_local: u32,
        bigint_tag_local: u32,
        word_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let heap_sign_local = self.reserve_temp_local();
        let heap_limbs_local = self.reserve_temp_local();

        self.emit_value_to_bigint_locals(
            value_tag_local,
            value_payload_local,
            false,
            bigint_payload_local,
            bigint_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(bigint_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            heap_sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            heap_limbs_local,
            function,
        );
        // ToBigInt64 and ToBigUint64 both store the low 64 bits. Negating the
        // least-significant magnitude limb gives the same modulo-2^64 bit
        // pattern for negative values, regardless of the remaining limbs.
        function.instruction(&Instruction::LocalGet(heap_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(heap_limbs_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(heap_limbs_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(bigint_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(word_payload_local));

        self.release_temp_local(heap_limbs_local);
        self.release_temp_local(heap_sign_local);
        Ok(())
    }

    pub(crate) fn emit_value_to_typed_array_element_payload(
        &mut self,
        element_kind_local: u32,
        value_tag_local: u32,
        value_payload_local: u32,
        element_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_bigint_u64_word_from_value_locals(
            value_tag_local,
            value_payload_local,
            element_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::End);

        Ok(())
    }

    pub(crate) fn emit_typed_array_element_write_from_locals(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_payload_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            bytes_per_element_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );
        self.emit_value_to_typed_array_element_payload(
            element_kind_local,
            value_tag_local,
            value_payload_local,
            number_payload_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_typed_array_current_byte_length(
            target_local,
            target_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));

        self.emit_store_number_payload_to_typed_array_address_by_kind(
            bytes_per_element_local,
            element_kind_local,
            address_local,
            number_payload_local,
            self.buffer_memory_index(),
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(address_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_payload_local);
        Ok(())
    }

    fn emit_arguments_length_delete(
        &mut self,
        arguments_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(descriptor_kind_local);
    }

    fn emit_arguments_callee_delete(
        &mut self,
        arguments_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(descriptor_kind_local);
    }

    fn emit_arguments_delete_property_key(
        &mut self,
        arguments_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length_delete(arguments_local, result_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_callee_delete(arguments_local, result_local, function);
        function.instruction(&Instruction::Else);
        self.emit_array_delete_property_key(arguments_local, key_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_delete_property_i32(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        strict: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;

        match target.kind {
            ValueKind::Object | ValueKind::Function => {
                let (key_local, converted_key_tag_local) =
                    if let PropertyKeyIr::StringExpr(key_expr) = key {
                        let key_local = self.reserve_temp_local();
                        let converted_tag_local = self.reserve_temp_local();
                        self.compile_expr_to_locals(
                            key_expr,
                            key_local,
                            converted_tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            key_local,
                            converted_tag_local,
                            function,
                        )?;
                        self.emit_value_to_property_key_locals(
                            key_local,
                            converted_tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            key_local,
                            converted_tag_local,
                            function,
                        )?;
                        (key_local, Some(converted_tag_local))
                    } else {
                        (self.compile_object_key_to_local(key, function)?, None)
                    };
                let array_index_local = if matches!(key, PropertyKeyIr::ArrayIndex(_)) {
                    Some(self.compile_array_index_to_local(key, function)?)
                } else {
                    None
                };
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_delete(target_local, array_index_local, result_local, function);
                    function.instruction(&Instruction::Else);
                    self.emit_object_delete(
                        target_local,
                        target_tag_local,
                        key_local,
                        result_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                } else {
                    if let Some(converted_key_tag_local) = converted_key_tag_local {
                        let canonical_numeric_index_local = self.reserve_temp_local();
                        let typed_array_key_handled_local = self.reserve_temp_local();
                        let typed_array_index_local = self.reserve_temp_local();
                        self.emit_typed_array_canonical_numeric_index_i32(
                            target_local,
                            target_tag_local,
                            key_local,
                            converted_key_tag_local,
                            canonical_numeric_index_local,
                            typed_array_key_handled_local,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalGet(typed_array_key_handled_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::I64Ne);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.emit_typed_array_valid_integer_index_i32(
                            target_local,
                            target_tag_local,
                            canonical_numeric_index_local,
                            typed_array_index_local,
                            result_local,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalGet(result_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::I64ExtendI32U);
                        function.instruction(&Instruction::LocalSet(result_local));
                        function.instruction(&Instruction::Else);
                        self.emit_object_delete(
                            target_local,
                            target_tag_local,
                            key_local,
                            result_local,
                            function,
                        )?;
                        function.instruction(&Instruction::End);
                        self.release_temp_local(typed_array_index_local);
                        self.release_temp_local(typed_array_key_handled_local);
                        self.release_temp_local(canonical_numeric_index_local);
                    } else {
                        self.emit_object_delete(
                            target_local,
                            target_tag_local,
                            key_local,
                            result_local,
                            function,
                        )?;
                    }
                }
                if let Some(converted_key_tag_local) = converted_key_tag_local {
                    self.release_temp_local(converted_key_tag_local);
                }
                self.release_temp_local(key_local);
            }
            ValueKind::Array => match key {
                PropertyKeyIr::ArrayIndex(_) => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_array_delete(target_local, index_local, result_local, function);
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::ArrayLength => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
                }
                PropertyKeyIr::StaticString(name) if name == "length" => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
                }
                PropertyKeyIr::StaticString(name) => {
                    if let Some(index) = static_array_index_name(name) {
                        let index_local = self.reserve_temp_local();
                        function.instruction(&Instruction::I64Const(index as i64));
                        function.instruction(&Instruction::LocalSet(index_local));
                        self.emit_array_delete(target_local, index_local, result_local, function);
                        self.release_temp_local(index_local);
                    } else {
                        let key_local = self.compile_object_key_to_local(key, function)?;
                        self.emit_array_named_prop_delete(
                            target_local,
                            key_local,
                            result_local,
                            function,
                        );
                        self.release_temp_local(key_local);
                    }
                }
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.emit_array_delete_property_key(
                        target_local,
                        key_local,
                        result_local,
                        function,
                    );
                    self.release_temp_local(key_local);
                }
            },
            ValueKind::Arguments => match key {
                PropertyKeyIr::ArrayIndex(_) => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_array_delete(target_local, index_local, result_local, function);
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::StaticString(name) if static_array_index_name(name).is_some() => {
                    let index_local = self.reserve_temp_local();
                    function.instruction(&Instruction::I64Const(
                        static_array_index_name(name).expect("arguments index") as i64,
                    ));
                    function.instruction(&Instruction::LocalSet(index_local));
                    self.emit_array_delete(target_local, index_local, result_local, function);
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::ArrayLength => {
                    self.emit_arguments_length_delete(target_local, result_local, function);
                }
                PropertyKeyIr::StaticString(name) if name == "length" => {
                    self.emit_arguments_length_delete(target_local, result_local, function);
                }
                PropertyKeyIr::StaticString(name) if name == "callee" => {
                    self.emit_arguments_callee_delete(target_local, result_local, function);
                }
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.emit_arguments_delete_property_key(
                        target_local,
                        key_local,
                        result_local,
                        function,
                    );
                    self.release_temp_local(key_local);
                }
                _ => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    self.emit_object_delete(
                        target_local,
                        target_tag_local,
                        key_local,
                        result_local,
                        function,
                    )?;
                    self.release_temp_local(key_local);
                }
            },
            ValueKind::Dynamic => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                let array_index_local = if matches!(key, PropertyKeyIr::ArrayIndex(_)) {
                    Some(self.compile_array_index_to_local(key, function)?)
                } else {
                    None
                };
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(result_local));
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_delete(target_local, array_index_local, result_local, function);
                    function.instruction(&Instruction::Else);
                } else {
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_delete_property_key(
                        target_local,
                        key_local,
                        result_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                }
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                if let Some(array_index_local) = array_index_local {
                    self.emit_array_delete(target_local, array_index_local, result_local, function);
                } else {
                    self.emit_arguments_delete_property_key(
                        target_local,
                        key_local,
                        result_local,
                        function,
                    );
                }
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_object_delete(
                    target_local,
                    target_tag_local,
                    key_local,
                    result_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                } else {
                    function.instruction(&Instruction::End);
                }
                self.release_temp_local(key_local);
            }
            // 13.5.1.2 step 5 performs `ToObject(baseValue)` on the reference
            // base, so `delete undefined.p` / `delete null[k]` is a runtime
            // TypeError rather than an unsupported construct.
            ValueKind::Undefined | ValueKind::Null => {
                // The property key expression belongs to the MemberExpression
                // and is evaluated (GetValue only, no ToPropertyKey) before
                // `delete` coerces the base, so its side effects still run.
                match key {
                    PropertyKeyIr::StringExpr(key_expr) | PropertyKeyIr::ArrayIndex(key_expr) => {
                        let key_payload_local = self.reserve_temp_local();
                        let key_tag_local = self.reserve_temp_local();
                        self.compile_expr_to_locals(
                            key_expr,
                            key_payload_local,
                            key_tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            key_payload_local,
                            key_tag_local,
                            function,
                        )?;
                        self.release_temp_local(key_tag_local);
                        self.release_temp_local(key_payload_local);
                    }
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
                }
                // The throw below always fires; seed the result so the strict
                // `Cannot delete property` check that follows stays inert on
                // the statically dead fall-through path.
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(result_local));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert undefined or null to object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
            }
            _ => {
                self.release_temp_local(result_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: delete on non-object target",
                ));
            }
        }

        if strict {
            function.instruction(&Instruction::LocalGet(result_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "Cannot delete property",
                self.result_local,
                self.result_tag_local,
                0,
                function,
            )?;
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(result_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    pub(crate) fn emit_in_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.compile_expr_to_primitive_locals(
            lhs,
            ToPrimitiveHint::String,
            lhs_payload_local,
            lhs_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        match rhs.kind {
            ValueKind::Object | ValueKind::Function => {
                self.emit_value_to_property_key_payload(
                    lhs_payload_local,
                    lhs_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_property_key_tag_from_source_tag(lhs_tag_local, key_tag_local, function);
                self.emit_object_has_property_with_key_tag_i32(
                    rhs_payload_local,
                    rhs_tag_local,
                    key_local,
                    key_tag_local,
                    result_local,
                    function,
                )?;
            }
            ValueKind::Array => {
                self.emit_value_to_property_key_payload(
                    lhs_payload_local,
                    lhs_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_property_key_tag_from_source_tag(lhs_tag_local, key_tag_local, function);
                self.emit_object_has_property_with_key_tag_i32(
                    rhs_payload_local,
                    rhs_tag_local,
                    key_local,
                    key_tag_local,
                    result_local,
                    function,
                )?;
            }
            ValueKind::Arguments => {
                if lhs.kind == ValueKind::Number {
                    function.instruction(&Instruction::LocalGet(lhs_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::I64TruncF64U);
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_arguments_has_index_i32(
                        rhs_payload_local,
                        key_local,
                        result_local,
                        function,
                    )?;
                } else {
                    self.emit_value_to_property_key_payload(
                        lhs_payload_local,
                        lhs_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        rhs_payload_local,
                        HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
                        result_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(result_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::Else);
                    for (digit, name) in ["0", "1", "2", "3", "4"].iter().enumerate() {
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        self.emit_string_payload_equality_i32(
                            key_local,
                            self.scratch_local,
                            function,
                        );
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(digit as i64));
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        self.emit_arguments_has_index_i32(
                            rhs_payload_local,
                            self.scratch_local,
                            result_local,
                            function,
                        )?;
                        function.instruction(&Instruction::End);
                    }
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
            }
            ValueKind::Dynamic => {
                self.emit_value_to_property_key_payload(
                    lhs_payload_local,
                    lhs_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_property_key_tag_from_source_tag(lhs_tag_local, key_tag_local, function);
                self.emit_is_heap_object_like_tag_i32(rhs_tag_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_object_has_property_with_key_tag_i32(
                    rhs_payload_local,
                    rhs_tag_local,
                    key_local,
                    key_tag_local,
                    result_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    "TypeError",
                    "right-hand side of `in` is not an object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 1, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
            }
            _ => {
                self.emit_throw_runtime_error(
                    "TypeError",
                    "right-hand side of `in` is not an object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 0, function);
                } else {
                    self.emit_return_current_completion(function);
                }
            }
        }

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(result_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(crate) fn emit_current_private_environment_to_local(
        &mut self,
        private_environment_local: u32,
        function: &mut Function,
    ) {
        if let Some(active_private_environment_local) =
            self.active_private_environment_locals.last().copied()
        {
            function.instruction(&Instruction::LocalGet(active_private_environment_local));
            function.instruction(&Instruction::LocalSet(private_environment_local));
            return;
        }
        if self
            .current_function_meta()
            .is_some_and(WasmFunctionMeta::has_function_context)
        {
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local,
                function,
            );
            return;
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(private_environment_local));
    }

    pub(crate) fn emit_private_name_token_to_local(
        &mut self,
        private_name_id: PrivateNameId,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.active_private_environment_locals.is_empty()
            && !self
                .current_function_meta()
                .is_some_and(WasmFunctionMeta::has_function_context)
        {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: private name outside class execution context",
            ));
        }
        self.emit_current_private_environment_to_local(token_local, function);

        let stored_class_scope_local = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "private environment is missing its declared name",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 3, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            token_local,
            HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
            stored_class_scope_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_class_scope_local));
        function.instruction(&Instruction::I64Const(private_name_id.class_scope() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            token_local,
            HEAP_PRIVATE_ENV_PARENT_OFFSET,
            token_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Const(
            (HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET
                + private_name_id.name_ordinal() as u64 * HEAP_PRIVATE_ENV_SLOT_SIZE)
                as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(token_local));
        self.release_temp_local(stored_class_scope_local);
        Ok(())
    }

    pub(crate) fn emit_private_brand_add(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            Some((receiver_payload_local, receiver_tag_local)),
            token_local,
            PRIVATE_ELEMENT_KIND_BRAND,
            None,
            function,
        )
    }

    pub(crate) fn emit_private_field_add(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        token_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            Some((receiver_payload_local, receiver_tag_local)),
            token_local,
            PRIVATE_ELEMENT_KIND_FIELD,
            Some((value_payload_local, value_tag_local)),
            function,
        )
    }

    pub(crate) fn emit_private_setter_definition_add(
        &mut self,
        token_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            None,
            token_local,
            PRIVATE_ELEMENT_KIND_SETTER,
            Some((setter_payload_local, setter_tag_local)),
            function,
        )
    }

    pub(crate) fn emit_private_method_definition_add(
        &mut self,
        token_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            None,
            token_local,
            PRIVATE_ELEMENT_KIND_METHOD,
            Some((method_payload_local, method_tag_local)),
            function,
        )
    }

    pub(crate) fn emit_private_getter_definition_add(
        &mut self,
        token_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            None,
            token_local,
            PRIVATE_ELEMENT_KIND_GETTER,
            Some((getter_payload_local, getter_tag_local)),
            function,
        )
    }

    fn emit_private_element_entry_add(
        &mut self,
        receiver_locals: Option<(u32, u32)>,
        token_local: u32,
        kind: u64,
        value_locals: Option<(u32, u32)>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let previous_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        if let Some((receiver_payload_local, receiver_tag_local)) = receiver_locals {
            let extensible_local = self.reserve_temp_local();
            self.emit_object_is_extensible_i32(
                receiver_payload_local,
                receiver_tag_local,
                extensible_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(extensible_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "private element cannot be installed on non-extensible object",
                self.result_local,
                self.result_tag_local,
                1,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.release_temp_local(extensible_local);

            let existing_entry_local = self.reserve_temp_local();
            self.emit_private_element_find(
                receiver_payload_local,
                token_local,
                existing_entry_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_entry_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "private element already installed on object",
                self.result_local,
                self.result_tag_local,
                1,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.release_temp_local(existing_entry_local);
        }

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            previous_local,
            function,
        );
        self.emit_heap_alloc_const(HEAP_PRIVATE_ELEMENT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            previous_local,
            function,
        );
        if let Some((receiver_payload_local, _)) = receiver_locals {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                receiver_payload_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                0,
                function,
            );
        }
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            token_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            kind,
            function,
        );
        if let Some((value_payload_local, value_tag_local)) = value_locals {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                value_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                value_payload_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                0,
                function,
            );
        }
        self.store_i64_local_at_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );

        self.release_temp_local(entry_local);
        self.release_temp_local(previous_local);
        self.release_temp_local(realm_local);
        Ok(())
    }

    pub(crate) fn emit_private_element_find(
        &mut self,
        receiver_local: u32,
        token_local: u32,
        entry_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let stored_receiver_local = self.reserve_temp_local();
        let stored_token_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
            stored_receiver_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            stored_token_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_receiver_local));
        function.instruction(&Instruction::LocalGet(receiver_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stored_token_local));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_token_local);
        self.release_temp_local(stored_receiver_local);
        self.release_temp_local(realm_local);
    }

    fn emit_private_element_definition_find(
        &mut self,
        token_local: u32,
        kind: u64,
        entry_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let stored_token_local = self.reserve_temp_local();
        let stored_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            stored_token_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            stored_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_token_local));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stored_kind_local));
        function.instruction(&Instruction::I64Const(kind as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_kind_local);
        self.release_temp_local(stored_token_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_private_brand_has_i32(
        &mut self,
        receiver_local: u32,
        token_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let entry_local = self.reserve_temp_local();
        self.emit_private_element_find(receiver_local, token_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        self.release_temp_local(entry_local);
    }

    pub(crate) fn compile_private_read_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_private_read_from_locals(
            target_payload_local,
            target_tag_local,
            private_name_id,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_private_read_from_locals(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_token_local = self.reserve_temp_local();
        let has_brand_local = self.reserve_temp_local();
        let private_entry_local = self.reserve_temp_local();
        let private_kind_local = self.reserve_temp_local();
        let definition_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_token_local,
            has_brand_local,
            function,
        )?;
        self.emit_private_element_find(
            target_payload_local,
            brand_token_local,
            private_entry_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            private_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(PRIVATE_ELEMENT_KIND_FIELD as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_private_element_definition_find(
            brand_token_local,
            PRIVATE_ELEMENT_KIND_METHOD,
            definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_private_element_definition_find(
            brand_token_local,
            PRIVATE_ELEMENT_KIND_GETTER,
            definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "private accessor has no getter",
            self.result_local,
            self.result_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(PRIVATE_ELEMENT_KIND_GETTER as i64));
        function.instruction(&Instruction::LocalSet(private_kind_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(PRIVATE_ELEMENT_KIND_METHOD as i64));
        function.instruction(&Instruction::LocalSet(private_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(PRIVATE_ELEMENT_KIND_GETTER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_throw_extra_depth(
            getter_payload_local,
            getter_tag_local,
            Some((target_payload_local, Some(target_tag_local))),
            &[],
            payload_local,
            tag_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(definition_local);
        self.release_temp_local(private_kind_local);
        self.release_temp_local(private_entry_local);
        self.release_temp_local(has_brand_local);
        self.release_temp_local(brand_token_local);
        Ok(())
    }

    pub(crate) fn compile_private_write_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        value: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.emit_private_write_from_locals(
            target_payload_local,
            target_tag_local,
            private_name_id,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_private_write_from_locals(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_token_local = self.reserve_temp_local();
        let has_brand_local = self.reserve_temp_local();
        let private_entry_local = self.reserve_temp_local();
        let private_kind_local = self.reserve_temp_local();
        let setter_definition_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();

        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_token_local,
            has_brand_local,
            function,
        )?;
        self.emit_private_element_find(
            target_payload_local,
            brand_token_local,
            private_entry_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            private_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(PRIVATE_ELEMENT_KIND_FIELD as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_private_element_definition_find(
            brand_token_local,
            PRIVATE_ELEMENT_KIND_SETTER,
            setter_definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(setter_definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "private element has no setter",
            self.result_local,
            self.result_tag_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            setter_definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            setter_definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_function_handle_call_with_throw_extra_depth(
            setter_payload_local,
            setter_tag_local,
            Some((target_payload_local, Some(target_tag_local))),
            &[(value_payload_local, value_tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(setter_definition_local);
        self.release_temp_local(private_kind_local);
        self.release_temp_local(private_entry_local);
        self.release_temp_local(has_brand_local);
        self.release_temp_local(brand_token_local);
        Ok(())
    }

    pub(crate) fn emit_private_brand_guard(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        brand_token_local: u32,
        has_brand_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_private_name_token_to_local(private_name_id, brand_token_local, function)?;
        self.emit_private_brand_has_i32(
            target_payload_local,
            brand_token_local,
            has_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_brand_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 2, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 1, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_object_key_to_locals(
        &mut self,
        key: &PropertyKeyIr,
        key_local: u32,
        key_tag_output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match key {
            PropertyKeyIr::StaticString(value) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_output_local));
            }
            PropertyKeyIr::ArrayLength => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: object key kind",
                ));
            }
            PropertyKeyIr::StringExpr(expr) => {
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(expr, key_payload_local, key_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                let property_key_kinds = KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Symbol));
                if !expr.possible_kinds.is_subset_of(property_key_kinds) {
                    self.emit_value_to_property_key_locals(
                        key_payload_local,
                        key_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        key_payload_local,
                        key_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(key_payload_local));
                function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(key_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(key_payload_local));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::LocalSet(key_tag_output_local));
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
            }
            PropertyKeyIr::ArrayIndex(expr) => {
                let index_payload_local = self.reserve_temp_local();
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(index_payload_local));
                self.emit_number_to_string_payload(index_payload_local, function)?;
                self.release_temp_local(index_payload_local);
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_output_local));
            }
        }
        Ok(())
    }

    pub(crate) fn compile_object_key_to_local(
        &mut self,
        key: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        if let Err(err) = self.compile_object_key_to_locals(key, key_local, key_tag_local, function)
        {
            self.release_temp_local(key_tag_local);
            self.release_temp_local(key_local);
            return Err(err);
        }
        self.release_temp_local(key_tag_local);
        Ok(key_local)
    }

    pub(crate) fn compile_array_index_to_local(
        &mut self,
        key: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let index_local = self.reserve_temp_local();
        let PropertyKeyIr::ArrayIndex(expr) = key else {
            self.release_temp_local(index_local);
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: array index kind",
            ));
        };
        self.compile_expr_payload(expr, function)?;
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            MAX_ARRAY_LENGTH as f64,
        )));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            MAX_ARRAY_LENGTH as f64,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(index_local));
        Ok(index_local)
    }

    pub(crate) fn emit_property_key_tag_from_payload(
        &self,
        key_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        self.emit_property_key_payload_is_symbol_i32(key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_property_key_payload_is_symbol_i32(
        &self,
        key_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
    }

    pub(crate) fn emit_property_key_payload_to_value_payload(
        &self,
        key_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(!PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64And);
    }

    /// Materializes the observable JS *value* payload of an internal property
    /// key into `value_payload_local`.
    ///
    /// Internal property-key payloads carry `PROPERTY_KEY_SYMBOL_MARKER` for
    /// symbol keys, but a symbol *value* never does. Any site that hands a
    /// property key back to user code — every proxy trap argument, in
    /// particular — must strip the marker first, otherwise `key === symbol`
    /// is observably false inside the trap and string concatenation of the key
    /// reads a bogus payload.
    pub(crate) fn emit_property_key_value_payload_to_local(
        &self,
        key_local: u32,
        value_payload_local: u32,
        function: &mut Function,
    ) {
        self.emit_property_key_payload_to_value_payload(key_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
    }

    /// Inverse of [`Self::emit_property_key_value_payload_to_local`]: converts a
    /// String/Symbol *value* (payload plus tag) that already is a property key
    /// — e.g. an element of a `Reflect.ownKeys` result, or a key produced by
    /// `ToPropertyKey` — into the internal property-key payload encoding by
    /// re-applying `PROPERTY_KEY_SYMBOL_MARKER` for symbols.
    pub(crate) fn emit_property_key_payload_from_value_local(
        &self,
        value_payload_local: u32,
        value_tag_local: u32,
        key_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_property_key_payload_equality_i32(
        &mut self,
        stored_key_local: u32,
        key_local: u32,
        function: &mut Function,
    ) {
        self.emit_property_key_payload_is_symbol_i32(key_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_property_key_payload_is_symbol_i32(stored_key_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(stored_key_local));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_property_key_payload_is_symbol_i32(stored_key_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32(stored_key_local, key_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_property_key_tag_from_source_tag(
        &self,
        source_tag_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_object_read(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_with_key_tag(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            None,
            payload_local,
            tag_local,
            function,
        )
    }

    /// Emits a proxy-aware `[[Get]]` while leaving an abrupt completion in
    /// `payload_local`/`tag_local` for the caller to handle. This is needed by
    /// operations that are currently inside raw Wasm control blocks, where a
    /// direct completion branch would not have a tracked control-stack depth.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_object_read_without_throw_propagation(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_without_throw_propagation_inner(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            None,
            payload_local,
            tag_local,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_object_read_without_throw_propagation_with_key_tag(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_without_throw_propagation_inner(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            Some(key_tag_local),
            payload_local,
            tag_local,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_object_read_without_throw_propagation_inner(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_override_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            if let Some(helper) = self.object_read_proxy_helper_function_index() {
                let key_tag_local = self.reserve_temp_local();
                if let Some(key_tag_override_local) = key_tag_override_local {
                    function.instruction(&Instruction::LocalGet(key_tag_override_local));
                    function.instruction(&Instruction::LocalSet(key_tag_local));
                } else {
                    self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
                }
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalGet(object_tag_local));
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                self.release_temp_local(key_tag_local);
                return Ok(());
            }
        }

        self.emit_object_read_ordinary_without_accessor_throw_propagation(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_object_read_with_key_tag(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_override_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            return self.emit_object_read_ordinary(
                object_local,
                object_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                payload_local,
                tag_local,
                function,
            );
        }

        if self.outline_object_read_proxy {
            if let Some(helper) = self.object_read_proxy_helper_function_index() {
                // The helper takes the key tag explicitly (param 5); compute it
                // here (respecting an override) so the helper body has a fixed
                // signature.
                let key_tag_local = self.reserve_temp_local();
                if let Some(key_tag_override_local) = key_tag_override_local {
                    function.instruction(&Instruction::LocalGet(key_tag_override_local));
                    function.instruction(&Instruction::LocalSet(key_tag_local));
                } else {
                    self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
                }
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalGet(object_tag_local));
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                // Store the value (or, on a throw, the thrown value) into the
                // caller's value locals and set the completion tuple, mirroring
                // the outlined ordinary-read path (`emit_object_read_ordinary_inner`).
                // `result_local` is not touched, so no caller value held across
                // the read is clobbered.
                self.store_call_results(payload_local, tag_local, function);
                // Inside the outlined helper a proxy-trap throw has no active
                // handler, so it is surfaced as a throw completion with the thrown
                // value in the caller's `payload`/`tag` locals. Propagate it here
                // (to the active handler, or by returning) so callers that do not
                // separately check the read's completion — e.g. the JSON.stringify
                // builtin's `LengthOfArrayLike` — still see the abrupt completion,
                // matching the inline wrapper's throw discipline.
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_tag_local);
                return Ok(());
            }
        }

        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let nested_target_payload_local = self.reserve_temp_local();
        let nested_target_tag_local = self.reserve_temp_local();
        let nested_trap_payload_local = self.reserve_temp_local();
        let nested_trap_tag_local = self.reserve_temp_local();
        let internal_key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let trap_key_payload_local = self.reserve_temp_local();

        // The `get` trap observes the key, so it must see the unmarked symbol
        // value rather than the internal property-key payload.
        self.emit_property_key_value_payload_to_local(key_local, trap_key_payload_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
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
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(internal_key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            internal_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(key_tag_override_local) = key_tag_override_local {
            function.instruction(&Instruction::LocalGet(key_tag_override_local));
            function.instruction(&Instruction::LocalSet(key_tag_local));
        } else {
            self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        }
        self.emit_function_handle_call_without_throw_propagation(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (trap_key_payload_local, key_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_proxy_get_invariant_check(
            target_payload_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            nested_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            nested_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(internal_key_local));
        self.emit_object_own_data_field_read(
            handler_payload_local,
            handler_tag_local,
            internal_key_local,
            key_tag_local,
            nested_trap_payload_local,
            nested_trap_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nested_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(key_tag_override_local) = key_tag_override_local {
            function.instruction(&Instruction::LocalGet(key_tag_override_local));
            function.instruction(&Instruction::LocalSet(key_tag_local));
        } else {
            self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        }
        self.emit_function_handle_call(
            nested_trap_payload_local,
            nested_trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (nested_target_payload_local, nested_target_tag_local),
                (trap_key_payload_local, key_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(nested_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(nested_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy get trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(nested_target_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(nested_target_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_ordinary_get_by_tag(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy get trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_object_read_ordinary(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(trap_key_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(internal_key_local);
        self.release_temp_local(nested_trap_tag_local);
        self.release_temp_local(nested_trap_payload_local);
        self.release_temp_local(nested_target_tag_local);
        self.release_temp_local(nested_target_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_get_invariant_check(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_local: u32,
        trap_result_payload_local: u32,
        trap_result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let target_value_payload_local = self.reserve_temp_local();
        let target_value_tag_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            target_value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            target_value_tag_local,
            function,
        );
        self.emit_tagged_payload_same_value_i32(
            trap_result_tag_local,
            trap_result_payload_local,
            target_value_tag_local,
            target_value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy get trap returned inconsistent frozen data property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(trap_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy get trap returned value for accessor without getter",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(target_value_tag_local);
        self.release_temp_local(target_value_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_has_invariant_check(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_local: u32,
        trap_result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(trap_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy has trap returned false for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_CAP_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy has trap returned false for non-extensible target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cap_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_empty_own_keys_invariant_check(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_CAP_OFFSET,
            cap_local,
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
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cap_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_object_keys_from_own_keys_result(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        handler_payload_local: u32,
        handler_tag_local: u32,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_len_local = self.reserve_temp_local();
        let snapshot_payload_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_key_payload_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let desc_payload_local = self.reserve_temp_local();
        let desc_tag_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let target_buffer_local = self.reserve_temp_local();
        let target_len_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let target_entry_local = self.reserve_temp_local();
        let target_entry_key_local = self.reserve_temp_local();
        let target_descriptor_kind_local = self.reserve_temp_local();
        let duplicate_index_local = self.reserve_temp_local();
        let duplicate_key_payload_local = self.reserve_temp_local();
        let duplicate_key_tag_local = self.reserve_temp_local();
        let target_cap_local = self.reserve_temp_local();
        let target_entry_key_tag_local = self.reserve_temp_local();
        let invariant_found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_len_local));
        function.instruction(&Instruction::LocalGet(own_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(own_keys_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_read(
            own_keys_payload_local,
            own_keys_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            index_key_payload_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            key_tag_local,
            key_payload_local,
            list_len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_empty_own_keys_invariant_check(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(
            list_len_local,
            snapshot_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(own_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            own_keys_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_read(
            own_keys_payload_local,
            own_keys_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            index_key_payload_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_write(
            snapshot_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            target_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            target_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_CAP_OFFSET,
            target_cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_buffer_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_entry_local));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            target_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            target_entry_key_local,
            function,
        );
        self.emit_property_key_tag_from_payload(
            target_entry_key_local,
            target_entry_key_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            target_entry_key_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        self.emit_property_key_payload_to_value_payload(target_entry_key_local, function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_buffer_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_entry_local));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            target_entry_key_local,
            function,
        );
        self.emit_property_key_tag_from_payload(
            target_entry_key_local,
            target_entry_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            target_entry_key_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        self.emit_property_key_payload_to_value_payload(target_entry_key_local, function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(list_len_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("getOwnPropertyDescriptor"),
        ));
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            index_key_payload_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[
                (target_payload_local, target_tag_local),
                (key_payload_local, key_tag_local),
            ],
            desc_payload_local,
            desc_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            target_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            target_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_buffer_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_entry_local));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            target_entry_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(target_entry_key_local, key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            target_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(desc_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(desc_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy getOwnPropertyDescriptor trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(desc_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_own_data_field_read(
            desc_payload_local,
            desc_tag_local,
            index_key_payload_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        self.compile_truthy_tagged_i32(enumerable_tag_local, enumerable_payload_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
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
        self.emit_return_current_completion(function);

        self.release_temp_local(invariant_found_local);
        self.release_temp_local(target_entry_key_tag_local);
        self.release_temp_local(target_cap_local);
        self.release_temp_local(duplicate_key_tag_local);
        self.release_temp_local(duplicate_key_payload_local);
        self.release_temp_local(duplicate_index_local);
        self.release_temp_local(target_descriptor_kind_local);
        self.release_temp_local(target_entry_key_local);
        self.release_temp_local(target_entry_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(target_len_local);
        self.release_temp_local(target_buffer_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(desc_tag_local);
        self.release_temp_local(desc_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(index_key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(snapshot_payload_local);
        self.release_temp_local(list_len_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_own_keys_trap_result(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        handled_local: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        handler_payload_local: u32,
        handler_tag_local: u32,
        trap_payload_local: u32,
        trap_tag_local: u32,
        trap_result_payload_local: u32,
        trap_result_tag_local: u32,
        key_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
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
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("ownKeys")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            key_payload_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_proxy_own_keys_validated_snapshot(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        list_len_local: u32,
        snapshot_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let duplicate_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let duplicate_key_payload_local = self.reserve_temp_local();
        let duplicate_key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_key_payload_local = self.reserve_temp_local();
        let target_buffer_local = self.reserve_temp_local();
        let target_len_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let target_entry_local = self.reserve_temp_local();
        let target_entry_key_local = self.reserve_temp_local();
        let target_entry_key_tag_local = self.reserve_temp_local();
        let target_descriptor_kind_local = self.reserve_temp_local();
        let target_cap_local = self.reserve_temp_local();
        let invariant_found_local = self.reserve_temp_local();
        let expected_keys_function_payload_local = self.reserve_temp_local();
        let expected_keys_function_tag_local = self.reserve_temp_local();
        let expected_keys_payload_local = self.reserve_temp_local();
        let expected_keys_tag_local = self.reserve_temp_local();
        let expected_keys_len_local = self.reserve_temp_local();
        let expected_key_index_local = self.reserve_temp_local();
        let expected_key_payload_local = self.reserve_temp_local();
        let expected_key_tag_local = self.reserve_temp_local();

        let reflect_own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_len_local));
        function.instruction(&Instruction::LocalGet(own_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(own_keys_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_read(
            own_keys_payload_local,
            own_keys_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            index_key_payload_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            key_tag_local,
            key_payload_local,
            list_len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(
            list_len_local,
            snapshot_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(own_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            own_keys_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_key_payload_local));
        self.emit_object_read(
            own_keys_payload_local,
            own_keys_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            index_key_payload_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result contained a non-property key",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result contained a duplicate key",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result contained a duplicate key",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_write(
            snapshot_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            target_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            target_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_CAP_OFFSET,
            target_cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_buffer_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_entry_local));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            target_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            target_entry_key_local,
            function,
        );
        self.emit_property_key_tag_from_payload(
            target_entry_key_local,
            target_entry_key_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            target_entry_key_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        self.emit_property_key_payload_to_value_payload(target_entry_key_local, function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::LocalGet(target_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_buffer_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_entry_local));
        self.load_i64_to_local_from_offset(
            target_entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            target_entry_key_local,
            function,
        );
        self.emit_property_key_tag_from_payload(
            target_entry_key_local,
            target_entry_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(target_entry_key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            target_entry_key_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        self.emit_property_key_payload_to_value_payload(target_entry_key_local, function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result contains an extra key for a non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expected_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_key_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            expected_key_index_local,
            expected_key_payload_local,
            expected_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(expected_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_string_payload_equality_i32(
            expected_key_payload_local,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(expected_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(expected_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result omitted target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
            target_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&reflect_own_keys_meta, function)?;
        function.instruction(&Instruction::LocalSet(expected_keys_function_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(expected_keys_function_tag_local));
        self.emit_function_handle_call(
            expected_keys_function_payload_local,
            expected_keys_function_tag_local,
            None,
            &[(target_payload_local, target_tag_local)],
            expected_keys_payload_local,
            expected_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            expected_keys_payload_local,
            HEAP_LEN_OFFSET,
            expected_keys_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(expected_keys_len_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result does not match non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expected_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_key_index_local));
        function.instruction(&Instruction::LocalGet(expected_keys_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            expected_keys_payload_local,
            expected_key_index_local,
            expected_key_payload_local,
            expected_key_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            duplicate_index_local,
            duplicate_key_payload_local,
            duplicate_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(duplicate_key_tag_local));
        function.instruction(&Instruction::LocalGet(expected_key_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            duplicate_key_payload_local,
            expected_key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(duplicate_key_payload_local));
        function.instruction(&Instruction::LocalGet(expected_key_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invariant_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duplicate_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invariant_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy ownKeys trap result does not match non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(expected_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(expected_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(expected_key_tag_local);
        self.release_temp_local(expected_key_payload_local);
        self.release_temp_local(expected_key_index_local);
        self.release_temp_local(expected_keys_len_local);
        self.release_temp_local(expected_keys_tag_local);
        self.release_temp_local(expected_keys_payload_local);
        self.release_temp_local(expected_keys_function_tag_local);
        self.release_temp_local(expected_keys_function_payload_local);
        self.release_temp_local(invariant_found_local);
        self.release_temp_local(target_cap_local);
        self.release_temp_local(target_descriptor_kind_local);
        self.release_temp_local(target_entry_key_tag_local);
        self.release_temp_local(target_entry_key_local);
        self.release_temp_local(target_entry_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(target_len_local);
        self.release_temp_local(target_buffer_local);
        self.release_temp_local(index_key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(duplicate_key_tag_local);
        self.release_temp_local(duplicate_key_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(duplicate_index_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_own_keys_filtered_result(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        expected_key_tag: ValueKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_len_local = self.reserve_temp_local();
        let snapshot_payload_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.emit_proxy_own_keys_validated_snapshot(
            target_payload_local,
            target_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            list_len_local,
            snapshot_payload_local,
            function,
        )?;

        self.emit_alloc_array_payload_with_length(list_len_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            snapshot_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(expected_key_tag.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        self.emit_return_current_completion(function);

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(snapshot_payload_local);
        self.release_temp_local(list_len_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_own_keys_array_result(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_len_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();

        self.emit_proxy_own_keys_validated_snapshot(
            target_payload_local,
            target_tag_local,
            own_keys_payload_local,
            own_keys_tag_local,
            list_len_local,
            result_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);

        self.release_temp_local(result_payload_local);
        self.release_temp_local(list_len_local);
        Ok(())
    }

    pub(crate) fn emit_object_own_data_field_read(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        present_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    /// ToPropertyDescriptor followed by FromPropertyDescriptor: converts an
    /// arbitrary attributes object into a freshly allocated ordinary object
    /// that owns exactly the fields the conversion found present.
    ///
    /// Reading descriptor fields straight out of the caller's object with
    /// `emit_object_own_data_field_read` misses inherited fields and own
    /// accessors, and re-reading the same object later would run user getters
    /// more than once.  The normalized object answers both problems: it is
    /// safe to read back with own-data reads, and absent fields stay absent.
    pub(crate) fn emit_to_property_descriptor_object(
        &mut self,
        descriptor_payload_local: u32,
        descriptor_tag_local: u32,
        type_error_message: &str,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let field_key_local = self.reserve_temp_local();
        let field_key_tag_local = self.reserve_temp_local();
        let value_present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let writable_present_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let writable_tag_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let configurable_present_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let configurable_tag_local = self.reserve_temp_local();
        let getter_present_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_present_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(descriptor_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            type_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // ToPropertyDescriptor observes the fields in this order, and reads
        // each present field exactly once after its HasProperty check.
        for (key, present_local, payload_local, tag_local) in [
            (
                "enumerable",
                enumerable_present_local,
                enumerable_payload_local,
                enumerable_tag_local,
            ),
            (
                "configurable",
                configurable_present_local,
                configurable_payload_local,
                configurable_tag_local,
            ),
            (
                "value",
                value_present_local,
                value_payload_local,
                value_tag_local,
            ),
            (
                "writable",
                writable_present_local,
                writable_payload_local,
                writable_tag_local,
            ),
            (
                "get",
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
            ),
            (
                "set",
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
            ),
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            function.instruction(&Instruction::LocalSet(field_key_local));
            self.emit_property_key_tag_from_payload(field_key_local, field_key_tag_local, function);
            self.emit_object_has_property_with_key_tag_i32(
                descriptor_payload_local,
                descriptor_tag_local,
                field_key_local,
                field_key_tag_local,
                present_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_object_read_without_throw_propagation(
                descriptor_payload_local,
                descriptor_tag_local,
                descriptor_payload_local,
                descriptor_tag_local,
                field_key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

            if matches!(key, "enumerable" | "configurable" | "writable") {
                // These fields are stored as the result of ToBoolean, so the
                // normalized object never carries the original tagged value.
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_to_boolean_payload_from_tagged_locals(
                    tag_local,
                    payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::End);
            }

            if matches!(key, "get" | "set") {
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
                self.emit_throw_current_function_realm_type_error(
                    "Property descriptor getter/setter must be callable or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Property descriptor cannot be both accessor and data",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // FromPropertyDescriptor: an ordinary object with the standard
        // prototype, carrying the present fields in specification order.
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        for (key, present_local, payload_local, tag_local) in [
            (
                "value",
                value_present_local,
                value_payload_local,
                value_tag_local,
            ),
            (
                "writable",
                writable_present_local,
                writable_payload_local,
                writable_tag_local,
            ),
            (
                "get",
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
            ),
            (
                "set",
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
            ),
            (
                "enumerable",
                enumerable_present_local,
                enumerable_payload_local,
                enumerable_tag_local,
            ),
            (
                "configurable",
                configurable_present_local,
                configurable_payload_local,
                configurable_tag_local,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            function.instruction(&Instruction::LocalSet(field_key_local));
            self.emit_object_define_enumerable_data(
                result_payload_local,
                field_key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(setter_present_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(getter_present_local);
        self.release_temp_local(configurable_tag_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(configurable_present_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(writable_tag_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(writable_present_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(value_present_local);
        self.release_temp_local(field_key_tag_local);
        self.release_temp_local(field_key_local);
        Ok(())
    }

    /// Read an Array instance's own named property, including invoking an own
    /// accessor. The lower-level `emit_array_named_prop_read` only publishes
    /// stored data and a presence bit for prototype-chain traversal.
    fn emit_array_own_named_property_read(
        &mut self,
        array_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        found_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
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
            stored_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(stored_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
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
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            getter_payload_local,
            getter_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_break_current_completion_if_throw(0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(stored_key_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_object_own_property_present(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        present_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(present_local));
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_ordinary_get_by_tag(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(object_local, payload_local, tag_local, function);
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            object_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read_ordinary(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_read_ordinary(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
        Ok(())
    }

    /// Emits a `call` into the shared object-read runtime helper, leaving the
    /// read value in `payload_local`/`tag_local` and the read's completion in
    /// the current completion locals. Returns `false` if outlining is disabled
    /// (e.g. while compiling the helper itself), in which case the caller must
    /// fall back to inlining.
    fn emit_object_read_ordinary_via_helper(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> bool {
        if !self.outline_object_read {
            return false;
        }
        let Some(helper) = self.object_read_helper_function_index() else {
            return false;
        };
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(payload_local, tag_local, function);
        true
    }

    pub(crate) fn emit_object_read_ordinary(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.emit_object_read_ordinary_via_helper(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        ) {
            return self.emit_propagate_throw_from_locals_if_needed(
                payload_local,
                tag_local,
                function,
            );
        }
        self.emit_object_read_ordinary_inner(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            Some(8),
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_object_read_ordinary_without_accessor_throw_propagation(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.emit_object_read_ordinary_via_helper(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        ) {
            return Ok(());
        }
        self.emit_object_read_ordinary_inner(
            object_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            None,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_object_read_ordinary_inner(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        accessor_throw_extra_depth: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_found_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let proxy_internal_key_local = self.reserve_temp_local();
        let proxy_trap_key_payload_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let own_found_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_found_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(current_local, payload_local, tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get(
            current_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            Some(found_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_own_named_property_read(
            current_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            found_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_prototype_to_current_locals(
            current_local,
            current_tag_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        // Array-like exotic elements and named properties live in
        // Array-specific storage.
        // Continue at the actual prototype rather than scanning that storage as
        // an ordinary object property table.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::Else);

        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::LocalGet(current_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                current_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                current_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                getter_payload_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                current_local,
                HEAP_OBJECT_BOXED_TAG_OFFSET,
                getter_tag_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(proxy_key_tag_local));
            function.instruction(&Instruction::I64Const(self.strings.payload("get")));
            function.instruction(&Instruction::LocalSet(proxy_internal_key_local));
            self.emit_object_own_data_field_read(
                descriptor_kind_local,
                proxy_key_tag_local,
                proxy_internal_key_local,
                proxy_trap_found_local,
                proxy_trap_payload_local,
                proxy_trap_tag_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(proxy_trap_found_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_property_key_tag_from_payload(key_local, proxy_key_tag_local, function);
            self.emit_property_key_value_payload_to_local(
                key_local,
                proxy_trap_key_payload_local,
                function,
            );
            self.emit_function_handle_call(
                proxy_trap_payload_local,
                proxy_trap_tag_local,
                Some((descriptor_kind_local, Some(proxy_key_tag_local))),
                &[
                    (getter_payload_local, getter_tag_local),
                    (proxy_trap_key_payload_local, proxy_key_tag_local),
                    (receiver_payload_local, receiver_tag_local),
                ],
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Proxy get trap is not callable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(current_local));
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(current_tag_local));
            function.instruction(&Instruction::Br(3));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("length")));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_length(getter_payload_local, payload_local, tag_local, function);
            function.instruction(&Instruction::Else);
            self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_index_get(
                getter_payload_local,
                index_local,
                receiver_payload_local,
                receiver_tag_local,
                payload_local,
                tag_local,
                Some(found_local),
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_array_named_prop_read(
                getter_payload_local,
                key_local,
                payload_local,
                tag_local,
                None,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(found_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::BrIf(4));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("length")));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_boxed_string_length_number_payload(
                getter_payload_local,
                payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Else);
            self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_string_index_read(
                getter_payload_local,
                index_local,
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
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_local));
            function.instruction(&Instruction::Br(4));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.load_i64_to_local_from_offset(current_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(current_local, HEAP_LEN_OFFSET, len_local, function);
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(own_found_local));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(extra_depth) = accessor_throw_extra_depth {
            self.emit_function_or_proxy_call_leave_throw_completion(
                getter_payload_local,
                getter_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                &[],
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_break_current_completion_if_throw(extra_depth.saturating_sub(1), function);
        } else {
            self.emit_function_or_proxy_call_leave_throw_completion(
                getter_payload_local,
                getter_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                &[],
                payload_local,
                tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.load_i64_from_offset(current_local, HEAP_OBJECT_BOXED_KIND_OFFSET, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.emit_boxed_string_length_number_payload(getter_payload_local, payload_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(own_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.emit_string_index_read(
            getter_payload_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(own_found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        self.emit_load_prototype_to_current_locals(
            current_local,
            current_tag_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Function objects expose their constructor prototype through an internal
        // slot that Object.defineProperty keeps in sync for the AOT object model.
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(current_tag_local);
        self.release_temp_local(own_found_local);
        self.release_temp_local(found_local);
        self.release_temp_local(proxy_trap_key_payload_local);
        self.release_temp_local(proxy_internal_key_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(proxy_trap_found_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(current_local);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_object_grow_buffer(
        &mut self,
        object_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

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
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_data(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )
    }

    pub(crate) fn emit_object_define_enumerable_data(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            true,
            true,
            function,
        )
    }

    pub(crate) fn emit_object_create_data_property_silent(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let can_define_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(can_define_local));
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(can_define_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(can_define_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_define_enumerable_data(
            object_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(can_define_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_data_with_configurable(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(i64::from(writable)));
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        function.instruction(&Instruction::I64Const(i64::from(enumerable)));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(i64::from(configurable)));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_object_define_data_with_flag_locals(
            object_local,
            key_local,
            payload_local,
            tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            function,
        )?;
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_data_with_flag_locals(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_object_define_data {
            if let Some(helper) = self.object_define_data_helper_function_index() {
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::LocalGet(writable_payload_local));
                function.instruction(&Instruction::LocalGet(enumerable_payload_local));
                function.instruction(&Instruction::LocalGet(configurable_payload_local));
                function.instruction(&Instruction::Call(helper));
                return Ok(());
            }
        }
        self.emit_object_define_entry(
            object_local,
            None,
            key_local,
            Some((payload_local, tag_local)),
            None,
            None,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            None,
            None,
            None,
            None,
            None,
            None,
            function,
        )
    }

    pub(crate) fn emit_object_define_accessor(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let configurable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_object_define_accessor_with_flag_local(
            object_local,
            key_local,
            getter,
            setter,
            enumerable_payload_local,
            configurable_payload_local,
            function,
        )?;
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(configurable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_enumerable_accessor(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let configurable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_object_define_accessor_with_flag_local(
            object_local,
            key_local,
            getter,
            setter,
            enumerable_payload_local,
            configurable_payload_local,
            function,
        )?;
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(configurable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_accessor_with_flag_local(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let writable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_object_define_entry(
            object_local,
            None,
            key_local,
            None,
            getter,
            setter,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            None,
            None,
            None,
            None,
            None,
            None,
            function,
        )?;
        self.release_temp_local(writable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_define_entry(
        &mut self,
        object_local: u32,
        object_tag_local: Option<u32>,
        key_local: u32,
        data: Option<(u32, u32)>,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        data_present_local: Option<u32>,
        getter_present_local: Option<u32>,
        setter_present_local: Option<u32>,
        writable_present_local: Option<u32>,
        enumerable_present_local: Option<u32>,
        configurable_present_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let stored_data_tag_local = self.reserve_temp_local();
        let stored_data_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        let has_data = data.is_some();
        let has_getter = getter.is_some();
        let has_setter = setter.is_some();
        let descriptor_kind = if has_data {
            OBJECT_DESCRIPTOR_DATA
        } else {
            OBJECT_DESCRIPTOR_ACCESSOR
        };
        function.instruction(&Instruction::I64Const(descriptor_kind as i64));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        if has_data {
            function.instruction(&Instruction::LocalGet(writable_payload_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        if let Some((data_payload_local, data_tag_local)) = data {
            function.instruction(&Instruction::LocalGet(data_tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(data_payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        }
        if let Some((getter_payload, getter_tag)) = getter {
            function.instruction(&Instruction::LocalGet(getter_tag));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload));
            function.instruction(&Instruction::LocalSet(getter_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(getter_payload_local));
        }
        if let Some((setter_payload, setter_tag)) = setter {
            function.instruction(&Instruction::LocalGet(setter_tag));
            function.instruction(&Instruction::LocalSet(setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload));
            function.instruction(&Instruction::LocalSet(setter_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(setter_payload_local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(present_local) = configurable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(configurable_payload_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
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
        }
        if let Some(present_local) = enumerable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
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
                TYPE_ERROR_NAME,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if data_present_local.is_some()
            && writable_present_local.is_some()
            && getter_present_local.is_some()
            && setter_present_local.is_some()
        {
            function.instruction(&Instruction::LocalGet(getter_present_local.unwrap()));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(setter_present_local.unwrap()));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
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

            function.instruction(&Instruction::LocalGet(data_present_local.unwrap()));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(writable_present_local.unwrap()));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
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
        }
        if let Some(present_local) = writable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(writable_payload_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
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
        }
        if let Some(present_local) = data_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                getter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                getter_payload_local,
                function,
            );
            // ValidateAndApplyPropertyDescriptor step 4.a.ii compares the
            // existing and incoming [[Value]] with SameValue, not strict
            // equality: redefining a non-writable property with NaN (or with
            // the same signed zero) must be an accepted no-op, and redefining
            // +0 as -0 must be rejected.
            self.emit_tagged_payload_same_value_i32(
                getter_tag_local,
                getter_payload_local,
                stored_data_tag_local,
                stored_data_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
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
        }
        if let Some(present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            self.emit_tagged_payload_equality_i32(
                stored_data_tag_local,
                stored_data_payload_local,
                getter_tag_local,
                getter_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
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
        }
        if let Some(present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            self.emit_tagged_payload_equality_i32(
                stored_data_tag_local,
                stored_data_payload_local,
                setter_tag_local,
                setter_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
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
        }
        function.instruction(&Instruction::End);
        if let Some(present_local) = data_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        // ValidateAndApplyPropertyDescriptor only carries [[Writable]] over
        // from the existing property when both the existing and the incoming
        // descriptors are data descriptors (step 7.b).  Converting an accessor
        // property into a data property keeps only [[Configurable]] and
        // [[Enumerable]] and resets everything else to its default (step
        // 7.c.i), so writable must come out false; and an accessor entry must
        // never pick up a writable bit of its own, because that stale bit is
        // what a later accessor-to-data conversion would read back.
        if has_data {
            if let Some(present_local) = writable_present_local {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(descriptor_kind_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
        if let Some(present_local) = enumerable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if let Some(present_local) = configurable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(
                OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
            ));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(
                OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
            ));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        if data_present_local.is_some()
            && writable_present_local.is_some()
            && getter_present_local.is_some()
            && setter_present_local.is_some()
        {
            function.instruction(&Instruction::LocalGet(data_present_local.unwrap()));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(writable_present_local.unwrap()));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(getter_present_local.unwrap()));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(setter_present_local.unwrap()));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            stored_data_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            stored_data_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                getter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                getter_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        } else if !has_getter {
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                getter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                getter_payload_local,
                function,
            );
        }
        if let Some(present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                setter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                setter_payload_local,
                function,
            );
            function.instruction(&Instruction::End);
        } else if !has_setter {
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                setter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                setter_payload_local,
                function,
            );
        }
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if let Some(object_tag_local) = object_tag_local {
            self.emit_ordinary_is_extensible_i32(
                object_local,
                object_tag_local,
                existing_descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        } else {
            function.instruction(&Instruction::LocalGet(cap_local));
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPE_ERROR_NAME),
        ));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(TYPE_ERROR_NAME) as i64,
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        if has_data {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
        } else {
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                0,
                function,
            );
            if has_getter {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_TAG_OFFSET,
                    getter_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    getter_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_TAG_OFFSET,
                    0,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    0,
                    function,
                );
            }
            if has_setter {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_TAG_OFFSET,
                    setter_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    setter_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_TAG_OFFSET,
                    0,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    0,
                    function,
                );
            }
        }
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_data_payload_local);
        self.release_temp_local(stored_data_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_create_data_property_or_throw(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        non_configurable_message: &'static str,
        non_extensible_message: &'static str,
        iterator_close_on_throw: Option<IteratorCloseOnThrowLocals>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let object_kind_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let bool_payload_local = self.reserve_temp_local();
        let bool_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let define_property_payload_local = self.reserve_temp_local();
        let define_property_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(bool_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(bool_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            self.scratch_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            self.scratch_local,
            bool_payload_local,
            bool_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            self.scratch_local,
            bool_payload_local,
            bool_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            self.scratch_local,
            bool_payload_local,
            bool_tag_local,
            function,
        )?;

        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                )
            })?;
        self.emit_function_value_payload(&define_property_meta, function)?;
        function.instruction(&Instruction::LocalSet(define_property_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_function_handle_call(
            define_property_payload_local,
            define_property_tag_local,
            None,
            &[
                (object_local, object_tag_local),
                (key_local, key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
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
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            non_configurable_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(close) = iterator_close_on_throw {
            self.emit_iterator_close_preserving_current_throw(close, function)?;
        }
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_CONFIGURABLE
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            non_extensible_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(close) = iterator_close_on_throw {
            self.emit_iterator_close_preserving_current_throw(close, function)?;
        }
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_CONFIGURABLE
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(define_property_tag_local);
        self.release_temp_local(define_property_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(bool_tag_local);
        self.release_temp_local(bool_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(object_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Emits a `call` into the shared object-write runtime helper. Mirrors the
    /// inline `emit_object_write` contract: on a setter/proxy throw the thrown
    /// value lands in the current result locals and the completion becomes
    /// `Throw`; on success the pre-call result locals are preserved.
    fn emit_object_write_via_helper(
        &mut self,
        helper: u32,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_result_local = self.reserve_temp_local();
        let saved_result_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(saved_result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(saved_result_tag_local));

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        // Parameter 5: the calling function's strictness. The shared write helper
        // is emitted once with a fixed (mode-less) body, so sloppy vs. strict
        // `[[Set]]` failure behavior must be selected at runtime from this flag.
        match self.object_write_strict_flag_local {
            Some(strict_override) => {
                function.instruction(&Instruction::LocalGet(strict_override));
            }
            None => {
                function.instruction(&Instruction::I64Const(i64::from(
                    self.is_current_function_strict(),
                )));
            }
        }
        // Only created-realm standard builtins use a self-backed environment
        // that the shared helper may interpret as realm metadata. User
        // functions can have nonzero lexical environments with a different
        // layout, so pass zero for every non-standard caller.
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
        function.instruction(&Instruction::Call(helper));
        self.store_call_results_to(
            self.result_local,
            self.result_tag_local,
            self.completion_local,
            self.completion_aux_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(saved_result_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(saved_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(saved_result_tag_local);
        self.release_temp_local(saved_result_local);
        Ok(())
    }

    /// Emits the `Else` branch of an ordinary `[[Set]]` on an existing
    /// non-writable data property / accessor-without-setter. Spec: the write is
    /// a silent no-op in sloppy mode and a `TypeError` only in strict mode.
    ///
    /// When emitted inline the enclosing function's compile-time strictness is
    /// authoritative. When emitted as the shared outlined write helper (a
    /// fixed, mode-less body) the decision is deferred to the runtime strict
    /// flag threaded through helper parameter 5.
    pub(crate) fn emit_object_write_set_failure_else(
        &mut self,
        message: &str,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.object_write_strict_flag_local {
            Some(strict_local) => {
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(strict_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error_to_active_handler(
                    TYPE_ERROR_NAME,
                    message,
                    self.result_local,
                    self.result_tag_local,
                    extra_depth,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            None => {
                if self.is_current_function_strict() {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error_to_active_handler(
                        TYPE_ERROR_NAME,
                        message,
                        self.result_local,
                        self.result_tag_local,
                        extra_depth,
                        function,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Emits the sloppy/strict-guarded throw for a Proxy `set` trap that
    /// returned a falsy value. Spec: `OrdinarySetWithOwnDescriptor`/`PutValue`
    /// throws only in strict mode; sloppy code silently ignores it. Runtime
    /// gating mirrors [`Self::emit_object_write_set_failure_else`].
    fn emit_object_write_proxy_set_false_throw(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.object_write_strict_flag_local {
            Some(strict_local) => {
                function.instruction(&Instruction::LocalGet(strict_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Proxy set trap returned false",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
            None => {
                if self.is_current_function_strict() {
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "Proxy set trap returned false",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
            }
        }
        Ok(())
    }

    /// Emits the sloppy/strict-guarded outcome for adding a new property to a
    /// non-extensible object. Spec: strict mode throws a `TypeError`; sloppy
    /// mode silently abandons the write. `sloppy_br_depth` is the branch depth
    /// used to abandon the write in the inline (compile-time) case; when emitted
    /// as the outlined helper the runtime guard nests one extra block, so the
    /// sloppy branch targets `sloppy_br_depth + 1`.
    fn emit_object_write_non_extensible_failure(
        &mut self,
        sloppy_br_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.object_write_strict_flag_local {
            Some(strict_local) => {
                function.instruction(&Instruction::LocalGet(strict_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error_to_active_handler(
                    TYPE_ERROR_NAME,
                    "Cannot add property to non-extensible object",
                    self.result_local,
                    self.result_tag_local,
                    5,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::Br(sloppy_br_depth + 1));
                function.instruction(&Instruction::End);
            }
            None => {
                if self.is_current_function_strict() {
                    self.emit_throw_runtime_error_to_active_handler(
                        TYPE_ERROR_NAME,
                        "Cannot add property to non-extensible object",
                        self.result_local,
                        self.result_tag_local,
                        5,
                        function,
                    )?;
                } else {
                    function.instruction(&Instruction::Br(sloppy_br_depth));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn emit_object_write_strict(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let strict_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(strict_local));
        let previous_strict_local = self.object_write_strict_flag_local;
        self.object_write_strict_flag_local = Some(strict_local);
        let result = self.emit_object_write(
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        );
        self.object_write_strict_flag_local = previous_strict_local;
        self.release_temp_local(strict_local);
        result
    }

    pub(crate) fn emit_object_write(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_object_write {
            if let Some(helper) = self.object_write_helper_function_index() {
                return self.emit_object_write_via_helper(
                    helper,
                    object_local,
                    object_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let inherited_descriptor_kind_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype_buffer_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();
        let prototype_index_local = self.reserve_temp_local();
        let prototype_proxy_kind_local = self.reserve_temp_local();
        let prototype_proxy_set_handled_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let array_index_write_handled_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_trap_truthy_local = self.reserve_temp_local();
        let proxy_internal_key_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let proxy_trap_key_payload_local = self.reserve_temp_local();
        let proxy_descriptor_payload_local = self.reserve_temp_local();
        let proxy_descriptor_tag_local = self.reserve_temp_local();
        let proxy_bool_payload_local = self.reserve_temp_local();
        let proxy_bool_tag_local = self.reserve_temp_local();
        let proxy_reflect_set_payload_local = self.reserve_temp_local();
        let proxy_reflect_set_tag_local = self.reserve_temp_local();
        let array_length_key_local = self.reserve_temp_local();
        let array_length_success_local = self.reserve_temp_local();
        let array_length_writable_present_local = self.reserve_temp_local();
        let array_length_allow_define_local = self.reserve_temp_local();
        let array_length_initial_writable_local = self.reserve_temp_local();
        let typed_array_numeric_index_local = self.reserve_temp_local();
        let typed_array_index_local = self.reserve_temp_local();
        let typed_array_index_valid_local = self.reserve_temp_local();
        let typed_array_index_handled_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;
        // The proxy write-forwarding paths below (guarded by `proxy_kind >=
        // PROXY_HANDLER_PAYLOAD_MIN`) dispatch through `Reflect.set` and can only
        // run when the object is a Proxy exotic object. A Proxy value requires the
        // `Proxy` constructor to be planned; when it is not, those branches are
        // dead, so materialize `Reflect.set` without recording it (which would
        // otherwise force the whole `Reflect` object through the fixpoint).
        let proxy_reachable = self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
        self.emit_property_key_tag_from_payload(key_local, proxy_key_tag_local, function);
        self.emit_typed_array_set_same_receiver_if_handled(
            object_local,
            object_tag_local,
            key_local,
            proxy_key_tag_local,
            payload_local,
            tag_local,
            array_length_success_local,
            array_index_write_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_callee_write(object_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_index_write_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
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
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(proxy_internal_key_local));
        self.emit_object_read_ordinary(
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_internal_key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_property_key_tag_from_payload(key_local, proxy_key_tag_local, function);
        self.emit_property_key_value_payload_to_local(
            key_local,
            proxy_trap_key_payload_local,
            function,
        );
        self.emit_function_handle_call(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            Some((proxy_handler_payload_local, Some(proxy_handler_tag_local))),
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (proxy_trap_key_payload_local, proxy_key_tag_local),
                (payload_local, tag_local),
                (object_local, object_tag_local),
            ],
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
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
        self.emit_object_write_proxy_set_false_throw(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_trap_truthy_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_set_invariant_check(
            proxy_target_payload_local,
            proxy_target_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_result_without_receiver_fallback_via_helper(
            proxy_target_payload_local,
            proxy_target_tag_local,
            object_local,
            object_tag_local,
            key_local,
            proxy_key_tag_local,
            payload_local,
            tag_local,
            proxy_trap_result_payload_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_trap_result_tag_local));
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
        self.emit_object_write_proxy_set_false_throw(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy set trap is not callable",
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
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_assignment_write(
            object_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_index_write_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(array_length_key_local));
        self.emit_string_payload_equality_i32(key_local, array_length_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_length_writable_present_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_length_allow_define_local));
        self.emit_array_length_writable_i64(
            object_local,
            array_length_initial_writable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_length_initial_writable_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_length_from_value(
            object_local,
            payload_local,
            tag_local,
            array_length_writable_present_local,
            array_length_writable_present_local,
            array_length_allow_define_local,
            array_length_success_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_length_success_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to array length", 1, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_index_write_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_result_without_receiver_fallback_via_helper(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            proxy_key_tag_local,
            payload_local,
            tag_local,
            array_length_success_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to array property", 1, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_index_write_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_standard_builtin_constructor_payload(object_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.emit_object_write_set_failure_else(
            "Cannot assign to read only property",
            9,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            setter_payload_local,
            setter_tag_local,
            object_local,
            object_tag_local,
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(setter_result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(setter_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.emit_break_current_completion_if_throw(0, function);
        self.emit_object_write_set_failure_else(
            "Cannot assign to read only property",
            9,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(inherited_descriptor_kind_local));
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_get_prototype_of(
            object_local,
            object_tag_local,
            prototype_local,
            prototype_tag_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_canonical_numeric_index_i32(
            prototype_local,
            prototype_tag_local,
            key_local,
            proxy_key_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            prototype_local,
            prototype_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_local,
            typed_array_index_valid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_data_on_receiver_result(
            object_local,
            object_tag_local,
            key_local,
            proxy_key_tag_local,
            payload_local,
            tag_local,
            array_length_success_local,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        match self.object_write_strict_flag_local {
            Some(strict_local) => {
                function.instruction(&Instruction::LocalGet(strict_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot assign inherited typed array index on receiver",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
            None => {
                if self.is_current_function_strict() {
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "Cannot assign inherited typed array index on receiver",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_proxy_kind_local));
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            prototype_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prototype_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_reachable {
            self.emit_function_value_payload(&reflect_set_meta, function)?;
        } else {
            self.emit_function_value_payload_unrecorded(&reflect_set_meta, function)?;
        }
        function.instruction(&Instruction::LocalSet(proxy_reflect_set_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_reflect_set_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        self.emit_property_key_tag_from_payload(key_local, proxy_key_tag_local, function);
        self.emit_function_handle_call(
            proxy_reflect_set_payload_local,
            proxy_reflect_set_tag_local,
            None,
            &[
                (prototype_local, proxy_handler_tag_local),
                (key_local, proxy_key_tag_local),
                (payload_local, tag_local),
                (object_local, object_tag_local),
            ],
            proxy_trap_result_payload_local,
            proxy_trap_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
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
        self.emit_object_write_proxy_set_false_throw(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_PTR_OFFSET,
            prototype_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(prototype_buffer_local));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
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
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(inherited_descriptor_kind_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(inherited_descriptor_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(inherited_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_get_prototype_of(
            prototype_local,
            prototype_tag_local,
            prototype_local,
            prototype_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(inherited_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else(
            "Cannot assign to inherited read only property",
            3,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inherited_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            setter_payload_local,
            setter_tag_local,
            object_local,
            object_tag_local,
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(setter_result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(setter_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.emit_break_current_completion_if_throw(0, function);
        self.emit_object_write_set_failure_else(
            "Cannot assign to inherited accessor without setter",
            2,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_ordinary_is_extensible_i32(
            object_local,
            object_tag_local,
            array_length_success_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_non_extensible_failure(1, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_CONFIGURABLE
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_handled_local);
        self.release_temp_local(typed_array_index_valid_local);
        self.release_temp_local(typed_array_index_local);
        self.release_temp_local(typed_array_numeric_index_local);
        self.release_temp_local(array_length_initial_writable_local);
        self.release_temp_local(array_length_allow_define_local);
        self.release_temp_local(array_length_writable_present_local);
        self.release_temp_local(array_length_success_local);
        self.release_temp_local(array_length_key_local);
        self.release_temp_local(proxy_reflect_set_tag_local);
        self.release_temp_local(proxy_reflect_set_payload_local);
        self.release_temp_local(proxy_bool_tag_local);
        self.release_temp_local(proxy_bool_payload_local);
        self.release_temp_local(proxy_descriptor_tag_local);
        self.release_temp_local(proxy_descriptor_payload_local);
        self.release_temp_local(proxy_trap_key_payload_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(proxy_internal_key_local);
        self.release_temp_local(proxy_trap_truthy_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(array_index_write_handled_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(prototype_proxy_set_handled_local);
        self.release_temp_local(prototype_proxy_kind_local);
        self.release_temp_local(prototype_index_local);
        self.release_temp_local(prototype_len_local);
        self.release_temp_local(prototype_buffer_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(inherited_descriptor_kind_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_is_object_entry_backed_tag_i32(
        &self,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [ValueKind::Function, ValueKind::Arguments] {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_ordinary_set_data_on_receiver_result(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        allow_generic_write_fallback: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ordinary_set_data_on_receiver_result_with_depth(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            4,
            allow_generic_write_fallback,
            function,
        )
    }

    pub(crate) fn emit_ordinary_set_data_on_receiver_result_with_depth(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        proxy_depth: u8,
        allow_generic_write_fallback: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.ordinary_set_data_on_receiver_emission
            == OrdinarySetDataOnReceiverEmission::Outlined
            && proxy_depth == 4
        {
            let helper = if allow_generic_write_fallback {
                self.ordinary_set_data_on_receiver_with_fallback_helper_function_index()
                    .expect("ordinary receiver-set fallback helper index must exist")
            } else {
                self.ordinary_set_data_on_receiver_helper_function_index()
                    .expect("ordinary receiver-set helper index must exist")
            };
            return self.emit_ordinary_set_data_on_receiver_via_helper(
                helper,
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                result_local,
                function,
            );
        }
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let proxy_kind_local = self.reserve_temp_local();
        let get_own_payload_local = self.reserve_temp_local();
        let get_own_tag_local = self.reserve_temp_local();
        let get_own_result_payload_local = self.reserve_temp_local();
        let get_own_result_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let descriptor_value_payload_local = self.reserve_temp_local();
        let descriptor_value_tag_local = self.reserve_temp_local();
        let reflect_define_payload_local = self.reserve_temp_local();
        let reflect_define_tag_local = self.reserve_temp_local();
        let reflect_define_result_payload_local = self.reserve_temp_local();
        let reflect_define_result_tag_local = self.reserve_temp_local();
        let array_length_writable_present_local = self.reserve_temp_local();
        let array_length_allow_define_local = self.reserve_temp_local();
        let array_length_initial_writable_local = self.reserve_temp_local();
        let typed_array_numeric_index_local = self.reserve_temp_local();
        let typed_array_index_local = self.reserve_temp_local();
        let typed_array_index_valid_local = self.reserve_temp_local();
        let typed_array_index_handled_local = self.reserve_temp_local();

        let object_get_own_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                )
            })?;
        let reflect_define_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                )
            })?;
        // The `proxy_depth > 0` block below forwards a define through
        // `Reflect.defineProperty` only when `receiver` is a Proxy exotic object,
        // which cannot exist unless the `Proxy` constructor is planned. When it is
        // not, that materialization is in a dead branch — emit it without recording
        // so it does not force the whole `Reflect` object through the fixpoint.
        let proxy_reachable = self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

        self.emit_typed_array_canonical_numeric_index_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            receiver_payload_local,
            receiver_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_local,
            typed_array_index_valid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            typed_array_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        if proxy_depth > 0 {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                proxy_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(proxy_kind_local));
            function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(proxy_kind_local));
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

            self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
            self.emit_function_value_payload(&object_get_own_meta, function)?;
            function.instruction(&Instruction::LocalSet(get_own_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(get_own_tag_local));
            self.emit_function_handle_call(
                get_own_payload_local,
                get_own_tag_local,
                None,
                &[
                    (receiver_payload_local, receiver_tag_local),
                    (key_local, key_tag_local),
                ],
                get_own_result_payload_local,
                get_own_result_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);

            function.instruction(&Instruction::LocalGet(value_payload_local));
            function.instruction(&Instruction::LocalSet(descriptor_value_payload_local));
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::LocalSet(descriptor_value_tag_local));
            function.instruction(&Instruction::LocalGet(get_own_result_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_data_property_descriptor_object_from_locals(
                descriptor_value_payload_local,
                descriptor_value_tag_local,
                true,
                true,
                true,
                descriptor_payload_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_alloc_value_descriptor_from_locals(
                descriptor_value_payload_local,
                descriptor_value_tag_local,
                descriptor_payload_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(descriptor_tag_local));

            if proxy_reachable {
                self.emit_function_value_payload(&reflect_define_meta, function)?;
            } else {
                self.emit_function_value_payload_unrecorded(&reflect_define_meta, function)?;
            }
            function.instruction(&Instruction::LocalSet(reflect_define_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(reflect_define_tag_local));
            self.emit_function_handle_call(
                reflect_define_payload_local,
                reflect_define_tag_local,
                None,
                &[
                    (receiver_payload_local, receiver_tag_local),
                    (key_local, key_tag_local),
                    (descriptor_payload_local, descriptor_tag_local),
                ],
                reflect_define_result_payload_local,
                reflect_define_result_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.compile_truthy_tagged_i32(
                reflect_define_result_tag_local,
                reflect_define_result_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(result_local));
            function.instruction(&Instruction::Br(2));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        // An Array's own "length" is a data property even though it is not
        // stored in the ordinary named-property table. Apply the receiver-side
        // DefineProperty semantics before the generic Array write fallback.
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_length_writable_present_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_length_allow_define_local));
        self.emit_array_length_writable_i64(
            receiver_payload_local,
            array_length_initial_writable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_length_initial_writable_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_length_from_value(
            receiver_payload_local,
            value_payload_local,
            value_tag_local,
            array_length_writable_present_local,
            array_length_writable_present_local,
            array_length_allow_define_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // Array-index receiver writes use Array [[DefineOwnProperty]], not the
        // named-property side table. Preserve an existing descriptor and
        // report blocked additions as `false`.
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_local,
            index_local,
            found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_index_result(
            receiver_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_object_entry_backed_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_CAP_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_array_define_named_data_descriptor(
            receiver_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            self.scratch_local,
            self.scratch_local,
            self.scratch_local,
            None,
            None,
            None,
            None,
            Some(result_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(
            receiver_payload_local,
            buffer_local,
            len_local,
            cap_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            OBJECT_DESCRIPTOR_DATA
                | OBJECT_DESCRIPTOR_CONFIGURABLE
                | OBJECT_DESCRIPTOR_WRITABLE
                | OBJECT_DESCRIPTOR_ENUMERABLE,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        if allow_generic_write_fallback {
            self.emit_object_write(
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(result_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_handled_local);
        self.release_temp_local(typed_array_index_valid_local);
        self.release_temp_local(typed_array_index_local);
        self.release_temp_local(typed_array_numeric_index_local);
        self.release_temp_local(array_length_initial_writable_local);
        self.release_temp_local(array_length_allow_define_local);
        self.release_temp_local(array_length_writable_present_local);
        self.release_temp_local(reflect_define_result_tag_local);
        self.release_temp_local(reflect_define_result_payload_local);
        self.release_temp_local(reflect_define_tag_local);
        self.release_temp_local(reflect_define_payload_local);
        self.release_temp_local(descriptor_value_tag_local);
        self.release_temp_local(descriptor_value_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(get_own_result_tag_local);
        self.release_temp_local(get_own_result_payload_local);
        self.release_temp_local(get_own_tag_local);
        self.release_temp_local(get_own_payload_local);
        self.release_temp_local(proxy_kind_local);
        self.release_temp_local(found_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ordinary_set_data_on_receiver_via_helper(
        &mut self,
        helper: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper_result_local = self.reserve_temp_local();
        let helper_result_tag_local = self.reserve_temp_local();
        for local in [
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            self.current_env_local,
        ] {
            function.instruction(&Instruction::LocalGet(local));
        }
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(helper_result_local, helper_result_tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed(
            helper_result_local,
            helper_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(helper_result_local));
        function.instruction(&Instruction::LocalSet(result_local));
        self.release_temp_local(helper_result_tag_local);
        self.release_temp_local(helper_result_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_set_result(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ordinary_set_result_with_receiver_fallback(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            true,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_ordinary_set_result_via_helper(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ordinary_set_result_via_selected_helper(
            self.ordinary_set_helper_function_index()
                .expect("ordinary-set helper index must exist"),
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_ordinary_set_result_without_receiver_fallback_via_helper(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ordinary_set_result_via_selected_helper(
            self.ordinary_set_without_receiver_fallback_helper_function_index()
                .expect("ordinary-set no-fallback helper index must exist"),
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ordinary_set_result_via_selected_helper(
        &mut self,
        helper: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let realm_environment_tag_local = self.reserve_temp_local();
        let helper_result_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(realm_environment_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (target_payload_local, target_tag_local),
                (receiver_payload_local, receiver_tag_local),
                (key_local, key_tag_local),
                (value_payload_local, value_tag_local),
                (self.current_env_local, realm_environment_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(result_local, helper_result_tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed(
            result_local,
            helper_result_tag_local,
            function,
        )?;
        self.release_temp_local(helper_result_tag_local);
        self.release_temp_local(realm_environment_tag_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        allow_receiver_generic_write_fallback: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let current_proxy_kind_local = self.reserve_temp_local();
        let reflect_set_payload_local = self.reserve_temp_local();
        let reflect_set_tag_local = self.reserve_temp_local();
        let reflect_set_result_tag_local = self.reserve_temp_local();
        let boxed_payload_local = self.reserve_temp_local();
        let boxed_tag_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_unit_len_local = self.reserve_temp_local();
        let array_length_success_local = self.reserve_temp_local();
        let array_length_writable_present_local = self.reserve_temp_local();
        let array_length_allow_define_local = self.reserve_temp_local();
        let array_length_initial_writable_local = self.reserve_temp_local();
        let array_length_key_local = self.reserve_temp_local();
        let array_index_found_local = self.reserve_temp_local();
        let typed_array_numeric_index_local = self.reserve_temp_local();
        let typed_array_index_local = self.reserve_temp_local();
        let typed_array_index_valid_local = self.reserve_temp_local();
        let typed_array_index_handled_local = self.reserve_temp_local();

        // The proxy write-forwarding branch below dispatches through `Reflect.set`
        // and is only reachable when `current` is a Proxy exotic object. A Proxy
        // value can only exist in a module that planned the `Proxy` constructor, so
        // when it is absent the branch is dead: skip both its emission and the
        // `Reflect.set` materialization that would otherwise force the whole
        // `Reflect` object (and its 13 methods) through the emission fixpoint.
        let proxy_reachable = self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor);
        let reflect_set_meta = if proxy_reachable {
            Some(
                self.functions
                    .get(&StandardBuiltinId::ReflectSet.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                        )
                    })?,
            )
        } else {
            None
        };

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        // Array's `length` is an own data property regardless of the receiver.
        // Its writable attribute belongs to the source descriptor; a distinct
        // receiver is handled by OrdinarySet's receiver-side data-property path.
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(array_length_key_local));
        self.emit_string_payload_equality_i32(key_local, array_length_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_length_writable_present_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_length_allow_define_local));
        self.emit_array_length_writable_i64(
            target_payload_local,
            array_length_initial_writable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_length_initial_writable_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_length_from_value(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            array_length_writable_present_local,
            array_length_writable_present_local,
            array_length_allow_define_local,
            array_length_success_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_ordinary_set_data_on_receiver_result(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            array_length_success_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_canonical_numeric_index_i32(
            current_payload_local,
            current_tag_local,
            key_local,
            key_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_set_same_receiver_if_handled(
            current_payload_local,
            current_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            typed_array_index_handled_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_valid_integer_index_i32(
            current_payload_local,
            current_tag_local,
            typed_array_numeric_index_local,
            typed_array_index_local,
            typed_array_index_valid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_data_on_receiver_result(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        if let Some(reflect_set_meta) = &reflect_set_meta {
            function.instruction(&Instruction::LocalGet(current_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                current_payload_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                current_proxy_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(current_proxy_kind_local));
            function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_function_value_payload(reflect_set_meta, function)?;
            function.instruction(&Instruction::LocalSet(reflect_set_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(reflect_set_tag_local));
            self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
            self.emit_function_handle_call(
                reflect_set_payload_local,
                reflect_set_tag_local,
                None,
                &[
                    (current_payload_local, current_tag_local),
                    (key_local, key_tag_local),
                    (value_payload_local, value_tag_local),
                    (receiver_payload_local, receiver_tag_local),
                ],
                result_local,
                reflect_set_result_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.compile_truthy_tagged_i32(reflect_set_result_tag_local, result_local, function)?;
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(result_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            current_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_proxy_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            boxed_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(
            boxed_payload_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_unit_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_unit_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Dense and sparse Array indices live outside the named-property
        // table.  Find their descriptor before walking the prototype chain so
        // OrdinarySet can honor writability and define on the receiver.
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_local,
            index_local,
            array_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_kind_for_index(
            current_payload_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_accessor_setter_for_index(
            current_payload_local,
            index_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[(value_payload_local, value_tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_data_on_receiver_result(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_object_entry_backed_tag_i32(current_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            setter_payload_local,
            setter_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[(value_payload_local, value_tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_data_on_receiver_result(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_prototype_to_current_locals(
            current_payload_local,
            current_tag_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_data_on_receiver_result(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            result_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_handled_local);
        self.release_temp_local(typed_array_index_valid_local);
        self.release_temp_local(typed_array_index_local);
        self.release_temp_local(typed_array_numeric_index_local);
        self.release_temp_local(array_index_found_local);
        self.release_temp_local(array_length_key_local);
        self.release_temp_local(array_length_initial_writable_local);
        self.release_temp_local(array_length_allow_define_local);
        self.release_temp_local(array_length_writable_present_local);
        self.release_temp_local(array_length_success_local);
        self.release_temp_local(string_unit_len_local);
        self.release_temp_local(string_byte_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(boxed_tag_local);
        self.release_temp_local(boxed_payload_local);
        self.release_temp_local(reflect_set_result_tag_local);
        self.release_temp_local(reflect_set_tag_local);
        self.release_temp_local(reflect_set_payload_local);
        self.release_temp_local(current_proxy_kind_local);
        self.release_temp_local(found_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_is_standard_builtin_constructor_payload(
        &self,
        object_local: u32,
        function: &mut Function,
    ) {
        const BUILTIN_CONSTRUCTOR_GLOBALS: &[u32] = &[
            FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
            OBJECT_CONSTRUCTOR_GLOBAL_INDEX,
            ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            NUMBER_CONSTRUCTOR_GLOBAL_INDEX,
            STRING_CONSTRUCTOR_GLOBAL_INDEX,
            BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX,
            ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
            ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX,
            SHARED_ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX,
            DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX,
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            BIGINT_CONSTRUCTOR_GLOBAL_INDEX,
            PROXY_CONSTRUCTOR_GLOBAL_INDEX,
            DATE_CONSTRUCTOR_GLOBAL_INDEX,
        ];

        function.instruction(&Instruction::I32Const(0));
        for global_index in BUILTIN_CONSTRUCTOR_GLOBALS {
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::GlobalGet(*global_index));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_object_delete(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_delete_with_depth(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            4,
            function,
        )
    }

    pub(crate) fn emit_object_delete_with_depth(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        proxy_depth: u8,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let internal_key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let trap_key_payload_local = self.reserve_temp_local();

        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        self.emit_property_key_value_payload_to_local(key_local, trap_key_payload_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
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
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("deleteProperty"),
        ));
        function.instruction(&Instruction::LocalSet(internal_key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            internal_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (trap_key_payload_local, key_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_proxy_delete_invariant_check(
            target_payload_local,
            target_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_depth == 0 {
            self.emit_delete_ordinary_by_tag(
                target_payload_local,
                target_tag_local,
                key_local,
                result_local,
                function,
            )?;
        } else {
            self.emit_object_delete_with_depth(
                target_payload_local,
                target_tag_local,
                key_local,
                result_local,
                proxy_depth - 1,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy deleteProperty trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_delete_ordinary_by_tag(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(trap_key_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(internal_key_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_delete_ordinary_by_tag(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_tag_local = self.reserve_temp_local();
        let typed_array_index_handled_local = self.reserve_temp_local();

        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        self.emit_typed_array_integer_index_validity_i32(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            typed_array_index_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_delete_property_key(object_local, key_local, result_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_delete_property_key(object_local, key_local, result_local, function);
        function.instruction(&Instruction::Else);
        self.emit_object_delete_ordinary(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_handled_local);
        self.release_temp_local(key_tag_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_delete_invariant_check(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let found_local = self.reserve_temp_local();
        let non_configurable_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(non_configurable_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(non_configurable_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(non_configurable_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(non_configurable_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(non_configurable_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy deleteProperty trap returned true for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_CAP_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy deleteProperty trap returned true for non-extensible target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cap_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(non_configurable_local);
        self.release_temp_local(found_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_set_invariant_check(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let present_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let target_value_payload_local = self.reserve_temp_local();
        let target_value_tag_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_value_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(setter_tag_local));

        self.emit_is_object_entry_backed_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
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
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            target_value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            target_value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_tagged_payload_same_value_i32(
            value_tag_local,
            value_payload_local,
            target_value_tag_local,
            target_value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy set trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy set trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(target_value_tag_local);
        self.release_temp_local(target_value_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(present_local);
        Ok(())
    }

    pub(crate) fn emit_object_delete_ordinary(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let shift_index_local = self.reserve_temp_local();
        let current_entry_local = self.reserve_temp_local();
        let next_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_unpack_string_payload(key_payload_local, buffer_local, len_local, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            len_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.emit_property_key_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_entry_local));
        function.instruction(&Instruction::LocalGet(current_entry_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(next_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                current_entry_local,
                offset,
                self.scratch_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_entry_local);
        self.release_temp_local(current_entry_local);
        self.release_temp_local(shift_index_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    /// Routes a pending throw completion (left in `result_local`/`result_tag_local`
    /// by a shared-helper call) to the active handler, or returns it, mirroring
    /// [`Self::emit_throw_runtime_error_to_active_handler`] but for a completion
    /// that already exists. Uses `BrIf` so no extra wasm frame is introduced and
    /// `extra_throw_depth` keeps the same meaning callers already pass (the count
    /// of untracked wasm frames wrapping the call site).
    fn emit_route_pending_throw_to_active_handler(
        &mut self,
        extra_throw_depth: u32,
        function: &mut Function,
    ) {
        if self.is_main() {
            if let Some(target) = self.active_throw_target() {
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Eq);
                self.emit_branch_if_to_target(target, extra_throw_depth, function);
                return;
            }
        }
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
    }

    /// Emits a `call` to the shared proxy-aware `[[GetPrototypeOf]]` helper,
    /// storing the prototype into the result locals and routing a proxy-trap
    /// throw to the active handler. Replaces the inline proxy state machine at
    /// every call site.
    fn emit_call_object_get_prototype_of_helper(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self
            .object_get_prototype_of_helper_function_index()
            .expect("get-prototype-of helper index must exist");
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(self.result_local, self.result_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        self.emit_route_pending_throw_to_active_handler(extra_throw_depth, function);
        Ok(())
    }

    /// Emits a `call` to the shared proxy-aware `[[IsExtensible]]` helper,
    /// storing the boolean result into `result_local` and routing a proxy-trap
    /// throw to the active handler.
    fn emit_call_object_is_extensible_helper(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper_payload_local = self.reserve_temp_local();
        let helper_tag_local = self.reserve_temp_local();
        let helper = self
            .object_is_extensible_helper_function_index()
            .expect("is-extensible helper index must exist");
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(helper_payload_local, helper_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(helper_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(helper_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(helper_payload_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        self.emit_route_pending_throw_to_active_handler(extra_throw_depth, function);
        self.release_temp_local(helper_tag_local);
        self.release_temp_local(helper_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_get_prototype_of(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_get_prototype_of_with_depth(
            object_payload_local,
            object_tag_local,
            result_payload_local,
            result_tag_local,
            0,
            function,
        )
    }

    pub(crate) fn emit_object_get_prototype_of_without_proxy(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_ordinary_get_prototype_of(
                object_payload_local,
                object_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            );
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        for (kind, prototype_global_index) in [
            (ValueKind::Number, NUMBER_PROTOTYPE_GLOBAL_INDEX),
            (ValueKind::String, STRING_PROTOTYPE_GLOBAL_INDEX),
            (ValueKind::Boolean, BOOLEAN_PROTOTYPE_GLOBAL_INDEX),
            (ValueKind::Symbol, SYMBOL_PROTOTYPE_GLOBAL_INDEX),
        ] {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(result_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(result_tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.load_i64_to_local_from_offset(
            result_payload_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            result_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert undefined or null to object",
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_object_get_prototype_of_with_depth(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_object_get_prototype_of {
            if self
                .object_get_prototype_of_helper_function_index()
                .is_some()
            {
                return self.emit_call_object_get_prototype_of_helper(
                    object_payload_local,
                    object_tag_local,
                    result_payload_local,
                    result_tag_local,
                    extra_throw_depth,
                    function,
                );
            }
        }
        let handled_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let target_extensible_local = self.reserve_temp_local();
        let target_proto_payload_local = self.reserve_temp_local();
        let target_proto_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 3,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("getPrototypeOf"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        // This call site sits 3 untracked `If`s deep (object-tag check, proxy-handler
        // check, trap-is-callable check above), matching the nesting the sibling
        // `emit_throw_runtime_error_to_active_handler(..., extra_throw_depth + 3, ...)`
        // calls in this function account for. `emit_function_handle_call` bakes in a
        // fixed extra depth of 1 (matched to being called from untracked-nesting depth
        // 0), which under-counts here and misroutes an abrupt completion from the trap
        // itself (e.g. the trap throwing directly) to the wrong wasm block instead of
        // the active catch/return path — silently swallowing the throw when this is
        // reached from inside a non-top-level function. `throw_extra_depth` must be
        // `extra_throw_depth + 2` (not `+ 3`) because the underlying propagate helper
        // already adds its own `+ 1` for its internal `if` wrapper.
        self.emit_function_or_proxy_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
            extra_throw_depth + 2,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(trap_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(trap_result_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy getPrototypeOf trap result must be object or null",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_call_object_is_extensible_helper(
            target_payload_local,
            target_tag_local,
            target_extensible_local,
            extra_throw_depth + 3,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_call_object_get_prototype_of_helper(
            target_payload_local,
            target_tag_local,
            target_proto_payload_local,
            target_proto_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        self.emit_tagged_payload_same_value_i32(
            trap_result_tag_local,
            trap_result_payload_local,
            target_proto_tag_local,
            target_proto_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy getPrototypeOf trap result does not match target",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Nested helper calls use the builder's result locals as their ABI
        // scratch. In the outlined [[GetPrototypeOf]] helper those locals are
        // also this operation's final outputs, so publish the trap result only
        // after the extensibility/invariant checks have finished clobbering the
        // ABI scratch slots.
        function.instruction(&Instruction::LocalGet(trap_result_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::LocalGet(trap_result_tag_local));
        function.instruction(&Instruction::LocalSet(result_tag_local));

        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_call_object_get_prototype_of_helper(
            target_payload_local,
            target_tag_local,
            result_payload_local,
            result_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy getPrototypeOf trap is not callable",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_get_prototype_of(
            object_payload_local,
            object_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(target_proto_tag_local);
        self.release_temp_local(target_proto_payload_local);
        self.release_temp_local(target_extensible_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(handled_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_get_prototype_of(
        &self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            result_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::GlobalGet(
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::GlobalGet(ERROR_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_object_set_prototype_of_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        proto_payload_local: u32,
        proto_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_set_prototype_of_i32_with_depth(
            object_payload_local,
            object_tag_local,
            proto_payload_local,
            proto_tag_local,
            result_local,
            0,
            function,
        )
    }

    pub(crate) fn emit_object_set_prototype_of_i32_with_depth(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        proto_payload_local: u32,
        proto_tag_local: u32,
        result_local: u32,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let handled_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let trap_truthy_local = self.reserve_temp_local();
        let target_extensible_local = self.reserve_temp_local();
        let target_proto_payload_local = self.reserve_temp_local();
        let target_proto_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 3,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("setPrototypeOf"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;

        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[
                (target_payload_local, target_tag_local),
                (proto_payload_local, proto_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            extra_throw_depth + 2,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(trap_truthy_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_call_object_is_extensible_helper(
            target_payload_local,
            target_tag_local,
            target_extensible_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_call_object_get_prototype_of_helper(
            target_payload_local,
            target_tag_local,
            target_proto_payload_local,
            target_proto_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        self.emit_tagged_payload_same_value_i32(
            proto_tag_local,
            proto_payload_local,
            target_proto_tag_local,
            target_proto_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy setPrototypeOf trap result incompatible with non-extensible target",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 6,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy setPrototypeOf trap is not callable",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_prototype_of_i32(
            object_payload_local,
            object_tag_local,
            proto_payload_local,
            proto_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(target_proto_tag_local);
        self.release_temp_local(target_proto_payload_local);
        self.release_temp_local(target_extensible_local);
        self.release_temp_local(trap_truthy_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(handled_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_set_prototype_of_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        proto_payload_local: u32,
        proto_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_proto_payload_local = self.reserve_temp_local();
        let current_proto_tag_local = self.reserve_temp_local();
        let same_value_local = self.reserve_temp_local();
        let cycle_payload_local = self.reserve_temp_local();
        let cycle_tag_local = self.reserve_temp_local();
        let next_cycle_payload_local = self.reserve_temp_local();
        let next_cycle_tag_local = self.reserve_temp_local();
        let cycle_found_local = self.reserve_temp_local();
        let internal_brand_local = self.reserve_temp_local();
        let cycle_boxed_kind_local = self.reserve_temp_local();

        self.emit_ordinary_get_prototype_of(
            object_payload_local,
            object_tag_local,
            current_proto_payload_local,
            current_proto_tag_local,
            function,
        );
        self.emit_tagged_payload_same_value_i32(
            proto_tag_local,
            proto_payload_local,
            current_proto_tag_local,
            current_proto_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(same_value_local));
        function.instruction(&Instruction::LocalGet(same_value_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            internal_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(internal_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_IMMUTABLE_PROTOTYPE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_ordinary_is_extensible_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_ordinary_is_extensible_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cycle_found_local));
        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proto_payload_local));
        function.instruction(&Instruction::LocalSet(cycle_payload_local));
        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::LocalSet(cycle_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cycle_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.emit_is_heap_object_like_tag_i32(cycle_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(cycle_payload_local));
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cycle_found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_tagged_payload_same_value_i32(
            cycle_tag_local,
            cycle_payload_local,
            object_tag_local,
            object_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cycle_found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(cycle_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cycle_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            cycle_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            cycle_boxed_kind_local,
            function,
        );
        // OrdinarySetPrototypeOf stops its cycle walk at a non-ordinary
        // [[GetPrototypeOf]] implementation.
        function.instruction(&Instruction::LocalGet(cycle_boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        self.emit_ordinary_get_prototype_of(
            cycle_payload_local,
            cycle_tag_local,
            next_cycle_payload_local,
            next_cycle_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_cycle_payload_local));
        function.instruction(&Instruction::LocalSet(cycle_payload_local));
        function.instruction(&Instruction::LocalGet(next_cycle_tag_local));
        function.instruction(&Instruction::LocalSet(cycle_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cycle_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(object_payload_local, HEAP_PROTOTYPE_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Null.tag() as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Null.tag() as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Null.tag() as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            proto_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            proto_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            proto_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            proto_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cycle_boxed_kind_local);
        self.release_temp_local(internal_brand_local);
        self.release_temp_local(cycle_found_local);
        self.release_temp_local(next_cycle_tag_local);
        self.release_temp_local(next_cycle_payload_local);
        self.release_temp_local(cycle_tag_local);
        self.release_temp_local(cycle_payload_local);
        self.release_temp_local(same_value_local);
        self.release_temp_local(current_proto_tag_local);
        self.release_temp_local(current_proto_payload_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_prevent_extensions_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let internal_brand_local = self.reserve_temp_local();
        let typed_array_length_tracking_local = self.reserve_temp_local();
        let typed_array_buffer_local = self.reserve_temp_local();
        let typed_array_buffer_flags_local = self.reserve_temp_local();
        let prevention_allowed_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(prevention_allowed_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
            1,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET,
            1,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            internal_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(internal_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,
            typed_array_length_tracking_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_length_tracking_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prevention_allowed_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            typed_array_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_buffer_local,
            HEAP_ARRAY_BUFFER_FLAGS_OFFSET,
            typed_array_buffer_flags_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_buffer_flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(typed_array_buffer_flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_SHARED as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prevention_allowed_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prevention_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(object_payload_local, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prevention_allowed_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(prevention_allowed_local);
        self.release_temp_local(typed_array_buffer_flags_local);
        self.release_temp_local(typed_array_buffer_local);
        self.release_temp_local(typed_array_length_tracking_local);
        self.release_temp_local(internal_brand_local);
    }

    pub(crate) fn emit_ordinary_is_extensible_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_CAP_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_object_prevent_extensions_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_prevent_extensions_i32_with_depth(
            object_payload_local,
            object_tag_local,
            result_local,
            4,
            0,
            function,
        )
    }

    pub(crate) fn emit_object_prevent_extensions_i32_with_depth(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        proxy_depth: u8,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let handled_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let trap_truthy_local = self.reserve_temp_local();
        let target_extensible_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));

        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("preventExtensions"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
            extra_throw_depth + 3,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(trap_truthy_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_depth == 0 {
            self.emit_ordinary_is_extensible_i32(
                target_payload_local,
                target_tag_local,
                target_extensible_local,
                function,
            );
        } else {
            self.emit_object_is_extensible_i32_with_depth(
                target_payload_local,
                target_tag_local,
                target_extensible_local,
                extra_throw_depth + 5,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap returned true for extensible target",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 6,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_ordinary_prevent_extensions_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_depth == 0 {
            self.emit_ordinary_prevent_extensions_i32(
                target_payload_local,
                target_tag_local,
                result_local,
                function,
            );
        } else {
            self.emit_object_prevent_extensions_i32_with_depth(
                target_payload_local,
                target_tag_local,
                result_local,
                proxy_depth - 1,
                extra_throw_depth + 5,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_prevent_extensions_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap is not callable",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_prevent_extensions_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(target_extensible_local);
        self.release_temp_local(trap_truthy_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(handled_local);
        Ok(())
    }

    pub(crate) fn emit_object_is_extensible_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_is_extensible_i32_with_depth(
            object_payload_local,
            object_tag_local,
            result_local,
            0,
            function,
        )
    }

    pub(crate) fn emit_object_is_extensible_i32_with_depth(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        extra_throw_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_object_is_extensible {
            if self.object_is_extensible_helper_function_index().is_some() {
                return self.emit_call_object_is_extensible_helper(
                    object_payload_local,
                    object_tag_local,
                    result_local,
                    extra_throw_depth,
                    function,
                );
            }
        }
        let handled_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let trap_truthy_local = self.reserve_temp_local();
        let target_result_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));

        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("isExtensible")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
            extra_throw_depth + 3,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(trap_truthy_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_call_object_is_extensible_helper(
            target_payload_local,
            target_tag_local,
            target_result_local,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalGet(target_result_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy isExtensible trap result does not match target",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_call_object_is_extensible_helper(
            target_payload_local,
            target_tag_local,
            result_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy isExtensible trap is not callable",
            self.result_local,
            self.result_tag_local,
            extra_throw_depth + 5,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_is_extensible_i32(
            object_payload_local,
            object_tag_local,
            result_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(target_result_local);
        self.release_temp_local(trap_truthy_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(handled_local);
        Ok(())
    }

    fn emit_typed_array_canonical_numeric_index_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        numeric_index_payload_local: u32,
        handled_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let canonical_numeric_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(canonical_numeric_index_local));
        self.emit_is_typed_array_i32(object_payload_local, object_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_canonical_numeric_index_string(
            key_payload_local,
            numeric_index_payload_local,
            canonical_numeric_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(canonical_numeric_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(canonical_numeric_index_local);
        Ok(())
    }

    fn emit_typed_array_integer_index_validity_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        result_local: u32,
        handled_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let numeric_index_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        self.emit_typed_array_canonical_numeric_index_i32(
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            numeric_index_payload_local,
            handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            object_payload_local,
            object_tag_local,
            numeric_index_payload_local,
            index_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(numeric_index_payload_local);
        Ok(())
    }

    fn emit_typed_array_set_same_receiver_if_handled(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        handled_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let numeric_index_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        self.emit_typed_array_canonical_numeric_index_i32(
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            numeric_index_payload_local,
            handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            18_446_744_073_709_551_616.0,
        )));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        self.emit_typed_array_element_write_from_locals(
            object_payload_local,
            object_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(numeric_index_payload_local);
        Ok(())
    }

    pub(crate) fn emit_object_has_property_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_tag_local = self.reserve_temp_local();
        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        let emit_result = self.emit_object_has_property_with_key_tag_i32(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        );
        self.release_temp_local(key_tag_local);
        emit_result
    }

    pub(crate) fn emit_object_has_property_with_key_tag_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let internal_key_local = self.reserve_temp_local();
        let trap_key_payload_local = self.reserve_temp_local();

        self.emit_property_key_value_payload_to_local(key_local, trap_key_payload_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
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
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("has")));
        function.instruction(&Instruction::LocalSet(internal_key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            internal_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (trap_key_payload_local, key_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_proxy_has_invariant_check(
            target_payload_local,
            target_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_has_property_by_tag_with_key_tag_i32(
            target_payload_local,
            target_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy has trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_ordinary_has_property_by_tag_with_key_tag_i32(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(trap_key_payload_local);
        self.release_temp_local(internal_key_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_ordinary_has_property_by_tag_with_key_tag_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let named_payload_local = self.reserve_temp_local();
        let named_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        self.load_i64_from_offset(
            object_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(object_local, index_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_read(
            object_local,
            key_local,
            named_payload_local,
            named_tag_local,
            Some(result_local),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32_ordinary_with_key_tag(
            prototype_local,
            prototype_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32_ordinary_with_key_tag(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(named_tag_local);
        self.release_temp_local(named_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_object_has_property_i32_ordinary(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_tag_local = self.reserve_temp_local();
        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        let emit_result = self.emit_object_has_property_i32_ordinary_with_key_tag(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            result_local,
            function,
        );
        self.release_temp_local(key_tag_local);
        emit_result
    }

    pub(crate) fn emit_object_has_property_i32_ordinary_with_key_tag(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        key_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let internal_key_local = self.reserve_temp_local();
        let trap_key_payload_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let named_payload_local = self.reserve_temp_local();
        let named_tag_local = self.reserve_temp_local();
        let integer_indexed_has_handled_local = self.reserve_temp_local();

        self.emit_property_key_value_payload_to_local(key_local, trap_key_payload_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_integer_index_validity_i32(
            current_local,
            current_tag_local,
            key_local,
            key_tag_local,
            result_local,
            integer_indexed_has_handled_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(integer_indexed_has_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(current_local, index_local, result_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_read(
            current_local,
            key_local,
            named_payload_local,
            named_tag_local,
            Some(result_local),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Else);
        self.emit_load_prototype_to_current_locals(
            current_local,
            current_tag_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_object_boxed_kind_for_tag(
            current_local,
            current_tag_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(target_payload_local, buffer_local, len_local, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            len_local,
            entry_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
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
            current_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("has")));
        function.instruction(&Instruction::LocalSet(internal_key_local));
        self.emit_object_read_ordinary(
            descriptor_kind_local,
            handler_tag_local,
            descriptor_kind_local,
            handler_tag_local,
            internal_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            trap_payload_local,
            trap_tag_local,
            Some((descriptor_kind_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (trap_key_payload_local, key_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        // Leave the traversal before propagating: the completion check is
        // nested inside the callable branch, Proxy branch, loop, and block.
        self.emit_break_current_completion_if_throw(4, function);
        function.instruction(&Instruction::LocalGet(trap_result_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_proxy_has_invariant_check(
            target_payload_local,
            target_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(target_payload_local, index_local, result_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_read(
            target_payload_local,
            key_local,
            named_payload_local,
            named_tag_local,
            Some(result_local),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy has trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(current_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(current_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.emit_property_key_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_load_prototype_to_current_locals(
            current_local,
            current_tag_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;

        self.release_temp_local(integer_indexed_has_handled_local);
        self.release_temp_local(named_tag_local);
        self.release_temp_local(named_payload_local);
        self.release_temp_local(done_local);
        self.release_temp_local(trap_key_payload_local);
        self.release_temp_local(internal_key_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_local);
        Ok(())
    }

    pub(crate) fn emit_data_property_read_no_call(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let current_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_object_own_data_field_read(
            current_local,
            current_tag_local,
            key_local,
            present_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        self.release_temp_local(present_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_local);
    }
}
