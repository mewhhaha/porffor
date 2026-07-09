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
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(current_tag_local));
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
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(None, Some(prototype_global_index), function)?;
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        if self.is_main()
            && self
                .standard_builtin_for_function_meta(meta)
                .is_some_and(|builtin| {
                    !self
                        .runtime_bootstrap_plan
                        .should_initialize_standard_builtin(builtin)
                })
        {
            return Ok(());
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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

    fn standard_builtin_for_function_meta(
        &self,
        meta: &WasmFunctionMeta,
    ) -> Option<StandardBuiltinId> {
        self.functions
            .iter()
            .find(|(_, candidate)| {
                candidate.wasm_index == meta.wasm_index
                    && candidate.table_index == meta.table_index
                    && candidate.name == meta.name
                    && candidate.to_string_value == meta.to_string_value
            })
            .and_then(|(function_id, _)| StandardBuiltinId::from_function_id(function_id))
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
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
        let capacity = (properties.len() as u64).max(MIN_HEAP_CAPACITY);
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
            let key_local = self.reserve_temp_local();
            match property {
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

        match target.kind {
            ValueKind::Object | ValueKind::Function | ValueKind::Dynamic => {
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
                        self.release_temp_local(target_tag_local);
                        self.release_temp_local(target_local);
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
                    self.release_temp_local(target_tag_local);
                    self.release_temp_local(target_local);
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
                        self.release_temp_local(target_tag_local);
                        self.release_temp_local(target_local);
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
                                                .and_then(standard_builtin_constructor_global_index)
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
                                        self.release_temp_local(target_tag_local);
                                        self.release_temp_local(target_local);
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
                    let byte_length_payload_local = self.reserve_temp_local();
                    let byte_length_tag_local = self.reserve_temp_local();
                    let buffer_payload_local = self.reserve_temp_local();
                    let buffer_tag_local = self.reserve_temp_local();
                    let data_ptr_local = self.reserve_temp_local();
                    let byte_offset_local = self.reserve_temp_local();
                    let buffer_byte_length_local = self.reserve_temp_local();
                    let tracking_payload_local = self.reserve_temp_local();
                    let tracking_tag_local = self.reserve_temp_local();
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
                        );
                        function.instruction(&Instruction::Else);
                    }
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
                    ));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read_ordinary(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        byte_length_payload_local,
                        byte_length_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(byte_length_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
                    ));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        buffer_payload_local,
                        buffer_tag_local,
                        function,
                    )?;
                    self.emit_object_read_number_slot_to_i64_local(
                        buffer_payload_local,
                        ARRAY_BUFFER_DATA_PTR_SLOT,
                        data_ptr_local,
                        function,
                    )?;
                    self.emit_object_read_number_slot_to_i64_local(
                        target_local,
                        TYPED_ARRAY_BYTE_OFFSET_SLOT,
                        byte_offset_local,
                        function,
                    )?;
                    self.emit_object_read_number_slot_to_i64_local(
                        target_local,
                        TYPED_ARRAY_BYTE_LENGTH_SLOT,
                        typed_byte_length_local,
                        function,
                    )?;
                    self.emit_object_read_number_slot_to_i64_local(
                        target_local,
                        TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
                        typed_bytes_per_element_local,
                        function,
                    )?;
                    self.emit_object_read_number_slot_to_i64_local(
                        buffer_payload_local,
                        ARRAY_BUFFER_BYTE_LENGTH_SLOT,
                        buffer_byte_length_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(TYPED_ARRAY_LENGTH_TRACKING_SLOT),
                    ));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        tracking_payload_local,
                        tracking_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(tracking_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(tracking_payload_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(byte_offset_local));
                    function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
                    function.instruction(&Instruction::I64GtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(typed_byte_length_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
                    function.instruction(&Instruction::LocalGet(byte_offset_local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(typed_byte_length_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(byte_offset_local));
                    function.instruction(&Instruction::LocalGet(typed_byte_length_local));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
                    function.instruction(&Instruction::I64GtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(typed_byte_length_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
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
                    self.release_temp_local(tracking_tag_local);
                    self.release_temp_local(tracking_payload_local);
                    self.release_temp_local(buffer_byte_length_local);
                    self.release_temp_local(byte_offset_local);
                    self.release_temp_local(data_ptr_local);
                    self.release_temp_local(buffer_tag_local);
                    self.release_temp_local(buffer_payload_local);
                    self.release_temp_local(byte_length_tag_local);
                    self.release_temp_local(byte_length_payload_local);
                    self.release_temp_local(key_local);
                    self.release_temp_local(target_tag_local);
                    self.release_temp_local(target_local);
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
                    self.emit_array_read(
                        target_local,
                        array_index_local,
                        payload_local,
                        tag_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    self.emit_array_named_prop_read(
                        target_local,
                        key_local,
                        payload_local,
                        tag_local,
                        None,
                        function,
                    );
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
                            true,
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
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::LocalGet(target_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
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
                        self.emit_array_named_prop_read(
                            target_local,
                            key_local,
                            payload_local,
                            tag_local,
                            None,
                            function,
                        );
                        function.instruction(&Instruction::End);
                        self.release_temp_local(key_local);
                    }
                },
                PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    let array_index_local = self.reserve_temp_local();
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
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_is_concat_spreadable_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_array_named_prop_read(
                        target_local,
                        key_local,
                        payload_local,
                        tag_local,
                        None,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
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
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
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
                PropertyKeyIr::ArrayIndex(_) => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    let end_local = self.reserve_temp_local();
                    let byte_start_local = self.reserve_temp_local();
                    let byte_end_local = self.reserve_temp_local();
                    let byte_len_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(end_local));
                    self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
                        target_local,
                        index_local,
                        byte_start_local,
                        function,
                    );
                    self.emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
                        target_local,
                        end_local,
                        byte_end_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(byte_end_local));
                    function.instruction(&Instruction::LocalGet(byte_start_local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(byte_len_local));
                    self.emit_string_slice_payload_from_locals(
                        target_local,
                        byte_start_local,
                        byte_len_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.release_temp_local(byte_len_local);
                    self.release_temp_local(byte_end_local);
                    self.release_temp_local(byte_start_local);
                    self.release_temp_local(end_local);
                    self.release_temp_local(index_local);
                }
                PropertyKeyIr::StaticString(name)
                    if matches!(
                        name.as_str(),
                        "match"
                            | "matchAll"
                            | "replace"
                            | "replaceAll"
                            | "search"
                            | "indexOf"
                            | "lastIndexOf"
                            | "charAt"
                            | "charCodeAt"
                            | "codePointAt"
                            | "at"
                            | "slice"
                            | "split"
                            | "padStart"
                            | "padEnd"
                            | "repeat"
                            | "endsWith"
                            | "includes"
                            | "startsWith"
                            | "toUpperCase"
                            | "toString"
                            | "valueOf"
                            | "isWellFormed"
                            | "toWellFormed"
                    ) =>
                {
                    let builtin = match name.as_str() {
                        "match" => StandardBuiltinId::StringPrototypeMatch,
                        "matchAll" => StandardBuiltinId::StringPrototypeMatchAll,
                        "replace" => StandardBuiltinId::StringPrototypeReplace,
                        "replaceAll" => StandardBuiltinId::StringPrototypeReplaceAll,
                        "search" => StandardBuiltinId::StringPrototypeSearch,
                        "indexOf" => StandardBuiltinId::StringPrototypeIndexOf,
                        "lastIndexOf" => StandardBuiltinId::StringPrototypeLastIndexOf,
                        "charAt" => StandardBuiltinId::StringPrototypeCharAt,
                        "charCodeAt" => StandardBuiltinId::StringPrototypeCharCodeAt,
                        "codePointAt" => StandardBuiltinId::StringPrototypeCodePointAt,
                        "at" => StandardBuiltinId::StringPrototypeAt,
                        "slice" => StandardBuiltinId::StringPrototypeSlice,
                        "split" => StandardBuiltinId::StringPrototypeSplit,
                        "padStart" => StandardBuiltinId::StringPrototypePadStart,
                        "padEnd" => StandardBuiltinId::StringPrototypePadEnd,
                        "repeat" => StandardBuiltinId::StringPrototypeRepeat,
                        "endsWith" => StandardBuiltinId::StringPrototypeEndsWith,
                        "includes" => StandardBuiltinId::StringPrototypeIncludes,
                        "startsWith" => StandardBuiltinId::StringPrototypeStartsWith,
                        "toUpperCase" => StandardBuiltinId::StringPrototypeToUpperCase,
                        "toString" => StandardBuiltinId::StringPrototypeToString,
                        "valueOf" => StandardBuiltinId::StringPrototypeValueOf,
                        "isWellFormed" => StandardBuiltinId::StringPrototypeIsWellFormed,
                        "toWellFormed" => StandardBuiltinId::StringPrototypeToWellFormed,
                        _ => unreachable!(),
                    };
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
                }
                PropertyKeyIr::StaticString(name) if name == "Symbol.iterator" => {
                    let meta = self
                        .functions
                        .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                        .ok_or_else(|| {
                            EmitError::unsupported(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype[Symbol.iterator]`",
                            )
                        })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                PropertyKeyIr::StaticString(name) if name == "description" => {
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
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                }
                _ => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            },
            ValueKind::Arguments => match key {
                PropertyKeyIr::ArrayLength => {
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
                }
                PropertyKeyIr::StaticString(name) if name == "length" => {
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
                }
                PropertyKeyIr::StaticString(name) if name == "callee" => {
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "%ThrowTypeError%",
                        payload_local,
                        tag_local,
                        function,
                    )?;
                }
                PropertyKeyIr::StaticString(name) if name == "Symbol.iterator" => {
                    let values_meta = self
                        .functions
                        .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                        .ok_or_else(|| {
                            EmitError::unsupported(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `Arguments @@iterator`",
                            )
                        })?;
                    self.emit_function_value_payload(values_meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                PropertyKeyIr::StaticString(name) if name == "Symbol.isConcatSpreadable" => {
                    self.emit_arguments_is_concat_spreadable_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    );
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
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_is_concat_spreadable_read(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    );
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
                        true,
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
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
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
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
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
            ValueKind::BigInt => match key {
                PropertyKeyIr::StaticString(name) => {
                    let builtin = match name.as_str() {
                        "toString" => Some(StandardBuiltinId::BigIntPrototypeToString),
                        "toLocaleString" => Some(StandardBuiltinId::BigIntPrototypeToLocaleString),
                        "valueOf" => Some(StandardBuiltinId::BigIntPrototypeValueOf),
                        _ => None,
                    };
                    if let Some(builtin) = builtin {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        self.emit_function_value_payload(meta, function)?;
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
                }
                _ => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            },
            _ => {
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: property access on non-object target",
                ));
            }
        }

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    pub(crate) fn emit_typed_array_or_object_index_read_from_locals(
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
        let buffer_tag_local = self.reserve_temp_local();
        let typed_array_slot_present_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();

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
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            target_local,
            target_tag_local,
            key_local,
            typed_array_slot_present_local,
            buffer_payload_local,
            buffer_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_slot_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_DATA_PTR_SLOT,
            data_ptr_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            bytes_per_element_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_ELEMENT_KIND_SLOT,
            element_kind_local,
            function,
        )?;
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
        function.instruction(&Instruction::F64Load(Self::memarg64(0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::F32Load(Self::memarg32(0)));
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
        function.instruction(&Instruction::I32Load8S(Self::memarg8(0)));
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
        function.instruction(&Instruction::I32Load16S(Self::memarg16(0)));
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
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
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
        function.instruction(&Instruction::I32Load16U(Self::memarg16(0)));
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
        function.instruction(&Instruction::I32Load(Self::memarg32(0)));
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
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load(Self::memarg32(0)));
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
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(address_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(typed_array_slot_present_local);
        self.release_temp_local(buffer_tag_local);
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
        let typed_array_present_local = self.reserve_temp_local();
        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let typed_array_key_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(typed_array_key_local));
        self.emit_object_own_data_field_read(
            target_local,
            target_tag_local,
            typed_array_key_local,
            typed_array_present_local,
            typed_array_payload_local,
            typed_array_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(typed_array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
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

        self.release_temp_local(typed_array_key_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);
        self.release_temp_local(typed_array_present_local);
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
                if let PropertyKeyIr::StaticString(key_name) = key {
                    if let Some(ObjectShapeProperty::Accessor {
                        setter: Some(setter),
                        ..
                    }) = target
                        .heap_shape
                        .as_deref()
                        .and_then(|shape| read_static_heap_shape_property(shape, key_name))
                    {
                        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        let setter_payload_local = self.reserve_temp_local();
                        let setter_tag_local = self.reserve_temp_local();
                        let setter_result_payload_local = self.reserve_temp_local();
                        let setter_result_tag_local = self.reserve_temp_local();
                        let setter_meta =
                            self.functions.get(&setter.function_id).ok_or_else(|| {
                                EmitError::unsupported(format!(
                                    "unsupported in porffor wasm-aot first slice: missing setter `{}`",
                                    setter.function_id
                                ))
                            })?;
                        self.emit_function_value_payload(setter_meta, function)?;
                        function.instruction(&Instruction::LocalSet(setter_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(setter_tag_local));
                        self.emit_function_handle_call(
                            setter_payload_local,
                            setter_tag_local,
                            Some((target_local, Some(target_tag_local))),
                            &[(payload_local, tag_local)],
                            setter_result_payload_local,
                            setter_result_tag_local,
                            function,
                        )?;
                        self.release_temp_local(setter_result_tag_local);
                        self.release_temp_local(setter_result_payload_local);
                        self.release_temp_local(setter_tag_local);
                        self.release_temp_local(setter_payload_local);
                        self.release_temp_local(target_tag_local);
                        self.release_temp_local(target_local);
                        return Ok(());
                    }
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
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                if matches!(key, PropertyKeyIr::ArrayLength)
                    || matches!(key, PropertyKeyIr::StaticString(name) if name == "length")
                {
                    self.emit_value_to_number_payload(tag_local, payload_local, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    self.emit_return_current_completion_if_throw(function);
                    self.emit_array_set_length_from_number_payload(
                        target_local,
                        payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
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
                } else if matches!(key, PropertyKeyIr::StaticString(name) if name == "constructor")
                {
                    self.store_i64_local_at_offset(
                        target_local,
                        HEAP_ARRAY_CONSTRUCTOR_PAYLOAD_OFFSET,
                        payload_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        target_local,
                        HEAP_ARRAY_CONSTRUCTOR_TAG_OFFSET,
                        tag_local,
                        function,
                    );
                    self.store_i64_const_at_offset(
                        target_local,
                        HEAP_ARRAY_CONSTRUCTOR_DESCRIPTOR_KIND_OFFSET,
                        ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
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
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
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
                        self.emit_array_define_named_data_property(
                            target_local,
                            key_local,
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(key_local);
                    }
                } else {
                    let index_local = self.compile_array_index_to_local(key, function)?;
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
                        HEAP_LEN_OFFSET,
                        len_local,
                        function,
                    );
                    self.release_temp_local(len_local);
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
                        self.strings.payload("Symbol.isConcatSpreadable"),
                    ));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalSet(is_spreadable_key_local));
                    function.instruction(&Instruction::LocalGet(is_spreadable_key_local));
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_is_concat_spreadable_write(
                        target_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    let len_local = self.reserve_temp_local();
                    self.load_i64_to_local_from_offset(
                        target_local,
                        HEAP_LEN_OFFSET,
                        len_local,
                        function,
                    );
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.store_i64_local_at_offset(
                        target_local,
                        HEAP_LEN_OFFSET,
                        len_local,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    self.release_temp_local(len_local);
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
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_write(
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(key_local);
                }
            }
            _ => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(target_tag_local));
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
        let key_local = self.reserve_temp_local();
        let slot_payload_local = self.reserve_temp_local();
        let slot_tag_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let requested_length_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray.from constructed target is not a typed array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            key_local,
            slot_payload_local,
            slot_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(slot_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray.from constructed target is not a typed array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for slot in [
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            TYPED_ARRAY_ELEMENT_KIND_SLOT,
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(slot)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                target_payload_local,
                target_tag_local,
                target_payload_local,
                target_tag_local,
                key_local,
                slot_payload_local,
                slot_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(slot_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "TypedArray.from constructed target is not a typed array",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            if slot == TYPED_ARRAY_BYTE_LENGTH_SLOT {
                function.instruction(&Instruction::LocalGet(slot_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncSatF64U);
                function.instruction(&Instruction::LocalSet(byte_length_local));
            } else if slot == TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT {
                function.instruction(&Instruction::LocalGet(slot_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncSatF64U);
                function.instruction(&Instruction::LocalSet(bytes_per_element_local));
            }
        }

        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray.from constructed target is not a typed array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

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
            "TypedArray.from constructed target is too small",
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
        self.release_temp_local(slot_tag_local);
        self.release_temp_local(slot_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_integer_typed_array_value_i64(
        &mut self,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_store_number_payload_to_typed_array_address_by_kind(
        &mut self,
        bytes_per_element_local: u32,
        element_kind_local: u32,
        address_local: u32,
        number_payload_local: u32,
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
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Store(Self::memarg64(0)));
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
        function.instruction(&Instruction::F32Store(Self::memarg32(0)));
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
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
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
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
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
        function.instruction(&Instruction::I32Store16(Self::memarg16(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store(Self::memarg32(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
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
        let key_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            buffer_payload_local,
            buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_DATA_PTR_SLOT,
            data_ptr_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            bytes_per_element_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            target_local,
            TYPED_ARRAY_ELEMENT_KIND_SLOT,
            element_kind_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            target_local,
            target_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_bigint_payload(value_tag_local, value_payload_local, false, function)?;
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);

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
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Store(Self::memarg64(0)));
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
        function.instruction(&Instruction::F32Store(Self::memarg32(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I32TruncSatF64S);
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
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
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
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
        function.instruction(&Instruction::I32Store16(Self::memarg16(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store(Self::memarg32(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(address_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(key_local);
        Ok(())
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
                let key_local = self.compile_object_key_to_local(key, function)?;
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
                    self.emit_object_delete(
                        target_local,
                        target_tag_local,
                        key_local,
                        result_local,
                        function,
                    )?;
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
                PropertyKeyIr::ArrayLength => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(result_local));
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
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::End);
                    self.release_temp_local(array_index_local);
                } else {
                    function.instruction(&Instruction::End);
                }
                self.release_temp_local(key_local);
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
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
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
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
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
        let brand_key_local = self.reserve_temp_local();
        let brand_payload_local = self.reserve_temp_local();
        let brand_tag_local = self.reserve_temp_local();
        let data_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_data_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(data_key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            data_key_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(data_key_local);
        self.release_temp_local(brand_tag_local);
        self.release_temp_local(brand_payload_local);
        self.release_temp_local(brand_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
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
        let brand_key_local = self.reserve_temp_local();
        let brand_payload_local = self.reserve_temp_local();
        let brand_tag_local = self.reserve_temp_local();
        let data_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_data_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(data_key_local));
        self.emit_object_write(
            target_payload_local,
            target_tag_local,
            data_key_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(data_key_local);
        self.release_temp_local(brand_tag_local);
        self.release_temp_local(brand_payload_local);
        self.release_temp_local(brand_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_private_brand_guard(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        brand_key_local: u32,
        brand_payload_local: u32,
        brand_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_brand_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(brand_key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(brand_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(brand_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
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
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
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
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_output_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_output_local));
                function.instruction(&Instruction::End);
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
        let mut first = true;
        for symbol_name in [
            "Symbol()",
            "Symbol.iterator",
            "Symbol.dispose",
            "Symbol.species",
            "Symbol.isConcatSpreadable",
            "Symbol.match",
            "Symbol.matchAll",
            "Symbol.replace",
            "Symbol.search",
            "Symbol.split",
            "Symbol.toStringTag",
            "Symbol.toPrimitive",
        ] {
            function.instruction(&Instruction::LocalGet(key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(symbol_name)));
            function.instruction(&Instruction::I64Eq);
            if first {
                first = false;
            } else {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
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
        function.instruction(&Instruction::LocalGet(trap_tag_local));
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
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (key_local, key_tag_local),
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
                (key_local, key_tag_local),
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(list_len_local));
        function.instruction(&Instruction::End);
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
        function.instruction(&Instruction::LocalGet(target_entry_key_local));
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
        function.instruction(&Instruction::LocalGet(target_entry_key_local));
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
                (key_payload_local, key_tag_local),
            ],
            desc_payload_local,
            desc_tag_local,
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

    pub(crate) fn emit_proxy_own_keys_filtered_result(
        &mut self,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        expected_key_tag: ValueKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_len_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_key_payload_local = self.reserve_temp_local();

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
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(list_len_local));
        function.instruction(&Instruction::End);
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

        self.release_temp_local(index_key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(list_len_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_own_keys_array_result(
        &mut self,
        own_keys_payload_local: u32,
        own_keys_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_len_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let index_key_payload_local = self.reserve_temp_local();

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
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(list_len_local));
        function.instruction(&Instruction::End);
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

        self.emit_alloc_array_payload_with_length(list_len_local, result_payload_local, function)?;
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
        self.emit_array_write(
            result_payload_local,
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

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);

        self.release_temp_local(index_key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        self.emit_array_named_prop_read(
            object_local,
            key_local,
            payload_local,
            tag_local,
            None,
            function,
        );
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
            return self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function);
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
        self.emit_array_named_prop_read(
            current_local,
            key_local,
            payload_local,
            tag_local,
            Some(found_local),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.load_i64_to_local_from_offset(
                current_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
            function.instruction(&Instruction::I64GeU);
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
            self.emit_function_handle_call(
                proxy_trap_payload_local,
                proxy_trap_tag_local,
                Some((descriptor_kind_local, Some(proxy_key_tag_local))),
                &[
                    (getter_payload_local, getter_tag_local),
                    (key_local, proxy_key_tag_local),
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(extra_depth) = accessor_throw_extra_depth {
            self.emit_function_handle_call_with_throw_extra_depth(
                getter_payload_local,
                getter_tag_local,
                Some((receiver_payload_local, Some(receiver_tag_local))),
                &[],
                payload_local,
                tag_local,
                extra_depth,
                function,
            )?;
        } else {
            self.emit_function_handle_call_without_throw_propagation(
                getter_payload_local,
                getter_tag_local,
                Some((receiver_payload_local, Some(receiver_tag_local))),
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
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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

    pub(crate) fn emit_object_define_enumerable_accessor(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
            self.emit_tagged_payload_equality_i32(
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
        if let Some(present_local) = writable_present_local {
            function.instruction(&Instruction::LocalGet(present_local));
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

        function.instruction(&Instruction::LocalGet(cap_local));
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
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
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
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
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
        let inherited_setter_found_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
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
        let proxy_descriptor_payload_local = self.reserve_temp_local();
        let proxy_descriptor_tag_local = self.reserve_temp_local();
        let proxy_bool_payload_local = self.reserve_temp_local();
        let proxy_bool_tag_local = self.reserve_temp_local();
        let proxy_reflect_set_payload_local = self.reserve_temp_local();
        let proxy_reflect_set_tag_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_index_write_handled_local));
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
        self.emit_function_handle_call(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            Some((proxy_handler_payload_local, Some(proxy_handler_tag_local))),
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
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
        self.emit_ordinary_set_result_with_receiver_fallback(
            proxy_target_payload_local,
            proxy_target_tag_local,
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            proxy_trap_result_payload_local,
            false,
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
        self.emit_array_write(
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        if self.is_current_function_strict() {
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "Cannot assign to read only property",
                self.result_local,
                self.result_tag_local,
                9,
                function,
            )?;
        }
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
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((object_local, Some(object_tag_local))),
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        if self.is_current_function_strict() {
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "Cannot assign to read only property",
                self.result_local,
                self.result_tag_local,
                9,
                function,
            )?;
        }
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
        function.instruction(&Instruction::LocalSet(inherited_setter_found_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            prototype_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&reflect_set_meta, function)?;
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(inherited_setter_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(inherited_setter_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_proxy_set_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(inherited_setter_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((object_local, Some(object_tag_local))),
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
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
            function.instruction(&Instruction::Br(1));
        }
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

        self.release_temp_local(proxy_reflect_set_tag_local);
        self.release_temp_local(proxy_reflect_set_payload_local);
        self.release_temp_local(proxy_bool_tag_local);
        self.release_temp_local(proxy_bool_payload_local);
        self.release_temp_local(proxy_descriptor_tag_local);
        self.release_temp_local(proxy_descriptor_payload_local);
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
        self.release_temp_local(prototype_local);
        self.release_temp_local(inherited_setter_found_local);
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
        value_payload_local: u32,
        value_tag_local: u32,
        result_local: u32,
        proxy_depth: u8,
        allow_generic_write_fallback: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let proxy_kind_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
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

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

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

            self.emit_function_value_payload(&reflect_define_meta, function)?;
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

        self.emit_is_object_entry_backed_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        self.release_temp_local(key_tag_local);
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

    pub(crate) fn emit_ordinary_set_result(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
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
            value_payload_local,
            value_tag_local,
            result_local,
            true,
            function,
        )
    }

    pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
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
        let key_tag_local = self.reserve_temp_local();
        let reflect_set_payload_local = self.reserve_temp_local();
        let reflect_set_tag_local = self.reserve_temp_local();
        let reflect_set_result_tag_local = self.reserve_temp_local();
        let boxed_payload_local = self.reserve_temp_local();
        let boxed_tag_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_unit_len_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));

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
        self.emit_function_value_payload(&reflect_set_meta, function)?;
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_object_entry_backed_tag_i32(current_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[(value_payload_local, value_tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
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
            value_payload_local,
            value_tag_local,
            result_local,
            allow_receiver_generic_write_fallback,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(string_unit_len_local);
        self.release_temp_local(string_byte_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(boxed_tag_local);
        self.release_temp_local(boxed_payload_local);
        self.release_temp_local(reflect_set_result_tag_local);
        self.release_temp_local(reflect_set_tag_local);
        self.release_temp_local(reflect_set_payload_local);
        self.release_temp_local(key_tag_local);
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
        self.emit_property_key_tag_from_payload(key_local, key_tag_local, function);
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (key_local, key_tag_local),
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
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_delete_property_key(object_local, key_local, result_local, function);
        function.instruction(&Instruction::Else);
        self.emit_object_delete_ordinary(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
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
        self.emit_string_payload_equality_i32(key_payload_local, key_local, function);
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
            4,
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
            (ValueKind::Symbol, OBJECT_PROTOTYPE_GLOBAL_INDEX),
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
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
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
        self.emit_function_handle_call_with_throw_extra_depth(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
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

        function.instruction(&Instruction::LocalGet(trap_result_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::LocalGet(trap_result_tag_local));
        function.instruction(&Instruction::LocalSet(result_tag_local));

        self.emit_object_is_extensible_i32_with_depth(
            target_payload_local,
            target_tag_local,
            target_extensible_local,
            proxy_depth,
            extra_throw_depth + 3,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_depth == 0 {
            self.emit_ordinary_get_prototype_of(
                target_payload_local,
                target_tag_local,
                target_proto_payload_local,
                target_proto_tag_local,
                function,
            );
        } else {
            self.emit_object_get_prototype_of_with_depth(
                target_payload_local,
                target_tag_local,
                target_proto_payload_local,
                target_proto_tag_local,
                proxy_depth - 1,
                extra_throw_depth + 4,
                function,
            )?;
        }
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
            self.emit_ordinary_get_prototype_of(
                target_payload_local,
                target_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            );
        } else {
            self.emit_object_get_prototype_of_with_depth(
                target_payload_local,
                target_tag_local,
                result_payload_local,
                result_tag_local,
                proxy_depth - 1,
                extra_throw_depth + 4,
                function,
            )?;
        }
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
            4,
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
        let target_proto_payload_local = self.reserve_temp_local();
        let target_proto_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
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
                (proto_payload_local, proto_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
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
        self.emit_object_is_extensible_i32_with_depth(
            target_payload_local,
            target_tag_local,
            target_extensible_local,
            proxy_depth,
            extra_throw_depth + 4,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if proxy_depth == 0 {
            self.emit_ordinary_get_prototype_of(
                target_payload_local,
                target_tag_local,
                target_proto_payload_local,
                target_proto_tag_local,
                function,
            );
        } else {
            self.emit_object_get_prototype_of_with_depth(
                target_payload_local,
                target_tag_local,
                target_proto_payload_local,
                target_proto_tag_local,
                proxy_depth - 1,
                extra_throw_depth + 5,
                function,
            )?;
        }
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
        if proxy_depth == 0 {
            self.emit_ordinary_set_prototype_of_i32(
                target_payload_local,
                target_tag_local,
                proto_payload_local,
                proto_tag_local,
                result_local,
                function,
            )?;
        } else {
            self.emit_object_set_prototype_of_i32_with_depth(
                target_payload_local,
                target_tag_local,
                proto_payload_local,
                proto_tag_local,
                result_local,
                proxy_depth - 1,
                extra_throw_depth + 5,
                function,
            )?;
        }
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

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
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
        let cap_local = self.reserve_temp_local();
        let cycle_payload_local = self.reserve_temp_local();
        let cycle_tag_local = self.reserve_temp_local();
        let next_cycle_payload_local = self.reserve_temp_local();
        let next_cycle_tag_local = self.reserve_temp_local();
        let cycle_found_local = self.reserve_temp_local();

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
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_CAP_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
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
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cycle_found_local);
        self.release_temp_local(next_cycle_tag_local);
        self.release_temp_local(next_cycle_payload_local);
        self.release_temp_local(cycle_tag_local);
        self.release_temp_local(cycle_payload_local);
        self.release_temp_local(cap_local);
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
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(object_payload_local, HEAP_CAP_OFFSET, 0, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
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
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
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
            self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            self.load_i64_from_offset(target_payload_local, HEAP_CAP_OFFSET, function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(target_extensible_local));
        } else {
            self.emit_object_is_extensible_i32_with_depth(
                target_payload_local,
                target_tag_local,
                target_extensible_local,
                proxy_depth - 1,
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
        self.store_i64_const_at_offset(object_payload_local, HEAP_CAP_OFFSET, 0, function);
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
        self.store_i64_const_at_offset(object_payload_local, HEAP_CAP_OFFSET, 0, function);
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
            4,
            0,
            function,
        )
    }

    pub(crate) fn emit_object_is_extensible_i32_with_depth(
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
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[(target_payload_local, target_tag_local)],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(trap_truthy_local));
        function.instruction(&Instruction::LocalGet(trap_truthy_local));
        function.instruction(&Instruction::LocalSet(result_local));
        if proxy_depth == 0 {
            self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            self.load_i64_from_offset(target_payload_local, HEAP_CAP_OFFSET, function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(target_result_local));
        } else {
            self.emit_object_is_extensible_i32_with_depth(
                target_payload_local,
                target_tag_local,
                target_result_local,
                proxy_depth - 1,
                extra_throw_depth + 4,
                function,
            )?;
        }
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
        if proxy_depth == 0 {
            self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            self.load_i64_from_offset(target_payload_local, HEAP_CAP_OFFSET, function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(result_local));
        } else {
            self.emit_object_is_extensible_i32_with_depth(
                target_payload_local,
                target_tag_local,
                result_local,
                proxy_depth - 1,
                extra_throw_depth + 5,
                function,
            )?;
        }
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
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_CAP_OFFSET,
            target_result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
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
                (key_local, key_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
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
        function.instruction(&Instruction::LocalGet(object_tag_local));
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
        self.emit_array_has_index_i32(object_local, index_local, result_local, function);
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
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
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
        let done_local = self.reserve_temp_local();

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
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((descriptor_kind_local, Some(handler_tag_local))),
            &[
                (target_payload_local, target_tag_local),
                (key_local, key_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
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
        self.emit_string_payload_equality_i32(key_payload_local, key_local, function);
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

        self.release_temp_local(done_local);
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
