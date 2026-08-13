use lila_ir::ValueKind;

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
pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 344;
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
pub(crate) const HEAP_PROMISE_RECORD_SIZE: u64 = 72;
pub(crate) const HEAP_MAP_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_MAP_ENTRY_SIZE: u64 = 40;
pub(crate) const HEAP_WEAK_MAP_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_WEAK_MAP_ENTRY_SIZE: u64 = 40;
pub(crate) const HEAP_WEAK_SET_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_WEAK_SET_ENTRY_SIZE: u64 = 24;
pub(crate) const HEAP_WEAK_REF_RECORD_SIZE: u64 = 16;
pub(crate) const HEAP_FINALIZATION_REGISTRY_RECORD_SIZE: u64 = 40;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_SIZE: u64 = 56;
/// `[[AsyncDisposableState]]` plus the `[[DisposeCapability]]`'s
/// `[[DisposableResourceStack]]` (pointer, length, capacity).
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE: u64 = 32;
/// One `DisposableResource` record: its kind, its `[[ResourceValue]]` and its
/// `[[DisposeMethod]]`.
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE: u64 = 40;
pub(crate) const HEAP_TEMPORAL_INSTANT_RECORD_SIZE: u64 = 16;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE: u64 = 48;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_TEMPORAL_DURATION_RECORD_SIZE: u64 = 80;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_RECORD_SIZE: u64 = 48;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_SIZE: u64 = 80;
pub(crate) const HEAP_INTL_LOCALE_RECORD_SIZE: u64 = 40;
pub(crate) const HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE: u64 = 184;
pub(crate) const HEAP_MAP_ITERATOR_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_SET_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_SET_ENTRY_SIZE: u64 = 24;
pub(crate) const HEAP_SET_ITERATOR_RECORD_SIZE: u64 = 32;
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE: u64 = 32;
#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_REACTION_RECORD_SIZE: u64 = 56;
#[allow(dead_code)]
pub(crate) const HEAP_PENDING_JOB_RECORD_SIZE: u64 = 56;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE: u64 = 48;
#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_CAPABILITY_RECORD_SIZE: u64 = 48;
pub(crate) const HEAP_ASYNC_ACTIVATION_RECORD_SIZE: u64 = 136;
#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE: u64 = 184;
#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE: u64 = 56;
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
// The generic Array exotic slots occupy every word through offset 264 and are
// also reachable for Arguments objects. Keep the remaining tagged length
// state after that shared layout instead of aliasing an Array fast-property
// slot.
pub(crate) const HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET: u64 = 272;
pub(crate) const HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET: u64 = 280;
pub(crate) const HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET: u64 = 288;
pub(crate) const HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET: u64 = HEAP_ARRAY_NON_EXTENSIBLE_OFFSET;
pub(crate) const HEAP_ARGUMENTS_RECORD_SIZE: u64 = 296;
pub(crate) const HEAP_OBJECT_BOXED_KIND_OFFSET: u64 = 32;
pub(crate) const HEAP_OBJECT_BOXED_TAG_OFFSET: u64 = 40;
pub(crate) const HEAP_OBJECT_BOXED_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_OBJECT_INTERNAL_BRAND_OFFSET: u64 = 56;
pub(crate) const HEAP_OBJECT_PROTOTYPE_TAG_OFFSET: u64 = 64;
pub(crate) const HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET: u64 = 72;
pub(crate) const HEAP_PROXY_HANDLER_TAG_OFFSET: u64 = 80;
pub(crate) const HEAP_GENERATOR_STATE_OFFSET: u64 = 80;
pub(crate) const HEAP_GENERATOR_FUNCTION_OFFSET: u64 = 88;
pub(crate) const HEAP_GENERATOR_THIS_PAYLOAD_OFFSET: u64 = 96;
pub(crate) const HEAP_GENERATOR_THIS_TAG_OFFSET: u64 = 104;
pub(crate) const HEAP_GENERATOR_ARGC_OFFSET: u64 = 112;
pub(crate) const HEAP_GENERATOR_ARGV_OFFSET: u64 = 120;
pub(crate) const HEAP_GENERATOR_RESUME_STATE_OFFSET: u64 = 128;
pub(crate) const HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET: u64 = 136;
pub(crate) const HEAP_GENERATOR_RESUME_TAG_OFFSET: u64 = 144;
pub(crate) const HEAP_GENERATOR_ENV_OFFSET: u64 = 152;
pub(crate) const HEAP_GENERATOR_INITIALIZED_OFFSET: u64 = 160;
pub(crate) const HEAP_GENERATOR_RESUME_KIND_OFFSET: u64 = 168;
pub(crate) const HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET: u64 = 176;
pub(crate) const HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET: u64 = 184;
pub(crate) const HEAP_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET: u64 = 192;
pub(crate) const HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET: u64 = 200;
pub(crate) const HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET: u64 = 208;
pub(crate) const HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET: u64 = 216;
pub(crate) const HEAP_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET: u64 = 224;
pub(crate) const HEAP_GENERATOR_DELEGATE_RECORD_OFFSET: u64 = 232;
pub(crate) const HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET: u64 = 0;
pub(crate) const HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET: u64 = 32;
pub(crate) const HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET: u64 = 40;
pub(crate) const HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET: u64 = 48;
pub(crate) const HEAP_GENERATOR_DELEGATE_ASYNC_ITERATOR_OFFSET: u64 = 56;
pub(crate) const HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET: u64 = 64;
pub(crate) const HEAP_GENERATOR_DELEGATE_RESULT_DONE_PAYLOAD_OFFSET: u64 = 72;
pub(crate) const HEAP_GENERATOR_DELEGATE_RESULT_DONE_TAG_OFFSET: u64 = 80;
pub(crate) const HEAP_GENERATOR_DELEGATE_RECORD_SIZE: u64 = 88;
pub(crate) const HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET: u64 = 80;
pub(crate) const HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET: u64 = 0;
pub(crate) const HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET: u64 = 8;
pub(crate) const HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET: u64 = 16;
pub(crate) const HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET: u64 = 32;
pub(crate) const HEAP_ASYNC_GENERATOR_FUNCTION_ENV_OFFSET: u64 = 40;
pub(crate) const HEAP_ASYNC_GENERATOR_THIS_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_ASYNC_GENERATOR_THIS_TAG_OFFSET: u64 = 56;
pub(crate) const HEAP_ASYNC_GENERATOR_ARGC_OFFSET: u64 = 64;
pub(crate) const HEAP_ASYNC_GENERATOR_ARGV_OFFSET: u64 = 72;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET: u64 = 80;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET: u64 = 88;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET: u64 = 96;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET: u64 = 104;
pub(crate) const HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET: u64 = 112;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET: u64 = 120;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET: u64 = 128;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET: u64 = 136;
pub(crate) const HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET: u64 = 144;
pub(crate) const HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET: u64 = 152;
pub(crate) const HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET: u64 = 160;
pub(crate) const HEAP_ASYNC_GENERATOR_INITIALIZED_OFFSET: u64 = 168;
pub(crate) const HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET: u64 = 176;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET: u64 = 0;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_CAPABILITY_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET: u64 = 40;
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET: u64 = 48;
pub(crate) const HEAP_PENDING_COMPLETION_NEXT_OFFSET: u64 = 0;
pub(crate) const HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_PENDING_COMPLETION_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_PENDING_COMPLETION_KIND_OFFSET: u64 = 24;
pub(crate) const HEAP_PENDING_COMPLETION_AUX_OFFSET: u64 = 32;
pub(crate) const HEAP_PENDING_COMPLETION_RECORD_SIZE: u64 = 40;
// ArrayBuffer and SharedArrayBuffer instances use a brand-selected private
// record in the generic object header. These slots must not be represented as
// ordinary properties: user code can legally create or overwrite the legacy
// `$ArrayBuffer...` names.
pub(crate) const HEAP_ARRAY_BUFFER_DATA_OFFSET: u64 = 80;
pub(crate) const HEAP_ARRAY_BUFFER_BYTE_LENGTH_OFFSET: u64 = 88;
pub(crate) const HEAP_ARRAY_BUFFER_MAX_BYTE_LENGTH_OFFSET: u64 = 96;
pub(crate) const HEAP_ARRAY_BUFFER_DETACH_KEY_TAG_OFFSET: u64 = 104;
pub(crate) const HEAP_ARRAY_BUFFER_DETACH_KEY_PAYLOAD_OFFSET: u64 = 112;
pub(crate) const HEAP_ARRAY_BUFFER_FLAGS_OFFSET: u64 = 120;
pub(crate) const ARRAY_BUFFER_FLAG_RESIZABLE: u64 = 1;
pub(crate) const ARRAY_BUFFER_FLAG_SHARED: u64 = 2;
pub(crate) const ARRAY_BUFFER_FLAG_IMMUTABLE: u64 = 4;
pub(crate) const ARRAY_BUFFER_FLAG_DETACHED: u64 = 8;
// TypedArray instances are ordinary heap objects with integer-indexed exotic
// behavior.  Their internal slots must not alias user-visible properties: JS
// can legally create or overwrite names such as `$TypedArrayByteLength`.
pub(crate) const HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET: u64 = 80;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_OFFSET: u64 = 88;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET: u64 = 96;
pub(crate) const HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET: u64 = 104;
pub(crate) const HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET: u64 = 112;
pub(crate) const HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET: u64 = 120;
pub(crate) const HEAP_DATA_VIEW_VIEWED_BUFFER_OFFSET: u64 = 80;
pub(crate) const HEAP_DATA_VIEW_BYTE_OFFSET: u64 = 88;
pub(crate) const HEAP_DATA_VIEW_BYTE_LENGTH_OFFSET: u64 = 96;
pub(crate) const HEAP_DATA_VIEW_LENGTH_TRACKING_OFFSET: u64 = 104;
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

/// The closed wire domain stored in a private-element heap entry.
///
/// This is a backend storage protocol, not ECMA-262's `field`/`method`/
/// `accessor` domain. Methods and accessors install one receiver brand and
/// share their callable definitions across instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrivateElementHeapKind {
    Brand,
    Field,
    SetterDefinition,
    MethodDefinition,
    GetterDefinition,
}

impl PrivateElementHeapKind {
    pub(crate) const fn wire_word(self) -> u64 {
        match self {
            Self::Brand => 0,
            Self::Field => 1,
            Self::SetterDefinition => 2,
            Self::MethodDefinition => 3,
            Self::GetterDefinition => 4,
        }
    }

    pub(crate) const fn has_receiver(self) -> bool {
        match self {
            Self::Brand | Self::Field => true,
            Self::SetterDefinition | Self::MethodDefinition | Self::GetterDefinition => false,
        }
    }

    pub(crate) const fn has_value(self) -> bool {
        match self {
            Self::Brand => false,
            Self::Field
            | Self::SetterDefinition
            | Self::MethodDefinition
            | Self::GetterDefinition => true,
        }
    }
}

/// The subset legal for a shared private callable-definition lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrivateElementDefinitionKind {
    Setter,
    Method,
    Getter,
}

impl PrivateElementDefinitionKind {
    pub(crate) const fn heap_kind(self) -> PrivateElementHeapKind {
        match self {
            Self::Setter => PrivateElementHeapKind::SetterDefinition,
            Self::Method => PrivateElementHeapKind::MethodDefinition,
            Self::Getter => PrivateElementHeapKind::GetterDefinition,
        }
    }
}
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
pub(crate) const HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET: u64 = 208;
pub(crate) const HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET: u64 = 216;
pub(crate) const HEAP_REALM_INTRINSICS_MAP_ITERATOR_PROTOTYPE_OFFSET: u64 = 224;
pub(crate) const HEAP_REALM_INTRINSICS_SET_ITERATOR_PROTOTYPE_OFFSET: u64 = 232;
pub(crate) const HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET: u64 = 240;
pub(crate) const HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET: u64 = 248;
pub(crate) const HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET: u64 = 256;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_ITERATOR_PROTOTYPE_OFFSET: u64 = 264;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET: u64 = 272;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_CONSTRUCTOR_OFFSET: u64 = 280;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET: u64 = 288;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET: u64 = 296;
pub(crate) const HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET: u64 = 304;
pub(crate) const HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET: u64 = 312;
pub(crate) const HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET: u64 = 320;
pub(crate) const HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET: u64 = 328;
pub(crate) const HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET: u64 = 336;
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
pub(crate) const HEAP_BIGINT_VALUE_TAG: i64 = 12;
pub(crate) const HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET: u64 = 8;
/// `Intl.Locale` internal slots. Every slot but `base_name_len` holds a packed
/// string payload (`offset << 32 | len`); an absent optional subtag is stored
/// as `0`, which no real payload can collide with because a zero length means
/// the empty string and the canonicalizer never emits one.
pub(crate) const HEAP_INTL_LOCALE_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_INTL_LOCALE_LANGUAGE_OFFSET: u64 = 8;
pub(crate) const HEAP_INTL_LOCALE_SCRIPT_OFFSET: u64 = 16;
pub(crate) const HEAP_INTL_LOCALE_REGION_OFFSET: u64 = 24;
pub(crate) const HEAP_INTL_LOCALE_BASE_NAME_OFFSET: u64 = 32;

/// `Intl.DateTimeFormat` internal slots (ECMA-402 11.5, Table 8).
///
/// The four string slots hold string payloads; every remaining slot holds a
/// small integer code from [`crate::builtins::intl_datetimeformat`] where 0
/// always means "the option was absent" (`undefined` in `resolvedOptions`).
pub(crate) const HEAP_INTL_DTF_LOCALE_OFFSET: u64 = 0;
pub(crate) const HEAP_INTL_DTF_CALENDAR_OFFSET: u64 = 8;
pub(crate) const HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET: u64 = 16;
pub(crate) const HEAP_INTL_DTF_TIME_ZONE_OFFSET: u64 = 24;
pub(crate) const HEAP_INTL_DTF_HOUR_CYCLE_OFFSET: u64 = 32;
pub(crate) const HEAP_INTL_DTF_WEEKDAY_OFFSET: u64 = 40;
pub(crate) const HEAP_INTL_DTF_ERA_OFFSET: u64 = 48;
pub(crate) const HEAP_INTL_DTF_YEAR_OFFSET: u64 = 56;
pub(crate) const HEAP_INTL_DTF_MONTH_OFFSET: u64 = 64;
pub(crate) const HEAP_INTL_DTF_DAY_OFFSET: u64 = 72;
pub(crate) const HEAP_INTL_DTF_DAY_PERIOD_OFFSET: u64 = 80;
pub(crate) const HEAP_INTL_DTF_HOUR_OFFSET: u64 = 88;
pub(crate) const HEAP_INTL_DTF_MINUTE_OFFSET: u64 = 96;
pub(crate) const HEAP_INTL_DTF_SECOND_OFFSET: u64 = 104;
pub(crate) const HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET: u64 = 112;
pub(crate) const HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET: u64 = 120;
pub(crate) const HEAP_INTL_DTF_DATE_STYLE_OFFSET: u64 = 128;
pub(crate) const HEAP_INTL_DTF_TIME_STYLE_OFFSET: u64 = 136;
/// Resolved `hour12`: 0 absent, 1 false, 2 true.
pub(crate) const HEAP_INTL_DTF_HOUR12_OFFSET: u64 = 144;
/// Memoised `[[BoundFormat]]` function object payload, 0 until first read.
pub(crate) const HEAP_INTL_DTF_BOUND_FORMAT_OFFSET: u64 = 152;
/// `needDefaults` as computed by `CreateDateTimeFormat`: 1 when the options
/// bag named no date/time component and no dateStyle/timeStyle, so the
/// Temporal `toLocaleString` path may substitute the type's own defaults.
pub(crate) const HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET: u64 = 160;
/// The resolved time zone's offset from UTC, in whole signed minutes.
///
/// This is the *other half* of [`HEAP_INTL_DTF_TIME_ZONE_OFFSET`]: that slot
/// holds the identifier `resolvedOptions().timeZone` reports, this one holds
/// the shift `PartitionDateTimePattern` applies to an exact time value before
/// breaking it into components. `"UTC"`, `"Etc/GMT+7"` and `"-07:00"` are three
/// identifiers, two offsets and one formatted output for two of them, so
/// neither slot can be derived from the other and both are stored.
///
/// A raw signed `i64` holding a value in `-1439..=1439` — the `TzOffsetMinutes`
/// range of `crate::builtins::intl_datetimeformat` — never an f64 bit pattern.
pub(crate) const HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET: u64 = 168;
/// The localized GMT name (`"GMT-07:00"`) of a **non-zero** offset zone, or `0`
/// when the offset is zero and CLDR `en`'s real UTC names apply instead.
///
/// Pre-rendered by the constructor rather than built inside the format walk.
/// That walk is emitted once per `format`, `formatToParts`, `formatRange` and
/// `formatRangeToParts` body and is already the largest thing this crate emits;
/// the string concatenations this slot replaces would have been paid for four
/// times over, in the one function whose size budget is known to be tight.
pub(crate) const HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET: u64 = 176;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET: u64 = 32;
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET: u64 = 40;
/// `Temporal.PlainDate` internal slots. The ISO fields are plain signed
/// integers, not tag/payload pairs: `RejectISODate` bounds year to
/// ±275760-ish and month/day to two digits, so they always fit an `i64` and
/// never need the BigInt escape hatch the epoch-nanosecond types use. The
/// calendar slot holds a bare interned string payload because
/// `emit_temporal_iso_calendar_or_throw` only ever yields `"iso8601"`.
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET: u64 = 8;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET: u64 = 16;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET: u64 = 24;
/// `Temporal.Duration` internal slots. Every field is a plain signed `i64`:
/// `IsValidDuration` caps years/months/weeks below 2^32 and forces the whole
/// day-through-nanosecond tail below 2^53 seconds, so no field can escape an
/// `i64` once construction has succeeded.
/// `Temporal.PlainTime` internal slots. `RejectTime` bounds every field to two
/// or three digits, so a plain signed `i64` per field is always enough and
/// there is no calendar slot to carry — a `PlainTime` has no calendar.
/// `Temporal.PlainDateTime` internal slots: the three ISO date fields of a
/// `PlainDate` followed by the six wall-clock fields of a `PlainTime`, then the
/// interned calendar payload. Every numeric slot is a plain signed `i64` for the
/// same reason the two component types are - `RejectDateTime` bounds all nine.
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET: u64 = 8;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET: u64 = 16;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET: u64 = 24;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET: u64 = 32;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET: u64 = 40;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET: u64 = 48;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET: u64 = 56;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET: u64 = 64;
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET: u64 = 72;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET: u64 = 8;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET: u64 = 16;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET: u64 = 24;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET: u64 = 32;
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET: u64 = 40;
pub(crate) const HEAP_TEMPORAL_DURATION_YEARS_OFFSET: u64 = 0;
pub(crate) const HEAP_TEMPORAL_DURATION_MONTHS_OFFSET: u64 = 8;
pub(crate) const HEAP_TEMPORAL_DURATION_WEEKS_OFFSET: u64 = 16;
pub(crate) const HEAP_TEMPORAL_DURATION_DAYS_OFFSET: u64 = 24;
pub(crate) const HEAP_TEMPORAL_DURATION_HOURS_OFFSET: u64 = 32;
pub(crate) const HEAP_TEMPORAL_DURATION_MINUTES_OFFSET: u64 = 40;
pub(crate) const HEAP_TEMPORAL_DURATION_SECONDS_OFFSET: u64 = 48;
pub(crate) const HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET: u64 = 56;
pub(crate) const HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET: u64 = 64;
pub(crate) const HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET: u64 = 72;
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
// Intrusive link used by the host unhandled-rejection tracker. A promise is
// appended to the tracked list at most once - when RejectPromise runs while
// [[IsHandled]] is still false - so one link field per record is enough.
pub(crate) const HEAP_PROMISE_UNHANDLED_NEXT_OFFSET: u64 = 64;
pub(crate) const HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET: u64 = 32;
pub(crate) const HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET: u64 = 40;
pub(crate) const HEAP_MAP_ENTRIES_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_MAP_ENTRIES_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_MAP_ENTRIES_CAP_OFFSET: u64 = 16;
pub(crate) const HEAP_MAP_LIVE_COUNT_OFFSET: u64 = 24;
pub(crate) const HEAP_MAP_ENTRY_PRESENT_OFFSET: u64 = 0;
pub(crate) const HEAP_MAP_ENTRY_KEY_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_MAP_ENTRY_VALUE_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_WEAK_MAP_ENTRIES_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_WEAK_MAP_ENTRIES_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_WEAK_MAP_ENTRIES_CAP_OFFSET: u64 = 16;
pub(crate) const HEAP_WEAK_MAP_LIVE_COUNT_OFFSET: u64 = 24;
pub(crate) const HEAP_WEAK_MAP_ENTRY_PRESENT_OFFSET: u64 = 0;
pub(crate) const HEAP_WEAK_MAP_ENTRY_KEY_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_WEAK_MAP_ENTRY_KEY_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_WEAK_MAP_ENTRY_VALUE_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_WEAK_MAP_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_WEAK_REF_TARGET_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET: u64 = 16;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET: u64 = 24;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET: u64 = 32;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_PRESENT_OFFSET: u64 = 0;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET: u64 = 32;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET: u64 = 40;
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET: u64 = 0;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET: u64 = 8;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET: u64 = 16;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET: u64 = 0;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET: u64 = 32;
/// `[[AsyncDisposableState]]` is a two-element domain, so it is stored as a
/// flag rather than a string: `pending` is the value `AsyncDisposableStack()`
/// installs, and `disposed` is what `disposeAsync` and `move` set. The
/// `disposed` getter reads exactly this word, which is why
/// `prototype/disposed/returns-true-when-disposed.js` observes the transition
/// synchronously.
///
/// This is an enum rather than the pair of `u64` constants it started as
/// because those constants shared a type — and a *value* — with the entry-kind
/// words below: `..._STATE_DISPOSED` and `..._ENTRY_KIND_ADOPT` were both
/// `u64 = 1`, so every emitter site accepted either one. The two domains index
/// different words of different records and are never interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncDisposableStackState {
    Pending,
    Disposed,
}

impl AsyncDisposableStackState {
    /// The word stored at [`HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET`].
    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Pending => 0,
            Self::Disposed => 1,
        }
    }
}

/// The closed domain of `[[DisposableResourceStack]]` entry shapes, stored at
/// [`HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET`].
///
/// # Why this is an enum and not four `u64` constants
///
/// The disposal walk in `builtins/async_disposable_stack.rs` dispatches on this
/// word by emitting a comparison chain, and the chain's **last arm is an
/// emitted `Else`**. A fifth `..._ENTRY_KIND_FOO: u64 = 4` would have compiled
/// cleanly next to its siblings and then been disposed *as a `Defer`* — called
/// with an undefined receiver and no arguments — with nothing to notice. That
/// is the same silent-fallthrough class [`crate::data::RuntimeRegExpEntryKind`]
/// was introduced for in batch 7, one record over.
///
/// So the decision the emitter makes is stated here once, as the exhaustive
/// [`Self::dispose_call`], and the emitter builds its comparison chain by
/// iterating [`Self::ALL`]. Adding a variant is then an `error[E0004]` here and
/// the emitted chain extends itself.
///
/// Residual, stated rather than papered over: [`Self::ALL`] is hand-written,
/// because stable Rust cannot enumerate an enum's variants. The trigger to
/// extend it is the `error[E0004]` a new variant produces at the two matches
/// below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncDisposableStackEntryKind {
    /// A `use(V)` entry: `Call(method, V)` with no arguments.
    Use,
    /// An `adopt(V, onDisposeAsync)` entry: the spec's captured closure is
    /// `Call(onDisposeAsync, undefined, « V »)`, which is stored flat here
    /// instead of minting a builtin function object per call.
    Adopt,
    /// A `defer(onDisposeAsync)` entry: `Call(onDisposeAsync, undefined, « »)`.
    Defer,
    /// A `use(null)` / `use(undefined)` entry. CreateDisposableResource leaves
    /// both `[[ResourceValue]]` and `[[DisposeMethod]]` undefined, so disposal
    /// performs no call — but the entry is still on the stack, so `Dispose`
    /// still awaits, which is what
    /// `disposeAsync/explicit-await-for-null.js` measures.
    Empty,
}

/// How the disposal walk calls one entry.
///
/// "No call at all" is spelled as the `None` of the [`Option`] returned by
/// [`AsyncDisposableStackEntryKind::dispose_call`], not as a variant here, so
/// the emitter's match over the shapes has no arm it must prove unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncDisposableStackDisposeCall {
    /// `Call(method, V)` — the resource is the receiver, no arguments.
    ResourceReceiver,
    /// `Call(onDisposeAsync, undefined, « V »)` — the resource is the sole
    /// argument.
    UndefinedReceiverWithResourceArgument,
    /// `Call(onDisposeAsync, undefined, « »)`.
    UndefinedReceiverNoArguments,
}

impl AsyncDisposableStackEntryKind {
    /// Every kind, in the order the emitted comparison chain tests them. See
    /// the type's doc for why this is hand-written and what keeps it honest.
    pub(crate) const ALL: [Self; 4] = [Self::Use, Self::Adopt, Self::Defer, Self::Empty];

    /// The word stored at
    /// [`HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET`].
    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Use => 0,
            Self::Adopt => 1,
            Self::Defer => 2,
            Self::Empty => 3,
        }
    }

    /// What disposal does with an entry of this kind, or `None` when the entry
    /// carries no dispose method and the walk only awaits. This is the whole
    /// dispatch policy; the emitter transcribes nothing.
    pub(crate) const fn dispose_call(self) -> Option<AsyncDisposableStackDisposeCall> {
        match self {
            Self::Use => Some(AsyncDisposableStackDisposeCall::ResourceReceiver),
            Self::Adopt => {
                Some(AsyncDisposableStackDisposeCall::UndefinedReceiverWithResourceArgument)
            }
            Self::Defer => Some(AsyncDisposableStackDisposeCall::UndefinedReceiverNoArguments),
            Self::Empty => None,
        }
    }
}
pub(crate) const HEAP_WEAK_SET_ENTRIES_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_WEAK_SET_ENTRIES_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_WEAK_SET_ENTRIES_CAP_OFFSET: u64 = 16;
pub(crate) const HEAP_WEAK_SET_LIVE_COUNT_OFFSET: u64 = 24;
pub(crate) const HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET: u64 = 0;
pub(crate) const HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET: u64 = 0;
pub(crate) const HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET: u64 = 8;
pub(crate) const HEAP_MAP_ITERATOR_KIND_OFFSET: u64 = 16;
pub(crate) const HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET: u64 = 24;
pub(crate) const HEAP_SET_ENTRIES_PTR_OFFSET: u64 = 0;
pub(crate) const HEAP_SET_ENTRIES_LEN_OFFSET: u64 = 8;
pub(crate) const HEAP_SET_ENTRIES_CAP_OFFSET: u64 = 16;
pub(crate) const HEAP_SET_LIVE_COUNT_OFFSET: u64 = 24;
pub(crate) const HEAP_SET_ENTRY_PRESENT_OFFSET: u64 = 0;
pub(crate) const HEAP_SET_ENTRY_VALUE_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET: u64 = 0;
pub(crate) const HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET: u64 = 8;
pub(crate) const HEAP_SET_ITERATOR_KIND_OFFSET: u64 = 16;
pub(crate) const HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET: u64 = 24;
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET: u64 = 0;
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET: u64 = 8;
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET: u64 = 16;
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_REACTION_CAPABILITY_OFFSET: u64 = 0;
pub(crate) const HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_PROMISE_REACTION_REALM_OFFSET: u64 = 24;
pub(crate) const HEAP_PROMISE_REACTION_NEXT_OFFSET: u64 = 32;
pub(crate) const HEAP_PROMISE_REACTION_TYPE_OFFSET: u64 = 40;
pub(crate) const HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET: u64 = 48;
pub(crate) const HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET: u64 = 0;
pub(crate) const HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const HEAP_PENDING_JOB_ARG_TAG_OFFSET: u64 = 16;
pub(crate) const HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET: u64 = 24;
pub(crate) const HEAP_PENDING_JOB_REALM_OFFSET: u64 = 32;
pub(crate) const HEAP_PENDING_JOB_NEXT_OFFSET: u64 = 40;
pub(crate) const HEAP_PENDING_JOB_KIND_OFFSET: u64 = 48;

macro_rules! promise_wire_domain {
    ($name:ident, $first_word:literal, { $($variant:ident = $word:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum $name {
            $($variant),+
        }

        impl $name {
            pub(crate) const ALL: [Self; promise_wire_domain!(@count $($variant),+)] =
                [$(Self::$variant),+];

            pub(crate) const fn word(self) -> u64 {
                match self {
                    $(Self::$variant => $word),+
                }
            }
        }

        const _: () = {
            let all = $name::ALL;
            let mut index = 0;
            while index < all.len() {
                assert!(all[index].word() == ($first_word as u64) + index as u64);
                index += 1;
            }
        };
    };
    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(promise_wire_domain!(@unit $variant)),+])
    };
    (@unit $variant:ident) => { () };
}

// The closed domain stored in a Promise reaction record's `type` word.
//
// A reaction is selected for exactly one terminal Promise path. It cannot be
// pending, even though its stable wire words intentionally match the fulfilled
// and rejected Promise-state words. The job runner decodes this domain once;
// no callback shape may reinterpret the raw word independently.
promise_wire_domain!(PromiseReactionType, 1, {
    Fulfill = 1,
    Reject = 2,
});

impl PromiseReactionType {
    /// The normalized runtime branch consumed by every reaction callback.
    pub(crate) const fn is_rejected(self) -> bool {
        match self {
            Self::Fulfill => false,
            Self::Reject => true,
        }
    }
}

// The closed domain stored in a Promise reaction record's `callback_kind`
// word.
//
// Promise reactions use the default ECMAScript handler path or one of five
// internal async-continuation paths. Keeping the wire word and its job-realm
// policy on one type means a new continuation cannot be initialized without
// also selecting how it runs and which realm a queued job carries.
promise_wire_domain!(PromiseReactionCallbackKind, 0, {
    Default = 0,
    AsyncFunction = 1,
    AsyncGeneratorAwaitReturn = 2,
    AsyncGeneratorAwait = 3,
    AsyncGeneratorYield = 4,
    AsyncGeneratorYieldReturn = 5,
});

/// Where a Promise reaction job obtains its host job realm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromiseReactionRealmSource {
    /// `GetFunctionRealm(handler)` when the handler is callable, otherwise the
    /// null realm required for an empty Promise reaction handler.
    HandlerOrNull,
    /// The realm captured when an internal async continuation was created.
    Captured,
}

impl PromiseReactionCallbackKind {
    /// The complete realm-selection policy for this reaction shape.
    pub(crate) const fn realm_source(self) -> PromiseReactionRealmSource {
        match self {
            Self::Default => PromiseReactionRealmSource::HandlerOrNull,
            Self::AsyncFunction
            | Self::AsyncGeneratorAwaitReturn
            | Self::AsyncGeneratorAwait
            | Self::AsyncGeneratorYield
            | Self::AsyncGeneratorYieldReturn => PromiseReactionRealmSource::Captured,
        }
    }
}

// The closed domain stored in a pending Promise job record's `kind` word.
//
// The job drain derives its emitted comparison chain from `ALL` and selects
// the implementation through an exhaustive Rust match. A new job kind
// therefore extends the run-time dispatch and fails to compile until its
// behavior is supplied, rather than falling through as a thenable job.
//
promise_wire_domain!(PromiseJobKind, 1, {
    Reaction = 1,
    ResolveThenable = 2,
});

pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET: u64 = 0;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET: u64 = 8;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET: u64 = 16;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET: u64 = 24;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET: u64 = 32;
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET: u64 = 40;
pub(crate) const HEAP_ASYNC_FUNCTION_ENV_OFFSET: u64 = 0;
pub(crate) const HEAP_ASYNC_FUNCTION_TABLE_INDEX_OFFSET: u64 = 8;
pub(crate) const HEAP_ASYNC_THIS_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_ASYNC_THIS_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_ARGC_OFFSET: u64 = 32;
pub(crate) const HEAP_ASYNC_ARGV_OFFSET: u64 = 40;
pub(crate) const HEAP_ASYNC_RESUME_STATE_OFFSET: u64 = 48;
pub(crate) const HEAP_ASYNC_RESUME_PAYLOAD_OFFSET: u64 = 56;
pub(crate) const HEAP_ASYNC_RESUME_TAG_OFFSET: u64 = 64;
pub(crate) const HEAP_ASYNC_RESUME_KIND_OFFSET: u64 = 72;
pub(crate) const HEAP_ASYNC_ENV_OFFSET: u64 = 80;
pub(crate) const HEAP_ASYNC_INITIALIZED_OFFSET: u64 = 88;
pub(crate) const HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET: u64 = 96;
pub(crate) const HEAP_ASYNC_PROMISE_RECORD_OFFSET: u64 = 104;
pub(crate) const HEAP_ASYNC_COMPLETED_OFFSET: u64 = 112;
pub(crate) const HEAP_ASYNC_PENDING_COMPLETION_HEAD_OFFSET: u64 = 120;
pub(crate) const HEAP_ASYNC_PENDING_COMPLETION_DEPTH_OFFSET: u64 = 128;
pub(crate) const ASYNC_RESUME_KIND_FULFILL: u64 = 0;
pub(crate) const ASYNC_RESUME_KIND_REJECT: u64 = 1;
pub(crate) const ENV_PARENT_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_BASE_OFFSET: u64 = 8;
pub(crate) const ENV_SLOT_SIZE: u64 = 16;
pub(crate) const ENV_SLOT_TAG_OFFSET: u64 = 0;
pub(crate) const ENV_SLOT_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const ENV_SLOT_UNINITIALIZED_TAG: i64 = -1;
pub(crate) const OBJECT_DESCRIPTOR_ACCESSOR: u64 = DescriptorBit::Accessor.word();
pub(crate) const OBJECT_DESCRIPTOR_CONFIGURABLE: u64 = DescriptorBit::Configurable.word();
pub(crate) const OBJECT_DESCRIPTOR_WRITABLE: u64 = DescriptorBit::Writable.word();
pub(crate) const OBJECT_DESCRIPTOR_ENUMERABLE: u64 = DescriptorBit::Enumerable.word();
/// Data is the **absence** of the accessor bit, which is why the illegal word
/// `ACCESSOR | WRITABLE` was representable: nothing forced a choice.
/// [`DescriptorWord::of_data`] is the constructor that makes the absence a
/// decision rather than a default.
pub(crate) const OBJECT_DESCRIPTOR_DATA: u64 = 0;
pub(crate) const PROPERTY_KEY_SYMBOL_MARKER: u64 = 1 << 63;
pub(crate) const ARRAY_DESCRIPTOR_OWN_PROPERTY: u64 = DescriptorBit::ArrayOwnProperty.word();
pub(crate) const ARGUMENTS_DESCRIPTOR_MAPPED: u64 = DescriptorBit::ArgumentsMapped.word();
pub(crate) const ARRAY_DESCRIPTOR_NORMAL_DATA: u64 =
    DescriptorWord::of_data(true, true, true).bits();

// These eight are not tautologies. They pin the **wire format** of a word that
// is written at nine distinct heap offsets and read by 176 references across 11
// files, so reordering `DescriptorBit`'s variants is a compile error rather
// than a silent, total corruption of every object's property attributes.
const _: () = assert!(OBJECT_DESCRIPTOR_ACCESSOR == 1);
const _: () = assert!(OBJECT_DESCRIPTOR_CONFIGURABLE == 2);
const _: () = assert!(OBJECT_DESCRIPTOR_WRITABLE == 4);
const _: () = assert!(OBJECT_DESCRIPTOR_ENUMERABLE == 8);
const _: () = assert!(OBJECT_DESCRIPTOR_DATA == 0);
const _: () = assert!(ARRAY_DESCRIPTOR_OWN_PROPERTY == 16);
const _: () = assert!(ARGUMENTS_DESCRIPTOR_MAPPED == 32);
const _: () = assert!(ARRAY_DESCRIPTOR_NORMAL_DATA == 14);

/// One bit of the descriptor-kind word stored at every
/// `*_DESCRIPTOR_KIND_OFFSET`.
///
/// The word has **three** axes, not one:
///
/// | Bits | Meaning | Axis |
/// |---|---|---|
/// | 0 | `[[Get]]`/`[[Set]]` kind: 1 = accessor, 0 = data | descriptor kind |
/// | 1 | `[[Configurable]]` | attribute |
/// | 2 | `[[Writable]]` | attribute |
/// | 3 | `[[Enumerable]]` | attribute |
/// | 4 | array exotic: this index has an own property record | exotic flag |
/// | 5 | mapped arguments: this index is mapped | exotic flag |
/// | 32..63 | mapped-arguments environment slot index | exotic payload |
///
/// The third axis is the one that decides the shape of these types: a type that
/// modelled only the four attribute bits would corrupt mapped arguments, whose
/// environment slot rides in the top half of the same `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorBit {
    Accessor = 0,
    Configurable = 1,
    Writable = 2,
    Enumerable = 3,
    ArrayOwnProperty = 4,
    ArgumentsMapped = 5,
}

impl DescriptorBit {
    pub(crate) const fn word(self) -> u64 {
        1u64 << (self as u32)
    }
}

/// A descriptor-kind word as **stored** in a heap slot.
///
/// The constructors are exactly the two 6.2.6.6 licenses, and `of_accessor`
/// takes no `writable` argument — so the bit pattern `ACCESSOR | WRITABLE`
/// (= 5), an accessor property carrying a stale writable bit that a later
/// accessor-to-data conversion reads back as `writable: true`, has **no
/// constructor**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DescriptorWord(u64);

impl DescriptorWord {
    /// 6.2.6.6 for a data property: `[[Writable]]`, `[[Enumerable]]`,
    /// `[[Configurable]]`, and the accessor bit clear.
    pub(crate) const fn of_data(writable: bool, enumerable: bool, configurable: bool) -> Self {
        let mut bits = 0u64;
        if writable {
            bits |= DescriptorBit::Writable.word();
        }
        if enumerable {
            bits |= DescriptorBit::Enumerable.word();
        }
        if configurable {
            bits |= DescriptorBit::Configurable.word();
        }
        Self(bits)
    }

    /// 6.2.6.6 for an accessor property. 10.1.6.3 steps 6.b and 7 say a
    /// conversion between kinds preserves only `[[Enumerable]]` and
    /// `[[Configurable]]`; there is no `writable` parameter because an accessor
    /// property has no `[[Writable]]` attribute to preserve.
    pub(crate) const fn of_accessor(enumerable: bool, configurable: bool) -> Self {
        let mut bits = DescriptorBit::Accessor.word();
        if enumerable {
            bits |= DescriptorBit::Enumerable.word();
        }
        if configurable {
            bits |= DescriptorBit::Configurable.word();
        }
        Self(bits)
    }

    /// Attach the orthogonal exotic axis. Separate from the two constructors so
    /// that the descriptor kind is never decided *by* an exotic flag.
    pub(crate) const fn with_flags(self, flags: DescriptorFlags) -> Self {
        let mut bits = self.0;
        if flags.array_own_property {
            bits |= DescriptorBit::ArrayOwnProperty.word();
        }
        match flags.mapped {
            None => {}
            Some(slot) => bits |= DescriptorBit::ArgumentsMapped.word() | slot.packed(),
        }
        Self(bits)
    }

    pub(crate) const fn bits(self) -> u64 {
        self.0
    }

    pub(crate) const fn as_i64(self) -> i64 {
        self.0 as i64
    }
}

/// A **test** against a descriptor word.
///
/// Deliberately a different type from [`DescriptorWord`], with no conversion in
/// either direction: composites like `ACCESSOR | WRITABLE` are illegal as
/// stored values and *legal and needed* as masks — the one at
/// [`DescriptorMask::ACCESSOR_OR_WRITABLE`] asks "is the existing entry a data
/// property that is not writable" in a single `I64And`, and banning the bit
/// pattern outright would break correct code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DescriptorMask(u64);

impl DescriptorMask {
    pub(crate) const ACCESSOR: Self = Self(DescriptorBit::Accessor.word());
    pub(crate) const WRITABLE: Self = Self(DescriptorBit::Writable.word());
    pub(crate) const ENUMERABLE: Self = Self(DescriptorBit::Enumerable.word());
    pub(crate) const CONFIGURABLE: Self = Self(DescriptorBit::Configurable.word());
    /// "The existing entry is a data descriptor **and** is not writable", in
    /// one `I64And`. A legal mask; not a legal word.
    pub(crate) const ACCESSOR_OR_WRITABLE: Self =
        Self(DescriptorBit::Accessor.word() | DescriptorBit::Writable.word());
    /// Bits 0..3: the descriptor kind and the three attributes.
    pub(crate) const KIND_AND_ATTRIBUTES: Self = Self(
        DescriptorBit::Accessor.word()
            | DescriptorBit::Configurable.word()
            | DescriptorBit::Writable.word()
            | DescriptorBit::Enumerable.word(),
    );
    /// Bits 4..5: the two exotic flags.
    pub(crate) const EXOTIC_FLAGS: Self =
        Self(DescriptorBit::ArrayOwnProperty.word() | DescriptorBit::ArgumentsMapped.word());
    /// Bits that survive an update whose incoming descriptor is generic.
    ///
    /// `[[Enumerable]]` and `[[Configurable]]` are re-applied from the
    /// incoming descriptor, while the existing kind, data-only
    /// `[[Writable]]`, and orthogonal exotic markers remain intact.
    pub(crate) const PRESERVED_BY_GENERIC_UPDATE: Self = Self(
        DescriptorBit::Accessor.word()
            | DescriptorBit::Writable.word()
            | DescriptorBit::ArrayOwnProperty.word()
            | DescriptorBit::ArgumentsMapped.word(),
    );

    pub(crate) const fn of(bit: DescriptorBit) -> Self {
        Self(bit.word())
    }

    pub(crate) const fn bits(self) -> u64 {
        self.0
    }

    pub(crate) const fn as_i64(self) -> i64 {
        self.0 as i64
    }

    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

const _: () = assert!(
    DescriptorMask::PRESERVED_BY_GENERIC_UPDATE
        .union(DescriptorMask::ENUMERABLE)
        .union(DescriptorMask::CONFIGURABLE)
        .bits()
        == DescriptorMask::KIND_AND_ATTRIBUTES
            .union(DescriptorMask::EXOTIC_FLAGS)
            .bits(),
    "a generic descriptor update must preserve exactly kind, writability, and exotic flags",
);

/// The exotic axis. Orthogonal to the descriptor kind: an array's own-property
/// marker and an arguments object's mapping are not descriptor kinds and must
/// not share the kind namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DescriptorFlags {
    pub(crate) array_own_property: bool,
    /// `Some(slot)` sets bit 5 **and** packs `slot` into bits 32..63. The two
    /// cannot be set independently, which is what the mapped-arguments writer
    /// in `functions.rs` does by hand today.
    pub(crate) mapped: Option<MappedSlot>,
}

/// A mapped-arguments environment slot index, living in bits 32..63 of the
/// descriptor-kind word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MappedSlot(u32);

impl MappedSlot {
    pub(crate) const SHIFT: u32 = 32;

    pub(crate) const fn new(slot: u32) -> Self {
        Self(slot)
    }

    pub(crate) const fn packed(self) -> u64 {
        (self.0 as u64) << Self::SHIFT
    }
}

// The layout the three axes depend on, asserted at build time.
const _: () = assert!(
    DescriptorMask::KIND_AND_ATTRIBUTES.bits() & DescriptorMask::EXOTIC_FLAGS.bits() == 0,
    "the descriptor kind bits and the exotic flag bits must not overlap",
);
const _: () = assert!(
    (DescriptorMask::KIND_AND_ATTRIBUTES.bits() | DescriptorMask::EXOTIC_FLAGS.bits())
        < (1u64 << MappedSlot::SHIFT),
    "every flag bit must sit below the mapped-slot payload at bit 32",
);
// The three bare literals this used to guard are gone: `functions.rs`'s writer
// now goes through `DescriptorWord::with_flags` and both readers shift by
// `MappedSlot::SHIFT`. What is left to pin is the *reproduction* assertion
// directly below, which spells the shift as a literal `32` in order to state
// the old wire format independently — if `SHIFT` moved and that literal did
// not, the assertion below would be comparing against a word no writer
// produces, and would fail rather than silently agreeing.
const _: () = assert!(
    MappedSlot::SHIFT == 32,
    "the mapped-arguments wire format below is written against a literal 32",
);
const _: () = assert!(
    DescriptorWord::of_data(false, false, false)
        .with_flags(DescriptorFlags {
            array_own_property: false,
            mapped: Some(MappedSlot::new(7)),
        })
        .bits()
        == (ARGUMENTS_DESCRIPTOR_MAPPED | (7u64 << 32)),
    "DescriptorFlags must reproduce the mapped-arguments writer's \
     `ARGUMENTS_DESCRIPTOR_MAPPED | ((slot as i64) << 32)`",
);
const _: () = assert!(
    DescriptorWord::of_accessor(false, false)
        .with_flags(DescriptorFlags {
            array_own_property: true,
            mapped: None,
        })
        .bits()
        == (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR),
    "DescriptorFlags must reproduce the two hand-built \
     `ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR` words",
);
const _: () = assert!(
    DescriptorMask::ACCESSOR
        .union(DescriptorMask::WRITABLE)
        .bits()
        == DescriptorMask::ACCESSOR_OR_WRITABLE.bits(),
);
// A mask is not a word: the bit pattern 5 is reachable as a `DescriptorMask`
// and unreachable as a `DescriptorWord`. There is no constructor that produces
// it, which is the whole point, so this asserts the negation on the two that
// exist.
const _: () = assert!(
    DescriptorWord::of_accessor(true, true).bits() & DescriptorBit::Writable.word() == 0,
    "an accessor word can never carry [[Writable]]",
);
const _: () = assert!(
    DescriptorWord::of_data(true, true, true).bits() & DescriptorBit::Accessor.word() == 0,
    "a data word can never carry the accessor bit",
);
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
pub(crate) const OBJECT_INTERNAL_BRAND_PROMISE: u64 = 11;
pub(crate) const OBJECT_INTERNAL_BRAND_MAP: u64 = 12;
pub(crate) const OBJECT_INTERNAL_BRAND_SET: u64 = 13;
pub(crate) const OBJECT_INTERNAL_BRAND_MAP_ITERATOR: u64 = 14;
pub(crate) const OBJECT_INTERNAL_BRAND_SET_ITERATOR: u64 = 15;
pub(crate) const OBJECT_INTERNAL_BRAND_ARRAY_BUFFER: u64 = 16;
pub(crate) const OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER: u64 = 17;
pub(crate) const OBJECT_INTERNAL_BRAND_GENERATOR: u64 = 18;
#[allow(dead_code)]
pub(crate) const OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR: u64 = 19;
pub(crate) const OBJECT_INTERNAL_BRAND_TYPED_ARRAY_ITERATOR: u64 = 20;
pub(crate) const OBJECT_INTERNAL_BRAND_DATA_VIEW: u64 = 21;
pub(crate) const OBJECT_INTERNAL_BRAND_WEAK_MAP: u64 = 22;
pub(crate) const OBJECT_INTERNAL_BRAND_DATE: u64 = 23;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT: u64 = 24;
pub(crate) const OBJECT_INTERNAL_BRAND_WEAK_REF: u64 = 25;
pub(crate) const OBJECT_INTERNAL_BRAND_FINALIZATION_REGISTRY: u64 = 26;
pub(crate) const OBJECT_INTERNAL_BRAND_WEAK_SET: u64 = 27;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME: u64 = 28;
pub(crate) const OBJECT_INTERNAL_BRAND_ITERATOR_CONCAT_HELPER: u64 = 29;
pub(crate) const OBJECT_INTERNAL_BRAND_IMMUTABLE_PROTOTYPE: u64 = 30;
pub(crate) const OBJECT_INTERNAL_BRAND_INTL_LOCALE: u64 = 31;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE: u64 = 32;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_DURATION: u64 = 33;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME: u64 = 34;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME: u64 = 35;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH: u64 = 36;
pub(crate) const OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY: u64 = 37;
pub(crate) const OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT: u64 = 38;
/// The `[[AsyncDisposableState]]` brand. Every
/// `prototype/*/this-does-not-have-internal-asyncdisposablestate-throws.js`
/// case turns on this word being absent from an ordinary object, from
/// `AsyncDisposableStack.prototype` itself, and from the constructor.
pub(crate) const OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK: u64 = 39;
pub(crate) const GENERATOR_STATE_SUSPENDED_START: u64 = 0;
pub(crate) const GENERATOR_STATE_EXECUTING: u64 = 1;
pub(crate) const GENERATOR_STATE_COMPLETED: u64 = 2;
pub(crate) const GENERATOR_STATE_SUSPENDED_YIELD: u64 = 3;
pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING: u64 = u64::MAX;
pub(crate) const GENERATOR_RESUME_KIND_NORMAL: u64 = 0;
pub(crate) const GENERATOR_RESUME_KIND_RETURN: u64 = 1;
pub(crate) const GENERATOR_RESUME_KIND_THROW: u64 = 2;
pub(crate) const GENERATOR_DELEGATED_RESULT_AUX_FLAG: i64 = i64::MIN;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_SUSPENDED_START: u64 = 0;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_SUSPENDED_YIELD: u64 = 1;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_EXECUTING: u64 = 2;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_DRAINING_QUEUE: u64 = 3;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_COMPLETED: u64 = 4;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT: u64 = 5;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_STATE_INITIALIZING: u64 = u64::MAX;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_KIND_NORMAL: u64 = 0;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_KIND_RETURN: u64 = 1;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_KIND_THROW: u64 = 2;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_KIND_FULFILL: u64 = 3;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_KIND_REJECT: u64 = 4;
pub(crate) const ASYNC_GENERATOR_RETURN_VALUE_ALREADY_AWAITED: u64 = 1;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_IDLE: u64 = 0;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_RUNNING: u64 = 1;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_AWAIT: u64 = 2;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_YIELD: u64 = 3;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_COMPLETE: u64 = 4;
pub(crate) const ASYNC_GENERATOR_BODY_STATUS_THROW: u64 = 5;
pub(crate) const PROMISE_STATE_PENDING: u64 = 0;
pub(crate) const PROMISE_STATE_FULFILLED: u64 = 1;
pub(crate) const PROMISE_STATE_REJECTED: u64 = 2;
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
pub(crate) const FUNCTION_FLAG_GENERATOR: u64 = 1024;
pub(crate) const FUNCTION_FLAG_ASYNC: u64 = 2048;
pub(crate) const FUNCTION_FLAG_ASYNC_GENERATOR: u64 = 4096;
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
        record: "proxy-object-header",
        name: "handler_tag",
        offset: HEAP_PROXY_HANDLER_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "data",
        offset: HEAP_ARRAY_BUFFER_DATA_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "byte_length",
        offset: HEAP_ARRAY_BUFFER_BYTE_LENGTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "max_byte_length",
        offset: HEAP_ARRAY_BUFFER_MAX_BYTE_LENGTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "detach_key_tag",
        offset: HEAP_ARRAY_BUFFER_DETACH_KEY_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "detach_key_payload",
        offset: HEAP_ARRAY_BUFFER_DETACH_KEY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "array-buffer-object-header",
        name: "flags",
        offset: HEAP_ARRAY_BUFFER_FLAGS_OFFSET,
        width: 8,
        pointer: false,
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
        record: "data-view-object-header",
        name: "viewed_array_buffer",
        offset: HEAP_DATA_VIEW_VIEWED_BUFFER_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "data-view-object-header",
        name: "byte_offset",
        offset: HEAP_DATA_VIEW_BYTE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "data-view-object-header",
        name: "byte_length",
        offset: HEAP_DATA_VIEW_BYTE_LENGTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "data-view-object-header",
        name: "length_tracking",
        offset: HEAP_DATA_VIEW_LENGTH_TRACKING_OFFSET,
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
pub(crate) const HEAP_GENERATOR_OBJECT_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "generator-object",
        name: "state",
        offset: HEAP_GENERATOR_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "function",
        offset: HEAP_GENERATOR_FUNCTION_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "this_payload",
        offset: HEAP_GENERATOR_THIS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "this_tag",
        offset: HEAP_GENERATOR_THIS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "argc",
        offset: HEAP_GENERATOR_ARGC_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "argv",
        offset: HEAP_GENERATOR_ARGV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "resume_state",
        offset: HEAP_GENERATOR_RESUME_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "resume_payload",
        offset: HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "resume_tag",
        offset: HEAP_GENERATOR_RESUME_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "environment",
        offset: HEAP_GENERATOR_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "initialized",
        offset: HEAP_GENERATOR_INITIALIZED_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "resume_kind",
        offset: HEAP_GENERATOR_RESUME_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "pending_completion_head",
        offset: HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "pending_completion_depth",
        offset: HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "pending_completion_capacity",
        offset: HEAP_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "assignment_target_payload",
        offset: HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "assignment_target_tag",
        offset: HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "assignment_key_payload",
        offset: HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "assignment_key_tag",
        offset: HEAP_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-object",
        name: "delegate_record",
        offset: HEAP_GENERATOR_DELEGATE_RECORD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_GENERATOR_DELEGATE_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "iterator_payload",
        offset: HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "iterator_tag",
        offset: HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "next_payload",
        offset: HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "next_tag",
        offset: HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "pending_kind",
        offset: HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "pending_payload",
        offset: HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "pending_tag",
        offset: HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "async_iterator",
        offset: HEAP_GENERATOR_DELEGATE_ASYNC_ITERATOR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "awaiting_sync_value",
        offset: HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "result_done_payload",
        offset: HEAP_GENERATOR_DELEGATE_RESULT_DONE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "generator-delegate-record",
        name: "result_done_tag",
        offset: HEAP_GENERATOR_DELEGATE_RESULT_DONE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT: &[HeapLayoutSlot] = &[HeapLayoutSlot {
    record: "async-generator-object",
    name: "activation",
    offset: HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,
    width: 8,
    pointer: true,
}];

#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "queue_head",
        offset: HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "queue_tail",
        offset: HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "active_request",
        offset: HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "execution_state",
        offset: HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "function",
        offset: HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "function_environment",
        offset: HEAP_ASYNC_GENERATOR_FUNCTION_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "this_payload",
        offset: HEAP_ASYNC_GENERATOR_THIS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "this_tag",
        offset: HEAP_ASYNC_GENERATOR_THIS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "argc",
        offset: HEAP_ASYNC_GENERATOR_ARGC_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "argv",
        offset: HEAP_ASYNC_GENERATOR_ARGV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "resume_state",
        offset: HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "resume_payload",
        offset: HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "resume_tag",
        offset: HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "resume_kind",
        offset: HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "lexical_environment",
        offset: HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "pending_completion_head",
        offset: HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "pending_completion_depth",
        offset: HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "pending_completion_capacity",
        offset: HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "body_status",
        offset: HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "body_result_payload",
        offset: HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "body_result_tag",
        offset: HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "initialized",
        offset: HEAP_ASYNC_GENERATOR_INITIALIZED_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-activation",
        name: "delegate_record",
        offset: HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "completion_kind",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "completion_tag",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "completion_payload",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "promise_capability",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_CAPABILITY_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "promise_payload",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "promise_record",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-generator-request",
        name: "next",
        offset: HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_PENDING_COMPLETION_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "pending-completion-record",
        name: "next",
        offset: HEAP_PENDING_COMPLETION_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-completion-record",
        name: "payload",
        offset: HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "pending-completion-record",
        name: "tag",
        offset: HEAP_PENDING_COMPLETION_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "pending-completion-record",
        name: "kind",
        offset: HEAP_PENDING_COMPLETION_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "pending-completion-record",
        name: "aux",
        offset: HEAP_PENDING_COMPLETION_AUX_OFFSET,
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
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Map.prototype%",
        offset: HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Set.prototype%",
        offset: HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%MapIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_MAP_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%SetIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_SET_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%GeneratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%GeneratorFunction.prototype%",
        offset: HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%GeneratorFunction%",
        offset: HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncIteratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_ITERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncFunction.prototype%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncFunction%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_CONSTRUCTOR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncGeneratorPrototype%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncGeneratorFunction.prototype%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%AsyncGeneratorFunction%",
        offset: HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%WeakMap.prototype%",
        offset: HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%WeakRef.prototype%",
        offset: HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%FinalizationRegistry.prototype%",
        offset: HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%WeakSet.prototype%",
        offset: HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,
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
    HeapLayoutSlot {
        record: "promise-record",
        name: "unhandled_next",
        offset: HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_PROMISE_CAPABILITY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "promise_tag",
        offset: HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "promise_payload",
        offset: HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "resolve_tag",
        offset: HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "resolve_payload",
        offset: HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "reject_tag",
        offset: HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "promise-capability-record",
        name: "reject_payload",
        offset: HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_MAP_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "map-record",
        name: "entries_ptr",
        offset: HEAP_MAP_ENTRIES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "map-record",
        name: "entries_len",
        offset: HEAP_MAP_ENTRIES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-record",
        name: "entries_cap",
        offset: HEAP_MAP_ENTRIES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-record",
        name: "live_count",
        offset: HEAP_MAP_LIVE_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_MAP_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "map-entry",
        name: "present",
        offset: HEAP_MAP_ENTRY_PRESENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-entry",
        name: "key_tag",
        offset: HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-entry",
        name: "key_payload",
        offset: HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "map-entry",
        name: "value_tag",
        offset: HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-entry",
        name: "value_payload",
        offset: HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_MAP_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "weak-map-record",
        name: "entries_ptr",
        offset: HEAP_WEAK_MAP_ENTRIES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "weak-map-record",
        name: "entries_len",
        offset: HEAP_WEAK_MAP_ENTRIES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-record",
        name: "entries_cap",
        offset: HEAP_WEAK_MAP_ENTRIES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-record",
        name: "live_count",
        offset: HEAP_WEAK_MAP_LIVE_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_MAP_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "weak-map-entry",
        name: "present",
        offset: HEAP_WEAK_MAP_ENTRY_PRESENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-entry",
        name: "key_tag",
        offset: HEAP_WEAK_MAP_ENTRY_KEY_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-entry",
        name: "key_payload",
        offset: HEAP_WEAK_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-entry",
        name: "value_tag",
        offset: HEAP_WEAK_MAP_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-map-entry",
        name: "value_payload",
        offset: HEAP_WEAK_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-instant-record",
        name: "epoch_nanoseconds_tag",
        offset: HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-instant-record",
        name: "epoch_nanoseconds_payload",
        offset: HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

pub(crate) const HEAP_INTL_LOCALE_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "intl-locale-record",
        name: "tag_payload",
        offset: HEAP_INTL_LOCALE_TAG_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-locale-record",
        name: "language_payload",
        offset: HEAP_INTL_LOCALE_LANGUAGE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-locale-record",
        name: "script_payload",
        offset: HEAP_INTL_LOCALE_SCRIPT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-locale-record",
        name: "region_payload",
        offset: HEAP_INTL_LOCALE_REGION_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-locale-record",
        name: "base_name_payload",
        offset: HEAP_INTL_LOCALE_BASE_NAME_OFFSET,
        width: 8,
        pointer: true,
    },
];

pub(crate) const HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "locale_payload",
        offset: HEAP_INTL_DTF_LOCALE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "calendar_payload",
        offset: HEAP_INTL_DTF_CALENDAR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "numbering_system_payload",
        offset: HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "time_zone_payload",
        offset: HEAP_INTL_DTF_TIME_ZONE_OFFSET,
        width: 8,
        pointer: true,
    },
    // Sits beside the identifier it belongs to rather than at the end of the
    // record, because the two are written and read as one value.
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "time_zone_offset_minutes",
        offset: HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "time_zone_gmt_name_payload",
        offset: HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "hour_cycle_code",
        offset: HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "weekday_code",
        offset: HEAP_INTL_DTF_WEEKDAY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "era_code",
        offset: HEAP_INTL_DTF_ERA_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "year_code",
        offset: HEAP_INTL_DTF_YEAR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "month_code",
        offset: HEAP_INTL_DTF_MONTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "day_code",
        offset: HEAP_INTL_DTF_DAY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "day_period_code",
        offset: HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "hour_code",
        offset: HEAP_INTL_DTF_HOUR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "minute_code",
        offset: HEAP_INTL_DTF_MINUTE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "second_code",
        offset: HEAP_INTL_DTF_SECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "fractional_second_digits",
        offset: HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "time_zone_name_code",
        offset: HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "date_style_code",
        offset: HEAP_INTL_DTF_DATE_STYLE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "time_style_code",
        offset: HEAP_INTL_DTF_TIME_STYLE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "hour12_code",
        offset: HEAP_INTL_DTF_HOUR12_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "bound_format_payload",
        offset: HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "intl-date-time-format-record",
        name: "need_defaults",
        offset: HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "epoch_nanoseconds_tag",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "epoch_nanoseconds_payload",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "time_zone_tag",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "time_zone_payload",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "calendar_tag",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-zoned-date-time-record",
        name: "calendar_payload",
        offset: HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-plain-date-record",
        name: "iso_year",
        offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-record",
        name: "iso_month",
        offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-record",
        name: "iso_day",
        offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-record",
        name: "calendar_payload",
        offset: HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "iso_year",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "iso_month",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "iso_day",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "hour",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "minute",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "second",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "millisecond",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "microsecond",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "nanosecond",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-date-time-record",
        name: "calendar_payload",
        offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "hour",
        offset: HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "minute",
        offset: HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "second",
        offset: HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "millisecond",
        offset: HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "microsecond",
        offset: HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-plain-time-record",
        name: "nanosecond",
        offset: HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TEMPORAL_DURATION_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "years",
        offset: HEAP_TEMPORAL_DURATION_YEARS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "months",
        offset: HEAP_TEMPORAL_DURATION_MONTHS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "weeks",
        offset: HEAP_TEMPORAL_DURATION_WEEKS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "days",
        offset: HEAP_TEMPORAL_DURATION_DAYS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "hours",
        offset: HEAP_TEMPORAL_DURATION_HOURS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "minutes",
        offset: HEAP_TEMPORAL_DURATION_MINUTES_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "seconds",
        offset: HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "milliseconds",
        offset: HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "microseconds",
        offset: HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "temporal-duration-record",
        name: "nanoseconds",
        offset: HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_REF_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "weak-ref-record",
        name: "target_tag",
        offset: HEAP_WEAK_REF_TARGET_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-ref-record",
        name: "target_payload",
        offset: HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "finalization-registry-record",
        name: "cleanup_callback_tag",
        offset: HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-record",
        name: "cleanup_callback_payload",
        offset: HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "finalization-registry-record",
        name: "cells_ptr",
        offset: HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "finalization-registry-record",
        name: "cells_len",
        offset: HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-record",
        name: "cells_cap",
        offset: HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "present",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_PRESENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "target_tag",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "target_payload",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "holdings_tag",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "holdings_payload",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "unregister_token_tag",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "finalization-registry-cell",
        name: "unregister_token_payload",
        offset: HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "async-disposable-stack-record",
        name: "state",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-record",
        name: "entries_ptr",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-record",
        name: "entries_len",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-record",
        name: "entries_cap",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
];

/// Both the resource value and the dispose method are strongly reachable: an
/// `AsyncDisposableStack` keeps every registered resource alive until disposal,
/// which is the whole point of the type and the opposite of a
/// `FinalizationRegistry` cell.
#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "async-disposable-stack-entry",
        name: "kind",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-entry",
        name: "value_tag",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-entry",
        name: "value_payload",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-entry",
        name: "method_tag",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-disposable-stack-entry",
        name: "method_payload",
        offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_MAP_ITERATOR_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "map-iterator-record",
        name: "map_payload",
        offset: HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "map-iterator-record",
        name: "next_index",
        offset: HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-iterator-record",
        name: "kind",
        offset: HEAP_MAP_ITERATOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "map-iterator-record",
        name: "cursor_state",
        offset: HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_SET_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "set-record",
        name: "entries_ptr",
        offset: HEAP_SET_ENTRIES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "set-record",
        name: "entries_len",
        offset: HEAP_SET_ENTRIES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-record",
        name: "entries_cap",
        offset: HEAP_SET_ENTRIES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-record",
        name: "live_count",
        offset: HEAP_SET_LIVE_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_SET_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "set-entry",
        name: "present",
        offset: HEAP_SET_ENTRY_PRESENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-entry",
        name: "value_tag",
        offset: HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-entry",
        name: "value_payload",
        offset: HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_SET_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "weak-set-record",
        name: "entries_ptr",
        offset: HEAP_WEAK_SET_ENTRIES_PTR_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "weak-set-record",
        name: "entries_len",
        offset: HEAP_WEAK_SET_ENTRIES_LEN_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-set-record",
        name: "entries_cap",
        offset: HEAP_WEAK_SET_ENTRIES_CAP_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-set-record",
        name: "live_count",
        offset: HEAP_WEAK_SET_LIVE_COUNT_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_WEAK_SET_ENTRY_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "weak-set-entry",
        name: "present",
        offset: HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-set-entry",
        name: "value_tag",
        offset: HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "weak-set-entry",
        name: "value_payload",
        offset: HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_SET_ITERATOR_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "set-iterator-record",
        name: "set_payload",
        offset: HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "set-iterator-record",
        name: "next_index",
        offset: HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-iterator-record",
        name: "kind",
        offset: HEAP_SET_ITERATOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "set-iterator-record",
        name: "cursor_state",
        offset: HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "typed-array-iterator-record",
        name: "typed_array_payload",
        offset: HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "typed-array-iterator-record",
        name: "next_index",
        offset: HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-iterator-record",
        name: "kind",
        offset: HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "typed-array-iterator-record",
        name: "done",
        offset: HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET,
        width: 8,
        pointer: false,
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
        name: "callback_kind",
        offset: HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
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
pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "state",
        offset: HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "address",
        offset: HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "promise_record",
        offset: HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "deadline_nanos",
        offset: HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "next",
        offset: HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "atomics-async-waiter",
        name: "host_id",
        offset: HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
        width: 8,
        pointer: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ARRAY_BUFFER_BACKING_STORE_LAYOUT: HeapByteSpanLayout = HeapByteSpanLayout {
    record: "array-buffer-backing-store",
    length_source: "array-buffer-object-header.max_byte_length",
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
        key: "$IteratorZipKeys",
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
pub(crate) const HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatIterables",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatMethods",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatCurrentIterator",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatCurrentNext",
        strong_reference: true,
        scans_target: true,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatIndex",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatActive",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatDone",
        strong_reference: false,
        scans_target: false,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatExecuting",
        strong_reference: false,
        scans_target: false,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_NAMED_SLOT_LAYOUTS: &[&[HeapNamedSlot]] = &[
    HEAP_ARRAY_ITERATOR_NAMED_SLOTS,
    HEAP_STRING_ITERATOR_NAMED_SLOTS,
    HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS,
    HEAP_ITERATOR_HELPER_NAMED_SLOTS,
    HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS,
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
        name: "value",
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
    pub(crate) fn emit_alloc_bigint_literal(
        &mut self,
        sign: i64,
        limbs: &[u64],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if limbs.is_empty() {
            return Err(EmitError::unsupported(
                "heap-backed BigInt literal requires at least one magnitude limb",
            ));
        }
        let limb_count = u64::try_from(limbs.len()).map_err(|_| {
            EmitError::unsupported("heap-backed BigInt literal has too many magnitude limbs")
        })?;
        let limbs_size = limb_count.checked_mul(8).ok_or_else(|| {
            EmitError::unsupported("heap-backed BigInt literal limb storage exceeds u64")
        })?;
        let record_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.emit_heap_alloc_const(limbs_size, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        for (index, limb) in (0_u64..).zip(limbs.iter().copied()) {
            self.store_i64_const_at_offset(limbs_local, index * 8, limb, function);
        }
        self.store_i64_const_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign as u64,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_CAP_OFFSET,
            limb_count,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_local));

        self.release_temp_local(limbs_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_one_limb_bigint(
        &mut self,
        sign: i64,
        magnitude_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.emit_heap_alloc_const(8, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        self.store_i64_const_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign as u64,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_const_at_offset(record_local, HEAP_BIGINT_LIMBS_LEN_OFFSET, 1, function);
        self.store_i64_const_at_offset(record_local, HEAP_BIGINT_LIMBS_CAP_OFFSET, 1, function);
        function.instruction(&Instruction::LocalGet(record_local));

        self.release_temp_local(limbs_local);
        self.release_temp_local(record_local);
        Ok(())
    }

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
                "unsupported in lila wasm-aot first slice: heap value without memory",
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
        Self::memarg64_in(0, offset)
    }

    pub(crate) const fn memarg64_in(memory_index: u32, offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 3,
            memory_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::lower;
    use std::collections::BTreeSet;
    use wasmparser::{Operator, Parser, Payload};

    #[test]
    fn promise_reaction_wire_domains_and_realm_policies_are_stable() {
        assert_eq!(
            PromiseReactionType::ALL.map(PromiseReactionType::word),
            [1, 2]
        );
        assert_eq!(
            PromiseReactionType::ALL.map(PromiseReactionType::is_rejected),
            [false, true]
        );
        assert_eq!(
            PromiseReactionCallbackKind::ALL.map(PromiseReactionCallbackKind::word),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            PromiseReactionCallbackKind::ALL.map(PromiseReactionCallbackKind::realm_source),
            [
                PromiseReactionRealmSource::HandlerOrNull,
                PromiseReactionRealmSource::Captured,
                PromiseReactionRealmSource::Captured,
                PromiseReactionRealmSource::Captured,
                PromiseReactionRealmSource::Captured,
                PromiseReactionRealmSource::Captured,
            ]
        );
    }

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
        assert_eq!(HEAP_REALM_INTRINSICS_RECORD_SIZE, 344);
        assert_eq!(HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET, 320);
        assert_eq!(
            HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,
            328
        );
        assert_eq!(HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET, 336);
        assert_eq!(HEAP_PROMISE_RECORD_SIZE, 72);
        assert_eq!(HEAP_PROMISE_CAPABILITY_RECORD_SIZE, 48);
        assert_eq!(HEAP_PROMISE_REACTION_RECORD_SIZE, 56);
        assert_eq!(HEAP_PENDING_JOB_RECORD_SIZE, 56);
        assert_eq!(HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE, 48);
        assert_eq!(HEAP_PENDING_COMPLETION_RECORD_SIZE, 40);
        assert_eq!(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE, 184);
        assert_eq!(HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE, 56);
        assert_eq!(HEAP_MAP_RECORD_SIZE, 32);
        assert_eq!(HEAP_MAP_ENTRY_SIZE, 40);
        assert_eq!(HEAP_WEAK_MAP_RECORD_SIZE, 32);
        assert_eq!(HEAP_WEAK_MAP_ENTRY_SIZE, 40);
        assert_eq!(HEAP_WEAK_SET_RECORD_SIZE, 32);
        assert_eq!(HEAP_WEAK_SET_ENTRY_SIZE, 24);
        assert_eq!(HEAP_WEAK_REF_RECORD_SIZE, 16);
        assert_eq!(HEAP_FINALIZATION_REGISTRY_RECORD_SIZE, 40);
        assert_eq!(HEAP_FINALIZATION_REGISTRY_CELL_SIZE, 56);
        assert_eq!(HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE, 32);
        assert_eq!(HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE, 40);
        assert_eq!(HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE, 48);
        assert_eq!(HEAP_MAP_ITERATOR_RECORD_SIZE, 32);
        assert_eq!(HEAP_SET_RECORD_SIZE, 32);
        assert_eq!(HEAP_SET_ENTRY_SIZE, 24);
        assert_eq!(HEAP_SET_ITERATOR_RECORD_SIZE, 32);
        assert_eq!(HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE, 32);
    }

    #[test]
    fn heap_layout_registry_has_no_slot_collisions() {
        assert_layout(HEAP_OBJECT_HEADER_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(HEAP_GENERATOR_OBJECT_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(
            HEAP_GENERATOR_DELEGATE_RECORD_LAYOUT,
            HEAP_GENERATOR_DELEGATE_RECORD_SIZE,
        );
        assert_layout(HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(
            HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT,
            HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE,
        );
        assert_layout(
            HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT,
            HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE,
        );
        assert_layout(
            HEAP_PENDING_COMPLETION_LAYOUT,
            HEAP_PENDING_COMPLETION_RECORD_SIZE,
        );
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
            HEAP_PROMISE_CAPABILITY_LAYOUT,
            HEAP_PROMISE_CAPABILITY_RECORD_SIZE,
        );
        assert_layout(HEAP_MAP_RECORD_LAYOUT, HEAP_MAP_RECORD_SIZE);
        assert_layout(HEAP_MAP_ENTRY_LAYOUT, HEAP_MAP_ENTRY_SIZE);
        assert_layout(HEAP_WEAK_MAP_RECORD_LAYOUT, HEAP_WEAK_MAP_RECORD_SIZE);
        assert_layout(HEAP_WEAK_MAP_ENTRY_LAYOUT, HEAP_WEAK_MAP_ENTRY_SIZE);
        assert_layout(HEAP_WEAK_SET_RECORD_LAYOUT, HEAP_WEAK_SET_RECORD_SIZE);
        assert_layout(HEAP_WEAK_SET_ENTRY_LAYOUT, HEAP_WEAK_SET_ENTRY_SIZE);
        assert_layout(HEAP_WEAK_REF_RECORD_LAYOUT, HEAP_WEAK_REF_RECORD_SIZE);
        assert_layout(
            HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT,
            HEAP_FINALIZATION_REGISTRY_RECORD_SIZE,
        );
        assert_layout(
            HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT,
            HEAP_FINALIZATION_REGISTRY_CELL_SIZE,
        );
        assert_layout(
            HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT,
            HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE,
        );
        assert_layout(
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT,
            HEAP_TEMPORAL_INSTANT_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT,
            HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT,
            HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_DURATION_RECORD_LAYOUT,
            HEAP_TEMPORAL_DURATION_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT,
            HEAP_TEMPORAL_PLAIN_TIME_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT,
            HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_SIZE,
        );
        assert_layout(HEAP_INTL_LOCALE_RECORD_LAYOUT, HEAP_INTL_LOCALE_RECORD_SIZE);
        assert_layout(
            HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT,
            HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE,
        );
        assert_layout(
            HEAP_MAP_ITERATOR_RECORD_LAYOUT,
            HEAP_MAP_ITERATOR_RECORD_SIZE,
        );
        assert_layout(HEAP_SET_RECORD_LAYOUT, HEAP_SET_RECORD_SIZE);
        assert_layout(HEAP_SET_ENTRY_LAYOUT, HEAP_SET_ENTRY_SIZE);
        assert_layout(
            HEAP_SET_ITERATOR_RECORD_LAYOUT,
            HEAP_SET_ITERATOR_RECORD_SIZE,
        );
        assert_layout(
            HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT,
            HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE,
        );
        assert_layout(
            HEAP_PROMISE_REACTION_LAYOUT,
            HEAP_PROMISE_REACTION_RECORD_SIZE,
        );
        assert_layout(HEAP_PENDING_JOB_LAYOUT, HEAP_PENDING_JOB_RECORD_SIZE);
        assert_layout(
            HEAP_ATOMICS_ASYNC_WAITER_LAYOUT,
            HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE,
        );
        assert_layout(
            HEAP_ENVIRONMENT_LAYOUT,
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_SIZE,
        );
    }

    #[test]
    fn heap_layout_registry_marks_gc_pointer_fields() {
        let pointer_slots = HEAP_OBJECT_HEADER_LAYOUT
            .iter()
            .chain(HEAP_GENERATOR_OBJECT_LAYOUT.iter())
            .chain(HEAP_GENERATOR_DELEGATE_RECORD_LAYOUT.iter())
            .chain(HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT.iter())
            .chain(HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT.iter())
            .chain(HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT.iter())
            .chain(HEAP_PENDING_COMPLETION_LAYOUT.iter())
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
            .chain(HEAP_PROMISE_CAPABILITY_LAYOUT.iter())
            .chain(HEAP_MAP_RECORD_LAYOUT.iter())
            .chain(HEAP_MAP_ENTRY_LAYOUT.iter())
            .chain(HEAP_WEAK_REF_RECORD_LAYOUT.iter())
            .chain(HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT.iter())
            .chain(HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT.iter())
            .chain(HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT.iter())
            .chain(HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_DURATION_RECORD_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT.iter())
            .chain(HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT.iter())
            .chain(HEAP_INTL_LOCALE_RECORD_LAYOUT.iter())
            .chain(HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT.iter())
            .chain(HEAP_MAP_ITERATOR_RECORD_LAYOUT.iter())
            .chain(HEAP_SET_RECORD_LAYOUT.iter())
            .chain(HEAP_SET_ENTRY_LAYOUT.iter())
            .chain(HEAP_SET_ITERATOR_RECORD_LAYOUT.iter())
            .chain(HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT.iter())
            .chain(HEAP_PROMISE_REACTION_LAYOUT.iter())
            .chain(HEAP_PENDING_JOB_LAYOUT.iter())
            .chain(HEAP_ATOMICS_ASYNC_WAITER_LAYOUT.iter())
            .chain(HEAP_ENVIRONMENT_LAYOUT.iter())
            .filter(|slot| slot.pointer)
            .count();
        assert!(pointer_slots >= 64, "expected GC-visible pointer slots");
        assert!(HEAP_OBJECT_HEADER_LAYOUT.iter().any(|slot| {
            slot.name == "prototype_payload" && slot.offset == HEAP_PROTOTYPE_OFFSET && slot.pointer
        }));
        assert!(HEAP_OBJECT_HEADER_LAYOUT.iter().any(|slot| {
            slot.record == "proxy-object-header"
                && slot.name == "handler_tag"
                && slot.offset == HEAP_PROXY_HANDLER_TAG_OFFSET
                && !slot.pointer
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
            slot.name == "%Map.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%Set.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET
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
    fn async_generator_records_expose_queue_activation_and_promise_edges_to_gc() {
        assert_eq!(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE, 184);
        assert_eq!(HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE, 56);
        assert_ne!(
            OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR,
            OBJECT_INTERNAL_BRAND_GENERATOR
        );
        assert_eq!(
            [
                ASYNC_GENERATOR_STATE_SUSPENDED_START,
                ASYNC_GENERATOR_STATE_SUSPENDED_YIELD,
                ASYNC_GENERATOR_STATE_EXECUTING,
                ASYNC_GENERATOR_STATE_DRAINING_QUEUE,
                ASYNC_GENERATOR_STATE_COMPLETED,
                ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT,
            ],
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(ASYNC_GENERATOR_RESUME_STATE_INITIALIZING, u64::MAX);
        assert_eq!(
            [
                ASYNC_GENERATOR_BODY_STATUS_IDLE,
                ASYNC_GENERATOR_BODY_STATUS_RUNNING,
                ASYNC_GENERATOR_BODY_STATUS_AWAIT,
                ASYNC_GENERATOR_BODY_STATUS_YIELD,
                ASYNC_GENERATOR_BODY_STATUS_COMPLETE,
                ASYNC_GENERATOR_BODY_STATUS_THROW,
            ],
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            [
                ASYNC_GENERATOR_RESUME_KIND_NORMAL,
                ASYNC_GENERATOR_RESUME_KIND_RETURN,
                ASYNC_GENERATOR_RESUME_KIND_THROW,
                ASYNC_GENERATOR_RESUME_KIND_FULFILL,
                ASYNC_GENERATOR_RESUME_KIND_REJECT,
            ],
            [0, 1, 2, 3, 4]
        );

        assert!(HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "activation"
                && slot.offset == HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET
                && slot.pointer
        }));
        for name in [
            "queue_head",
            "queue_tail",
            "active_request",
            "function",
            "function_environment",
            "this_payload",
            "argv",
            "resume_payload",
            "lexical_environment",
            "pending_completion_head",
            "body_result_payload",
            "delegate_record",
        ] {
            assert!(
                HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT
                    .iter()
                    .any(|slot| slot.name == name && slot.pointer),
                "async-generator activation must trace {name}"
            );
        }
        for name in [
            "execution_state",
            "this_tag",
            "argc",
            "resume_state",
            "resume_tag",
            "resume_kind",
            "pending_completion_depth",
            "pending_completion_capacity",
            "body_status",
            "body_result_tag",
            "initialized",
        ] {
            assert!(
                HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT
                    .iter()
                    .any(|slot| slot.name == name && !slot.pointer),
                "async-generator activation must not trace scalar {name}"
            );
        }

        for name in [
            "completion_payload",
            "promise_capability",
            "promise_payload",
            "promise_record",
            "next",
        ] {
            assert!(
                HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT
                    .iter()
                    .any(|slot| slot.name == name && slot.pointer),
                "async-generator request must trace {name}"
            );
        }
        assert!(HEAP_PROMISE_REACTION_LAYOUT.iter().any(|slot| {
            slot.name == "callback_kind"
                && slot.offset == HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET
                && !slot.pointer
        }));
        for name in ["completion_kind", "completion_tag"] {
            assert!(
                HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT
                    .iter()
                    .any(|slot| slot.name == name && !slot.pointer),
                "async-generator request must not trace scalar {name}"
            );
        }

        for name in ["promise_payload", "resolve_payload", "reject_payload"] {
            assert!(
                HEAP_PROMISE_CAPABILITY_LAYOUT
                    .iter()
                    .any(|slot| slot.name == name && slot.pointer),
                "Promise capability must trace {name}"
            );
        }
        assert!(HEAP_PENDING_COMPLETION_LAYOUT.iter().any(|slot| {
            slot.name == "next"
                && slot.offset == HEAP_PENDING_COMPLETION_NEXT_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_PENDING_COMPLETION_LAYOUT.iter().any(|slot| {
            slot.name == "payload"
                && slot.offset == HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET
                && slot.pointer
        }));
    }

    #[test]
    fn array_buffer_state_uses_a_private_brand_selected_header_record() {
        let array_buffer_slots = HEAP_OBJECT_HEADER_LAYOUT
            .iter()
            .filter(|slot| slot.record == "array-buffer-object-header")
            .collect::<Vec<_>>();
        assert_eq!(array_buffer_slots.len(), 6);
        assert!(array_buffer_slots.iter().any(|slot| {
            slot.name == "data" && slot.offset == HEAP_ARRAY_BUFFER_DATA_OFFSET && slot.pointer
        }));
        assert!(array_buffer_slots.iter().any(|slot| {
            slot.name == "detach_key_payload"
                && slot.offset == HEAP_ARRAY_BUFFER_DETACH_KEY_PAYLOAD_OFFSET
                && slot.pointer
        }));
        assert!(array_buffer_slots.iter().any(|slot| {
            slot.name == "flags" && slot.offset == HEAP_ARRAY_BUFFER_FLAGS_OFFSET && !slot.pointer
        }));
        assert_eq!(
            ARRAY_BUFFER_FLAG_RESIZABLE
                | ARRAY_BUFFER_FLAG_SHARED
                | ARRAY_BUFFER_FLAG_IMMUTABLE
                | ARRAY_BUFFER_FLAG_DETACHED,
            15
        );
    }

    #[test]
    fn data_view_state_uses_a_private_brand_selected_header_record() {
        let data_view_slots = HEAP_OBJECT_HEADER_LAYOUT
            .iter()
            .filter(|slot| slot.record == "data-view-object-header")
            .collect::<Vec<_>>();
        assert_eq!(data_view_slots.len(), 4);
        assert!(data_view_slots.iter().any(|slot| {
            slot.name == "viewed_array_buffer"
                && slot.offset == HEAP_DATA_VIEW_VIEWED_BUFFER_OFFSET
                && slot.pointer
        }));
        assert!(data_view_slots.iter().any(|slot| {
            slot.name == "byte_offset" && slot.offset == HEAP_DATA_VIEW_BYTE_OFFSET && !slot.pointer
        }));
        assert!(data_view_slots.iter().any(|slot| {
            slot.name == "byte_length"
                && slot.offset == HEAP_DATA_VIEW_BYTE_LENGTH_OFFSET
                && !slot.pointer
        }));
        assert!(data_view_slots.iter().any(|slot| {
            slot.name == "length_tracking"
                && slot.offset == HEAP_DATA_VIEW_LENGTH_TRACKING_OFFSET
                && !slot.pointer
        }));
    }

    #[test]
    fn heap_named_slot_registry_marks_iterator_references() {
        for layout in HEAP_NAMED_SLOT_LAYOUTS {
            assert_named_slots(layout);
        }
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
            "$IteratorZipKeys",
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
    fn iterator_concat_state_slots_have_expected_gc_edges() {
        for key in [
            "$IteratorConcatIterables",
            "$IteratorConcatMethods",
            "$IteratorConcatCurrentIterator",
            "$IteratorConcatCurrentNext",
        ] {
            assert!(HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS
                .iter()
                .any(|slot| { slot.key == key && slot.strong_reference && slot.scans_target }));
        }
        for key in [
            "$IteratorConcatIndex",
            "$IteratorConcatActive",
            "$IteratorConcatDone",
            "$IteratorConcatExecuting",
        ] {
            assert!(HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS
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
            slot.record == "weak-set-entry"
                && slot.name == "value"
                && slot.kind == HeapWeakEdgeKind::EphemeronKey
                && !slot.keeps_target_alive
        }));
        assert!(HEAP_WEAK_SET_ENTRY_LAYOUT
            .iter()
            .find(|slot| slot.name == "value_payload")
            .is_some_and(|slot| !slot.pointer));
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
    fn weak_map_entries_are_ephemerons_not_strong_heap_edges() {
        assert!(HEAP_WEAK_MAP_ENTRY_LAYOUT.iter().all(|slot| !slot.pointer));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-map-entry"
                && slot.name == "key"
                && slot.kind == HeapWeakEdgeKind::EphemeronKey
        }));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-map-entry"
                && slot.name == "value"
                && slot.kind == HeapWeakEdgeKind::EphemeronValue
        }));
    }

    #[test]
    fn weak_ref_target_is_not_a_strong_heap_edge() {
        assert!(HEAP_WEAK_REF_RECORD_LAYOUT.iter().all(|slot| !slot.pointer));
        assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
            slot.record == "weak-ref-record"
                && slot.name == "target"
                && slot.kind == HeapWeakEdgeKind::WeakTarget
                && !slot.keeps_target_alive
        }));
    }

    #[test]
    fn finalization_registry_cells_keep_only_holdings_strongly_reachable() {
        assert!(HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT
            .iter()
            .any(|slot| { slot.name == "holdings_payload" && slot.pointer }));
        assert!(HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT.iter().all(|slot| {
            !matches!(slot.name, "target_payload" | "unregister_token_payload") || !slot.pointer
        }));
        for (name, kind, keeps_target_alive) in [
            ("target", HeapWeakEdgeKind::WeakTarget, false),
            ("holdings", HeapWeakEdgeKind::FinalizerHoldings, true),
            ("unregister-token", HeapWeakEdgeKind::FinalizerToken, false),
        ] {
            assert!(HEAP_WEAK_EDGE_SLOTS.iter().any(|slot| {
                slot.record == "finalization-registry-cell"
                    && slot.name == name
                    && slot.kind == kind
                    && slot.keeps_target_alive == keeps_target_alive
            }));
        }
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
            "array-buffer-object-header.max_byte_length"
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
            HEAP_MAP_RECORD_SIZE,
            HEAP_MAP_ENTRY_SIZE,
            HEAP_MAP_ITERATOR_RECORD_SIZE,
            HEAP_SET_RECORD_SIZE,
            HEAP_SET_ENTRY_SIZE,
            HEAP_SET_ITERATOR_RECORD_SIZE,
            HEAP_BOUND_FUNCTION_RECORD_SIZE,
            HEAP_REALM_RECORD_SIZE,
            HEAP_REALM_INTRINSICS_RECORD_SIZE,
            HEAP_STRING_RECORD_SIZE,
            HEAP_BIGINT_RECORD_SIZE,
            HEAP_SYMBOL_RECORD_SIZE,
            HEAP_PROMISE_RECORD_SIZE,
            HEAP_PROMISE_CAPABILITY_RECORD_SIZE,
            HEAP_PROMISE_REACTION_RECORD_SIZE,
            HEAP_PENDING_JOB_RECORD_SIZE,
            HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE,
            HEAP_PENDING_COMPLETION_RECORD_SIZE,
            HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE,
            HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE,
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
