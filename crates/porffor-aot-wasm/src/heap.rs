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

pub(crate) fn emit_heap_alloc_helper_function() -> Function {
    const SIZE_LOCAL: u32 = 0;
    const ALLOC_LOCAL: u32 = 1;
    const END_LOCAL: u32 = 2;
    const ALIGNED_SIZE_LOCAL: u32 = 3;

    let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 3));

    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::I64Const(7));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(ALIGNED_SIZE_LOCAL));

    function.instruction(&Instruction::LocalGet(ALIGNED_SIZE_LOCAL));
    function.instruction(&Instruction::LocalGet(SIZE_LOCAL));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(ALIGNED_SIZE_LOCAL));
    function.instruction(&Instruction::I64Const(-8));
    function.instruction(&Instruction::I64And);
    function.instruction(&Instruction::LocalSet(ALIGNED_SIZE_LOCAL));

    function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalSet(ALLOC_LOCAL));
    function.instruction(&Instruction::LocalGet(ALLOC_LOCAL));
    function.instruction(&Instruction::LocalGet(ALIGNED_SIZE_LOCAL));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(END_LOCAL));

    function.instruction(&Instruction::LocalGet(END_LOCAL));
    function.instruction(&Instruction::LocalGet(ALLOC_LOCAL));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(END_LOCAL));
    function.instruction(&Instruction::MemorySize(0));
    function.instruction(&Instruction::I64ExtendI32U);
    function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(END_LOCAL));
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
    function.instruction(&Instruction::I32Const(-1));
    function.instruction(&Instruction::I32Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(END_LOCAL));
    function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
    function.instruction(&Instruction::LocalGet(ALLOC_LOCAL));
    function.instruction(&Instruction::End);
    function
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapLayoutSlot {
    pub record: &'static str,
    pub name: &'static str,
    pub offset: u64,
    pub width: u64,
    pub pointer: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapByteSpanLayout {
    pub record: &'static str,
    pub length_source: &'static str,
    pub element_width: u64,
    pub pointer: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapNamedSlot {
    pub record: &'static str,
    pub key: &'static str,
    pub strong_reference: bool,
    pub scans_target: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapRootSource {
    pub name: &'static str,
    pub owner: &'static str,
    pub tagged_values: bool,
    pub transient: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapWeakEdgeKind {
    EphemeronKey,
    EphemeronValue,
    WeakTarget,
    FinalizerHoldings,
    FinalizerToken,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapWeakEdgeSlot {
    pub record: &'static str,
    pub name: &'static str,
    pub kind: HeapWeakEdgeKind,
    pub keeps_target_alive: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapCollectorPhaseKind {
    StopTheWorld,
    RootScan,
    MarkStrong,
    ProcessEphemerons,
    ClearWeakRefs,
    QueueFinalizers,
    Sweep,
    Resume,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapCollectorPhase {
    pub name: &'static str,
    pub kind: HeapCollectorPhaseKind,
    pub required_for_gc_builtin: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapCollectorCapability {
    DocumentedOnly,
    MetadataChecked,
    Executable,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapCollectorContract {
    pub name: &'static str,
    pub moving: bool,
    pub capability: HeapCollectorCapability,
    pub root_sources: &'static [HeapRootSource],
    pub weak_edges: &'static [HeapWeakEdgeSlot],
    pub phases: &'static [HeapCollectorPhase],
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostMemoryBorrowDuration {
    ImportCallOnly,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapHostBoundaryContract {
    pub name: &'static str,
    pub durable_host_pointers: bool,
    pub memory_borrow_duration: HostMemoryBorrowDuration,
    pub borrowed_root_source: &'static str,
    pub reentrant_imports_require_transient_roots: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValuePayloadEncoding {
    Immediate,
    BooleanBit,
    Ieee754Bits,
    HeapPointer,
    StaticOrHeapPointer,
    I64Temporary,
    I64TemporaryOrHeapPointer,
    DynamicTaggedPair,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueEncodingSlot {
    pub kind: ValueKind,
    pub payload: ValuePayloadEncoding,
    pub preserves_number_bits: bool,
    pub arbitrary_precision_ready: bool,
}

#[allow(dead_code)]
impl HeapLayoutSlot {
    pub(crate) const fn end(self) -> u64 {
        self.offset + self.width
    }
}

pub(crate) const HEAP_HEADER_SIZE: u64 = 256;
pub(crate) const HEAP_FUNCTION_OBJECT_SIZE: u64 = 304;
pub(crate) const HEAP_OBJECT_ENTRY_SIZE: u64 = 64;
pub(crate) const HEAP_REALM_RECORD_SIZE: u64 = 72;
pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 208;
pub(crate) const HEAP_ARRAY_ENTRY_SIZE: u64 = 40;
// Array offsets intentionally retain padding at boxed-object metadata positions:
// some generic object paths can still receive an Array pointer after tag erasure.
pub(crate) const HEAP_ARRAY_RECORD_SIZE: u64 = 272;
#[allow(dead_code)]
pub(crate) const HEAP_STRING_RECORD_SIZE: u64 = 32;
#[allow(dead_code)]
pub(crate) const HEAP_BIGINT_RECORD_SIZE: u64 = 32;
#[allow(dead_code)]
pub(crate) const HEAP_SYMBOL_RECORD_SIZE: u64 = 32;
#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_RECORD_SIZE: u64 = 64;
#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_REACTION_RECORD_SIZE: u64 = 64;
#[allow(dead_code)]
pub(crate) const HEAP_PENDING_JOB_RECORD_SIZE: u64 = 56;
pub(crate) const SPARSE_ARRAY_DENSE_GROW_FACTOR: u64 = 16;
pub(crate) const HEAP_BOUND_FUNCTION_RECORD_SIZE: u64 = 48;
// Arguments records reuse the generic array header (ptr/len/cap/prototype at
// 0/8/16/24) and are also inspected by generic object paths (e.g.
// `Object.prototype.toString`) that read the boxed-object cluster at
// 32/40/48 and the internal brand / prototype-tag / proxy fields up to 72.
// The arguments-specific fields therefore live past that cluster so mapped
// arguments objects are not misclassified as boxed primitives.
pub(crate) const HEAP_ARGUMENTS_IS_CONCAT_SPREADABLE_OFFSET: u64 = 48;
pub(crate) const HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET: u64 = 80;
pub(crate) const HEAP_ARGUMENTS_ENV_HANDLE_OFFSET: u64 = 88;
// The array-header length tracks the indexed backing extent independently of
// the observable, configurable `arguments.length` value. Keep its descriptor
// and accessor override out of line from that header as well.
pub(crate) const HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET: u64 = 96;
pub(crate) const HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET: u64 = 104;
pub(crate) const HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET: u64 = 112;
pub(crate) const HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET: u64 = 120;
pub(crate) const HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET: u64 = 128;
pub(crate) const HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET: u64 = 136;
pub(crate) const HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET: u64 = 144;
pub(crate) const HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET: u64 = 152;
pub(crate) const HEAP_ARGUMENTS_RECORD_SIZE: u64 = 160;
pub(crate) const HEAP_OBJECT_BOXED_KIND_OFFSET: u64 = 32;
pub(crate) const HEAP_OBJECT_BOXED_TAG_OFFSET: u64 = 40;
pub(crate) const HEAP_OBJECT_BOXED_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_OBJECT_INTERNAL_BRAND_OFFSET: u64 = 56;
pub(crate) const HEAP_OBJECT_PROTOTYPE_TAG_OFFSET: u64 = 64;
pub(crate) const HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 72;
// TypedArray instances are ordinary heap objects with integer-indexed exotic
// behavior.  Their internal slots must not alias user-visible properties: JS
// can legally create or overwrite names such as `$TypedArrayByteLength`.
pub(crate) const HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET: u64 = 80;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_OFFSET: u64 = 88;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET: u64 = 96;
pub(crate) const HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET: u64 = 104;
pub(crate) const HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET: u64 = 112;
pub(crate) const HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET: u64 = 120;
pub(crate) const HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET: u64 = 128;
pub(crate) const HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET: u64 = 136;
/// Absolute linear-memory address of an immutable, AOT-compiled RegExp program.
/// Zero means that the object has no attached program and must use the dynamic
/// construction/migration path.
pub(crate) const HEAP_REGEXP_PROGRAM_PTR_OFFSET: u64 = 144;
/// Number of fixed-width instructions in the compiled RegExp program.
pub(crate) const HEAP_REGEXP_PROGRAM_INSTRUCTION_COUNT_OFFSET: u64 = 152;
/// Number of numbered captures in the immutable AOT-compiled RegExp program.
pub(crate) const HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET: u64 = 160;
/// Number of `Split` instructions in the immutable AOT-compiled program.
pub(crate) const HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET: u64 = 168;
/// Number of `Split` instructions that belong to a control-flow cycle.
pub(crate) const HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET: u64 = 176;
/// Absolute linear-memory address of immutable named-capture metadata for the
/// compiled RegExp program. Zero means that no named-group table is attached.
pub(crate) const HEAP_REGEXP_NAMED_GROUP_TABLE_PTR_OFFSET: u64 = 184;
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
pub(crate) const HEAP_FUNCTION_DEFINING_REALM_OFFSET: u64 = 272;
pub(crate) const HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET: u64 = 280;
pub(crate) const HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET: u64 = 288;
pub(crate) const HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET: u64 = 296;
// Function objects that need execution context state own this immutable
// context through `HEAP_FUNCTION_ENV_HANDLE_OFFSET`. It separates lexical
// scope from a member's [[HomeObject]]; in particular, `super` must not derive
// its base from the receiver supplied by an eventual detached call.
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET: u64 = 0;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET: u64 = 8;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET: u64 = 32;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET: u64 = 40;
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_SIZE: u64 = 48;
pub(crate) const HEAP_PRIVATE_ENV_PARENT_OFFSET: u64 = 0;
pub(crate) const HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET: u64 = 8;
pub(crate) const HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET: u64 = 16;
pub(crate) const HEAP_PRIVATE_ENV_SLOT_SIZE: u64 = 8;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET: u64 = 0;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET: u64 = 8;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET: u64 = 16;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET: u64 = 24;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET: u64 = 32;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 40;
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_SIZE: u64 = 48;
pub(crate) const PRIVATE_ELEMENT_KIND_BRAND: u64 = 0;
pub(crate) const PRIVATE_ELEMENT_KIND_FIELD: u64 = 1;
pub(crate) const PRIVATE_ELEMENT_KIND_SETTER: u64 = 2;
pub(crate) const PRIVATE_ELEMENT_KIND_METHOD: u64 = 3;
pub(crate) const PRIVATE_ELEMENT_KIND_GETTER: u64 = 4;
#[allow(dead_code)]
pub(crate) const HEAP_PRIVATE_ENV_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "private-environment",
        name: "parent",
        offset: HEAP_PRIVATE_ENV_PARENT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "private-environment",
        name: "class_scope",
        offset: HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
        width: 8,
        pointer: false,
    },
];
#[allow(dead_code)]
pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "next",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "receiver",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "token",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "kind",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "value_tag",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "private-element-entry",
        name: "value_payload",
        offset: HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];
pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "class-function-context",
        name: "lexical_env",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "class-function-context",
        name: "active_function",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "class-function-context",
        name: "home_object_payload",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "class-function-context",
        name: "home_object_tag",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "class-function-context",
        name: "field_keys",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "class-function-context",
        name: "private_environment",
        offset: HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
];
pub(crate) const HEAP_REALM_ID_OFFSET: u64 = 0;
pub(crate) const HEAP_REALM_AGENT_ID_OFFSET: u64 = 8;
pub(crate) const HEAP_REALM_GLOBAL_OBJECT_OFFSET: u64 = 16;
pub(crate) const HEAP_REALM_GLOBAL_THIS_OFFSET: u64 = 24;
pub(crate) const HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET: u64 = 32;
pub(crate) const HEAP_REALM_INTRINSICS_OFFSET: u64 = 40;
pub(crate) const HEAP_REALM_HOST_HOOKS_OFFSET: u64 = 48;
pub(crate) const HEAP_REALM_MODULE_REGISTRY_OFFSET: u64 = 56;
pub(crate) const HEAP_REALM_PRIVATE_ELEMENTS_OFFSET: u64 = 64;
pub(crate) const HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 0;
pub(crate) const HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET: u64 = 8;
pub(crate) const HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET: u64 = 16;
pub(crate) const HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET: u64 = 24;
pub(crate) const HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET: u64 = 32;
pub(crate) const HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET: u64 = 40;
pub(crate) const HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET: u64 = 48;
pub(crate) const HEAP_REALM_INTRINSICS_FLOAT64_ARRAY_PROTOTYPE_OFFSET: u64 = 56;
pub(crate) const HEAP_REALM_INTRINSICS_FLOAT32_ARRAY_PROTOTYPE_OFFSET: u64 = 64;
pub(crate) const HEAP_REALM_INTRINSICS_INT32_ARRAY_PROTOTYPE_OFFSET: u64 = 72;
pub(crate) const HEAP_REALM_INTRINSICS_INT16_ARRAY_PROTOTYPE_OFFSET: u64 = 80;
pub(crate) const HEAP_REALM_INTRINSICS_INT8_ARRAY_PROTOTYPE_OFFSET: u64 = 88;
pub(crate) const HEAP_REALM_INTRINSICS_UINT32_ARRAY_PROTOTYPE_OFFSET: u64 = 96;
pub(crate) const HEAP_REALM_INTRINSICS_UINT16_ARRAY_PROTOTYPE_OFFSET: u64 = 104;
pub(crate) const HEAP_REALM_INTRINSICS_UINT8_ARRAY_PROTOTYPE_OFFSET: u64 = 112;
pub(crate) const HEAP_REALM_INTRINSICS_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET: u64 = 120;
pub(crate) const HEAP_REALM_INTRINSICS_BIGINT64_ARRAY_PROTOTYPE_OFFSET: u64 = 128;
pub(crate) const HEAP_REALM_INTRINSICS_BIGUINT64_ARRAY_PROTOTYPE_OFFSET: u64 = 136;
pub(crate) const HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET: u64 = 144;
pub(crate) const HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET: u64 = 152;
pub(crate) const HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET: u64 = 160;
pub(crate) const HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET: u64 = 168;
pub(crate) const HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET: u64 = 176;
pub(crate) const HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET: u64 = 184;
pub(crate) const HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET: u64 = 192;
pub(crate) const HEAP_REALM_INTRINSICS_STRING_ITERATOR_PROTOTYPE_OFFSET: u64 = 200;
pub(crate) const HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET: u64 = 32;
// The bound function's own payload is retained so [[Construct]] can apply
// BoundFunctionExoticObject's one-level `SameValue(F, newTarget)` rewrite.
pub(crate) const HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET: u64 = 40;
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
pub(crate) const HEAP_ARRAY_SETTER_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_ARRAY_SETTER_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET: u64 = 48;
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
pub(crate) const HEAP_ARRAY_PROTOTYPE_TAG_OFFSET: u64 = 256;
pub(crate) const HEAP_ARRAY_NON_EXTENSIBLE_OFFSET: u64 = 264;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_SIZE: u64 = 48;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET: u64 = 0;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_SETTER_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_SETTER_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET: u64 = 40;
pub(crate) const HEAP_STRING_CODE_UNITS_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_STRING_BYTE_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_STRING_CODE_UNIT_LEN_OFFSET: u64 = 16;
pub(crate) const HEAP_STRING_INTERN_ID_OFFSET: u64 = 24;
pub(crate) const HEAP_BIGINT_SIGN_OFFSET: u64 = 0;
pub(crate) const HEAP_BIGINT_LIMBS_PTR_OFFSET: u64 = 8;
pub(crate) const HEAP_BIGINT_LIMBS_LEN_OFFSET: u64 = 16;
pub(crate) const HEAP_BIGINT_LIMBS_CAP_OFFSET: u64 = 24;
pub(crate) const HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_SYMBOL_ID_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_STATE_OFFSET: u64 = 0;
pub(crate) const HEAP_PROMISE_RESULT_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_PROMISE_RESULT_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_PROMISE_FULFILL_REACTIONS_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_REJECT_REACTIONS_OFFSET: u64 = 32;
pub(crate) const HEAP_PROMISE_IS_HANDLED_OFFSET: u64 = 40;
pub(crate) const HEAP_PROMISE_REALM_OFFSET: u64 = 48;
pub(crate) const HEAP_PROMISE_HOST_DATA_OFFSET: u64 = 56;
pub(crate) const HEAP_PROMISE_REACTION_CAPABILITY_OFFSET: u64 = 0;
pub(crate) const HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_PROMISE_REACTION_REALM_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_REACTION_NEXT_OFFSET: u64 = 32;
pub(crate) const HEAP_PROMISE_REACTION_TYPE_OFFSET: u64 = 40;
pub(crate) const HEAP_PROMISE_REACTION_JOB_CALLBACK_OFFSET: u64 = 48;
pub(crate) const HEAP_PROMISE_REACTION_RESERVED_OFFSET: u64 = 56;
pub(crate) const HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_PENDING_JOB_ARG_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_PENDING_JOB_REALM_OFFSET: u64 = 32;
pub(crate) const HEAP_PENDING_JOB_NEXT_OFFSET: u64 = 40;
pub(crate) const HEAP_PENDING_JOB_KIND_OFFSET: u64 = 48;
pub(crate) const ENV_PARENT_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_BASE_OFFSET: u64 = 8;
pub(crate) const ENV_SLOT_SIZE: u64 = 16;
pub(crate) const ENV_SLOT_TAG_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const ENV_SLOT_UNINITIALIZED_TAG: i64 = -1;
pub(crate) const OBJECT_DESCRIPTOR_ACCESSOR: u64 = 1;
pub(crate) const OBJECT_DESCRIPTOR_CONFIGURABLE: u64 = 2;
pub(crate) const OBJECT_DESCRIPTOR_WRITABLE: u64 = 4;
pub(crate) const OBJECT_DESCRIPTOR_ENUMERABLE: u64 = 8;
pub(crate) const OBJECT_DESCRIPTOR_DATA: u64 = 0;
pub(crate) const ARRAY_DESCRIPTOR_OWN_PROPERTY: u64 = 16;
pub(crate) const ARGUMENTS_DESCRIPTOR_MAPPED: u64 = 32;
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
pub(crate) const OBJECT_INTERNAL_BRAND_TYPED_ARRAY: u64 = 3;
pub(crate) const OBJECT_INTERNAL_BRAND_REGEXP: u64 = 4;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_ZIP_HELPER: u64 = 5;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_MAP_HELPER: u64 = 6;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_FILTER_HELPER: u64 = 7;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_FLAT_MAP_HELPER: u64 = 8;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_TAKE_HELPER: u64 = 9;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_DROP_HELPER: u64 = 10;
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

#[allow(dead_code)]
pub(crate) const HEAP_OBJECT_HEADER_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "object-header",
        name: "elements_ptr",
        offset: HEAP_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "elements_len",
        offset: HEAP_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "elements_cap",
        offset: HEAP_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "prototype_payload",
        offset: HEAP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "boxed_kind",
        offset: HEAP_OBJECT_BOXED_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "boxed_tag",
        offset: HEAP_OBJECT_BOXED_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "boxed_payload",
        offset: HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "internal_brand",
        offset: HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "prototype_tag",
        offset: HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-header",
        name: "proxy_type_error_prototype",
        offset: HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "viewed_array_buffer",
        offset: HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "byte_offset",
        offset: HEAP_TYPED_ARRAY_BYTE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "byte_length",
        offset: HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "bytes_per_element",
        offset: HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "element_kind",
        offset: HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-object-header",
        name: "length_tracking",
        offset: HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "original_source_payload",
        offset: HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "original_flags_payload",
        offset: HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "program_ptr",
        offset: HEAP_REGEXP_PROGRAM_PTR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "program_instruction_count",
        offset: HEAP_REGEXP_PROGRAM_INSTRUCTION_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "program_capture_count",
        offset: HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "program_split_count",
        offset: HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "program_repeatable_split_count",
        offset: HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "regexp-object-header",
        name: "named_group_table_ptr",
        offset: HEAP_REGEXP_NAMED_GROUP_TABLE_PTR_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_FUNCTION_OBJECT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "function-object",
        name: "elements_ptr",
        offset: HEAP_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "elements_len",
        offset: HEAP_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "elements_cap",
        offset: HEAP_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "prototype_payload",
        offset: HEAP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "table_index",
        offset: HEAP_FUNCTION_TABLE_INDEX_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "env_handle",
        offset: HEAP_FUNCTION_ENV_HANDLE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "flags",
        offset: HEAP_FUNCTION_FLAGS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "prototype_tag",
        offset: HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "prototype_payload",
        offset: HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "to_string_payload",
        offset: HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_array_buffer_prototype",
        offset: HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_data_view_prototype",
        offset: HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_aggregate_error_prototype",
        offset: HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_float64_array_prototype",
        offset: HEAP_FUNCTION_REALM_FLOAT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_float32_array_prototype",
        offset: HEAP_FUNCTION_REALM_FLOAT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_int32_array_prototype",
        offset: HEAP_FUNCTION_REALM_INT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_int16_array_prototype",
        offset: HEAP_FUNCTION_REALM_INT16_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_int8_array_prototype",
        offset: HEAP_FUNCTION_REALM_INT8_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_uint32_array_prototype",
        offset: HEAP_FUNCTION_REALM_UINT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_uint16_array_prototype",
        offset: HEAP_FUNCTION_REALM_UINT16_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_uint8_array_prototype",
        offset: HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_uint8_clamped_array_prototype",
        offset: HEAP_FUNCTION_REALM_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_bigint64_array_prototype",
        offset: HEAP_FUNCTION_REALM_BIGINT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_biguint64_array_prototype",
        offset: HEAP_FUNCTION_REALM_BIGUINT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_number_prototype",
        offset: HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_type_error_prototype",
        offset: HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_error_prototype",
        offset: HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_eval_error_prototype",
        offset: HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_range_error_prototype",
        offset: HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_reference_error_prototype",
        offset: HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_syntax_error_prototype",
        offset: HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_uri_error_prototype",
        offset: HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_suppressed_error_prototype",
        offset: HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "realm_boolean_prototype",
        offset: HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "defining_realm",
        offset: HEAP_FUNCTION_DEFINING_REALM_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "internal_prototype_tag",
        offset: HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "typed_array_bytes_per_element",
        offset: HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "function-object",
        name: "typed_array_element_kind",
        offset: HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_REALM_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "realm-record",
        name: "realm_id",
        offset: HEAP_REALM_ID_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "agent_id",
        offset: HEAP_REALM_AGENT_ID_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "global_object",
        offset: HEAP_REALM_GLOBAL_OBJECT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "global_this",
        offset: HEAP_REALM_GLOBAL_THIS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "global_environment",
        offset: HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "intrinsics",
        offset: HEAP_REALM_INTRINSICS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "host_hooks",
        offset: HEAP_REALM_HOST_HOOKS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "module_registry",
        offset: HEAP_REALM_MODULE_REGISTRY_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-record",
        name: "private_elements",
        offset: HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_REALM_INTRINSICS_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "TypeError.prototype",
        offset: HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%ArrayIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Object.prototype%",
        offset: HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%String.prototype%",
        offset: HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Number.prototype%",
        offset: HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Boolean.prototype%",
        offset: HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Float64Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_FLOAT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Float32Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_FLOAT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Int32Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_INT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Int16Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_INT16_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Int8Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_INT8_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Uint32Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_UINT32_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Uint16Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_UINT16_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Uint8Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_UINT8_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Uint8ClampedArray.prototype%",
        offset: HEAP_REALM_INTRINSICS_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%BigInt64Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_BIGINT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%BigUint64Array.prototype%",
        offset: HEAP_REALM_INTRINSICS_BIGUINT64_ARRAY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Symbol.prototype%",
        offset: HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%BigInt.prototype%",
        offset: HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%IteratorHelperPrototype%",
        offset: HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Iterator.prototype%",
        offset: HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%WrapForValidIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%ThrowTypeError%",
        offset: HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%RegExp.prototype%",
        offset: HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%StringIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_STRING_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_BOUND_FUNCTION_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "bound-function",
        name: "target_payload",
        offset: HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "bound-function",
        name: "target_tag",
        offset: HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "bound-function",
        name: "this_payload",
        offset: HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "bound-function",
        name: "this_tag",
        offset: HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "bound-function",
        name: "args_payload",
        offset: HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "bound-function",
        name: "self_payload",
        offset: HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_OBJECT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "array-object",
        name: "elements_ptr",
        offset: HEAP_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "elements_len",
        offset: HEAP_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "elements_cap",
        offset: HEAP_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "prototype_payload",
        offset: HEAP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "prototype_tag",
        offset: HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "is_concat_spreadable",
        offset: HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "concat_spreadable_descriptor_kind",
        offset: HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "concat_spreadable_getter_tag",
        offset: HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "concat_spreadable_getter_payload",
        offset: HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_descriptor_kind",
        offset: HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_data_tag",
        offset: HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_data_payload",
        offset: HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_getter_tag",
        offset: HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_getter_payload",
        offset: HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_setter_tag",
        offset: HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_groups_setter_payload",
        offset: HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_index_descriptor_kind",
        offset: HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_index_data_tag",
        offset: HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_index_data_payload",
        offset: HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_input_descriptor_kind",
        offset: HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_input_data_tag",
        offset: HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "regexp_input_data_payload",
        offset: HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "present_indexes_ptr",
        offset: HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "present_indexes_len",
        offset: HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "present_indexes_cap",
        offset: HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "named_props_ptr",
        offset: HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "named_props_len",
        offset: HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "named_props_cap",
        offset: HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-object",
        name: "non_extensible",
        offset: HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_OBJECT_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "object-entry",
        name: "key",
        offset: HEAP_OBJECT_KEY_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "descriptor_kind",
        offset: HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "data_tag",
        offset: HEAP_OBJECT_DATA_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "data_payload",
        offset: HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "getter_tag",
        offset: HEAP_OBJECT_GETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "getter_payload",
        offset: HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "setter_tag",
        offset: HEAP_OBJECT_SETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "object-entry",
        name: "setter_payload",
        offset: HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "array-entry",
        name: "tag",
        offset: HEAP_ARRAY_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-entry",
        name: "payload",
        offset: HEAP_ARRAY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-entry",
        name: "descriptor_kind",
        offset: HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-entry",
        name: "setter_tag",
        offset: HEAP_ARRAY_SETTER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-entry",
        name: "setter_payload",
        offset: HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ENVIRONMENT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "environment",
        name: "parent",
        offset: ENV_PARENT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "environment-slot",
        name: "tag",
        offset: ENV_SLOT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "environment-slot",
        name: "payload",
        offset: ENV_SLOT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_STRING_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "string-record",
        name: "code_units_ptr",
        offset: HEAP_STRING_CODE_UNITS_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "string-record",
        name: "byte_len",
        offset: HEAP_STRING_BYTE_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "string-record",
        name: "code_unit_len",
        offset: HEAP_STRING_CODE_UNIT_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "string-record",
        name: "intern_id",
        offset: HEAP_STRING_INTERN_ID_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_BIGINT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "bigint-record",
        name: "sign",
        offset: HEAP_BIGINT_SIGN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "bigint-record",
        name: "limbs_ptr",
        offset: HEAP_BIGINT_LIMBS_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "bigint-record",
        name: "limbs_len",
        offset: HEAP_BIGINT_LIMBS_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "bigint-record",
        name: "limbs_cap",
        offset: HEAP_BIGINT_LIMBS_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_SYMBOL_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "symbol-record",
        name: "description_tag",
        offset: HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "symbol-record",
        name: "description_payload",
        offset: HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "symbol-record",
        name: "registry_key_payload",
        offset: HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "symbol-record",
        name: "symbol_id",
        offset: HEAP_SYMBOL_ID_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "promise-record",
        name: "state",
        offset: HEAP_PROMISE_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "result_tag",
        offset: HEAP_PROMISE_RESULT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "result_payload",
        offset: HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "fulfill_reactions",
        offset: HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "reject_reactions",
        offset: HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "is_handled",
        offset: HEAP_PROMISE_IS_HANDLED_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "realm",
        offset: HEAP_PROMISE_REALM_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-record",
        name: "host_data",
        offset: HEAP_PROMISE_HOST_DATA_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_REACTION_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "capability",
        offset: HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "handler_tag",
        offset: HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "handler_payload",
        offset: HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "realm",
        offset: HEAP_PROMISE_REACTION_REALM_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "next",
        offset: HEAP_PROMISE_REACTION_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "type",
        offset: HEAP_PROMISE_REACTION_TYPE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "job_callback",
        offset: HEAP_PROMISE_REACTION_JOB_CALLBACK_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-reaction-record",
        name: "reserved",
        offset: HEAP_PROMISE_REACTION_RESERVED_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_PENDING_JOB_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "callback_tag",
        offset: HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "callback_payload",
        offset: HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "arg_tag",
        offset: HEAP_PENDING_JOB_ARG_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "arg_payload",
        offset: HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "realm",
        offset: HEAP_PENDING_JOB_REALM_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "next",
        offset: HEAP_PENDING_JOB_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-job-record",
        name: "kind",
        offset: HEAP_PENDING_JOB_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_BUFFER_BACKING_STORE_LAYOUT: HeapByteSpanLayout = HeapByteSpanLayout {
    record: "array-buffer-backing-store",
    length_source: ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT,
    element_width: 1,
    pointer: false,
};

#[allow(dead_code)]
pub(crate) const HEAP_STRING_CODE_UNITS_LAYOUT: HeapByteSpanLayout = HeapByteSpanLayout {
    record: "string-code-units",
    length_source: "string-record.code_unit_len",
    element_width: 2,
    pointer: false,
};

#[allow(dead_code)]
pub(crate) const HEAP_BIGINT_LIMBS_LAYOUT: HeapByteSpanLayout = HeapByteSpanLayout {
    record: "bigint-limbs",
    length_source: "bigint-record.limbs_len",
    element_width: 8,
    pointer: false,
};

#[allow(dead_code)]
pub(crate) const HEAP_RAW_BYTE_SPAN_LAYOUTS: &[HeapByteSpanLayout] = &[
    HEAP_ARRAY_BUFFER_BACKING_STORE_LAYOUT,
    HEAP_STRING_CODE_UNITS_LAYOUT,
    HEAP_BIGINT_LIMBS_LAYOUT,
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_BUFFER_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "array-buffer-object",
        key: ARRAY_BUFFER_DATA_PTR_SLOT,
        strong_reference: true,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-buffer-object",
        key: ARRAY_BUFFER_BYTE_LENGTH_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-buffer-object",
        key: ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-buffer-object",
        key: ARRAY_BUFFER_SHARED_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-buffer-object",
        key: ARRAY_BUFFER_IMMUTABLE_SLOT,
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_DATA_VIEW_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "data-view-object",
        key: "buffer",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "data-view-object",
        key: DATA_VIEW_DATA_PTR_SLOT,
        strong_reference: true,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "data-view-object",
        key: DATA_VIEW_BYTE_OFFSET_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "data-view-object",
        key: DATA_VIEW_BYTE_LENGTH_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "data-view-object",
        key: DATA_VIEW_LENGTH_TRACKING_SLOT,
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TYPED_ARRAY_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT,
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_BYTE_OFFSET_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_BYTE_LENGTH_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_ELEMENT_KIND_SLOT,
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "typed-array-object",
        key: TYPED_ARRAY_LENGTH_TRACKING_SLOT,
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.array",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.index",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.done",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.kind",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_STRING_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "string-iterator-object",
        key: "$StringIterator.string",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "string-iterator-object",
        key: "$StringIterator.index",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.regexp",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.string",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.global",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.unicode",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.done",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ITERATOR_HELPER_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFromIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFromNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapMapper",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterPredicate",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapMapper",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapInnerIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapInnerNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropDone",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipIterators",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipNextMethods",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipOpen",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipPadding",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipMode",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipExecuting",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipStarted",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_NAMED_SLOT_LAYOUTS: &[&[HeapNamedSlot]] = &[
    HEAP_ARRAY_BUFFER_NAMED_SLOTS,
    HEAP_DATA_VIEW_NAMED_SLOTS,
    HEAP_TYPED_ARRAY_NAMED_SLOTS,
    HEAP_ARRAY_ITERATOR_NAMED_SLOTS,
    HEAP_STRING_ITERATOR_NAMED_SLOTS,
    HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS,
    HEAP_ITERATOR_HELPER_NAMED_SLOTS,
    HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS,
];

#[allow(dead_code)]
pub(crate) const HEAP_ROOT_SOURCES: &[HeapRootSource] = &[
    HeapRootSource {
        name: "realm-globals",
        owner: "module-globals",
        tagged_values: false,
        transient: false,
    },
    HeapRootSource {
        name: "active-frame-locals",
        owner: "function-locals",
        tagged_values: true,
        transient: true,
    },
    HeapRootSource {
        name: "lexical-environments",
        owner: "environment-chain",
        tagged_values: true,
        transient: false,
    },
    HeapRootSource {
        name: "completion-records",
        owner: "completion-abi",
        tagged_values: true,
        transient: true,
    },
    HeapRootSource {
        name: "function-table",
        owner: "indirect-call-table",
        tagged_values: false,
        transient: false,
    },
    HeapRootSource {
        name: "host-borrowed-values",
        owner: "host-import-boundary",
        tagged_values: true,
        transient: true,
    },
    HeapRootSource {
        name: "pending-jobs",
        owner: "job-queue",
        tagged_values: true,
        transient: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_EDGE_SLOTS: &[HeapWeakEdgeSlot] = &[
    HeapWeakEdgeSlot {
        record: "weak-map-entry",
        name: "key",
        kind: HeapWeakEdgeKind::EphemeronKey,
        keeps_target_alive: false,
    },
    HeapWeakEdgeSlot {
        record: "weak-map-entry",
        name: "value",
        kind: HeapWeakEdgeKind::EphemeronValue,
        keeps_target_alive: false,
    },
    HeapWeakEdgeSlot {
        record: "weak-set-entry",
        name: "key",
        kind: HeapWeakEdgeKind::EphemeronKey,
        keeps_target_alive: false,
    },
    HeapWeakEdgeSlot {
        record: "weak-ref-record",
        name: "target",
        kind: HeapWeakEdgeKind::WeakTarget,
        keeps_target_alive: false,
    },
    HeapWeakEdgeSlot {
        record: "finalization-registry-cell",
        name: "target",
        kind: HeapWeakEdgeKind::WeakTarget,
        keeps_target_alive: false,
    },
    HeapWeakEdgeSlot {
        record: "finalization-registry-cell",
        name: "holdings",
        kind: HeapWeakEdgeKind::FinalizerHoldings,
        keeps_target_alive: true,
    },
    HeapWeakEdgeSlot {
        record: "finalization-registry-cell",
        name: "unregister-token",
        kind: HeapWeakEdgeKind::FinalizerToken,
        keeps_target_alive: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_COLLECTOR_PHASES: &[HeapCollectorPhase] = &[
    HeapCollectorPhase {
        name: "stop-the-world",
        kind: HeapCollectorPhaseKind::StopTheWorld,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "scan-roots",
        kind: HeapCollectorPhaseKind::RootScan,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "mark-strong-graph",
        kind: HeapCollectorPhaseKind::MarkStrong,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "process-ephemerons",
        kind: HeapCollectorPhaseKind::ProcessEphemerons,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "clear-weakrefs",
        kind: HeapCollectorPhaseKind::ClearWeakRefs,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "queue-finalizers",
        kind: HeapCollectorPhaseKind::QueueFinalizers,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "sweep-unmarked",
        kind: HeapCollectorPhaseKind::Sweep,
        required_for_gc_builtin: true,
    },
    HeapCollectorPhase {
        name: "resume",
        kind: HeapCollectorPhaseKind::Resume,
        required_for_gc_builtin: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_COLLECTOR_CONTRACT: HeapCollectorContract = HeapCollectorContract {
    name: "non-moving-tracing-collector",
    moving: false,
    capability: HeapCollectorCapability::MetadataChecked,
    root_sources: HEAP_ROOT_SOURCES,
    weak_edges: HEAP_WEAK_EDGE_SLOTS,
    phases: HEAP_COLLECTOR_PHASES,
};

#[allow(dead_code)]
pub(crate) const HEAP_HOST_BOUNDARY_CONTRACT: HeapHostBoundaryContract = HeapHostBoundaryContract {
    name: "host-import-memory-borrow",
    durable_host_pointers: false,
    memory_borrow_duration: HostMemoryBorrowDuration::ImportCallOnly,
    borrowed_root_source: "host-borrowed-values",
    reentrant_imports_require_transient_roots: true,
};

#[allow(dead_code)]
pub(crate) const fn heap_collector_is_executable() -> bool {
    matches!(
        HEAP_COLLECTOR_CONTRACT.capability,
        HeapCollectorCapability::Executable
    )
}

#[allow(dead_code)]
pub(crate) const VALUE_ENCODING_SLOTS: &[ValueEncodingSlot] = &[
    ValueEncodingSlot {
        kind: ValueKind::Undefined,
        payload: ValuePayloadEncoding::Immediate,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Null,
        payload: ValuePayloadEncoding::Immediate,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Boolean,
        payload: ValuePayloadEncoding::BooleanBit,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Number,
        payload: ValuePayloadEncoding::Ieee754Bits,
        preserves_number_bits: true,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::String,
        payload: ValuePayloadEncoding::StaticOrHeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Symbol,
        payload: ValuePayloadEncoding::StaticOrHeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Object,
        payload: ValuePayloadEncoding::HeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Array,
        payload: ValuePayloadEncoding::HeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Function,
        payload: ValuePayloadEncoding::HeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::Arguments,
        payload: ValuePayloadEncoding::HeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
    ValueEncodingSlot {
        kind: ValueKind::BigInt,
        payload: ValuePayloadEncoding::I64TemporaryOrHeapPointer,
        preserves_number_bits: false,
        arbitrary_precision_ready: false,
    },
    ValueEncodingSlot {
        kind: ValueKind::Dynamic,
        payload: ValuePayloadEncoding::DynamicTaggedPair,
        preserves_number_bits: false,
        arbitrary_precision_ready: true,
    },
];

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
        if let Some(heap_alloc_function_index) = self.heap_alloc_function_index {
            function.instruction(&Instruction::LocalGet(size_local));
            function.instruction(&Instruction::Call(heap_alloc_function_index));
            return Ok(());
        }
        let alloc_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let aligned_size_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(size_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(aligned_size_local));

        function.instruction(&Instruction::LocalGet(aligned_size_local));
        function.instruction(&Instruction::LocalGet(size_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(aligned_size_local));
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(aligned_size_local));

        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(alloc_local));
        function.instruction(&Instruction::LocalGet(alloc_local));
        function.instruction(&Instruction::LocalGet(aligned_size_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(alloc_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

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
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(alloc_local));
        self.release_temp_local(aligned_size_local);
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
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::lower;
    use std::collections::BTreeSet;
    use wasmparser::{Operator, Parser, Payload};

    fn assert_layout(layout: &[HeapLayoutSlot], record_size: u64) {
        let mut offsets = BTreeSet::new();
        for slot in layout {
            assert_eq!(
                slot.offset % 8,
                0,
                "{}.{} offset should be 8-byte aligned",
                slot.record,
                slot.name
            );
            assert_eq!(
                slot.width % 8,
                0,
                "{}.{} width should be 8-byte aligned",
                slot.record,
                slot.name
            );
            assert!(
                slot.end() <= record_size,
                "{}.{} ends at {}, beyond record size {}",
                slot.record,
                slot.name,
                slot.end(),
                record_size
            );
            assert!(
                offsets.insert((slot.record, slot.offset)),
                "{} has duplicate offset {}",
                slot.record,
                slot.offset
            );
        }
    }

    fn assert_byte_span_layout(layout: HeapByteSpanLayout) {
        assert!(!layout.record.is_empty());
        assert!(!layout.length_source.is_empty());
        assert!(
            matches!(layout.element_width, 1 | 2 | 8),
            "{} has unsupported element width {}",
            layout.record,
            layout.element_width
        );
        assert!(
            !layout.pointer,
            "{} should not be traced as pointer storage",
            layout.record
        );
    }

    fn assert_named_slots(layout: &[HeapNamedSlot]) {
        let mut keys = BTreeSet::new();
        for slot in layout {
            assert!(!slot.record.is_empty());
            assert!(!slot.key.is_empty());
            assert!(
                !slot.scans_target || slot.strong_reference,
                "{}.{} cannot scan a weak or non-reference target",
                slot.record,
                slot.key
            );
            assert!(
                keys.insert((slot.record, slot.key)),
                "{} has duplicate metadata key {}",
                slot.record,
                slot.key
            );
        }
    }

    fn assert_root_sources(layout: &[HeapRootSource]) {
        let mut names = BTreeSet::new();
        for source in layout {
            assert!(!source.name.is_empty());
            assert!(!source.owner.is_empty());
            assert!(
                names.insert(source.name),
                "duplicate root source {}",
                source.name
            );
        }
    }

    fn assert_weak_edge_slots(layout: &[HeapWeakEdgeSlot]) {
        let mut names = BTreeSet::new();
        for slot in layout {
            assert!(!slot.record.is_empty());
            assert!(!slot.name.is_empty());
            assert!(
                names.insert((slot.record, slot.name)),
                "{} has duplicate weak edge slot {}",
                slot.record,
                slot.name
            );
            match slot.kind {
                HeapWeakEdgeKind::EphemeronKey
                | HeapWeakEdgeKind::EphemeronValue
                | HeapWeakEdgeKind::WeakTarget
                | HeapWeakEdgeKind::FinalizerToken => assert!(
                    !slot.keeps_target_alive,
                    "{:?} must not keep the weak target alive",
                    slot
                ),
                HeapWeakEdgeKind::FinalizerHoldings => assert!(
                    slot.keeps_target_alive,
                    "finalizer holdings must stay live until cleanup"
                ),
            }
        }
    }

    fn assert_collector_contract(contract: HeapCollectorContract) {
        assert!(!contract.name.is_empty());
        assert!(
            !contract.moving,
            "T05 collector contract must stay non-moving until all roots can be updated"
        );
        assert_eq!(
            contract.capability,
            HeapCollectorCapability::MetadataChecked,
            "gc() must not run until the collector is executable"
        );
        assert!(!heap_collector_is_executable());
        assert_eq!(contract.root_sources, HEAP_ROOT_SOURCES);
        assert_eq!(contract.weak_edges, HEAP_WEAK_EDGE_SLOTS);

        let mut phase_names = BTreeSet::new();
        let mut phase_kinds = BTreeSet::new();
        for phase in contract.phases {
            assert!(!phase.name.is_empty());
            assert!(
                phase.required_for_gc_builtin,
                "{} must be implemented before exposing gc()",
                phase.name
            );
            assert!(
                phase_names.insert(phase.name),
                "duplicate collector phase {}",
                phase.name
            );
            assert!(
                phase_kinds.insert(format!("{:?}", phase.kind)),
                "duplicate collector phase kind {:?}",
                phase.kind
            );
        }
    }

    fn assert_host_boundary_contract(contract: HeapHostBoundaryContract) {
        assert!(!contract.name.is_empty());
        assert!(
            !contract.durable_host_pointers,
            "host pointers must not be stored as durable Wasm payloads"
        );
        assert_eq!(
            contract.memory_borrow_duration,
            HostMemoryBorrowDuration::ImportCallOnly
        );
        assert!(
            contract.reentrant_imports_require_transient_roots,
            "host re-entrancy must keep tag/payload roots live"
        );
        assert!(
            HEAP_ROOT_SOURCES.iter().any(|source| {
                source.name == contract.borrowed_root_source
                    && source.owner == "host-import-boundary"
                    && source.tagged_values
                    && source.transient
            }),
            "host boundary contract must point at a transient tagged root source"
        );
    }

    fn assert_value_encoding_slots(layout: &[ValueEncodingSlot]) {
        let mut kinds = BTreeSet::new();
        for slot in layout {
            assert!(
                kinds.insert(slot.kind.tag()),
                "duplicate value encoding for {:?}",
                slot.kind
            );
            if slot.preserves_number_bits {
                assert_eq!(
                    slot.payload,
                    ValuePayloadEncoding::Ieee754Bits,
                    "only Number values should preserve IEEE-754 payload bits"
                );
            }
        }
    }

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
        assert_eq!(HEAP_ARRAY_RECORD_SIZE, 272);
        assert_eq!(HEAP_STRING_RECORD_SIZE, 32);
        assert_eq!(HEAP_BIGINT_RECORD_SIZE, 32);
        assert_eq!(HEAP_SYMBOL_RECORD_SIZE, 32);
        assert_eq!(HEAP_REALM_RECORD_SIZE, 72);
        assert_eq!(HEAP_REALM_INTRINSICS_RECORD_SIZE, 208);
        assert_eq!(HEAP_PROMISE_RECORD_SIZE, 64);
        assert_eq!(HEAP_PROMISE_REACTION_RECORD_SIZE, 64);
        assert_eq!(HEAP_PENDING_JOB_RECORD_SIZE, 56);
    }

    #[test]
    fn heap_layout_registry_has_no_slot_collisions() {
        assert_layout(HEAP_OBJECT_HEADER_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(HEAP_FUNCTION_OBJECT_LAYOUT, HEAP_FUNCTION_OBJECT_SIZE);
        assert_layout(
            HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT,
            HEAP_CLASS_FUNCTION_CONTEXT_SIZE,
        );
        assert_layout(HEAP_PRIVATE_ENV_LAYOUT, HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET);
        assert_layout(HEAP_BOUND_FUNCTION_LAYOUT, HEAP_BOUND_FUNCTION_RECORD_SIZE);
        assert_layout(
            HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT,
            HEAP_PRIVATE_ELEMENT_ENTRY_SIZE,
        );
        assert_layout(HEAP_ARRAY_OBJECT_LAYOUT, HEAP_ARRAY_RECORD_SIZE);
        assert_layout(HEAP_OBJECT_ENTRY_LAYOUT, HEAP_OBJECT_ENTRY_SIZE);
        assert_layout(HEAP_ARRAY_ENTRY_LAYOUT, HEAP_ARRAY_ENTRY_SIZE);
        assert_layout(HEAP_STRING_LAYOUT, HEAP_STRING_RECORD_SIZE);
        assert_layout(HEAP_BIGINT_LAYOUT, HEAP_BIGINT_RECORD_SIZE);
        assert_layout(HEAP_SYMBOL_LAYOUT, HEAP_SYMBOL_RECORD_SIZE);
        assert_layout(HEAP_REALM_RECORD_LAYOUT, HEAP_REALM_RECORD_SIZE);
        assert_layout(
            HEAP_REALM_INTRINSICS_LAYOUT,
            HEAP_REALM_INTRINSICS_RECORD_SIZE,
        );
        assert_layout(HEAP_PROMISE_LAYOUT, HEAP_PROMISE_RECORD_SIZE);
        assert_layout(
            HEAP_PROMISE_REACTION_LAYOUT,
            HEAP_PROMISE_REACTION_RECORD_SIZE,
        );
        assert_layout(HEAP_PENDING_JOB_LAYOUT, HEAP_PENDING_JOB_RECORD_SIZE);
        assert_layout(
            HEAP_ENVIRONMENT_LAYOUT,
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_SIZE,
        );
    }

    #[test]
    fn heap_layout_registry_marks_gc_pointer_fields() {
        let pointer_slots = HEAP_OBJECT_HEADER_LAYOUT
            .iter()
            .chain(HEAP_FUNCTION_OBJECT_LAYOUT.iter())
            .chain(HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT.iter())
            .chain(HEAP_PRIVATE_ENV_LAYOUT.iter())
            .chain(HEAP_BOUND_FUNCTION_LAYOUT.iter())
            .chain(HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT.iter())
            .chain(HEAP_ARRAY_OBJECT_LAYOUT.iter())
            .chain(HEAP_OBJECT_ENTRY_LAYOUT.iter())
            .chain(HEAP_ARRAY_ENTRY_LAYOUT.iter())
            .chain(HEAP_STRING_LAYOUT.iter())
            .chain(HEAP_BIGINT_LAYOUT.iter())
            .chain(HEAP_SYMBOL_LAYOUT.iter())
            .chain(HEAP_REALM_RECORD_LAYOUT.iter())
            .chain(HEAP_REALM_INTRINSICS_LAYOUT.iter())
            .chain(HEAP_PROMISE_LAYOUT.iter())
            .chain(HEAP_PROMISE_REACTION_LAYOUT.iter())
            .chain(HEAP_PENDING_JOB_LAYOUT.iter())
            .chain(HEAP_ENVIRONMENT_LAYOUT.iter())
            .filter(|slot| slot.pointer)
            .count();
        assert!(pointer_slots >= 64, "expected GC-visible pointer slots");
        assert!(HEAP_OBJECT_HEADER_LAYOUT.iter().any(|slot| {
            slot.name == "prototype_payload" && slot.offset == HEAP_PROTOTYPE_OFFSET && slot.pointer
        }));
        assert!(HEAP_ARRAY_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "prototype_tag"
                && slot.offset == HEAP_ARRAY_PROTOTYPE_TAG_OFFSET
                && !slot.pointer
        }));
        assert!(HEAP_FUNCTION_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "env_handle"
                && slot.offset == HEAP_FUNCTION_ENV_HANDLE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT.iter().all(|slot| {
            (slot.offset == HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET) == !slot.pointer
        }));
        assert!(HEAP_FUNCTION_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "defining_realm"
                && slot.offset == HEAP_FUNCTION_DEFINING_REALM_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_RECORD_LAYOUT.iter().any(|slot| {
            slot.name == "global_object"
                && slot.offset == HEAP_REALM_GLOBAL_OBJECT_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "TypeError.prototype"
                && slot.offset == HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%ArrayIteratorPrototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_ARRAY_ITERATOR_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%Object.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%IteratorHelperPrototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%Iterator.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%WrapForValidIteratorPrototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_BIGINT_LAYOUT.iter().any(|slot| {
            slot.name == "limbs_ptr" && slot.offset == HEAP_BIGINT_LIMBS_PTR_OFFSET && slot.pointer
        }));
        assert!(HEAP_SYMBOL_LAYOUT.iter().any(|slot| {
            slot.name == "description_payload"
                && slot.offset == HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_PROMISE_LAYOUT.iter().any(|slot| {
            slot.name == "result_payload"
                && slot.offset == HEAP_PROMISE_RESULT_PAYLOAD_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_PENDING_JOB_LAYOUT.iter().any(|slot| {
            slot.name == "next" && slot.offset == HEAP_PENDING_JOB_NEXT_OFFSET && slot.pointer
        }));
    }

    #[test]
    fn heap_named_slot_registry_marks_binary_data_references() {
        for layout in HEAP_NAMED_SLOT_LAYOUTS {
            assert_named_slots(layout);
        }
        assert!(HEAP_ARRAY_BUFFER_NAMED_SLOTS.iter().any(|slot| {
            slot.key == ARRAY_BUFFER_DATA_PTR_SLOT && slot.strong_reference && !slot.scans_target
        }));
        assert!(HEAP_DATA_VIEW_NAMED_SLOTS
            .iter()
            .any(|slot| slot.key == "buffer" && slot.strong_reference && slot.scans_target));
        assert!(HEAP_TYPED_ARRAY_NAMED_SLOTS.iter().any(|slot| {
            slot.key == TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT
                && slot.strong_reference
                && slot.scans_target
        }));
        assert!(HEAP_ARRAY_ITERATOR_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$ArrayIterator.array" && slot.strong_reference && slot.scans_target
        }));
        assert!(HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$RegExpStringIterator.regexp" && slot.strong_reference && slot.scans_target
        }));
        assert!(HEAP_ITERATOR_HELPER_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$IteratorFromIterator" && slot.strong_reference && slot.scans_target
        }));
        assert!(HEAP_ITERATOR_HELPER_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$IteratorFlatMapInnerNext" && slot.strong_reference && slot.scans_target
        }));
    }

    #[test]
    fn iterator_zip_state_slots_have_expected_gc_edges() {
        for key in [
            "$IteratorZipIterators",
            "$IteratorZipNextMethods",
            "$IteratorZipOpen",
            "$IteratorZipPadding",
        ] {
            assert!(HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS
                .iter()
                .any(|slot| { slot.key == key && slot.strong_reference && slot.scans_target }));
        }
        for key in [
            "$IteratorZipDone",
            "$IteratorZipExecuting",
            "$IteratorZipStarted",
            "$IteratorZipMode",
        ] {
            assert!(HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS
                .iter()
                .any(|slot| { slot.key == key && !slot.strong_reference && !slot.scans_target }));
        }
    }

    #[test]
    fn heap_root_registry_covers_gc_safepoint_sources() {
        assert_root_sources(HEAP_ROOT_SOURCES);
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            source.name == "active-frame-locals" && source.tagged_values && source.transient
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            source.name == "completion-records"
                && source.owner == "completion-abi"
                && source.tagged_values
                && source.transient
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            source.name == "host-borrowed-values" && source.tagged_values && source.transient
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            source.name == "pending-jobs" && source.tagged_values && !source.transient
        }));
    }

    #[test]
    fn heap_weak_edge_registry_models_ephemerons_and_finalizers() {
        assert_weak_edge_slots(HEAP_WEAK_EDGE_SLOTS);
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-map-entry"
                && slot.name == "key"
                && slot.kind == HeapWeakEdgeKind::EphemeronKey
                && !slot.keeps_target_alive
        }));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-map-entry"
                && slot.name == "value"
                && slot.kind == HeapWeakEdgeKind::EphemeronValue
                && !slot.keeps_target_alive
        }));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-ref-record"
                && slot.kind == HeapWeakEdgeKind::WeakTarget
                && !slot.keeps_target_alive
        }));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "finalization-registry-cell"
                && slot.name == "holdings"
                && slot.kind == HeapWeakEdgeKind::FinalizerHoldings
                && slot.keeps_target_alive
        }));
    }

    #[test]
    fn heap_collector_contract_requires_all_gc_builtin_phases() {
        assert_collector_contract(HEAP_COLLECTOR_CONTRACT);
        for kind in [
            HeapCollectorPhaseKind::StopTheWorld,
            HeapCollectorPhaseKind::RootScan,
            HeapCollectorPhaseKind::MarkStrong,
            HeapCollectorPhaseKind::ProcessEphemerons,
            HeapCollectorPhaseKind::ClearWeakRefs,
            HeapCollectorPhaseKind::QueueFinalizers,
            HeapCollectorPhaseKind::Sweep,
            HeapCollectorPhaseKind::Resume,
        ] {
            assert!(
                HEAP_COLLECTOR_PHASES
                    .iter()
                    .any(|phase| phase.kind == kind && phase.required_for_gc_builtin),
                "missing required collector phase {:?}",
                kind
            );
        }
    }

    #[test]
    fn heap_collector_contract_keeps_gc_builtin_unsupported_until_executable() {
        assert!(!heap_collector_is_executable());
        assert_eq!(
            HEAP_COLLECTOR_CONTRACT.capability,
            HeapCollectorCapability::MetadataChecked
        );
        assert!(HEAP_COLLECTOR_CONTRACT
            .phases
            .iter()
            .any(|phase| phase.kind == HeapCollectorPhaseKind::Sweep));
        assert!(HEAP_COLLECTOR_CONTRACT
            .weak_edges
            .iter()
            .any(|slot| slot.kind == HeapWeakEdgeKind::EphemeronKey));
        assert!(HEAP_COLLECTOR_CONTRACT
            .weak_edges
            .iter()
            .any(|slot| slot.kind == HeapWeakEdgeKind::FinalizerHoldings));
    }

    #[test]
    fn heap_host_boundary_contract_rejects_durable_host_pointers() {
        assert_host_boundary_contract(HEAP_HOST_BOUNDARY_CONTRACT);
    }

    #[test]
    fn heap_value_encoding_registry_covers_ecmascript_language_types() {
        assert_value_encoding_slots(VALUE_ENCODING_SLOTS);
        for kind in [
            ValueKind::Undefined,
            ValueKind::Null,
            ValueKind::Boolean,
            ValueKind::Number,
            ValueKind::String,
            ValueKind::Symbol,
            ValueKind::Object,
            ValueKind::BigInt,
        ] {
            assert!(
                VALUE_ENCODING_SLOTS.iter().any(|slot| slot.kind == kind),
                "missing value encoding for {:?}",
                kind
            );
        }

        let number = VALUE_ENCODING_SLOTS
            .iter()
            .find(|slot| slot.kind == ValueKind::Number)
            .expect("Number encoding should be registered");
        assert_eq!(number.payload, ValuePayloadEncoding::Ieee754Bits);
        assert!(number.preserves_number_bits);

        let bigint = VALUE_ENCODING_SLOTS
            .iter()
            .find(|slot| slot.kind == ValueKind::BigInt)
            .expect("BigInt encoding should be registered");
        assert_eq!(
            bigint.payload,
            ValuePayloadEncoding::I64TemporaryOrHeapPointer
        );
        assert!(
            !bigint.arbitrary_precision_ready,
            "T05 must keep the current BigInt storage gap visible"
        );
    }

    #[test]
    fn heap_raw_byte_span_registry_marks_non_pointer_storage() {
        assert_eq!(HEAP_RAW_BYTE_SPAN_LAYOUTS.len(), 3);
        for layout in HEAP_RAW_BYTE_SPAN_LAYOUTS {
            assert_byte_span_layout(*layout);
        }
        assert_eq!(
            HEAP_ARRAY_BUFFER_BACKING_STORE_LAYOUT.length_source,
            ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT
        );
        assert_eq!(HEAP_ARRAY_BUFFER_BACKING_STORE_LAYOUT.element_width, 1);
        assert_eq!(HEAP_STRING_CODE_UNITS_LAYOUT.element_width, 2);
        assert_eq!(HEAP_BIGINT_LIMBS_LAYOUT.element_width, 8);
    }

    #[test]
    fn heap_sizes_stay_aligned_for_memory_growth() {
        for size in [
            HEAP_HEADER_SIZE,
            HEAP_FUNCTION_OBJECT_SIZE,
            HEAP_OBJECT_ENTRY_SIZE,
            HEAP_ARRAY_ENTRY_SIZE,
            HEAP_BOUND_FUNCTION_RECORD_SIZE,
            HEAP_REALM_RECORD_SIZE,
            HEAP_REALM_INTRINSICS_RECORD_SIZE,
            HEAP_STRING_RECORD_SIZE,
            HEAP_BIGINT_RECORD_SIZE,
            HEAP_SYMBOL_RECORD_SIZE,
            HEAP_PROMISE_RECORD_SIZE,
            HEAP_PROMISE_REACTION_RECORD_SIZE,
            HEAP_PENDING_JOB_RECORD_SIZE,
            ENV_SLOT_SIZE,
        ] {
            assert_eq!(FunctionBuilder::align_heap_size(size), size);
        }
        assert_eq!(FunctionBuilder::align_heap_size(1), 8);
        assert_eq!(FunctionBuilder::align_heap_size(9), 16);
    }

    #[test]
    fn heap_allocation_emits_growth_failure_and_overflow_traps() {
        let source =
            parse("let o = { a: 1 }; o.a;", ParseOptions::script()).expect("script should parse");
        let artifact = crate::emit(&lower(&source)).expect("heap allocation should emit");

        let mut memory_grow_count = 0;
        let mut unreachable_count = 0;
        let mut dynamic_align_mask_count = 0;
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            let Payload::CodeSectionEntry(body) = payload.expect("module should parse") else {
                continue;
            };
            let mut operators = body.get_operators_reader().expect("operators should parse");
            while !operators.eof() {
                match operators.read().expect("operator should parse") {
                    Operator::MemoryGrow { .. } => memory_grow_count += 1,
                    Operator::Unreachable => unreachable_count += 1,
                    Operator::I64Const { value: -8 } => dynamic_align_mask_count += 1,
                    _ => {}
                }
            }
        }

        assert!(memory_grow_count > 0, "heap allocation should grow memory");
        assert!(
            dynamic_align_mask_count > 0,
            "heap allocation should align dynamic allocation sizes"
        );
        assert!(
            unreachable_count >= 3,
            "heap allocation should trap on size alignment overflow, pointer overflow, and memory.grow failure"
        );
    }
}
