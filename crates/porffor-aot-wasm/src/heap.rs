use porffor_ir::ValueKind;

use super::*;

pub(crate) const WASM_PAGE_SIZE: u64 = 65_536;
pub(crate) const STATIC_DATA_OFFSET: u32 = 4096;
pub(crate) const MIN_HEAP_CAPACITY: u64 = 1;
pub(crate) const MAX_ARRAY_BUFFER_BYTE_LENGTH: u64 = 1 << 32;
pub(crate) const MAX_DENSE_ARRAY_INDEX: u64 = 1_000_000;
pub(crate) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub(crate) const MAX_ARRAY_LENGTH: u64 = 4_294_967_295;
pub(crate) const HEAP_ARRAY_HOLE_TAG: i64 = ValueKind::Dynamic.tag() as i64;

pub(crate) const HEAP_HEADER_SIZE: u64 = 256;
pub(crate) const HEAP_FUNCTION_OBJECT_SIZE: u64 = 272;
pub(crate) const HEAP_OBJECT_ENTRY_SIZE: u64 = 64;
pub(crate) const HEAP_ARRAY_ENTRY_SIZE: u64 = 24;
pub(crate) const SPARSE_ARRAY_DENSE_GROW_FACTOR: u64 = 16;
pub(crate) const HEAP_BOUND_FUNCTION_RECORD_SIZE: u64 = 40;
pub(crate) const HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET: u64 = 32;
pub(crate) const HEAP_ARGUMENTS_ENV_HANDLE_OFFSET: u64 = 40;
pub(crate) const HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET: u64 = 48;
pub(crate) const HEAP_OBJECT_BOXED_KIND_OFFSET: u64 = 32;
pub(crate) const HEAP_OBJECT_BOXED_TAG_OFFSET: u64 = 40;
pub(crate) const HEAP_OBJECT_BOXED_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_OBJECT_INTERNAL_BRAND_OFFSET: u64 = 56;
pub(crate) const HEAP_OBJECT_PROTOTYPE_TAG_OFFSET: u64 = 64;
pub(crate) const HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 72;
pub(crate) const HEAP_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_CAP_OFFSET: u64 = 16;
pub(crate) const HEAP_PROTOTYPE_OFFSET: u64 = 24;
pub(crate) const HEAP_FUNCTION_TABLE_INDEX_OFFSET: u64 = 32;
pub(crate) const HEAP_FUNCTION_ENV_HANDLE_OFFSET: u64 = 40;
pub(crate) const HEAP_FUNCTION_FLAGS_OFFSET: u64 = 48;
pub(crate) const HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET: u64 = 56;
pub(crate) const HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET: u64 = 64;
pub(crate) const HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET: u64 = 72;
pub(crate) const HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET: u64 = 80;
pub(crate) const HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET: u64 = 88;
pub(crate) const HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET: u64 = 96;
pub(crate) const HEAP_FUNCTION_REALM_FLOAT64_ARRAY_PROTOTYPE_OFFSET: u64 = 104;
pub(crate) const HEAP_FUNCTION_REALM_FLOAT32_ARRAY_PROTOTYPE_OFFSET: u64 = 112;
pub(crate) const HEAP_FUNCTION_REALM_INT32_ARRAY_PROTOTYPE_OFFSET: u64 = 120;
pub(crate) const HEAP_FUNCTION_REALM_INT16_ARRAY_PROTOTYPE_OFFSET: u64 = 128;
pub(crate) const HEAP_FUNCTION_REALM_INT8_ARRAY_PROTOTYPE_OFFSET: u64 = 136;
pub(crate) const HEAP_FUNCTION_REALM_UINT32_ARRAY_PROTOTYPE_OFFSET: u64 = 144;
pub(crate) const HEAP_FUNCTION_REALM_UINT16_ARRAY_PROTOTYPE_OFFSET: u64 = 152;
pub(crate) const HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET: u64 = 160;
pub(crate) const HEAP_FUNCTION_REALM_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET: u64 = 168;
pub(crate) const HEAP_FUNCTION_REALM_BIGINT64_ARRAY_PROTOTYPE_OFFSET: u64 = 176;
pub(crate) const HEAP_FUNCTION_REALM_BIGUINT64_ARRAY_PROTOTYPE_OFFSET: u64 = 184;
pub(crate) const HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET: u64 = 192;
pub(crate) const HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 200;
pub(crate) const HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET: u64 = 208;
pub(crate) const HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET: u64 = 216;
pub(crate) const HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET: u64 = 224;
pub(crate) const HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET: u64 = 232;
pub(crate) const HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET: u64 = 240;
pub(crate) const HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET: u64 = 248;
pub(crate) const HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET: u64 = 256;
pub(crate) const HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET: u64 = 264;
pub(crate) const HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_OBJECT_KEY_OFFSET: u64 = 0;
pub(crate) const HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET: u64 = 8;
pub(crate) const HEAP_OBJECT_DATA_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_OBJECT_DATA_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_OBJECT_GETTER_TAG_OFFSET: u64 = 32;
pub(crate) const HEAP_OBJECT_GETTER_PAYLOAD_OFFSET: u64 = 40;
pub(crate) const HEAP_OBJECT_SETTER_TAG_OFFSET: u64 = 48;
pub(crate) const HEAP_OBJECT_SETTER_PAYLOAD_OFFSET: u64 = 56;
pub(crate) const HEAP_ARRAY_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_ARRAY_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET: u64 = 16;
pub(crate) const HEAP_ARRAY_CONSTRUCTOR_TAG_OFFSET: u64 = 32;
pub(crate) const HEAP_ARRAY_CONSTRUCTOR_PAYLOAD_OFFSET: u64 = 40;
pub(crate) const HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET: u64 = 48;
pub(crate) const HEAP_ARRAY_CONSTRUCTOR_DESCRIPTOR_KIND_OFFSET: u64 = 56;
pub(crate) const HEAP_ARRAY_CONSTRUCTOR_GETTER_TAG_OFFSET: u64 = 64;
pub(crate) const HEAP_ARRAY_CONSTRUCTOR_GETTER_PAYLOAD_OFFSET: u64 = 72;
pub(crate) const HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET: u64 = 80;
pub(crate) const HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET: u64 = 88;
pub(crate) const HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET: u64 = 96;
pub(crate) const HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET: u64 = 104;
pub(crate) const HEAP_ARRAY_PROP_DATA_TAG_OFFSET: u64 = 112;
pub(crate) const HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET: u64 = 120;
pub(crate) const HEAP_ARRAY_PROP_GETTER_TAG_OFFSET: u64 = 128;
pub(crate) const HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET: u64 = 136;
pub(crate) const HEAP_ARRAY_PROP_SETTER_TAG_OFFSET: u64 = 144;
pub(crate) const HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET: u64 = 152;
pub(crate) const HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET: u64 = 160;
pub(crate) const HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET: u64 = 168;
pub(crate) const HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET: u64 = 176;
pub(crate) const HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET: u64 = 184;
pub(crate) const HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET: u64 = 192;
pub(crate) const HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET: u64 = 200;
pub(crate) const HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET: u64 = 208;
pub(crate) const HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET: u64 = 216;
pub(crate) const HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET: u64 = 224;
pub(crate) const HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET: u64 = 232;
pub(crate) const HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET: u64 = 240;
pub(crate) const HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET: u64 = 248;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_SIZE: u64 = 32;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET: u64 = 0;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET: u64 = 24;
pub(crate) const ENV_PARENT_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_BASE_OFFSET: u64 = 8;
pub(crate) const ENV_SLOT_SIZE: u64 = 16;
pub(crate) const ENV_SLOT_TAG_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const OBJECT_DESCRIPTOR_ACCESSOR: u64 = 1;
pub(crate) const OBJECT_DESCRIPTOR_CONFIGURABLE: u64 = 2;
pub(crate) const OBJECT_DESCRIPTOR_WRITABLE: u64 = 4;
pub(crate) const OBJECT_DESCRIPTOR_ENUMERABLE: u64 = 8;
pub(crate) const OBJECT_DESCRIPTOR_DATA: u64 = 0;
pub(crate) const ARRAY_DESCRIPTOR_OWN_PROPERTY: u64 = 16;
pub(crate) const ARRAY_DESCRIPTOR_NORMAL_DATA: u64 =
    OBJECT_DESCRIPTOR_CONFIGURABLE | OBJECT_DESCRIPTOR_WRITABLE | OBJECT_DESCRIPTOR_ENUMERABLE;
pub(crate) const BOXED_PRIMITIVE_KIND_NONE: u64 = 0;
pub(crate) const BOXED_PRIMITIVE_KIND_NUMBER: u64 = 1;
pub(crate) const BOXED_PRIMITIVE_KIND_STRING: u64 = 2;
pub(crate) const BOXED_PRIMITIVE_KIND_BOOLEAN: u64 = 3;
pub(crate) const BOXED_PRIMITIVE_KIND_BIGINT: u64 = 4;
pub(crate) const BOXED_PRIMITIVE_KIND_SYMBOL: u64 = 5;

pub(crate) const PROXY_HANDLER_PAYLOAD_MIN: u64 = BOXED_PRIMITIVE_KIND_SYMBOL + 1;
pub(crate) const OBJECT_INTERNAL_BRAND_ERROR: u64 = 1;
pub(crate) const OBJECT_INTERNAL_BRAND_RAW_JSON: u64 = 2;
pub(crate) const FUNCTION_FLAG_CONSTRUCTABLE: u64 = 1;
pub(crate) const FUNCTION_FLAG_CLASS_CONSTRUCTOR: u64 = 2;
pub(crate) const FUNCTION_FLAG_BOUND: u64 = 4;
pub(crate) const FUNCTION_FLAG_DERIVED_CONSTRUCTOR: u64 = 8;
pub(crate) const FUNCTION_FLAG_SYNTHETIC_DEFAULT_DERIVED_CONSTRUCTOR: u64 = 16;
pub(crate) const FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR: u64 = 32;
pub(crate) const FUNCTION_FLAG_USES_SUPER: u64 = 64;
pub(crate) const FUNCTION_FLAG_THIS_BEFORE_SUPER: u64 = 128;
pub(crate) const FUNCTION_FLAG_STRICT: u64 = 256;
pub(crate) const FUNCTION_FLAG_IS_HTMLDDA: u64 = 512;
pub(crate) const JS_FUNCTION_PARAM_COUNT: usize = 7;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_heap_alloc_const(
        &mut self,
        size: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let size_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(Self::align_heap_size(size) as i64));
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        self.release_temp_local(size_local);
        Ok(())
    }

    pub(crate) fn emit_heap_alloc_from_local(
        &mut self,
        size_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.uses_heap {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: heap value without memory",
            ));
        }
        let alloc_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(alloc_local));
        function.instruction(&Instruction::LocalGet(alloc_local));
        function.instruction(&Instruction::LocalGet(size_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const((WASM_PAGE_SIZE - 1) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryGrow(0));
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(alloc_local));
        self.release_temp_local(end_local);
        self.release_temp_local(alloc_local);
        Ok(())
    }

    pub(crate) const fn align_heap_size(size: u64) -> u64 {
        (size + 7) & !7
    }

    pub(crate) fn load_i64_to_local_from_offset(
        &self,
        base_local: u32,
        offset: u64,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_from_offset(base_local, offset, function);
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    pub(crate) fn load_i64_from_offset(
        &self,
        base_local: u32,
        offset: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(offset)));
    }

    pub(crate) fn store_i64_const_at_offset(
        &self,
        base_local: u32,
        offset: u64,
        value: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(value as i64));
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    pub(crate) fn store_i64_local_at_offset(
        &self,
        base_local: u32,
        offset: u64,
        value_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    pub(crate) const fn memarg64(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 3,
            memory_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_limits_are_stable() {
        assert_eq!(WASM_PAGE_SIZE, 65_536);
        assert_eq!(STATIC_DATA_OFFSET, 4096);
        assert_eq!(MIN_HEAP_CAPACITY, 1);
        assert_eq!(MAX_ARRAY_BUFFER_BYTE_LENGTH, 1 << 32);
        assert_eq!(MAX_DENSE_ARRAY_INDEX, 1_000_000);
        assert_eq!(MAX_SAFE_INTEGER, 9_007_199_254_740_991);
        assert_eq!(MAX_ARRAY_LENGTH, 4_294_967_295);
        assert_eq!(HEAP_ARRAY_HOLE_TAG, ValueKind::Dynamic.tag() as i64);
    }
}
