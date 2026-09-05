use lila_ir::ValueKind;

#[cfg(test)]
use super::heap_async_disposable_stack_entry_layout::{
    AsyncDisposableStackEntryHeapSlot, HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT,
};
#[cfg(test)]
use super::heap_async_disposable_stack_record_layout::{
    AsyncDisposableStackRecordHeapSlot, HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_async_generator_object_layout::{
    AsyncGeneratorObjectHeapSlot, HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT,
};
#[cfg(test)]
use super::heap_atomics_async_waiter_layout::{
    AtomicsAsyncWaiterHeapSlot, HEAP_ATOMICS_ASYNC_WAITER_LAYOUT,
};
#[cfg(test)]
use super::heap_bigint_layout::{BigIntHeapSlot, HEAP_BIGINT_LAYOUT};
#[cfg(test)]
use super::heap_bound_function_layout::{BoundFunctionHeapSlot, HEAP_BOUND_FUNCTION_LAYOUT};
#[cfg(test)]
use super::heap_class_function_context_layout::{
    ClassFunctionContextHeapSlot, HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT,
};
#[cfg(test)]
use super::heap_collector_phases::{RequiredHeapCollectorPhase, REQUIRED_HEAP_COLLECTOR_PHASES};
#[cfg(test)]
use super::heap_collector_policy::HeapCollectorPolicy;
use super::heap_collector_policy::HEAP_COLLECTOR_POLICY;
#[cfg(test)]
use super::heap_disposable_stack_entry_layout::{
    DisposableStackEntryHeapSlot, HEAP_DISPOSABLE_STACK_ENTRY_LAYOUT,
};
#[cfg(test)]
use super::heap_disposable_stack_record_layout::{
    DisposableStackRecordHeapSlot, HEAP_DISPOSABLE_STACK_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_environment_layout::{EnvironmentHeapSlot, HEAP_ENVIRONMENT_LAYOUT};
#[cfg(test)]
use super::heap_finalization_registry_cell_layout::{
    FinalizationRegistryCellHeapSlot, HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT,
};
#[cfg(test)]
use super::heap_finalization_registry_record_layout::{
    FinalizationRegistryRecordHeapSlot, HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_host_boundary::{HeapHostBoundaryPolicy, HEAP_HOST_BOUNDARY_POLICY};
#[cfg(test)]
use super::heap_intl_date_time_format_layout::{
    IntlDateTimeFormatHeapSlot, HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_intl_locale_layout::{IntlLocaleHeapSlot, HEAP_INTL_LOCALE_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_map_entry_layout::{MapEntryHeapSlot, HEAP_MAP_ENTRY_LAYOUT};
#[cfg(test)]
use super::heap_map_iterator_layout::{MapIteratorHeapSlot, HEAP_MAP_ITERATOR_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_map_record_layout::{MapRecordHeapSlot, HEAP_MAP_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_object_entry_layout::{ObjectEntryHeapSlot, HEAP_OBJECT_ENTRY_LAYOUT};
#[cfg(test)]
use super::heap_pending_completion_layout::{
    PendingCompletionHeapSlot, HEAP_PENDING_COMPLETION_LAYOUT,
};
#[cfg(test)]
use super::heap_pending_job_layout::{PendingJobHeapSlot, HEAP_PENDING_JOB_LAYOUT};
#[cfg(test)]
use super::heap_private_element_entry_layout::{
    PrivateElementEntryHeapSlot, HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT,
};
#[cfg(test)]
use super::heap_private_environment_layout::{PrivateEnvironmentHeapSlot, HEAP_PRIVATE_ENV_LAYOUT};
#[cfg(test)]
use super::heap_promise_capability_layout::{
    PromiseCapabilityHeapSlot, HEAP_PROMISE_CAPABILITY_LAYOUT,
};
#[cfg(test)]
use super::heap_promise_reaction_layout::{PromiseReactionHeapSlot, HEAP_PROMISE_REACTION_LAYOUT};
#[cfg(test)]
use super::heap_realm_record_layout::{RealmRecordHeapSlot, HEAP_REALM_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_root_sources::{HeapRootKind, HeapRootSource, HEAP_ROOT_SOURCES};
#[cfg(test)]
use super::heap_set_entry_layout::{SetEntryHeapSlot, HEAP_SET_ENTRY_LAYOUT};
#[cfg(test)]
use super::heap_set_iterator_layout::{SetIteratorHeapSlot, HEAP_SET_ITERATOR_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_set_record_layout::{SetRecordHeapSlot, HEAP_SET_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_side_storage::{LinearSideStorage, LinearSideStorageElement, LINEAR_SIDE_STORAGES};
#[cfg(test)]
use super::heap_string_layout::{StringHeapSlot, HEAP_STRING_LAYOUT};
#[cfg(test)]
use super::heap_symbol_layout::{SymbolHeapSlot, HEAP_SYMBOL_LAYOUT};
#[cfg(test)]
use super::heap_temporal_duration_layout::{
    TemporalDurationHeapSlot, HEAP_TEMPORAL_DURATION_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_temporal_instant_layout::{
    TemporalInstantHeapSlot, HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_temporal_plain_date_layout::{
    TemporalPlainDateHeapSlot, HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_temporal_plain_date_time_layout::{
    TemporalPlainDateTimeHeapSlot, HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_temporal_plain_time_layout::{
    TemporalPlainTimeHeapSlot, HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_temporal_zoned_date_time_layout::{
    TemporalZonedDateTimeHeapSlot, HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_typed_array_iterator_layout::{
    TypedArrayIteratorHeapSlot, HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT,
};
#[cfg(test)]
use super::heap_value_encodings::{HeapValueEncoding, ValuePayloadEncoding, HEAP_VALUE_ENCODINGS};
#[cfg(test)]
use super::heap_weak_edges::{
    HeapWeakEdge, HeapWeakEdgeKind, HeapWeakEdgeRetention, HEAP_WEAK_EDGES,
};
#[cfg(test)]
use super::heap_weak_map_entry_layout::{WeakMapEntryHeapSlot, HEAP_WEAK_MAP_ENTRY_LAYOUT};
#[cfg(test)]
use super::heap_weak_map_record_layout::{WeakMapRecordHeapSlot, HEAP_WEAK_MAP_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_weak_ref_layout::{WeakRefHeapSlot, HEAP_WEAK_REF_RECORD_LAYOUT};
#[cfg(test)]
use super::heap_weak_set_entry_layout::{WeakSetEntryHeapSlot, HEAP_WEAK_SET_ENTRY_LAYOUT};
#[cfg(test)]
use super::heap_weak_set_record_layout::{WeakSetRecordHeapSlot, HEAP_WEAK_SET_RECORD_LAYOUT};
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
pub(crate) enum HeapNamedSlotStorage {
    StrongReference,
    Scalar,
}

#[allow(dead_code)]
impl HeapNamedSlotStorage {
    const fn is_strong_reference(self) -> bool {
        match self {
            Self::StrongReference => true,
            Self::Scalar => false,
        }
    }

    const fn scans_target(self) -> bool {
        match self {
            Self::StrongReference => true,
            Self::Scalar => false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeapNamedSlot {
    pub record: &'static str,
    pub key: &'static str,
    pub storage: HeapNamedSlotStorage,
}

#[allow(dead_code)]
impl HeapLayoutSlot {
    pub(crate) const fn end(self) -> u64 {
        self.offset + self.width
    }
}

pub(crate) const HEAP_HEADER_SIZE: u64 = 256;
pub(crate) const HEAP_FUNCTION_OBJECT_SIZE: u64 = 312;
pub(crate) const HEAP_OBJECT_ENTRY_SIZE: u64 = 64;
pub(crate) const HEAP_REALM_RECORD_SIZE: u64 = 72;
pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 424;
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
/// `[[DisposableState]]` plus the `[[DisposeCapability]]`'s
/// `[[DisposableResourceStack]]` (pointer, length, capacity).
pub(crate) const HEAP_DISPOSABLE_STACK_RECORD_SIZE: u64 = 32;
/// One synchronous `DisposableResource`: its closed call kind, resource value,
/// and acquired disposal method.
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_SIZE: u64 = 40;
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
pub(crate) const HEAP_ASYNC_ACTIVATION_RECORD_SIZE: u64 = 144;
#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE: u64 = 184;
#[allow(dead_code)]
pub(crate) const HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE: u64 = 56;
pub(crate) const SPARSE_ARRAY_DENSE_GROW_FACTOR: u64 = 16;
pub(crate) const HEAP_BOUND_FUNCTION_RECORD_SIZE: u64 = 48;
// Arguments records reuse the generic array header (ptr/len/cap/prototype at
// 0/8/16/24) and are also inspected by generic object paths (e.g.
// `Object.prototype.toString`) that read the boxed-object cluster at
// 32/40/48 and the internal brand / prototype-tag / proxy fields up to 80.
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
pub(crate) const HEAP_PROXY_HANDLER_TAG_OFFSET: u64 = 80;
const HEAP_GENERATOR_STATE_OFFSET: u64 = 80;
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
const HEAP_GENERATOR_RESUME_KIND_OFFSET: u64 = 168;
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
const HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET: u64 = 24;
pub(crate) const HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET: u64 = 32;
pub(crate) const HEAP_ASYNC_GENERATOR_FUNCTION_ENV_OFFSET: u64 = 40;
pub(crate) const HEAP_ASYNC_GENERATOR_THIS_PAYLOAD_OFFSET: u64 = 48;
pub(crate) const HEAP_ASYNC_GENERATOR_THIS_TAG_OFFSET: u64 = 56;
pub(crate) const HEAP_ASYNC_GENERATOR_ARGC_OFFSET: u64 = 64;
pub(crate) const HEAP_ASYNC_GENERATOR_ARGV_OFFSET: u64 = 72;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET: u64 = 80;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET: u64 = 88;
pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET: u64 = 96;
const HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET: u64 = 104;
pub(crate) const HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET: u64 = 112;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET: u64 = 120;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET: u64 = 128;
pub(crate) const HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_CAPACITY_OFFSET: u64 = 136;
const HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET: u64 = 144;
pub(crate) const HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET: u64 = 152;
pub(crate) const HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET: u64 = 160;
pub(crate) const HEAP_ASYNC_GENERATOR_INITIALIZED_OFFSET: u64 = 168;
pub(crate) const HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET: u64 = 176;
const HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET: u64 = 0;
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

pub(crate) enum ArrayBufferFlag {
    Resizable,
    Shared,
    Immutable,
    Detached,
}

impl ArrayBufferFlag {
    pub(crate) const fn word(&self) -> u64 {
        match self {
            Self::Resizable => 1,
            Self::Shared => 2,
            Self::Immutable => 4,
            Self::Detached => 8,
        }
    }
}

// TypedArray instances are ordinary heap objects with integer-indexed exotic
// behavior.  Their internal slots must not alias user-visible properties: JS
// can legally create or overwrite names such as `$TypedArrayByteLength`.
pub(crate) const HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET: u64 = 80;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_OFFSET: u64 = 88;
pub(crate) const HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET: u64 = 96;
pub(crate) const HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET: u64 = 104;
pub(crate) const HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET: u64 = 112;
pub(crate) const HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET: u64 = 120;

pub(crate) enum TypedArrayLengthMode {
    Fixed,
    Tracking,
}

impl TypedArrayLengthMode {
    pub(crate) const fn word(&self) -> u64 {
        match self {
            Self::Fixed => 0,
            Self::Tracking => 1,
        }
    }
}

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
/// Number of ordinary and progress-split choices in the immutable program.
pub(crate) const HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET: u64 = 168;
/// Number of ordinary and progress-split choices in a control-flow cycle.
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
/// Immutable algorithm state captured by a compiler-owned builtin closure.
/// The environment handle remains the callable function identity used for
/// defining-Realm and error-prototype lookup.
pub(crate) const HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET: u64 = 304;
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
pub(crate) const HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET: u64 = 344;
pub(crate) const HEAP_REALM_INTRINSICS_ERROR_PROTOTYPE_OFFSET: u64 = 352;
pub(crate) const HEAP_REALM_INTRINSICS_EVAL_ERROR_PROTOTYPE_OFFSET: u64 = 360;
pub(crate) const HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET: u64 = 368;
pub(crate) const HEAP_REALM_INTRINSICS_REFERENCE_ERROR_PROTOTYPE_OFFSET: u64 = 376;
pub(crate) const HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET: u64 = 384;
pub(crate) const HEAP_REALM_INTRINSICS_URI_ERROR_PROTOTYPE_OFFSET: u64 = 392;
pub(crate) const HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET: u64 = 400;
pub(crate) const HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET: u64 = 408;
pub(crate) const HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET: u64 = 416;
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
const HEAP_PROMISE_STATE_OFFSET: u64 = 0;
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
pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET: u64 = 0;
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
pub(crate) const HEAP_DISPOSABLE_STACK_STATE_OFFSET: u64 = 0;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET: u64 = 8;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET: u64 = 16;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET: u64 = 24;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET: u64 = 0;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET: u64 = 8;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET: u64 = 16;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET: u64 = 24;
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET: u64 = 32;

/// The closed `[[DisposableState]]` domain. Keeping this distinct from entry
/// kinds makes a state word impossible to pass to a kind-emission helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisposableStackState {
    Pending,
    Disposed,
}

impl DisposableStackState {
    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Pending => 0,
            Self::Disposed => 1,
        }
    }
}

/// The complete synchronous resource-entry domain. Nullish `use` values do
/// not create an entry, so the async stack's `Empty` kind has no sync analogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisposableStackEntryKind {
    Use,
    Adopt,
    Defer,
}

/// The only three call conventions a synchronous disposal entry can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisposableStackDisposeCall {
    ResourceReceiver,
    UndefinedReceiverWithResourceArgument,
    UndefinedReceiverNoArguments,
}

impl DisposableStackEntryKind {
    /// Every kind, in emitted comparison order. `dispose_call`'s exhaustive
    /// match forces a newly added variant to state its call convention.
    pub(crate) const ALL: [Self; 3] = [Self::Use, Self::Adopt, Self::Defer];

    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Use => 0,
            Self::Adopt => 1,
            Self::Defer => 2,
        }
    }

    pub(crate) const fn dispose_call(self) -> DisposableStackDisposeCall {
        match self {
            Self::Use => DisposableStackDisposeCall::ResourceReceiver,
            Self::Adopt => DisposableStackDisposeCall::UndefinedReceiverWithResourceArgument,
            Self::Defer => DisposableStackDisposeCall::UndefinedReceiverNoArguments,
        }
    }
}

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

/// The private lifecycle of one activation-backed async DisposeCapability.
///
/// This domain is deliberately distinct from [`AsyncDisposableStackState`]: a
/// lexical `await using` scope must remain parked in `Disposing` across each
/// Await, while the user-visible stack only exposes pending/disposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivationAsyncDisposeCapabilityState {
    Pending,
    Disposing,
    Disposed,
}

impl ActivationAsyncDisposeCapabilityState {
    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Pending => 0,
            Self::Disposing => 1,
            Self::Disposed => 2,
        }
    }
}

/// The three observably distinct disposal shapes acquired by `await using`.
///
/// `SyncFallbackMethod` cannot collapse into `AsyncMethod`: its unobservable
/// spec wrapper ignores a normal return (including a thenable) and converts a
/// synchronous throw into a rejected Promise before the disposal Await.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivationAsyncDisposeEntryKind {
    Empty,
    AsyncMethod,
    SyncFallbackMethod,
}

impl ActivationAsyncDisposeEntryKind {
    pub(crate) const ALL: [Self; 3] = [Self::Empty, Self::AsyncMethod, Self::SyncFallbackMethod];

    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::AsyncMethod => 1,
            Self::SyncFallbackMethod => 2,
        }
    }
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
        promise_wire_domain!(@define pub(crate), $name, $first_word, {
            $($variant = $word),+
        });
    };
    (private $name:ident, $first_word:literal, { $($variant:ident = $word:literal),+ $(,)? }) => {
        promise_wire_domain!(@define, $name, $first_word, {
            $($variant = $word),+
        });
    };
    (@define $method_visibility:vis, $name:ident, $first_word:literal, { $($variant:ident = $word:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum $name {
            $($variant),+
        }

        impl $name {
            $method_visibility const ALL: [Self; promise_wire_domain!(@count $($variant),+)] =
                [$(Self::$variant),+];

            $method_visibility const fn word(self) -> u64 {
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

// The closed domain stored in a Promise record's `[[PromiseState]]` word.
//
// Every raw access to this word stays behind the typed heap helpers below.
// Consumers strictly decode it before routing so an unknown word cannot be
// mistaken for rejection merely because it is neither pending nor fulfilled.
promise_wire_domain!(PromiseState, 0, {
    Pending = 0,
    Fulfilled = 1,
    Rejected = 2,
});

/// A terminal direction accepted by Promise settlement producers.
///
/// This is deliberately distinct from [`PromiseState`]: a caller settling a
/// Promise must choose fulfilment or rejection and cannot supply `Pending`.
/// It is also distinct from [`PromiseReactionType`], whose matching wire words
/// belong to a different specification record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromiseSettlement {
    Fulfill,
    Reject,
}

impl PromiseSettlement {
    pub(crate) const fn state(self) -> PromiseState {
        match self {
            Self::Fulfill => PromiseState::Fulfilled,
            Self::Reject => PromiseState::Rejected,
        }
    }

    pub(crate) const fn is_rejected(self) -> bool {
        match self {
            Self::Fulfill => false,
            Self::Reject => true,
        }
    }
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
const HEAP_ASYNC_RESUME_COMPLETION_OFFSET: u64 = 72;
pub(crate) const HEAP_ASYNC_ENV_OFFSET: u64 = 80;
pub(crate) const HEAP_ASYNC_INITIALIZED_OFFSET: u64 = 88;
pub(crate) const HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET: u64 = 96;
pub(crate) const HEAP_ASYNC_PROMISE_RECORD_OFFSET: u64 = 104;
pub(crate) const HEAP_ASYNC_COMPLETED_OFFSET: u64 = 112;
pub(crate) const HEAP_ASYNC_PENDING_COMPLETION_HEAD_OFFSET: u64 = 120;
pub(crate) const HEAP_ASYNC_PENDING_COMPLETION_DEPTH_OFFSET: u64 = 128;
pub(crate) const HEAP_ASYNC_FUNCTION_REALM_OFFSET: u64 = 136;

// The completion with which an ordinary async function resumes after Await.
//
// Unlike the Promise reaction type and async-generator resume kind, this field
// has exactly the two Completion Record shapes Await can supply. Its words and
// activation offset remain private to the typed heap accessors below.
promise_wire_domain!(private AsyncFunctionResumeCompletion, 0, {
    Normal = 0,
    Throw = 1,
});

impl AsyncFunctionResumeCompletion {
    const fn is_throw(self) -> bool {
        match self {
            Self::Normal => false,
            Self::Throw => true,
        }
    }
}
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
    const fn of_data(writable: bool, enumerable: bool, configurable: bool) -> Self {
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
    const fn of_accessor(enumerable: bool, configurable: bool) -> Self {
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

/// Static 6.2.6.6 attributes for a descriptor-kind word stored in the heap.
pub(crate) enum StoredPropertyAttributes {
    Data {
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        enumerable: bool,
        configurable: bool,
    },
}

impl StoredPropertyAttributes {
    pub(crate) const fn descriptor_word(self) -> DescriptorWord {
        match self {
            Self::Data {
                writable,
                enumerable,
                configurable,
            } => DescriptorWord::of_data(writable, enumerable, configurable),
            Self::Accessor {
                enumerable,
                configurable,
            } => DescriptorWord::of_accessor(enumerable, configurable),
        }
    }

    pub(crate) const fn descriptor_kind_bits(self) -> u64 {
        self.descriptor_word().bits()
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
/// `[[DisposableState]]` is intentionally not the async brand. The five
/// AsyncDisposableStack wrong-receiver witnesses depend on this distinction.
pub(crate) const OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK: u64 = 40;
/// The closed `[[GeneratorState]]` domain persisted in a synchronous
/// generator record.
///
/// The declaration order follows the stable heap words. The explicit
/// projection, rather than a Rust discriminant, owns that representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeneratorState {
    SuspendedStart,
    Executing,
    Completed,
    SuspendedYield,
}

impl GeneratorState {
    const ALL: [Self; 4] = [
        Self::SuspendedStart,
        Self::Executing,
        Self::Completed,
        Self::SuspendedYield,
    ];

    const fn word(self) -> u64 {
        match self {
            Self::SuspendedStart => 0,
            Self::Executing => 1,
            Self::Completed => 2,
            Self::SuspendedYield => 3,
        }
    }
}

/// One strictly validated snapshot of a synchronous generator's state word.
///
/// The raw local is private and the token is deliberately non-`Copy`. State
/// dispatch borrows it, then must consume it through the matching release
/// boundary once every comparison has been emitted.
#[must_use = "a loaded generator state must be compared and released"]
pub(crate) struct LoadedGeneratorState(u32);

/// The closed completion kind supplied when a synchronous generator resumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeneratorResumeKind {
    Normal,
    Return,
    Throw,
}

impl GeneratorResumeKind {
    const ALL: [Self; 3] = [Self::Normal, Self::Return, Self::Throw];

    const fn word(self) -> u64 {
        match self {
            Self::Normal => 0,
            Self::Return => 1,
            Self::Throw => 2,
        }
    }
}

/// One strictly validated snapshot of a synchronous generator resume kind.
#[must_use = "a loaded generator resume kind must be routed and released"]
pub(crate) struct LoadedGeneratorResumeKind(u32);

/// The exact resume-kind transport joining fresh and resumed delegation.
///
/// The raw local is private and the token is deliberately non-`Copy`. The
/// fresh path initializes it from a typed kind, while the resumed path can
/// only replace it from a validated activation snapshot.
#[must_use = "a generator delegation resume kind must be routed and released"]
pub(crate) struct GeneratorResumeKindTransport(u32);

/// The closed Completion Record subset persisted in an async-generator
/// request.
///
/// The semantic declaration order follows the specification operations. The
/// stable heap words come exclusively from the general completion ABI below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorRequestCompletionKind {
    Normal,
    Return,
    Throw,
}

impl AsyncGeneratorRequestCompletionKind {
    const ALL: [Self; 3] = [Self::Normal, Self::Throw, Self::Return];

    const fn completion_kind(self) -> CompletionKind {
        match self {
            Self::Normal => CompletionKind::Normal,
            Self::Return => CompletionKind::Return,
            Self::Throw => CompletionKind::Throw,
        }
    }

    const fn word(self) -> u64 {
        self.completion_kind().code() as u64
    }
}

/// One strictly validated snapshot of an async-generator request's completion
/// kind.
///
/// The raw local is private and the token is deliberately non-`Copy`. Routing
/// borrows it, then must consume it through the matching release boundary.
#[must_use = "a loaded request completion kind must be routed and released"]
pub(crate) struct LoadedAsyncGeneratorRequestCompletionKind(u32);

/// The closed `[[AsyncGeneratorState]]` lifecycle stored in an activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorExecutionState {
    SuspendedStart,
    SuspendedYield,
    Executing,
    DrainingQueue,
    Completed,
}

impl AsyncGeneratorExecutionState {
    const ALL: [Self; 5] = [
        Self::SuspendedStart,
        Self::SuspendedYield,
        Self::Executing,
        Self::DrainingQueue,
        Self::Completed,
    ];

    const fn word(self) -> u64 {
        match self {
            Self::SuspendedStart => 0,
            Self::SuspendedYield => 1,
            Self::Executing => 2,
            Self::DrainingQueue => 3,
            Self::Completed => 4,
        }
    }
}

/// One strictly validated snapshot of an async-generator execution state.
///
/// The raw local is private and the token is deliberately non-`Copy`. State
/// routing borrows it, then must consume it through the matching release
/// boundary once every comparison has been emitted.
#[must_use = "a loaded async-generator execution state must be routed and released"]
pub(crate) struct LoadedAsyncGeneratorExecutionState(u32);

/// The closed backend status stored around an async-generator body invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorBodyStatus {
    Idle,
    Running,
    Await,
    Yield,
    Complete,
    Throw,
}

impl AsyncGeneratorBodyStatus {
    const ALL: [Self; 6] = [
        Self::Idle,
        Self::Running,
        Self::Await,
        Self::Yield,
        Self::Complete,
        Self::Throw,
    ];

    const fn word(self) -> u64 {
        match self {
            Self::Idle => 0,
            Self::Running => 1,
            Self::Await => 2,
            Self::Yield => 3,
            Self::Complete => 4,
            Self::Throw => 5,
        }
    }
}

/// One strictly validated snapshot of an async-generator body status.
///
/// The raw local is private and the token is deliberately non-`Copy`. Status
/// routing borrows it, then must consume it through the matching release
/// boundary once every comparison has been emitted.
#[must_use = "a loaded async-generator body status must be routed and released"]
pub(crate) struct LoadedAsyncGeneratorBodyStatus(u32);

/// The closed completion kind supplied when an async-generator body resumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorResumeKind {
    Normal,
    Return,
    Throw,
    Fulfill,
    Reject,
}

impl AsyncGeneratorResumeKind {
    const ALL: [Self; 5] = [
        Self::Normal,
        Self::Return,
        Self::Throw,
        Self::Fulfill,
        Self::Reject,
    ];

    const fn word(self) -> u64 {
        match self {
            Self::Normal => 0,
            Self::Return => 1,
            Self::Throw => 2,
            Self::Fulfill => 3,
            Self::Reject => 4,
        }
    }
}

/// One strictly validated snapshot of an async-generator resume kind.
///
/// The raw local is private and the token is deliberately non-`Copy`. Routing
/// borrows it, then must consume it through the matching release boundary once
/// every comparison and validated transport copy has been emitted.
#[must_use = "a loaded async-generator resume kind must be routed and released"]
pub(crate) struct LoadedAsyncGeneratorResumeKind(u32);

pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING: u64 = u64::MAX;
pub(crate) const GENERATOR_DELEGATED_RESULT_AUX_FLAG: i64 = i64::MIN;
#[allow(dead_code)]
pub(crate) const ASYNC_GENERATOR_RESUME_STATE_INITIALIZING: u64 = u64::MAX;
pub(crate) const ASYNC_GENERATOR_RETURN_VALUE_ALREADY_AWAITED: u64 = 1;
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
pub(crate) const HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT: &[HeapLayoutSlot] = &[
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "function_environment",
        offset: HEAP_ASYNC_FUNCTION_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "function_table_index",
        offset: HEAP_ASYNC_FUNCTION_TABLE_INDEX_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "this_payload",
        offset: HEAP_ASYNC_THIS_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "this_tag",
        offset: HEAP_ASYNC_THIS_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "argc",
        offset: HEAP_ASYNC_ARGC_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "argv",
        offset: HEAP_ASYNC_ARGV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "resume_state",
        offset: HEAP_ASYNC_RESUME_STATE_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "resume_payload",
        offset: HEAP_ASYNC_RESUME_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "resume_tag",
        offset: HEAP_ASYNC_RESUME_TAG_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "resume_completion",
        offset: HEAP_ASYNC_RESUME_COMPLETION_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "lexical_environment",
        offset: HEAP_ASYNC_ENV_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "initialized",
        offset: HEAP_ASYNC_INITIALIZED_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "promise_payload",
        offset: HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "promise_record",
        offset: HEAP_ASYNC_PROMISE_RECORD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "completed",
        offset: HEAP_ASYNC_COMPLETED_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "pending_completion_head",
        offset: HEAP_ASYNC_PENDING_COMPLETION_HEAD_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "pending_completion_depth",
        offset: HEAP_ASYNC_PENDING_COMPLETION_DEPTH_OFFSET,
        width: 8,
        pointer: false,
    },
    HeapLayoutSlot {
        record: "async-function-activation",
        name: "realm",
        offset: HEAP_ASYNC_FUNCTION_REALM_OFFSET,
        width: 8,
        pointer: true,
    },
];

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
    HeapLayoutSlot {
        record: "function-object",
        name: "builtin_closure_context",
        offset: HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
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
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Date.prototype%",
        offset: HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Error.prototype%",
        offset: HEAP_REALM_INTRINSICS_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%EvalError.prototype%",
        offset: HEAP_REALM_INTRINSICS_EVAL_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%RangeError.prototype%",
        offset: HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%ReferenceError.prototype%",
        offset: HEAP_REALM_INTRINSICS_REFERENCE_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%SyntaxError.prototype%",
        offset: HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%URIError.prototype%",
        offset: HEAP_REALM_INTRINSICS_URI_ERROR_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Promise.prototype%",
        offset: HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Function.prototype%",
        offset: HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
        width: 8,
        pointer: true,
    },
    HeapLayoutSlot {
        record: "realm-intrinsics",
        name: "%Promise%",
        offset: HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET,
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
pub(crate) const HEAP_ARRAY_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.array",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.index",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.done",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "array-iterator-object",
        key: "$ArrayIterator.kind",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_STRING_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "string-iterator-object",
        key: "$StringIterator.string",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "string-iterator-object",
        key: "$StringIterator.index",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.regexp",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.string",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.global",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.unicode",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "regexp-string-iterator-object",
        key: "$RegExpStringIterator.done",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ITERATOR_HELPER_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFromIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFromNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapMapper",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterPredicate",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapMapper",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapInnerIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapInnerNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorMapDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFilterDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorFlatMapDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorTakeDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-helper-object",
        key: "$IteratorDropDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipIterators",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipNextMethods",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipOpen",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipPadding",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipKeys",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipMode",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipExecuting",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-zip-state-object",
        key: "$IteratorZipStarted",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) const HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS: &[HeapNamedSlot] = &[
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatIterables",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatMethods",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatCurrentIterator",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatCurrentNext",
        storage: HeapNamedSlotStorage::StrongReference,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatIndex",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatActive",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatDone",
        storage: HeapNamedSlotStorage::Scalar,
    },
    HeapNamedSlot {
        record: "iterator-concat-state-object",
        key: "$IteratorConcatExecuting",
        storage: HeapNamedSlotStorage::Scalar,
    },
];

#[allow(dead_code)]
pub(crate) enum HeapNamedSlotFamily {
    ArrayIterator,
    StringIterator,
    RegExpStringIterator,
    IteratorHelper,
    IteratorConcatState,
    IteratorZipState,
}

#[allow(dead_code)]
impl HeapNamedSlotFamily {
    pub(crate) const fn slots(&self) -> &'static [HeapNamedSlot] {
        match self {
            Self::ArrayIterator => HEAP_ARRAY_ITERATOR_NAMED_SLOTS,
            Self::StringIterator => HEAP_STRING_ITERATOR_NAMED_SLOTS,
            Self::RegExpStringIterator => HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS,
            Self::IteratorHelper => HEAP_ITERATOR_HELPER_NAMED_SLOTS,
            Self::IteratorConcatState => HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS,
            Self::IteratorZipState => HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS,
        }
    }
}

#[allow(dead_code)]
pub(crate) const HEAP_NAMED_SLOT_FAMILIES: &[HeapNamedSlotFamily] = &[
    HeapNamedSlotFamily::ArrayIterator,
    HeapNamedSlotFamily::StringIterator,
    HeapNamedSlotFamily::RegExpStringIterator,
    HeapNamedSlotFamily::IteratorHelper,
    HeapNamedSlotFamily::IteratorConcatState,
    HeapNamedSlotFamily::IteratorZipState,
];

#[allow(dead_code)]
pub(crate) const fn heap_collector_is_executable() -> bool {
    HEAP_COLLECTOR_POLICY.is_executable()
}

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

    /// Store one state from the closed synchronous-generator lifecycle.
    pub(crate) fn emit_store_generator_state(
        &self,
        generator_payload_local: u32,
        state: GeneratorState,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            generator_payload_local,
            HEAP_GENERATOR_STATE_OFFSET,
            state.word(),
            function,
        );
    }

    /// Load and strictly validate one snapshot of `[[GeneratorState]]`.
    ///
    /// An unknown word is an impossible generator record. Trap rather than
    /// letting the builtin dispatcher mistake it for `SuspendedStart`.
    pub(crate) fn emit_load_generator_state_strict(
        &mut self,
        generator_payload_local: u32,
        function: &mut Function,
    ) -> LoadedGeneratorState {
        let state_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            generator_payload_local,
            HEAP_GENERATOR_STATE_OFFSET,
            state_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for state in GeneratorState::ALL {
            function.instruction(&Instruction::LocalGet(state_word_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedGeneratorState(state_word_local)
    }

    /// Emit one comparison against a strictly loaded generator-state word.
    pub(crate) fn emit_generator_state_equals(
        &self,
        loaded: &LoadedGeneratorState,
        expected: GeneratorState,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Release the private local owned by a loaded generator-state snapshot.
    pub(crate) fn release_loaded_generator_state(&mut self, loaded: LoadedGeneratorState) {
        self.release_temp_local(loaded.0);
    }

    /// Store one kind from the closed synchronous-generator resume domain.
    pub(crate) fn emit_store_generator_resume_kind(
        &self,
        generator_payload_local: u32,
        kind: GeneratorResumeKind,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            generator_payload_local,
            HEAP_GENERATOR_RESUME_KIND_OFFSET,
            kind.word(),
            function,
        );
    }

    /// Load and strictly validate one generator resume-kind snapshot.
    pub(crate) fn emit_load_generator_resume_kind_strict(
        &mut self,
        generator_payload_local: u32,
        function: &mut Function,
    ) -> LoadedGeneratorResumeKind {
        let kind_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            generator_payload_local,
            HEAP_GENERATOR_RESUME_KIND_OFFSET,
            kind_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for kind in GeneratorResumeKind::ALL {
            function.instruction(&Instruction::LocalGet(kind_word_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedGeneratorResumeKind(kind_word_local)
    }

    /// Compare one strictly loaded generator resume kind.
    pub(crate) fn emit_generator_resume_kind_equals(
        &self,
        loaded: &LoadedGeneratorResumeKind,
        expected: GeneratorResumeKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Initialize the exact resume-kind transport for fresh delegation.
    pub(crate) fn emit_initialize_generator_resume_kind_transport(
        &mut self,
        kind: GeneratorResumeKind,
        function: &mut Function,
    ) -> GeneratorResumeKindTransport {
        let kind_word_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(kind.word() as i64));
        function.instruction(&Instruction::LocalSet(kind_word_local));
        GeneratorResumeKindTransport(kind_word_local)
    }

    /// Copy a validated activation snapshot into the delegation transport.
    pub(crate) fn emit_copy_generator_resume_kind_to_transport(
        &self,
        loaded: &LoadedGeneratorResumeKind,
        transport: &GeneratorResumeKindTransport,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::LocalSet(transport.0));
    }

    /// Compare one exact generator delegation resume-kind transport.
    pub(crate) fn emit_generator_resume_kind_transport_equals(
        &self,
        transport: &GeneratorResumeKindTransport,
        expected: GeneratorResumeKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(transport.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Release the private local owned by a resume-kind snapshot.
    pub(crate) fn release_loaded_generator_resume_kind(
        &mut self,
        loaded: LoadedGeneratorResumeKind,
    ) {
        self.release_temp_local(loaded.0);
    }

    /// Release the private local owned by a delegation resume-kind transport.
    pub(crate) fn release_generator_resume_kind_transport(
        &mut self,
        transport: GeneratorResumeKindTransport,
    ) {
        self.release_temp_local(transport.0);
    }

    /// Store one completion kind from the closed async-generator request
    /// domain.
    pub(crate) fn emit_store_async_generator_request_completion_kind(
        &self,
        request_local: u32,
        kind: AsyncGeneratorRequestCompletionKind,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            request_local,
            HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
            kind.word(),
            function,
        );
    }

    /// Load and strictly validate one async-generator request completion kind.
    ///
    /// An unknown or wrong-domain word is an impossible request record. Trap
    /// before either request consumer can mistake it for Normal.
    pub(crate) fn emit_load_async_generator_request_completion_kind_strict(
        &mut self,
        request_local: u32,
        function: &mut Function,
    ) -> LoadedAsyncGeneratorRequestCompletionKind {
        let kind_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            request_local,
            HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
            kind_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for kind in AsyncGeneratorRequestCompletionKind::ALL {
            function.instruction(&Instruction::LocalGet(kind_word_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedAsyncGeneratorRequestCompletionKind(kind_word_local)
    }

    /// Emit one comparison against a strictly loaded request completion kind.
    pub(crate) fn emit_async_generator_request_completion_kind_equals(
        &self,
        loaded: &LoadedAsyncGeneratorRequestCompletionKind,
        expected: AsyncGeneratorRequestCompletionKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Copy a validated request word into the generic completion transport
    /// consumed by async-generator complete-step.
    pub(crate) fn emit_copy_async_generator_request_completion_kind_to_step_completion(
        &self,
        loaded: &LoadedAsyncGeneratorRequestCompletionKind,
        step_completion_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::LocalSet(step_completion_kind_local));
    }

    /// Release the private local owned by a loaded request-kind snapshot.
    pub(crate) fn release_loaded_async_generator_request_completion_kind(
        &mut self,
        loaded: LoadedAsyncGeneratorRequestCompletionKind,
    ) {
        self.release_temp_local(loaded.0);
    }

    /// Store one state from the closed async-generator execution lifecycle.
    pub(crate) fn emit_store_async_generator_execution_state(
        &self,
        activation_local: u32,
        state: AsyncGeneratorExecutionState,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            state.word(),
            function,
        );
    }

    /// Load and strictly validate one async-generator execution-state snapshot.
    ///
    /// An unknown or wrong-domain word is an impossible activation record.
    pub(crate) fn emit_load_async_generator_execution_state_strict(
        &mut self,
        activation_local: u32,
        function: &mut Function,
    ) -> LoadedAsyncGeneratorExecutionState {
        let state_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            state_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for state in AsyncGeneratorExecutionState::ALL {
            function.instruction(&Instruction::LocalGet(state_word_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedAsyncGeneratorExecutionState(state_word_local)
    }

    /// Emit one comparison against a strictly loaded execution-state word.
    pub(crate) fn emit_async_generator_execution_state_equals(
        &self,
        loaded: &LoadedAsyncGeneratorExecutionState,
        expected: AsyncGeneratorExecutionState,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Release the private local owned by an execution-state snapshot.
    pub(crate) fn release_loaded_async_generator_execution_state(
        &mut self,
        loaded: LoadedAsyncGeneratorExecutionState,
    ) {
        self.release_temp_local(loaded.0);
    }

    /// Store one status from the closed async-generator body domain.
    pub(crate) fn emit_store_async_generator_body_status(
        &self,
        activation_local: u32,
        status: AsyncGeneratorBodyStatus,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            status.word(),
            function,
        );
    }

    /// Load and strictly validate one async-generator body-status snapshot.
    ///
    /// An unknown or wrong-domain word is an impossible activation record.
    pub(crate) fn emit_load_async_generator_body_status_strict(
        &mut self,
        activation_local: u32,
        function: &mut Function,
    ) -> LoadedAsyncGeneratorBodyStatus {
        let status_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            status_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for status in AsyncGeneratorBodyStatus::ALL {
            function.instruction(&Instruction::LocalGet(status_word_local));
            function.instruction(&Instruction::I64Const(status.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedAsyncGeneratorBodyStatus(status_word_local)
    }

    /// Emit one comparison against a strictly loaded body-status word.
    pub(crate) fn emit_async_generator_body_status_equals(
        &self,
        loaded: &LoadedAsyncGeneratorBodyStatus,
        expected: AsyncGeneratorBodyStatus,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Release the private local owned by a body-status snapshot.
    pub(crate) fn release_loaded_async_generator_body_status(
        &mut self,
        loaded: LoadedAsyncGeneratorBodyStatus,
    ) {
        self.release_temp_local(loaded.0);
    }

    /// Store one kind from the closed async-generator resume domain.
    pub(crate) fn emit_store_async_generator_resume_kind(
        &self,
        activation_local: u32,
        kind: AsyncGeneratorResumeKind,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            kind.word(),
            function,
        );
    }

    /// Load and strictly validate one async-generator resume-kind snapshot.
    ///
    /// An unknown or wrong-domain word is an impossible activation record.
    pub(crate) fn emit_load_async_generator_resume_kind_strict(
        &mut self,
        activation_local: u32,
        function: &mut Function,
    ) -> LoadedAsyncGeneratorResumeKind {
        let kind_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            kind_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for kind in AsyncGeneratorResumeKind::ALL {
            function.instruction(&Instruction::LocalGet(kind_word_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedAsyncGeneratorResumeKind(kind_word_local)
    }

    /// Emit one comparison against a strictly loaded resume-kind word.
    pub(crate) fn emit_async_generator_resume_kind_equals(
        &self,
        loaded: &LoadedAsyncGeneratorResumeKind,
        expected: AsyncGeneratorResumeKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Copy a validated activation resume kind into the widened delegation
    /// pending-kind transport.
    pub(crate) fn emit_copy_async_generator_resume_kind_to_delegate_pending_kind(
        &self,
        loaded: &LoadedAsyncGeneratorResumeKind,
        pending_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(loaded.0));
        function.instruction(&Instruction::LocalSet(pending_kind_local));
    }

    /// Initialize the delegation pending-kind transport from one typed resume
    /// kind without constructing an activation snapshot.
    pub(crate) fn emit_initialize_async_generator_delegate_pending_kind_from_resume_kind(
        &self,
        pending_kind_local: u32,
        kind: AsyncGeneratorResumeKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(kind.word() as i64));
        function.instruction(&Instruction::LocalSet(pending_kind_local));
    }

    /// Compare the widened delegation pending-kind transport with one resume
    /// kind without treating the pending field as the activation domain.
    pub(crate) fn emit_async_generator_delegate_pending_kind_equals_resume_kind(
        &self,
        pending_kind_local: u32,
        expected: AsyncGeneratorResumeKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(pending_kind_local));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    /// Release the private local owned by a resume-kind snapshot.
    pub(crate) fn release_loaded_async_generator_resume_kind(
        &mut self,
        loaded: LoadedAsyncGeneratorResumeKind,
    ) {
        self.release_temp_local(loaded.0);
    }

    /// Initialize a Promise record in the sole valid non-terminal state.
    pub(crate) fn emit_initialize_promise_state(
        &self,
        promise_record_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            PromiseState::Pending.word(),
            function,
        );
    }

    /// Store a terminal Promise state selected by the closed settlement domain.
    pub(crate) fn emit_store_promise_settlement(
        &self,
        promise_record_local: u32,
        settlement: PromiseSettlement,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            settlement.state().word(),
            function,
        );
    }

    /// Load and strictly validate a Promise record's lifecycle state word.
    ///
    /// The known stable word remains in `state_word_local` for the caller's
    /// emitted dispatch. An unknown word is an impossible Promise record and
    /// traps rather than falling through as rejection.
    pub(crate) fn emit_load_promise_state_strict(
        &self,
        promise_record_local: u32,
        state_word_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for state in PromiseState::ALL {
            function.instruction(&Instruction::LocalGet(state_word_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }
    }

    /// Store the completion with which an ordinary async function resumes.
    ///
    /// The activation offset and wire word stay inside this boundary so a
    /// producer cannot substitute an arbitrary integer for the closed domain.
    pub(crate) fn emit_store_async_function_resume_completion(
        &self,
        activation_local: u32,
        completion: AsyncFunctionResumeCompletion,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_RESUME_COMPLETION_OFFSET,
            completion.word(),
            function,
        );
    }

    /// Load and strictly decode an ordinary async function's resume
    /// completion into one normalized i64 boolean.
    ///
    /// An unknown heap word is an impossible activation state. Trap instead
    /// of letting consumers mistake it for `Normal`.
    pub(crate) fn emit_load_async_function_resume_is_throw(
        &mut self,
        activation_local: u32,
        is_throw_local: u32,
        function: &mut Function,
    ) {
        let completion_word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_RESUME_COMPLETION_OFFSET,
            completion_word_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for completion in AsyncFunctionResumeCompletion::ALL {
            function.instruction(&Instruction::LocalGet(completion_word_local));
            function.instruction(&Instruction::I64Const(completion.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(if completion.is_throw() {
                1
            } else {
                0
            }));
            function.instruction(&Instruction::LocalSet(is_throw_local));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(completion_word_local);
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

    #[test]
    fn promise_lifecycle_wire_domain_is_closed() {
        assert_eq!(PromiseState::ALL.map(PromiseState::word), [0, 1, 2]);
        assert_eq!(
            [PromiseSettlement::Fulfill, PromiseSettlement::Reject].map(PromiseSettlement::state),
            [PromiseState::Fulfilled, PromiseState::Rejected]
        );
        assert_eq!(
            [PromiseSettlement::Fulfill, PromiseSettlement::Reject]
                .map(PromiseSettlement::is_rejected),
            [false, true]
        );
    }

    #[test]
    fn promise_lifecycle_owns_every_raw_state_access() {
        let heap_source = include_str!("heap.rs");
        let heap_implementation = heap_source
            .split_once("#[cfg(test)]\nmod tests {")
            .expect(
                "heap test module should follow the implementation, including test-only imports",
            )
            .0;
        let promise_source = include_str!("builtins/promise.rs");
        let consumer_sources = [
            promise_source,
            include_str!("builtins/standard.rs"),
            include_str!("builtins/async_iterator.rs"),
            include_str!("builtins/async_disposable_stack.rs"),
            include_str!("control_flow.rs"),
            include_str!("functions.rs"),
        ];

        assert!(heap_implementation.contains("const HEAP_PROMISE_STATE_OFFSET: u64 = 0;"));
        assert_eq!(
            heap_implementation
                .matches("HEAP_PROMISE_STATE_OFFSET")
                .count(),
            5,
            "only the declaration, layout, initializer, terminal store and strict load own the raw offset"
        );
        for source in consumer_sources {
            assert!(!source.contains("HEAP_PROMISE_STATE_OFFSET"));
            assert!(!source.contains(concat!("PROMISE_STATE_", "PENDING")));
            assert!(!source.contains(concat!("PROMISE_STATE_", "FULFILLED")));
            assert!(!source.contains(concat!("PROMISE_STATE_", "REJECTED")));
            assert!(!source.contains("promise_state: u64"));
        }

        let decoder = heap_implementation
            .split_once("pub(crate) fn emit_load_promise_state_strict(")
            .expect("strict Promise-state decoder should exist")
            .1
            .split_once("/// Store the completion with which an ordinary async function resumes.")
            .expect("strict Promise-state decoder should have a stable boundary")
            .0;
        assert_eq!(decoder.matches("HEAP_PROMISE_STATE_OFFSET").count(), 1);
        assert_eq!(decoder.matches("PromiseState::ALL").count(), 1);
        assert_eq!(decoder.matches("Instruction::Unreachable").count(), 1);

        let settlement = promise_source
            .split_once("pub(crate) fn emit_settle_promise_record(")
            .expect("typed Promise settlement should exist")
            .1
            .split_once("/// Appends `promise_record_local`")
            .expect("Promise settlement should have a stable boundary")
            .0;
        assert!(settlement.contains("settlement: PromiseSettlement"));
        assert_eq!(
            settlement
                .matches("emit_load_promise_state_strict(")
                .count(),
            1
        );
        assert_eq!(
            settlement.matches("emit_store_promise_settlement(").count(),
            1
        );
        let capture = settlement
            .find("match settlement")
            .expect("settlement must select and capture one reaction list");
        let result = settlement
            .find("HEAP_PROMISE_RESULT_PAYLOAD_OFFSET")
            .expect("settlement must store its result");
        let clear = settlement
            .find("for offset in [")
            .expect("settlement must clear both obsolete reaction lists");
        let state = settlement
            .find("emit_store_promise_settlement(")
            .expect("settlement must store its terminal state");
        let tracker = settlement
            .find("emit_track_unhandled_rejection(")
            .expect("rejection must enter host tracking");
        let enqueue = settlement
            .find("emit_enqueue_promise_reaction_list(")
            .expect("settlement must enqueue the captured reactions");
        assert!(capture < result && result < clear && clear < state);
        assert!(state < tracker && tracker < enqueue);

        let router = promise_source
            .split_once("fn emit_route_promise_reaction_pair(")
            .expect("one Promise reaction-pair router should exist")
            .1
            .split_once("fn emit_intrinsic_promise_resolve_to_locals(")
            .expect("Promise reaction router should have a stable boundary")
            .0;
        assert_eq!(router.matches("emit_load_promise_state_strict(").count(), 1);
        assert_eq!(router.matches("PromiseState::ALL").count(), 1);
        assert_eq!(router.matches("PromiseState::Pending =>").count(), 1);
        assert_eq!(router.matches("PromiseState::Fulfilled =>").count(), 1);
        assert_eq!(router.matches("PromiseState::Rejected =>").count(), 1);
        assert_eq!(router.matches("Instruction::Unreachable").count(), 1);
        assert_eq!(
            promise_source
                .matches("emit_route_promise_reaction_pair(")
                .count(),
            4,
            "the one definition must serve ordinary then, await and async-generator return-await"
        );
        assert!(!promise_source.contains("state: u64"));
    }

    #[test]
    fn async_function_resume_completion_wire_domain_is_closed() {
        assert_eq!(
            AsyncFunctionResumeCompletion::ALL.map(AsyncFunctionResumeCompletion::word),
            [0, 1]
        );
        assert_eq!(
            AsyncFunctionResumeCompletion::ALL.map(AsyncFunctionResumeCompletion::is_throw),
            [false, true]
        );
    }

    #[test]
    fn async_function_resume_completion_owns_every_raw_access() {
        let heap_source = include_str!("heap.rs");
        let heap_implementation = heap_source
            .split_once("#[cfg(test)]\nmod tests {")
            .expect(
                "heap test module should follow the implementation, including test-only imports",
            )
            .0;
        let functions_source = include_str!("functions.rs");
        let promise_source = include_str!("builtins/promise.rs");
        let control_flow_source = include_str!("control_flow.rs");

        assert!(
            heap_implementation.contains("const HEAP_ASYNC_RESUME_COMPLETION_OFFSET: u64 = 72;")
        );
        assert_eq!(
            heap_implementation
                .matches("HEAP_ASYNC_RESUME_COMPLETION_OFFSET")
                .count(),
            4,
            "only the declaration, layout, typed store and strict decoder own the raw offset"
        );
        let resume_layout_slots = HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT
            .iter()
            .filter(|slot| slot.name == "resume_completion")
            .collect::<Vec<_>>();
        assert_eq!(resume_layout_slots.len(), 1);
        assert_eq!(
            resume_layout_slots[0].offset,
            HEAP_ASYNC_RESUME_COMPLETION_OFFSET
        );
        assert_eq!(resume_layout_slots[0].width, 8);
        assert!(!resume_layout_slots[0].pointer);
        for source in [functions_source, promise_source, control_flow_source] {
            assert!(!source.contains("HEAP_ASYNC_RESUME_COMPLETION_OFFSET"));
            assert!(!source.contains(concat!("HEAP_ASYNC_RESUME_", "KIND_OFFSET")));
            assert!(!source.contains(concat!("ASYNC_RESUME_KIND_", "FULFILL")));
            assert!(!source.contains(concat!("ASYNC_RESUME_KIND_", "REJECT")));
        }

        let store_boundary = heap_implementation
            .split_once("pub(crate) fn emit_store_async_function_resume_completion(")
            .expect("typed async-function resume store should exist")
            .1
            .split_once("pub(crate) fn emit_load_async_function_resume_is_throw(")
            .expect("typed store should end at the strict decoder")
            .0;
        assert_eq!(
            store_boundary
                .matches("HEAP_ASYNC_RESUME_COMPLETION_OFFSET")
                .count(),
            1
        );

        let load_boundary = heap_implementation
            .split_once("pub(crate) fn emit_load_async_function_resume_is_throw(")
            .expect("strict async-function resume decoder should exist")
            .1
            .split_once("pub(crate) fn load_i64_to_local_from_offset(")
            .expect("strict decoder should end before general heap loads")
            .0;
        assert_eq!(
            load_boundary
                .matches("HEAP_ASYNC_RESUME_COMPLETION_OFFSET")
                .count(),
            1
        );
        assert_eq!(
            load_boundary
                .matches("AsyncFunctionResumeCompletion::ALL")
                .count(),
            1
        );
        assert_eq!(load_boundary.matches("Instruction::Unreachable").count(), 1);

        assert_eq!(
            functions_source
                .matches("emit_store_async_function_resume_completion(")
                .count(),
            1
        );
        assert_eq!(
            functions_source
                .matches("AsyncFunctionResumeCompletion::Normal")
                .count(),
            1
        );
        assert_eq!(
            promise_source
                .matches("emit_store_async_function_resume_completion(")
                .count(),
            2
        );
        assert_eq!(
            promise_source
                .matches("AsyncFunctionResumeCompletion::Normal")
                .count(),
            1
        );
        assert_eq!(
            promise_source
                .matches("AsyncFunctionResumeCompletion::Throw")
                .count(),
            1
        );
        assert_eq!(
            control_flow_source
                .matches("emit_load_async_function_resume_is_throw(")
                .count(),
            3,
            "ordinary await, async disposal and for-await must share the strict decoder"
        );

        let layout_domain = control_flow_source
            .split_once("enum ForAwaitActivationLayout {")
            .expect("closed for-await activation layout should exist")
            .1
            .split_once("}\n\nimpl ForAwaitActivationLayout")
            .expect("for-await activation layout should have a bounded domain")
            .0;
        assert_eq!(layout_domain.matches("AsyncFunction,").count(), 1);
        assert_eq!(layout_domain.matches("AsyncGenerator,").count(), 1);
        assert_eq!(
            layout_domain
                .lines()
                .filter(|line| line.trim_end().ends_with(','))
                .count(),
            2
        );

        let layout_decoder = control_flow_source
            .split_once("fn emit_load_for_await_resume_is_throw(")
            .expect("for-await resume decoder should exist")
            .1
            .split_once("pub(crate) fn compile_async_for_of_iterator(")
            .expect("for-await resume decoder should be bounded")
            .0;
        assert_eq!(
            layout_decoder
                .matches("ForAwaitActivationLayout::AsyncFunction")
                .count(),
            1
        );
        assert_eq!(
            layout_decoder
                .matches("ForAwaitActivationLayout::AsyncGenerator")
                .count(),
            1
        );
        assert_eq!(
            layout_decoder
                .matches("emit_load_async_generator_resume_kind_strict")
                .count(),
            1
        );
        assert_eq!(
            layout_decoder
                .matches("AsyncGeneratorResumeKind::Reject")
                .count(),
            1,
            "the async-generator branch must preserve its existing rejection policy"
        );

        let for_await_body = control_flow_source
            .split_once("pub(crate) fn compile_async_for_of_iterator(")
            .expect("for-await emitter should exist")
            .1
            .split_once("pub(crate) fn compile_for_of_iterator(")
            .expect("for-await emitter should have a stable boundary")
            .0;
        assert_eq!(
            for_await_body
                .matches("emit_load_for_await_resume_is_throw(")
                .count(),
            2,
            "value and iterator-close resumes must both normalize their completion"
        );
        assert!(!for_await_body.contains(concat!("resume_kind_", "offset")));
        assert!(!for_await_body.contains(concat!("rejected_resume_", "kind")));
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

    fn assert_linear_side_storage(storage: &LinearSideStorage) {
        assert!(!storage.record().is_empty());
        assert!(!storage.length_source().is_empty());
        assert!(
            !storage.element().is_reference_storage(),
            "{} should not be traced as pointer storage",
            storage.record()
        );
    }

    fn assert_named_slots(layout: &[HeapNamedSlot]) {
        let mut keys = BTreeSet::new();
        for slot in layout {
            assert!(!slot.record.is_empty());
            assert!(!slot.key.is_empty());
            assert_eq!(
                slot.storage.scans_target(),
                slot.storage.is_strong_reference(),
                "{}.{} must derive tracing and reference strength from one storage class",
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
            assert!(!source.name().is_empty());
            assert!(!source.owner().is_empty());
            assert!(
                names.insert(source.name()),
                "duplicate root source {}",
                source.name()
            );
        }
    }

    fn assert_weak_edges(layout: &[HeapWeakEdge]) {
        let mut names = BTreeSet::new();
        for edge in layout {
            assert!(!edge.record().is_empty());
            assert!(!edge.name().is_empty());
            assert!(
                names.insert((edge.record(), edge.name())),
                "{} has duplicate weak edge slot {}",
                edge.record(),
                edge.name()
            );
            match edge.kind().retention() {
                HeapWeakEdgeRetention::DoesNotRetain => assert!(matches!(
                    edge.kind(),
                    HeapWeakEdgeKind::EphemeronKey
                        | HeapWeakEdgeKind::WeakTarget
                        | HeapWeakEdgeKind::FinalizerToken
                )),
                HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey => {
                    assert_eq!(edge.kind(), HeapWeakEdgeKind::EphemeronValue)
                }
                HeapWeakEdgeRetention::StrongUntilCleanup => {
                    assert_eq!(edge.kind(), HeapWeakEdgeKind::FinalizerHoldings)
                }
            }
        }
    }

    fn assert_collector_policy(policy: &HeapCollectorPolicy) {
        assert!(!policy.name().is_empty());
        assert!(
            !policy.moves_objects(),
            "T05 collector policy must stay non-moving until all roots can be updated"
        );
        assert!(!heap_collector_is_executable());
        assert_eq!(policy.root_sources(), HEAP_ROOT_SOURCES);
        assert_eq!(policy.weak_edges().len(), HEAP_WEAK_EDGES.len());
        for (actual, expected) in policy.weak_edges().iter().zip(HEAP_WEAK_EDGES) {
            assert_eq!(actual.record(), expected.record());
            assert_eq!(actual.name(), expected.name());
            assert_eq!(actual.kind(), expected.kind());
        }

        let mut phase_names = BTreeSet::new();
        let mut phase_kinds = BTreeSet::new();
        for phase in policy.required_phases() {
            assert!(!phase.name().is_empty());
            assert!(
                phase_names.insert(phase.name()),
                "duplicate collector phase {}",
                phase.name()
            );
            assert!(
                phase_kinds.insert(format!("{phase:?}")),
                "duplicate collector phase {phase:?}"
            );
        }
    }

    fn assert_host_boundary_policy(policy: &HeapHostBoundaryPolicy) {
        assert_eq!(policy.name(), "host-import-memory-borrow");
        let borrowed_root_source = policy.borrowed_root_source();
        assert_eq!(borrowed_root_source, HeapRootSource::HostBorrowedValues);
        assert_eq!(borrowed_root_source.owner(), "host-import-boundary");
        assert_eq!(
            borrowed_root_source.kind(),
            HeapRootKind::TransientTaggedValues
        );
        assert!(HEAP_ROOT_SOURCES.contains(&borrowed_root_source));
    }

    fn assert_value_encodings(encodings: &[HeapValueEncoding]) {
        let mut kinds = BTreeSet::new();
        for encoding in encodings {
            assert!(
                kinds.insert(encoding.kind().tag()),
                "duplicate value encoding for {:?}",
                encoding.kind()
            );
            if encoding.preserves_number_bits() {
                assert_eq!(
                    encoding.payload(),
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
        assert_eq!(HEAP_REALM_INTRINSICS_RECORD_SIZE, 424);
        assert_eq!(HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET, 320);
        assert_eq!(
            HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,
            328
        );
        assert_eq!(HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET, 336);
        assert_eq!(HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET, 344);
        assert_eq!(HEAP_REALM_INTRINSICS_ERROR_PROTOTYPE_OFFSET, 352);
        assert_eq!(HEAP_REALM_INTRINSICS_EVAL_ERROR_PROTOTYPE_OFFSET, 360);
        assert_eq!(HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET, 368);
        assert_eq!(HEAP_REALM_INTRINSICS_REFERENCE_ERROR_PROTOTYPE_OFFSET, 376);
        assert_eq!(HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET, 384);
        assert_eq!(HEAP_REALM_INTRINSICS_URI_ERROR_PROTOTYPE_OFFSET, 392);
        assert_eq!(HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET, 400);
        assert_eq!(HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET, 408);
        assert_eq!(HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET, 416);
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
        assert_eq!(HEAP_DISPOSABLE_STACK_RECORD_SIZE, 32);
        assert_eq!(HEAP_DISPOSABLE_STACK_ENTRY_SIZE, 40);
        assert_eq!(DisposableStackState::Pending.word(), 0);
        assert_eq!(DisposableStackState::Disposed.word(), 1);
        assert_eq!(DisposableStackEntryKind::Use.word(), 0);
        assert_eq!(DisposableStackEntryKind::Adopt.word(), 1);
        assert_eq!(DisposableStackEntryKind::Defer.word(), 2);
        assert_ne!(
            OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK,
            OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK
        );
        assert_eq!(HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE, 48);
        assert_eq!(HEAP_MAP_ITERATOR_RECORD_SIZE, 32);
        assert_eq!(HEAP_SET_RECORD_SIZE, 32);
        assert_eq!(HEAP_SET_ENTRY_SIZE, 24);
        assert_eq!(HEAP_SET_ITERATOR_RECORD_SIZE, 32);
        assert_eq!(HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE, 32);
    }

    #[test]
    fn heap_layout_registry_has_no_slot_collisions() {
        let atomics_async_waiter_layout = HEAP_ATOMICS_ASYNC_WAITER_LAYOUT
            .iter()
            .map(AtomicsAsyncWaiterHeapSlot::layout)
            .collect::<Vec<_>>();
        let symbol_layout = HEAP_SYMBOL_LAYOUT
            .iter()
            .map(SymbolHeapSlot::layout)
            .collect::<Vec<_>>();
        let bigint_layout = HEAP_BIGINT_LAYOUT
            .iter()
            .map(BigIntHeapSlot::layout)
            .collect::<Vec<_>>();
        let bound_function_layout = HEAP_BOUND_FUNCTION_LAYOUT
            .iter()
            .map(BoundFunctionHeapSlot::layout)
            .collect::<Vec<_>>();
        let class_function_context_layout = HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT
            .iter()
            .map(ClassFunctionContextHeapSlot::layout)
            .collect::<Vec<_>>();
        let string_layout = HEAP_STRING_LAYOUT
            .iter()
            .map(StringHeapSlot::layout)
            .collect::<Vec<_>>();
        let environment_layout = HEAP_ENVIRONMENT_LAYOUT
            .iter()
            .map(EnvironmentHeapSlot::layout)
            .collect::<Vec<_>>();
        let async_generator_object_layout = HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT
            .iter()
            .map(AsyncGeneratorObjectHeapSlot::layout)
            .collect::<Vec<_>>();
        let pending_completion_layout = HEAP_PENDING_COMPLETION_LAYOUT
            .iter()
            .map(PendingCompletionHeapSlot::layout)
            .collect::<Vec<_>>();
        let pending_job_layout = HEAP_PENDING_JOB_LAYOUT
            .iter()
            .map(PendingJobHeapSlot::layout)
            .collect::<Vec<_>>();
        let promise_reaction_layout = HEAP_PROMISE_REACTION_LAYOUT
            .iter()
            .map(PromiseReactionHeapSlot::layout)
            .collect::<Vec<_>>();
        let intl_locale_layout = HEAP_INTL_LOCALE_RECORD_LAYOUT
            .iter()
            .map(IntlLocaleHeapSlot::layout)
            .collect::<Vec<_>>();
        let intl_date_time_format_layout = HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT
            .iter()
            .map(IntlDateTimeFormatHeapSlot::layout)
            .collect::<Vec<_>>();
        let object_entry_layout = HEAP_OBJECT_ENTRY_LAYOUT
            .iter()
            .map(ObjectEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let realm_record_layout = HEAP_REALM_RECORD_LAYOUT
            .iter()
            .map(RealmRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let private_element_entry_layout = HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT
            .iter()
            .map(PrivateElementEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let map_record_layout = HEAP_MAP_RECORD_LAYOUT
            .iter()
            .map(MapRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let weak_map_record_layout = HEAP_WEAK_MAP_RECORD_LAYOUT
            .iter()
            .map(WeakMapRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let weak_set_record_layout = HEAP_WEAK_SET_RECORD_LAYOUT
            .iter()
            .map(WeakSetRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let set_record_layout = HEAP_SET_RECORD_LAYOUT
            .iter()
            .map(SetRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let set_iterator_layout = HEAP_SET_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(SetIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        let map_entry_layout = HEAP_MAP_ENTRY_LAYOUT
            .iter()
            .map(MapEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let weak_map_entry_layout = HEAP_WEAK_MAP_ENTRY_LAYOUT
            .iter()
            .map(WeakMapEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let map_iterator_layout = HEAP_MAP_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(MapIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        let weak_set_entry_layout = HEAP_WEAK_SET_ENTRY_LAYOUT
            .iter()
            .map(WeakSetEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let set_entry_layout = HEAP_SET_ENTRY_LAYOUT
            .iter()
            .map(SetEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let private_environment_layout = HEAP_PRIVATE_ENV_LAYOUT
            .iter()
            .map(PrivateEnvironmentHeapSlot::layout)
            .collect::<Vec<_>>();
        let weak_ref_layout = HEAP_WEAK_REF_RECORD_LAYOUT
            .iter()
            .map(WeakRefHeapSlot::layout)
            .collect::<Vec<_>>();
        let finalization_registry_record_layout = HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT
            .iter()
            .map(FinalizationRegistryRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let finalization_registry_cell_layout = HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT
            .iter()
            .map(FinalizationRegistryCellHeapSlot::layout)
            .collect::<Vec<_>>();
        let async_disposable_stack_record_layout = HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT
            .iter()
            .map(AsyncDisposableStackRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let async_disposable_stack_entry_layout = HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT
            .iter()
            .map(AsyncDisposableStackEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let disposable_stack_record_layout = HEAP_DISPOSABLE_STACK_RECORD_LAYOUT
            .iter()
            .map(DisposableStackRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let disposable_stack_entry_layout = HEAP_DISPOSABLE_STACK_ENTRY_LAYOUT
            .iter()
            .map(DisposableStackEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_duration_layout = HEAP_TEMPORAL_DURATION_RECORD_LAYOUT
            .iter()
            .map(TemporalDurationHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_instant_layout = HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT
            .iter()
            .map(TemporalInstantHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_zoned_date_time_layout = HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalZonedDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_plain_date_layout = HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainDateHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_plain_date_time_layout = HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_plain_time_layout = HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let typed_array_iterator_layout = HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(TypedArrayIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        let promise_capability_layout = HEAP_PROMISE_CAPABILITY_LAYOUT
            .iter()
            .map(PromiseCapabilityHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_layout(HEAP_OBJECT_HEADER_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(HEAP_GENERATOR_OBJECT_LAYOUT, HEAP_HEADER_SIZE);
        assert_layout(
            HEAP_GENERATOR_DELEGATE_RECORD_LAYOUT,
            HEAP_GENERATOR_DELEGATE_RECORD_SIZE,
        );
        assert_layout(&async_generator_object_layout, HEAP_HEADER_SIZE);
        assert_layout(
            HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT,
            HEAP_ASYNC_ACTIVATION_RECORD_SIZE,
        );
        assert_layout(
            HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT,
            HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE,
        );
        assert_layout(
            HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT,
            HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE,
        );
        assert_layout(
            &pending_completion_layout,
            HEAP_PENDING_COMPLETION_RECORD_SIZE,
        );
        assert_layout(HEAP_FUNCTION_OBJECT_LAYOUT, HEAP_FUNCTION_OBJECT_SIZE);
        assert_layout(
            &class_function_context_layout,
            HEAP_CLASS_FUNCTION_CONTEXT_SIZE,
        );
        assert_layout(
            &private_environment_layout,
            HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET,
        );
        assert_layout(&bound_function_layout, HEAP_BOUND_FUNCTION_RECORD_SIZE);
        assert_layout(
            &private_element_entry_layout,
            HEAP_PRIVATE_ELEMENT_ENTRY_SIZE,
        );
        assert_layout(HEAP_ARRAY_OBJECT_LAYOUT, HEAP_ARRAY_RECORD_SIZE);
        assert_layout(&object_entry_layout, HEAP_OBJECT_ENTRY_SIZE);
        assert_layout(HEAP_ARRAY_ENTRY_LAYOUT, HEAP_ARRAY_ENTRY_SIZE);
        assert_layout(&string_layout, HEAP_STRING_RECORD_SIZE);
        assert_layout(&bigint_layout, HEAP_BIGINT_RECORD_SIZE);
        assert_layout(&symbol_layout, HEAP_SYMBOL_RECORD_SIZE);
        assert_layout(&realm_record_layout, HEAP_REALM_RECORD_SIZE);
        assert_layout(
            HEAP_REALM_INTRINSICS_LAYOUT,
            HEAP_REALM_INTRINSICS_RECORD_SIZE,
        );
        assert_layout(HEAP_PROMISE_LAYOUT, HEAP_PROMISE_RECORD_SIZE);
        assert_layout(
            &promise_capability_layout,
            HEAP_PROMISE_CAPABILITY_RECORD_SIZE,
        );
        assert_layout(&map_record_layout, HEAP_MAP_RECORD_SIZE);
        assert_layout(&map_entry_layout, HEAP_MAP_ENTRY_SIZE);
        assert_layout(&weak_map_record_layout, HEAP_WEAK_MAP_RECORD_SIZE);
        assert_layout(&weak_map_entry_layout, HEAP_WEAK_MAP_ENTRY_SIZE);
        assert_layout(&weak_set_record_layout, HEAP_WEAK_SET_RECORD_SIZE);
        assert_layout(&weak_set_entry_layout, HEAP_WEAK_SET_ENTRY_SIZE);
        assert_layout(&weak_ref_layout, HEAP_WEAK_REF_RECORD_SIZE);
        assert_layout(
            &finalization_registry_record_layout,
            HEAP_FINALIZATION_REGISTRY_RECORD_SIZE,
        );
        assert_layout(
            &finalization_registry_cell_layout,
            HEAP_FINALIZATION_REGISTRY_CELL_SIZE,
        );
        assert_layout(
            &async_disposable_stack_record_layout,
            HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE,
        );
        assert_layout(
            &async_disposable_stack_entry_layout,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE,
        );
        assert_layout(
            &disposable_stack_record_layout,
            HEAP_DISPOSABLE_STACK_RECORD_SIZE,
        );
        assert_layout(
            &disposable_stack_entry_layout,
            HEAP_DISPOSABLE_STACK_ENTRY_SIZE,
        );
        assert_layout(&temporal_instant_layout, HEAP_TEMPORAL_INSTANT_RECORD_SIZE);
        assert_layout(
            &temporal_zoned_date_time_layout,
            HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE,
        );
        assert_layout(
            &temporal_plain_date_layout,
            HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE,
        );
        assert_layout(
            &temporal_duration_layout,
            HEAP_TEMPORAL_DURATION_RECORD_SIZE,
        );
        assert_layout(
            &temporal_plain_time_layout,
            HEAP_TEMPORAL_PLAIN_TIME_RECORD_SIZE,
        );
        assert_layout(
            &temporal_plain_date_time_layout,
            HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_SIZE,
        );
        assert_layout(&intl_locale_layout, HEAP_INTL_LOCALE_RECORD_SIZE);
        assert_layout(
            &intl_date_time_format_layout,
            HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE,
        );
        assert_layout(&map_iterator_layout, HEAP_MAP_ITERATOR_RECORD_SIZE);
        assert_layout(&set_record_layout, HEAP_SET_RECORD_SIZE);
        assert_layout(&set_entry_layout, HEAP_SET_ENTRY_SIZE);
        assert_layout(&set_iterator_layout, HEAP_SET_ITERATOR_RECORD_SIZE);
        assert_layout(
            &typed_array_iterator_layout,
            HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE,
        );
        assert_layout(&promise_reaction_layout, HEAP_PROMISE_REACTION_RECORD_SIZE);
        assert_layout(&pending_job_layout, HEAP_PENDING_JOB_RECORD_SIZE);
        assert_layout(
            &atomics_async_waiter_layout,
            HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE,
        );
        assert_layout(&environment_layout, ENV_SLOT_BASE_OFFSET + ENV_SLOT_SIZE);
    }

    #[test]
    fn heap_layout_registry_marks_gc_pointer_fields() {
        let atomics_async_waiter_layout = HEAP_ATOMICS_ASYNC_WAITER_LAYOUT
            .iter()
            .map(AtomicsAsyncWaiterHeapSlot::layout)
            .collect::<Vec<_>>();
        let bound_function_layout = HEAP_BOUND_FUNCTION_LAYOUT
            .iter()
            .map(BoundFunctionHeapSlot::layout)
            .collect::<Vec<_>>();
        let class_function_context_layout = HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT
            .iter()
            .map(ClassFunctionContextHeapSlot::layout)
            .collect::<Vec<_>>();
        let pending_completion_layout = HEAP_PENDING_COMPLETION_LAYOUT
            .iter()
            .map(PendingCompletionHeapSlot::layout)
            .collect::<Vec<_>>();
        let pending_job_layout = HEAP_PENDING_JOB_LAYOUT
            .iter()
            .map(PendingJobHeapSlot::layout)
            .collect::<Vec<_>>();
        let private_element_entry_layout = HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT
            .iter()
            .map(PrivateElementEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let finalization_registry_record_layout = HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT
            .iter()
            .map(FinalizationRegistryRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let finalization_registry_cell_layout = HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT
            .iter()
            .map(FinalizationRegistryCellHeapSlot::layout)
            .collect::<Vec<_>>();
        let promise_capability_layout = HEAP_PROMISE_CAPABILITY_LAYOUT
            .iter()
            .map(PromiseCapabilityHeapSlot::layout)
            .collect::<Vec<_>>();
        let promise_reaction_layout = HEAP_PROMISE_REACTION_LAYOUT
            .iter()
            .map(PromiseReactionHeapSlot::layout)
            .collect::<Vec<_>>();
        let intl_locale_layout = HEAP_INTL_LOCALE_RECORD_LAYOUT
            .iter()
            .map(IntlLocaleHeapSlot::layout)
            .collect::<Vec<_>>();
        let intl_date_time_format_layout = HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT
            .iter()
            .map(IntlDateTimeFormatHeapSlot::layout)
            .collect::<Vec<_>>();
        let object_entry_layout = HEAP_OBJECT_ENTRY_LAYOUT
            .iter()
            .map(ObjectEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        let realm_record_layout = HEAP_REALM_RECORD_LAYOUT
            .iter()
            .map(RealmRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_zoned_date_time_layout = HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalZonedDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_plain_time_layout = HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_plain_date_time_layout = HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        let temporal_duration_layout = HEAP_TEMPORAL_DURATION_RECORD_LAYOUT
            .iter()
            .map(TemporalDurationHeapSlot::layout)
            .collect::<Vec<_>>();
        let pointer_slots = HEAP_OBJECT_HEADER_LAYOUT
            .iter()
            .chain(HEAP_GENERATOR_OBJECT_LAYOUT.iter())
            .chain(HEAP_GENERATOR_DELEGATE_RECORD_LAYOUT.iter())
            .chain(HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT.iter())
            .chain(HEAP_ASYNC_GENERATOR_ACTIVATION_LAYOUT.iter())
            .chain(HEAP_ASYNC_GENERATOR_REQUEST_LAYOUT.iter())
            .chain(HEAP_FUNCTION_OBJECT_LAYOUT.iter())
            .chain(HEAP_ARRAY_OBJECT_LAYOUT.iter())
            .chain(HEAP_ARRAY_ENTRY_LAYOUT.iter())
            .chain(HEAP_REALM_INTRINSICS_LAYOUT.iter())
            .chain(HEAP_PROMISE_LAYOUT.iter())
            .filter(|slot| slot.pointer)
            .count()
            + intl_locale_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + intl_date_time_format_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + object_entry_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + realm_record_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + temporal_plain_time_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + temporal_plain_date_time_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + temporal_duration_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + promise_reaction_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + HEAP_SYMBOL_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_BIGINT_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_STRING_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_ENVIRONMENT_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_MAP_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_SET_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_SET_ITERATOR_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_MAP_ENTRY_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_MAP_ITERATOR_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_WEAK_SET_ENTRY_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_SET_ENTRY_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_PRIVATE_ENV_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_WEAK_REF_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_DISPOSABLE_STACK_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_DISPOSABLE_STACK_ENTRY_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT
                .iter()
                .filter(|slot| slot.layout().pointer)
                .count()
            + pending_completion_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + atomics_async_waiter_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + bound_function_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + class_function_context_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + finalization_registry_record_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + finalization_registry_cell_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + promise_capability_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + pending_job_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + private_element_entry_layout
                .iter()
                .filter(|slot| slot.pointer)
                .count()
            + temporal_zoned_date_time_layout
                .iter()
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
        assert!(class_function_context_layout.iter().all(|slot| {
            (slot.offset == HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET) == !slot.pointer
        }));
        assert!(HEAP_FUNCTION_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "defining_realm"
                && slot.offset == HEAP_FUNCTION_DEFINING_REALM_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_FUNCTION_OBJECT_LAYOUT.iter().any(|slot| {
            slot.name == "builtin_closure_context"
                && slot.offset == HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_RECORD_LAYOUT.iter().any(|slot| {
            let slot = slot.layout();
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
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%Date.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_REALM_INTRINSICS_LAYOUT.iter().any(|slot| {
            slot.name == "%Error.prototype%"
                && slot.offset == HEAP_REALM_INTRINSICS_ERROR_PROTOTYPE_OFFSET
                && slot.pointer
        }));
        for (name, offset) in [
            (
                "%EvalError.prototype%",
                HEAP_REALM_INTRINSICS_EVAL_ERROR_PROTOTYPE_OFFSET,
            ),
            (
                "%RangeError.prototype%",
                HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET,
            ),
            (
                "%ReferenceError.prototype%",
                HEAP_REALM_INTRINSICS_REFERENCE_ERROR_PROTOTYPE_OFFSET,
            ),
            (
                "%SyntaxError.prototype%",
                HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET,
            ),
            (
                "%URIError.prototype%",
                HEAP_REALM_INTRINSICS_URI_ERROR_PROTOTYPE_OFFSET,
            ),
            (
                "%Promise.prototype%",
                HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET,
            ),
            (
                "%Function.prototype%",
                HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            ),
            (
                "%Promise%",
                HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET,
            ),
        ] {
            assert!(HEAP_REALM_INTRINSICS_LAYOUT
                .iter()
                .any(|slot| { slot.name == name && slot.offset == offset && slot.pointer }));
        }
        assert!(HEAP_BIGINT_LAYOUT.iter().any(|slot| {
            let slot = slot.layout();
            slot.name == "limbs_ptr" && slot.offset == HEAP_BIGINT_LIMBS_PTR_OFFSET && slot.pointer
        }));
        assert!(HEAP_SYMBOL_LAYOUT.iter().any(|slot| {
            let slot = slot.layout();
            slot.name == "description_payload"
                && slot.offset == HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET
                && slot.pointer
        }));
        assert!(HEAP_PROMISE_LAYOUT.iter().any(|slot| {
            slot.name == "result_payload"
                && slot.offset == HEAP_PROMISE_RESULT_PAYLOAD_OFFSET
                && slot.pointer
        }));
        assert!(pending_job_layout.iter().any(|slot| {
            slot.name == "next" && slot.offset == HEAP_PENDING_JOB_NEXT_OFFSET && slot.pointer
        }));
        assert!(pending_completion_layout.iter().any(|slot| {
            slot.name == "next"
                && slot.offset == HEAP_PENDING_COMPLETION_NEXT_OFFSET
                && slot.pointer
        }));
        assert!(pending_completion_layout.iter().any(|slot| {
            slot.name == "payload"
                && slot.offset == HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET
                && slot.pointer
        }));
    }

    #[test]
    fn async_generator_records_expose_queue_activation_and_promise_edges_to_gc() {
        let promise_capability_layout = HEAP_PROMISE_CAPABILITY_LAYOUT
            .iter()
            .map(PromiseCapabilityHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE, 184);
        assert_eq!(HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE, 56);
        assert_ne!(
            OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR,
            OBJECT_INTERNAL_BRAND_GENERATOR
        );
        assert_eq!(
            AsyncGeneratorExecutionState::ALL.map(AsyncGeneratorExecutionState::word),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(ASYNC_GENERATOR_RESUME_STATE_INITIALIZING, u64::MAX);
        assert_eq!(
            AsyncGeneratorBodyStatus::ALL.map(AsyncGeneratorBodyStatus::word),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            AsyncGeneratorResumeKind::ALL.map(AsyncGeneratorResumeKind::word),
            [0, 1, 2, 3, 4]
        );

        assert!(HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT.iter().any(|slot| {
            let slot = slot.layout();
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
            let slot = slot.layout();
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
                promise_capability_layout
                    .iter()
                    .any(|slot| slot.name == name && slot.pointer),
                "Promise capability must trace {name}"
            );
        }
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
            ArrayBufferFlag::Resizable.word()
                | ArrayBufferFlag::Shared.word()
                | ArrayBufferFlag::Immutable.word()
                | ArrayBufferFlag::Detached.word(),
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
        for family in HEAP_NAMED_SLOT_FAMILIES {
            assert_named_slots(family.slots());
        }
        assert!(HEAP_ARRAY_ITERATOR_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$ArrayIterator.array"
                && slot.storage == HeapNamedSlotStorage::StrongReference
        }));
        assert!(HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$RegExpStringIterator.regexp"
                && slot.storage == HeapNamedSlotStorage::StrongReference
        }));
        assert!(HEAP_ITERATOR_HELPER_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$IteratorFromIterator"
                && slot.storage == HeapNamedSlotStorage::StrongReference
        }));
        assert!(HEAP_ITERATOR_HELPER_NAMED_SLOTS.iter().any(|slot| {
            slot.key == "$IteratorFlatMapInnerNext"
                && slot.storage == HeapNamedSlotStorage::StrongReference
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
            assert!(HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS.iter().any(|slot| {
                slot.key == key && slot.storage == HeapNamedSlotStorage::StrongReference
            }));
        }
        for key in [
            "$IteratorZipDone",
            "$IteratorZipExecuting",
            "$IteratorZipStarted",
            "$IteratorZipMode",
        ] {
            assert!(HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS
                .iter()
                .any(|slot| { slot.key == key && slot.storage == HeapNamedSlotStorage::Scalar }));
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
            assert!(HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS.iter().any(|slot| {
                slot.key == key && slot.storage == HeapNamedSlotStorage::StrongReference
            }));
        }
        for key in [
            "$IteratorConcatIndex",
            "$IteratorConcatActive",
            "$IteratorConcatDone",
            "$IteratorConcatExecuting",
        ] {
            assert!(HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS
                .iter()
                .any(|slot| { slot.key == key && slot.storage == HeapNamedSlotStorage::Scalar }));
        }
    }

    #[test]
    fn heap_root_registry_covers_gc_safepoint_sources() {
        assert_root_sources(HEAP_ROOT_SOURCES);
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            *source == HeapRootSource::ActiveFrameLocals
                && source.kind() == HeapRootKind::TransientTaggedValues
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            *source == HeapRootSource::CompletionRecords
                && source.owner() == "completion-abi"
                && source.kind() == HeapRootKind::TransientTaggedValues
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            *source == HeapRootSource::HostBorrowedValues
                && source.kind() == HeapRootKind::TransientTaggedValues
        }));
        assert!(HEAP_ROOT_SOURCES.iter().any(|source| {
            *source == HeapRootSource::PendingJobs
                && source.kind() == HeapRootKind::PersistentTaggedValues
        }));
    }

    #[test]
    fn heap_weak_edge_registry_models_ephemerons_and_finalizers() {
        assert_weak_edges(HEAP_WEAK_EDGES);
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-map-entry"
                && edge.name() == "key"
                && edge.kind() == HeapWeakEdgeKind::EphemeronKey
                && edge.kind().retention() == HeapWeakEdgeRetention::DoesNotRetain
        }));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-map-entry"
                && edge.name() == "value"
                && edge.kind() == HeapWeakEdgeKind::EphemeronValue
                && edge.kind().retention()
                    == HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey
        }));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-ref-record"
                && edge.kind() == HeapWeakEdgeKind::WeakTarget
                && edge.kind().retention() == HeapWeakEdgeRetention::DoesNotRetain
        }));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-set-entry"
                && edge.name() == "value"
                && edge.kind() == HeapWeakEdgeKind::EphemeronKey
                && edge.kind().retention() == HeapWeakEdgeRetention::DoesNotRetain
        }));
        assert!(HEAP_WEAK_SET_ENTRY_LAYOUT
            .iter()
            .map(WeakSetEntryHeapSlot::layout)
            .find(|slot| slot.name == "value_payload")
            .is_some_and(|slot| !slot.pointer));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "finalization-registry-cell"
                && edge.name() == "holdings"
                && edge.kind() == HeapWeakEdgeKind::FinalizerHoldings
                && edge.kind().retention() == HeapWeakEdgeRetention::StrongUntilCleanup
        }));
    }

    #[test]
    fn heap_weak_edge_kinds_own_their_retention_semantics() {
        for (kind, retention) in [
            (
                HeapWeakEdgeKind::EphemeronKey,
                HeapWeakEdgeRetention::DoesNotRetain,
            ),
            (
                HeapWeakEdgeKind::EphemeronValue,
                HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey,
            ),
            (
                HeapWeakEdgeKind::WeakTarget,
                HeapWeakEdgeRetention::DoesNotRetain,
            ),
            (
                HeapWeakEdgeKind::FinalizerHoldings,
                HeapWeakEdgeRetention::StrongUntilCleanup,
            ),
            (
                HeapWeakEdgeKind::FinalizerToken,
                HeapWeakEdgeRetention::DoesNotRetain,
            ),
        ] {
            assert_eq!(kind.retention(), retention);
        }
    }

    #[test]
    fn heap_collector_policy_requires_all_gc_builtin_phases() {
        assert_collector_policy(&HEAP_COLLECTOR_POLICY);
        for phase in [
            RequiredHeapCollectorPhase::StopTheWorld,
            RequiredHeapCollectorPhase::RootScan,
            RequiredHeapCollectorPhase::MarkStrong,
            RequiredHeapCollectorPhase::ProcessEphemerons,
            RequiredHeapCollectorPhase::ClearWeakRefs,
            RequiredHeapCollectorPhase::QueueFinalizers,
            RequiredHeapCollectorPhase::Sweep,
            RequiredHeapCollectorPhase::Resume,
        ] {
            assert!(
                REQUIRED_HEAP_COLLECTOR_PHASES.contains(&phase),
                "missing required collector phase {phase:?}"
            );
        }
    }

    #[test]
    fn heap_collector_policy_keeps_gc_builtin_unsupported_until_executable() {
        assert!(!heap_collector_is_executable());
        assert_eq!(HEAP_COLLECTOR_POLICY.name(), "non-moving-tracing-collector");
        assert!(!HEAP_COLLECTOR_POLICY.moves_objects());
        assert!(HEAP_COLLECTOR_POLICY
            .required_phases()
            .contains(&RequiredHeapCollectorPhase::Sweep));
        assert!(HEAP_COLLECTOR_POLICY
            .weak_edges()
            .iter()
            .any(|edge| edge.kind() == HeapWeakEdgeKind::EphemeronKey));
        assert!(HEAP_COLLECTOR_POLICY
            .weak_edges()
            .iter()
            .any(|edge| edge.kind() == HeapWeakEdgeKind::FinalizerHoldings));
    }

    #[test]
    fn weak_map_entries_are_ephemerons_not_strong_heap_edges() {
        assert!(HEAP_WEAK_MAP_ENTRY_LAYOUT
            .iter()
            .all(|slot| !slot.layout().pointer));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-map-entry"
                && edge.name() == "key"
                && edge.kind() == HeapWeakEdgeKind::EphemeronKey
        }));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-map-entry"
                && edge.name() == "value"
                && edge.kind() == HeapWeakEdgeKind::EphemeronValue
        }));
    }

    #[test]
    fn weak_ref_target_is_not_a_strong_heap_edge() {
        assert!(HEAP_WEAK_REF_RECORD_LAYOUT
            .iter()
            .all(|slot| !slot.layout().pointer));
        assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
            edge.record() == "weak-ref-record"
                && edge.name() == "target"
                && edge.kind() == HeapWeakEdgeKind::WeakTarget
                && edge.kind().retention() == HeapWeakEdgeRetention::DoesNotRetain
        }));
    }

    #[test]
    fn weak_ref_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_WEAK_REF_RECORD_LAYOUT
            .iter()
            .map(WeakRefHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 2);

        let tag = &layouts[0];
        assert_eq!(tag.record, "weak-ref-record");
        assert_eq!(tag.name, "target_tag");
        assert_eq!(tag.offset, HEAP_WEAK_REF_TARGET_TAG_OFFSET);
        assert_eq!(tag.width, 8);
        assert!(!tag.pointer);

        let payload = &layouts[1];
        assert_eq!(payload.record, "weak-ref-record");
        assert_eq!(payload.name, "target_payload");
        assert_eq!(payload.offset, HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET);
        assert_eq!(payload.width, 8);
        assert!(!payload.pointer);

        assert_layout(&layouts, HEAP_WEAK_REF_RECORD_SIZE);
    }

    #[test]
    fn promise_capability_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PROMISE_CAPABILITY_LAYOUT
            .iter()
            .map(PromiseCapabilityHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "promise_tag",
                HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "promise_payload",
                HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "resolve_tag",
                HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
                false,
            ),
            (
                &layouts[3],
                "resolve_payload",
                HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[4],
                "reject_tag",
                HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "reject_payload",
                HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "promise-capability-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_PROMISE_CAPABILITY_RECORD_SIZE);
    }

    #[test]
    fn intl_locale_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_INTL_LOCALE_RECORD_LAYOUT
            .iter()
            .map(IntlLocaleHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        for (slot, name, offset) in [
            (&layouts[0], "tag_payload", HEAP_INTL_LOCALE_TAG_OFFSET),
            (
                &layouts[1],
                "language_payload",
                HEAP_INTL_LOCALE_LANGUAGE_OFFSET,
            ),
            (
                &layouts[2],
                "script_payload",
                HEAP_INTL_LOCALE_SCRIPT_OFFSET,
            ),
            (
                &layouts[3],
                "region_payload",
                HEAP_INTL_LOCALE_REGION_OFFSET,
            ),
            (
                &layouts[4],
                "base_name_payload",
                HEAP_INTL_LOCALE_BASE_NAME_OFFSET,
            ),
        ] {
            assert_eq!(slot.record, "intl-locale-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert!(slot.pointer);
        }

        assert_layout(&layouts, HEAP_INTL_LOCALE_RECORD_SIZE);
    }

    #[test]
    fn intl_date_time_format_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT
            .iter()
            .map(IntlDateTimeFormatHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 23);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "locale_payload",
                HEAP_INTL_DTF_LOCALE_OFFSET,
                true,
            ),
            (
                &layouts[1],
                "calendar_payload",
                HEAP_INTL_DTF_CALENDAR_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "numbering_system_payload",
                HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "time_zone_payload",
                HEAP_INTL_DTF_TIME_ZONE_OFFSET,
                true,
            ),
            (
                &layouts[4],
                "time_zone_offset_minutes",
                HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "time_zone_gmt_name_payload",
                HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET,
                true,
            ),
            (
                &layouts[6],
                "hour_cycle_code",
                HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
                false,
            ),
            (
                &layouts[7],
                "weekday_code",
                HEAP_INTL_DTF_WEEKDAY_OFFSET,
                false,
            ),
            (&layouts[8], "era_code", HEAP_INTL_DTF_ERA_OFFSET, false),
            (&layouts[9], "year_code", HEAP_INTL_DTF_YEAR_OFFSET, false),
            (
                &layouts[10],
                "month_code",
                HEAP_INTL_DTF_MONTH_OFFSET,
                false,
            ),
            (&layouts[11], "day_code", HEAP_INTL_DTF_DAY_OFFSET, false),
            (
                &layouts[12],
                "day_period_code",
                HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
                false,
            ),
            (&layouts[13], "hour_code", HEAP_INTL_DTF_HOUR_OFFSET, false),
            (
                &layouts[14],
                "minute_code",
                HEAP_INTL_DTF_MINUTE_OFFSET,
                false,
            ),
            (
                &layouts[15],
                "second_code",
                HEAP_INTL_DTF_SECOND_OFFSET,
                false,
            ),
            (
                &layouts[16],
                "fractional_second_digits",
                HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
                false,
            ),
            (
                &layouts[17],
                "time_zone_name_code",
                HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
                false,
            ),
            (
                &layouts[18],
                "date_style_code",
                HEAP_INTL_DTF_DATE_STYLE_OFFSET,
                false,
            ),
            (
                &layouts[19],
                "time_style_code",
                HEAP_INTL_DTF_TIME_STYLE_OFFSET,
                false,
            ),
            (
                &layouts[20],
                "hour12_code",
                HEAP_INTL_DTF_HOUR12_OFFSET,
                false,
            ),
            (
                &layouts[21],
                "bound_format_payload",
                HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
                true,
            ),
            (
                &layouts[22],
                "need_defaults",
                HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
                false,
            ),
        ] {
            assert_eq!(slot.record, "intl-date-time-format-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE);
    }
    #[test]
    fn object_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_OBJECT_ENTRY_LAYOUT
            .iter()
            .map(ObjectEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 8);

        for (slot, name, offset, pointer) in [
            (&layouts[0], "key", HEAP_OBJECT_KEY_OFFSET, true),
            (
                &layouts[1],
                "descriptor_kind",
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                false,
            ),
            (&layouts[2], "data_tag", HEAP_OBJECT_DATA_TAG_OFFSET, false),
            (
                &layouts[3],
                "data_payload",
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[4],
                "getter_tag",
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "getter_payload",
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[6],
                "setter_tag",
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                false,
            ),
            (
                &layouts[7],
                "setter_payload",
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "object-entry");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_OBJECT_ENTRY_SIZE);
    }

    #[test]
    fn realm_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_REALM_RECORD_LAYOUT
            .iter()
            .map(RealmRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 9);

        for (slot, name, offset, pointer) in [
            (&layouts[0], "realm_id", HEAP_REALM_ID_OFFSET, false),
            (&layouts[1], "agent_id", HEAP_REALM_AGENT_ID_OFFSET, false),
            (
                &layouts[2],
                "global_object",
                HEAP_REALM_GLOBAL_OBJECT_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "global_this",
                HEAP_REALM_GLOBAL_THIS_OFFSET,
                true,
            ),
            (
                &layouts[4],
                "global_environment",
                HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
                true,
            ),
            (
                &layouts[5],
                "intrinsics",
                HEAP_REALM_INTRINSICS_OFFSET,
                true,
            ),
            (
                &layouts[6],
                "host_hooks",
                HEAP_REALM_HOST_HOOKS_OFFSET,
                true,
            ),
            (
                &layouts[7],
                "module_registry",
                HEAP_REALM_MODULE_REGISTRY_OFFSET,
                true,
            ),
            (
                &layouts[8],
                "private_elements",
                HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "realm-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_REALM_RECORD_SIZE);
    }

    #[test]
    fn promise_reaction_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PROMISE_REACTION_LAYOUT
            .iter()
            .map(PromiseReactionHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 7);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "capability",
                HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
                true,
            ),
            (
                &layouts[1],
                "handler_tag",
                HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
                false,
            ),
            (
                &layouts[2],
                "handler_payload",
                HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "realm",
                HEAP_PROMISE_REACTION_REALM_OFFSET,
                true,
            ),
            (&layouts[4], "next", HEAP_PROMISE_REACTION_NEXT_OFFSET, true),
            (
                &layouts[5],
                "type",
                HEAP_PROMISE_REACTION_TYPE_OFFSET,
                false,
            ),
            (
                &layouts[6],
                "callback_kind",
                HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
                false,
            ),
        ] {
            assert_eq!(slot.record, "promise-reaction-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_PROMISE_REACTION_RECORD_SIZE);
    }

    #[test]
    fn pending_job_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PENDING_JOB_LAYOUT
            .iter()
            .map(PendingJobHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 7);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "callback_tag",
                HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "callback_payload",
                HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "arg_tag",
                HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                false,
            ),
            (
                &layouts[3],
                "arg_payload",
                HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                true,
            ),
            (&layouts[4], "realm", HEAP_PENDING_JOB_REALM_OFFSET, true),
            (&layouts[5], "next", HEAP_PENDING_JOB_NEXT_OFFSET, true),
            (&layouts[6], "kind", HEAP_PENDING_JOB_KIND_OFFSET, false),
        ] {
            assert_eq!(slot.record, "pending-job-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_PENDING_JOB_RECORD_SIZE);
    }

    #[test]
    fn finalization_registry_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT
            .iter()
            .map(FinalizationRegistryRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "cleanup_callback_tag",
                HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "cleanup_callback_payload",
                HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "cells_ptr",
                HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "cells_len",
                HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
                false,
            ),
            (
                &layouts[4],
                "cells_cap",
                HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
                false,
            ),
        ] {
            assert_eq!(slot.record, "finalization-registry-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_FINALIZATION_REGISTRY_RECORD_SIZE);
    }

    #[test]
    fn finalization_registry_cell_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT
            .iter()
            .map(FinalizationRegistryCellHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 7);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "state",
                HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "target_tag",
                HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
                false,
            ),
            (
                &layouts[2],
                "target_payload",
                HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
                false,
            ),
            (
                &layouts[3],
                "holdings_tag",
                HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
                false,
            ),
            (
                &layouts[4],
                "holdings_payload",
                HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[5],
                "unregister_token_tag",
                HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
                false,
            ),
            (
                &layouts[6],
                "unregister_token_payload",
                HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
                false,
            ),
        ] {
            assert_eq!(slot.record, "finalization-registry-cell");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_FINALIZATION_REGISTRY_CELL_SIZE);
    }

    #[test]
    fn finalization_registry_cells_keep_only_holdings_strongly_reachable() {
        let layouts = HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT
            .iter()
            .map(FinalizationRegistryCellHeapSlot::layout)
            .collect::<Vec<_>>();
        assert!(layouts
            .iter()
            .any(|slot| { slot.name == "holdings_payload" && slot.pointer }));
        assert!(layouts.iter().all(|slot| {
            !matches!(slot.name, "target_payload" | "unregister_token_payload") || !slot.pointer
        }));
        for (name, kind, retention) in [
            (
                "target",
                HeapWeakEdgeKind::WeakTarget,
                HeapWeakEdgeRetention::DoesNotRetain,
            ),
            (
                "holdings",
                HeapWeakEdgeKind::FinalizerHoldings,
                HeapWeakEdgeRetention::StrongUntilCleanup,
            ),
            (
                "unregister-token",
                HeapWeakEdgeKind::FinalizerToken,
                HeapWeakEdgeRetention::DoesNotRetain,
            ),
        ] {
            assert!(HEAP_WEAK_EDGES.iter().any(|edge| {
                edge.record() == "finalization-registry-cell"
                    && edge.name() == name
                    && edge.kind() == kind
                    && edge.kind().retention() == retention
            }));
        }
    }

    #[test]
    fn heap_host_boundary_is_call_scoped_and_transiently_rooted() {
        assert_host_boundary_policy(&HEAP_HOST_BOUNDARY_POLICY);
    }

    #[test]
    fn heap_value_encoding_registry_covers_ecmascript_language_types() {
        assert_value_encodings(HEAP_VALUE_ENCODINGS);
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
                HEAP_VALUE_ENCODINGS
                    .iter()
                    .any(|encoding| encoding.kind() == kind),
                "missing value encoding for {:?}",
                kind
            );
        }

        let number = HEAP_VALUE_ENCODINGS
            .iter()
            .find(|encoding| encoding.kind() == ValueKind::Number)
            .expect("Number encoding should be registered");
        assert_eq!(number.payload(), ValuePayloadEncoding::Ieee754Bits);
        assert!(number.preserves_number_bits());

        let bigint = HEAP_VALUE_ENCODINGS
            .iter()
            .find(|encoding| encoding.kind() == ValueKind::BigInt)
            .expect("BigInt encoding should be registered");
        assert_eq!(
            bigint.payload(),
            ValuePayloadEncoding::I64TemporaryOrHeapPointer
        );
        assert!(
            !bigint.arbitrary_precision_ready(),
            "T05 must keep the current BigInt storage gap visible"
        );
    }

    #[test]
    fn linear_side_storage_identities_own_metadata_and_element_semantics() {
        assert_eq!(LINEAR_SIDE_STORAGES.len(), 3);
        for storage in LINEAR_SIDE_STORAGES {
            assert_linear_side_storage(storage);
        }
        assert_eq!(
            LinearSideStorage::ArrayBufferBackingStore.length_source(),
            "array-buffer-object-header.max_byte_length"
        );
        assert_eq!(
            LinearSideStorage::ArrayBufferBackingStore.element(),
            LinearSideStorageElement::Byte
        );
        assert_eq!(
            LinearSideStorage::StringCodeUnits.element(),
            LinearSideStorageElement::Utf16CodeUnit
        );
        assert_eq!(
            LinearSideStorage::BigIntLimbs.element(),
            LinearSideStorageElement::BigIntLimb
        );
        assert_eq!(
            LinearSideStorage::ArrayBufferBackingStore
                .element()
                .byte_width(),
            1
        );
        assert_eq!(LinearSideStorage::StringCodeUnits.element().byte_width(), 2);
        assert_eq!(LinearSideStorage::BigIntLimbs.element().byte_width(), 8);
    }

    #[test]
    fn temporal_instant_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT
            .iter()
            .map(TemporalInstantHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 2);

        let tag = &layouts[0];
        assert_eq!(tag.record, "temporal-instant-record");
        assert_eq!(tag.name, "epoch_nanoseconds_tag");
        assert_eq!(
            tag.offset,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET
        );
        assert_eq!(tag.width, 8);
        assert!(!tag.pointer);

        let payload = &layouts[1];
        assert_eq!(payload.record, "temporal-instant-record");
        assert_eq!(payload.name, "epoch_nanoseconds_payload");
        assert_eq!(
            payload.offset,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET
        );
        assert_eq!(payload.width, 8);
        assert!(payload.pointer);

        assert_layout(&layouts, HEAP_TEMPORAL_INSTANT_RECORD_SIZE);
    }

    #[test]
    fn temporal_zoned_date_time_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalZonedDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "epoch_nanoseconds_tag",
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "epoch_nanoseconds_payload",
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "time_zone_tag",
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                false,
            ),
            (
                &layouts[3],
                "time_zone_payload",
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[4],
                "calendar_tag",
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "calendar_payload",
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "temporal-zoned-date-time-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE);
    }

    #[test]
    fn class_function_context_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT
            .iter()
            .map(ClassFunctionContextHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "lexical_env",
                HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                true,
            ),
            (
                &layouts[1],
                "active_function",
                HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "home_object_payload",
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "home_object_tag",
                HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                false,
            ),
            (
                &layouts[4],
                "field_keys",
                HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                true,
            ),
            (
                &layouts[5],
                "private_environment",
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "class-function-context");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_CLASS_FUNCTION_CONTEXT_SIZE);
    }

    #[test]
    fn private_element_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT
            .iter()
            .map(PrivateElementEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "next",
                HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
                true,
            ),
            (
                &layouts[1],
                "receiver",
                HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                true,
            ),
            (
                &layouts[2],
                "token",
                HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
                true,
            ),
            (
                &layouts[3],
                "kind",
                HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
                false,
            ),
            (
                &layouts[4],
                "value_tag",
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "value_payload",
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "private-element-entry");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_PRIVATE_ELEMENT_ENTRY_SIZE);
    }

    #[test]
    fn bound_function_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_BOUND_FUNCTION_LAYOUT
            .iter()
            .map(BoundFunctionHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        let target_payload = &layouts[0];
        assert_eq!(target_payload.record, "bound-function");
        assert_eq!(target_payload.name, "target_payload");
        assert_eq!(
            target_payload.offset,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET
        );
        assert_eq!(target_payload.width, 8);
        assert!(target_payload.pointer);

        let target_tag = &layouts[1];
        assert_eq!(target_tag.record, "bound-function");
        assert_eq!(target_tag.name, "target_tag");
        assert_eq!(target_tag.offset, HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET);
        assert_eq!(target_tag.width, 8);
        assert!(!target_tag.pointer);

        let this_payload = &layouts[2];
        assert_eq!(this_payload.record, "bound-function");
        assert_eq!(this_payload.name, "this_payload");
        assert_eq!(this_payload.offset, HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET);
        assert_eq!(this_payload.width, 8);
        assert!(this_payload.pointer);

        let this_tag = &layouts[3];
        assert_eq!(this_tag.record, "bound-function");
        assert_eq!(this_tag.name, "this_tag");
        assert_eq!(this_tag.offset, HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET);
        assert_eq!(this_tag.width, 8);
        assert!(!this_tag.pointer);

        let arguments_payload = &layouts[4];
        assert_eq!(arguments_payload.record, "bound-function");
        assert_eq!(arguments_payload.name, "args_payload");
        assert_eq!(
            arguments_payload.offset,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET
        );
        assert_eq!(arguments_payload.width, 8);
        assert!(arguments_payload.pointer);

        let self_payload = &layouts[5];
        assert_eq!(self_payload.record, "bound-function");
        assert_eq!(self_payload.name, "self_payload");
        assert_eq!(self_payload.offset, HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET);
        assert_eq!(self_payload.width, 8);
        assert!(self_payload.pointer);

        assert_layout(&layouts, HEAP_BOUND_FUNCTION_RECORD_SIZE);
    }

    #[test]
    fn atomics_async_waiter_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_ATOMICS_ASYNC_WAITER_LAYOUT
            .iter()
            .map(AtomicsAsyncWaiterHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        let state = &layouts[0];
        assert_eq!(state.record, "atomics-async-waiter");
        assert_eq!(state.name, "state");
        assert_eq!(state.offset, HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET);
        assert_eq!(state.width, 8);
        assert!(!state.pointer);

        let address = &layouts[1];
        assert_eq!(address.record, "atomics-async-waiter");
        assert_eq!(address.name, "address");
        assert_eq!(address.offset, HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET);
        assert_eq!(address.width, 8);
        assert!(!address.pointer);

        let promise_record = &layouts[2];
        assert_eq!(promise_record.record, "atomics-async-waiter");
        assert_eq!(promise_record.name, "promise_record");
        assert_eq!(
            promise_record.offset,
            HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET
        );
        assert_eq!(promise_record.width, 8);
        assert!(promise_record.pointer);

        let deadline_nanos = &layouts[3];
        assert_eq!(deadline_nanos.record, "atomics-async-waiter");
        assert_eq!(deadline_nanos.name, "deadline_nanos");
        assert_eq!(
            deadline_nanos.offset,
            HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET
        );
        assert_eq!(deadline_nanos.width, 8);
        assert!(!deadline_nanos.pointer);

        let next = &layouts[4];
        assert_eq!(next.record, "atomics-async-waiter");
        assert_eq!(next.name, "next");
        assert_eq!(next.offset, HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET);
        assert_eq!(next.width, 8);
        assert!(next.pointer);

        let host_id = &layouts[5];
        assert_eq!(host_id.record, "atomics-async-waiter");
        assert_eq!(host_id.name, "host_id");
        assert_eq!(host_id.offset, HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET);
        assert_eq!(host_id.width, 8);
        assert!(!host_id.pointer);

        assert_layout(&layouts, HEAP_ATOMICS_ASYNC_WAITER_RECORD_SIZE);
    }

    #[test]
    fn pending_completion_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PENDING_COMPLETION_LAYOUT
            .iter()
            .map(PendingCompletionHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        let next = &layouts[0];
        assert_eq!(next.record, "pending-completion-record");
        assert_eq!(next.name, "next");
        assert_eq!(next.offset, HEAP_PENDING_COMPLETION_NEXT_OFFSET);
        assert_eq!(next.width, 8);
        assert!(next.pointer);

        let payload = &layouts[1];
        assert_eq!(payload.record, "pending-completion-record");
        assert_eq!(payload.name, "payload");
        assert_eq!(payload.offset, HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET);
        assert_eq!(payload.width, 8);
        assert!(payload.pointer);

        let tag = &layouts[2];
        assert_eq!(tag.record, "pending-completion-record");
        assert_eq!(tag.name, "tag");
        assert_eq!(tag.offset, HEAP_PENDING_COMPLETION_TAG_OFFSET);
        assert_eq!(tag.width, 8);
        assert!(!tag.pointer);

        let kind = &layouts[3];
        assert_eq!(kind.record, "pending-completion-record");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_PENDING_COMPLETION_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let aux = &layouts[4];
        assert_eq!(aux.record, "pending-completion-record");
        assert_eq!(aux.name, "aux");
        assert_eq!(aux.offset, HEAP_PENDING_COMPLETION_AUX_OFFSET);
        assert_eq!(aux.width, 8);
        assert!(!aux.pointer);

        assert_layout(&layouts, HEAP_PENDING_COMPLETION_RECORD_SIZE);
    }

    #[test]
    fn async_disposable_stack_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT
            .iter()
            .map(AsyncDisposableStackRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let state = &layouts[0];
        assert_eq!(state.record, "async-disposable-stack-record");
        assert_eq!(state.name, "state");
        assert_eq!(state.offset, HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET);
        assert_eq!(state.width, 8);
        assert!(!state.pointer);

        let entries_pointer = &layouts[1];
        assert_eq!(entries_pointer.record, "async-disposable-stack-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(
            entries_pointer.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET
        );
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[2];
        assert_eq!(entries_length.record, "async-disposable-stack-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(
            entries_length.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET
        );
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[3];
        assert_eq!(entries_capacity.record, "async-disposable-stack-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(
            entries_capacity.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET
        );
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        assert_layout(&layouts, HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE);
    }

    #[test]
    fn async_disposable_stack_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT
            .iter()
            .map(AsyncDisposableStackEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        let kind = &layouts[0];
        assert_eq!(kind.record, "async-disposable-stack-entry");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let value_tag = &layouts[1];
        assert_eq!(value_tag.record, "async-disposable-stack-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(
            value_tag.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET
        );
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[2];
        assert_eq!(value_payload.record, "async-disposable-stack-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(
            value_payload.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET
        );
        assert_eq!(value_payload.width, 8);
        assert!(value_payload.pointer);

        let method_tag = &layouts[3];
        assert_eq!(method_tag.record, "async-disposable-stack-entry");
        assert_eq!(method_tag.name, "method_tag");
        assert_eq!(
            method_tag.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET
        );
        assert_eq!(method_tag.width, 8);
        assert!(!method_tag.pointer);

        let method_payload = &layouts[4];
        assert_eq!(method_payload.record, "async-disposable-stack-entry");
        assert_eq!(method_payload.name, "method_payload");
        assert_eq!(
            method_payload.offset,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET
        );
        assert_eq!(method_payload.width, 8);
        assert!(method_payload.pointer);

        assert_layout(&layouts, HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE);
    }

    #[test]
    fn disposable_stack_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_DISPOSABLE_STACK_RECORD_LAYOUT
            .iter()
            .map(DisposableStackRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let state = &layouts[0];
        assert_eq!(state.record, "disposable-stack-record");
        assert_eq!(state.name, "state");
        assert_eq!(state.offset, HEAP_DISPOSABLE_STACK_STATE_OFFSET);
        assert_eq!(state.width, 8);
        assert!(!state.pointer);

        let entries_pointer = &layouts[1];
        assert_eq!(entries_pointer.record, "disposable-stack-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(
            entries_pointer.offset,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET
        );
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[2];
        assert_eq!(entries_length.record, "disposable-stack-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(
            entries_length.offset,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET
        );
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[3];
        assert_eq!(entries_capacity.record, "disposable-stack-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(
            entries_capacity.offset,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET
        );
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        assert_layout(&layouts, HEAP_DISPOSABLE_STACK_RECORD_SIZE);
    }

    #[test]
    fn disposable_stack_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_DISPOSABLE_STACK_ENTRY_LAYOUT
            .iter()
            .map(DisposableStackEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        let kind = &layouts[0];
        assert_eq!(kind.record, "disposable-stack-entry");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let value_tag = &layouts[1];
        assert_eq!(value_tag.record, "disposable-stack-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(
            value_tag.offset,
            HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET
        );
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[2];
        assert_eq!(value_payload.record, "disposable-stack-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(
            value_payload.offset,
            HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET
        );
        assert_eq!(value_payload.width, 8);
        assert!(value_payload.pointer);

        let method_tag = &layouts[3];
        assert_eq!(method_tag.record, "disposable-stack-entry");
        assert_eq!(method_tag.name, "method_tag");
        assert_eq!(
            method_tag.offset,
            HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET
        );
        assert_eq!(method_tag.width, 8);
        assert!(!method_tag.pointer);

        let method_payload = &layouts[4];
        assert_eq!(method_payload.record, "disposable-stack-entry");
        assert_eq!(method_payload.name, "method_payload");
        assert_eq!(
            method_payload.offset,
            HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET
        );
        assert_eq!(method_payload.width, 8);
        assert!(method_payload.pointer);

        assert_layout(&layouts, HEAP_DISPOSABLE_STACK_ENTRY_SIZE);
    }

    #[test]
    fn temporal_plain_date_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainDateHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let iso_year = &layouts[0];
        assert_eq!(iso_year.record, "temporal-plain-date-record");
        assert_eq!(iso_year.name, "iso_year");
        assert_eq!(iso_year.offset, HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET);
        assert_eq!(iso_year.width, 8);
        assert!(!iso_year.pointer);

        let iso_month = &layouts[1];
        assert_eq!(iso_month.record, "temporal-plain-date-record");
        assert_eq!(iso_month.name, "iso_month");
        assert_eq!(iso_month.offset, HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET);
        assert_eq!(iso_month.width, 8);
        assert!(!iso_month.pointer);

        let iso_day = &layouts[2];
        assert_eq!(iso_day.record, "temporal-plain-date-record");
        assert_eq!(iso_day.name, "iso_day");
        assert_eq!(iso_day.offset, HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET);
        assert_eq!(iso_day.width, 8);
        assert!(!iso_day.pointer);

        let calendar_payload = &layouts[3];
        assert_eq!(calendar_payload.record, "temporal-plain-date-record");
        assert_eq!(calendar_payload.name, "calendar_payload");
        assert_eq!(
            calendar_payload.offset,
            HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET
        );
        assert_eq!(calendar_payload.width, 8);
        assert!(calendar_payload.pointer);

        assert_layout(&layouts, HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE);
    }

    #[test]
    fn temporal_duration_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_DURATION_RECORD_LAYOUT
            .iter()
            .map(TemporalDurationHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 10);

        for (slot, name, offset) in [
            (&layouts[0], "years", HEAP_TEMPORAL_DURATION_YEARS_OFFSET),
            (&layouts[1], "months", HEAP_TEMPORAL_DURATION_MONTHS_OFFSET),
            (&layouts[2], "weeks", HEAP_TEMPORAL_DURATION_WEEKS_OFFSET),
            (&layouts[3], "days", HEAP_TEMPORAL_DURATION_DAYS_OFFSET),
            (&layouts[4], "hours", HEAP_TEMPORAL_DURATION_HOURS_OFFSET),
            (
                &layouts[5],
                "minutes",
                HEAP_TEMPORAL_DURATION_MINUTES_OFFSET,
            ),
            (
                &layouts[6],
                "seconds",
                HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,
            ),
            (
                &layouts[7],
                "milliseconds",
                HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,
            ),
            (
                &layouts[8],
                "microseconds",
                HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET,
            ),
            (
                &layouts[9],
                "nanoseconds",
                HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET,
            ),
        ] {
            assert_eq!(slot.record, "temporal-duration-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert!(!slot.pointer);
        }

        assert_layout(&layouts, HEAP_TEMPORAL_DURATION_RECORD_SIZE);
    }

    #[test]
    fn temporal_plain_time_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 6);

        for (slot, name, offset) in [
            (&layouts[0], "hour", HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET),
            (
                &layouts[1],
                "minute",
                HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
            ),
            (
                &layouts[2],
                "second",
                HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
            ),
            (
                &layouts[3],
                "millisecond",
                HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
            ),
            (
                &layouts[4],
                "microsecond",
                HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
            ),
            (
                &layouts[5],
                "nanosecond",
                HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
            ),
        ] {
            assert_eq!(slot.record, "temporal-plain-time-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert!(!slot.pointer);
        }

        assert_layout(&layouts, HEAP_TEMPORAL_PLAIN_TIME_RECORD_SIZE);
    }

    #[test]
    fn temporal_plain_date_time_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT
            .iter()
            .map(TemporalPlainDateTimeHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 10);

        for (slot, name, offset, pointer) in [
            (
                &layouts[0],
                "iso_year",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
                false,
            ),
            (
                &layouts[1],
                "iso_month",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
                false,
            ),
            (
                &layouts[2],
                "iso_day",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
                false,
            ),
            (
                &layouts[3],
                "hour",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,
                false,
            ),
            (
                &layouts[4],
                "minute",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
                false,
            ),
            (
                &layouts[5],
                "second",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
                false,
            ),
            (
                &layouts[6],
                "millisecond",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,
                false,
            ),
            (
                &layouts[7],
                "microsecond",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
                false,
            ),
            (
                &layouts[8],
                "nanosecond",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,
                false,
            ),
            (
                &layouts[9],
                "calendar_payload",
                HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                true,
            ),
        ] {
            assert_eq!(slot.record, "temporal-plain-date-time-record");
            assert_eq!(slot.name, name);
            assert_eq!(slot.offset, offset);
            assert_eq!(slot.width, 8);
            assert_eq!(slot.pointer, pointer);
        }

        assert_layout(&layouts, HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_SIZE);
    }

    #[test]
    fn private_environment_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_PRIVATE_ENV_LAYOUT
            .iter()
            .map(PrivateEnvironmentHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 2);

        let parent = &layouts[0];
        assert_eq!(parent.record, "private-environment");
        assert_eq!(parent.name, "parent");
        assert_eq!(parent.offset, HEAP_PRIVATE_ENV_PARENT_OFFSET);
        assert_eq!(parent.width, 8);
        assert!(parent.pointer);

        let class_scope = &layouts[1];
        assert_eq!(class_scope.record, "private-environment");
        assert_eq!(class_scope.name, "class_scope");
        assert_eq!(class_scope.offset, HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET);
        assert_eq!(class_scope.width, 8);
        assert!(!class_scope.pointer);

        assert_layout(&layouts, HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET);
    }

    #[test]
    fn set_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_SET_ENTRY_LAYOUT
            .iter()
            .map(SetEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 3);

        let present = &layouts[0];
        assert_eq!(present.record, "set-entry");
        assert_eq!(present.name, "present");
        assert_eq!(present.offset, HEAP_SET_ENTRY_PRESENT_OFFSET);
        assert_eq!(present.width, 8);
        assert!(!present.pointer);

        let value_tag = &layouts[1];
        assert_eq!(value_tag.record, "set-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(value_tag.offset, HEAP_SET_ENTRY_VALUE_TAG_OFFSET);
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[2];
        assert_eq!(value_payload.record, "set-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(value_payload.offset, HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET);
        assert_eq!(value_payload.width, 8);
        assert!(value_payload.pointer);

        assert_layout(&layouts, HEAP_SET_ENTRY_SIZE);
    }

    #[test]
    fn weak_set_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_WEAK_SET_ENTRY_LAYOUT
            .iter()
            .map(WeakSetEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 3);

        let present = &layouts[0];
        assert_eq!(present.record, "weak-set-entry");
        assert_eq!(present.name, "present");
        assert_eq!(present.offset, HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET);
        assert_eq!(present.width, 8);
        assert!(!present.pointer);

        let value_tag = &layouts[1];
        assert_eq!(value_tag.record, "weak-set-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(value_tag.offset, HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET);
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[2];
        assert_eq!(value_payload.record, "weak-set-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(
            value_payload.offset,
            HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET
        );
        assert_eq!(value_payload.width, 8);
        assert!(!value_payload.pointer);

        assert_layout(&layouts, HEAP_WEAK_SET_ENTRY_SIZE);
    }

    #[test]
    fn weak_map_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_WEAK_MAP_ENTRY_LAYOUT
            .iter()
            .map(WeakMapEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        let present = &layouts[0];
        assert_eq!(present.record, "weak-map-entry");
        assert_eq!(present.name, "present");
        assert_eq!(present.offset, HEAP_WEAK_MAP_ENTRY_PRESENT_OFFSET);
        assert_eq!(present.width, 8);
        assert!(!present.pointer);

        let key_tag = &layouts[1];
        assert_eq!(key_tag.record, "weak-map-entry");
        assert_eq!(key_tag.name, "key_tag");
        assert_eq!(key_tag.offset, HEAP_WEAK_MAP_ENTRY_KEY_TAG_OFFSET);
        assert_eq!(key_tag.width, 8);
        assert!(!key_tag.pointer);

        let key_payload = &layouts[2];
        assert_eq!(key_payload.record, "weak-map-entry");
        assert_eq!(key_payload.name, "key_payload");
        assert_eq!(key_payload.offset, HEAP_WEAK_MAP_ENTRY_KEY_PAYLOAD_OFFSET);
        assert_eq!(key_payload.width, 8);
        assert!(!key_payload.pointer);

        let value_tag = &layouts[3];
        assert_eq!(value_tag.record, "weak-map-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(value_tag.offset, HEAP_WEAK_MAP_ENTRY_VALUE_TAG_OFFSET);
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[4];
        assert_eq!(value_payload.record, "weak-map-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(
            value_payload.offset,
            HEAP_WEAK_MAP_ENTRY_VALUE_PAYLOAD_OFFSET
        );
        assert_eq!(value_payload.width, 8);
        assert!(!value_payload.pointer);

        assert_layout(&layouts, HEAP_WEAK_MAP_ENTRY_SIZE);
    }

    #[test]
    fn symbol_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_SYMBOL_LAYOUT
            .iter()
            .map(SymbolHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let description_tag = &layouts[0];
        assert_eq!(description_tag.record, "symbol-record");
        assert_eq!(description_tag.name, "description_tag");
        assert_eq!(description_tag.offset, HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET);
        assert_eq!(description_tag.width, 8);
        assert!(!description_tag.pointer);

        let description_payload = &layouts[1];
        assert_eq!(description_payload.record, "symbol-record");
        assert_eq!(description_payload.name, "description_payload");
        assert_eq!(
            description_payload.offset,
            HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET
        );
        assert_eq!(description_payload.width, 8);
        assert!(description_payload.pointer);

        let registry_key_payload = &layouts[2];
        assert_eq!(registry_key_payload.record, "symbol-record");
        assert_eq!(registry_key_payload.name, "registry_key_payload");
        assert_eq!(
            registry_key_payload.offset,
            HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET
        );
        assert_eq!(registry_key_payload.width, 8);
        assert!(registry_key_payload.pointer);

        let symbol_id = &layouts[3];
        assert_eq!(symbol_id.record, "symbol-record");
        assert_eq!(symbol_id.name, "symbol_id");
        assert_eq!(symbol_id.offset, HEAP_SYMBOL_ID_OFFSET);
        assert_eq!(symbol_id.width, 8);
        assert!(!symbol_id.pointer);

        assert_layout(&layouts, HEAP_SYMBOL_RECORD_SIZE);
    }

    #[test]
    fn bigint_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_BIGINT_LAYOUT
            .iter()
            .map(BigIntHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let sign = &layouts[0];
        assert_eq!(sign.record, "bigint-record");
        assert_eq!(sign.name, "sign");
        assert_eq!(sign.offset, HEAP_BIGINT_SIGN_OFFSET);
        assert_eq!(sign.width, 8);
        assert!(!sign.pointer);

        let limbs_pointer = &layouts[1];
        assert_eq!(limbs_pointer.record, "bigint-record");
        assert_eq!(limbs_pointer.name, "limbs_ptr");
        assert_eq!(limbs_pointer.offset, HEAP_BIGINT_LIMBS_PTR_OFFSET);
        assert_eq!(limbs_pointer.width, 8);
        assert!(limbs_pointer.pointer);

        let limbs_length = &layouts[2];
        assert_eq!(limbs_length.record, "bigint-record");
        assert_eq!(limbs_length.name, "limbs_len");
        assert_eq!(limbs_length.offset, HEAP_BIGINT_LIMBS_LEN_OFFSET);
        assert_eq!(limbs_length.width, 8);
        assert!(!limbs_length.pointer);

        let limbs_capacity = &layouts[3];
        assert_eq!(limbs_capacity.record, "bigint-record");
        assert_eq!(limbs_capacity.name, "limbs_cap");
        assert_eq!(limbs_capacity.offset, HEAP_BIGINT_LIMBS_CAP_OFFSET);
        assert_eq!(limbs_capacity.width, 8);
        assert!(!limbs_capacity.pointer);

        assert_layout(&layouts, HEAP_BIGINT_RECORD_SIZE);
    }

    #[test]
    fn string_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_STRING_LAYOUT
            .iter()
            .map(StringHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let code_units_pointer = &layouts[0];
        assert_eq!(code_units_pointer.record, "string-record");
        assert_eq!(code_units_pointer.name, "code_units_ptr");
        assert_eq!(code_units_pointer.offset, HEAP_STRING_CODE_UNITS_PTR_OFFSET);
        assert_eq!(code_units_pointer.width, 8);
        assert!(code_units_pointer.pointer);

        let byte_length = &layouts[1];
        assert_eq!(byte_length.record, "string-record");
        assert_eq!(byte_length.name, "byte_len");
        assert_eq!(byte_length.offset, HEAP_STRING_BYTE_LEN_OFFSET);
        assert_eq!(byte_length.width, 8);
        assert!(!byte_length.pointer);

        let code_unit_length = &layouts[2];
        assert_eq!(code_unit_length.record, "string-record");
        assert_eq!(code_unit_length.name, "code_unit_len");
        assert_eq!(code_unit_length.offset, HEAP_STRING_CODE_UNIT_LEN_OFFSET);
        assert_eq!(code_unit_length.width, 8);
        assert!(!code_unit_length.pointer);

        let intern_id = &layouts[3];
        assert_eq!(intern_id.record, "string-record");
        assert_eq!(intern_id.name, "intern_id");
        assert_eq!(intern_id.offset, HEAP_STRING_INTERN_ID_OFFSET);
        assert_eq!(intern_id.width, 8);
        assert!(!intern_id.pointer);

        assert_layout(&layouts, HEAP_STRING_RECORD_SIZE);
    }

    #[test]
    fn environment_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_ENVIRONMENT_LAYOUT
            .iter()
            .map(EnvironmentHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 3);

        let parent = &layouts[0];
        assert_eq!(parent.record, "environment");
        assert_eq!(parent.name, "parent");
        assert_eq!(parent.offset, ENV_PARENT_OFFSET);
        assert_eq!(parent.width, 8);
        assert!(parent.pointer);

        let binding_tag = &layouts[1];
        assert_eq!(binding_tag.record, "environment-slot");
        assert_eq!(binding_tag.name, "tag");
        assert_eq!(binding_tag.offset, ENV_SLOT_TAG_OFFSET);
        assert_eq!(binding_tag.width, 8);
        assert!(!binding_tag.pointer);

        let binding_payload = &layouts[2];
        assert_eq!(binding_payload.record, "environment-slot");
        assert_eq!(binding_payload.name, "payload");
        assert_eq!(binding_payload.offset, ENV_SLOT_PAYLOAD_OFFSET);
        assert_eq!(binding_payload.width, 8);
        assert!(binding_payload.pointer);

        assert_layout(&layouts, ENV_SLOT_BASE_OFFSET + ENV_SLOT_SIZE);
    }

    #[test]
    fn async_generator_object_heap_slot_identity_owns_layout_metadata() {
        let layouts = HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT
            .iter()
            .map(AsyncGeneratorObjectHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 1);

        let activation = &layouts[0];
        assert_eq!(activation.record, "async-generator-object");
        assert_eq!(activation.name, "activation");
        assert_eq!(activation.offset, HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET);
        assert_eq!(activation.width, 8);
        assert!(activation.pointer);

        assert_layout(&layouts, HEAP_HEADER_SIZE);
    }

    #[test]
    fn map_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_MAP_RECORD_LAYOUT
            .iter()
            .map(MapRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let entries_pointer = &layouts[0];
        assert_eq!(entries_pointer.record, "map-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(entries_pointer.offset, HEAP_MAP_ENTRIES_PTR_OFFSET);
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[1];
        assert_eq!(entries_length.record, "map-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(entries_length.offset, HEAP_MAP_ENTRIES_LEN_OFFSET);
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[2];
        assert_eq!(entries_capacity.record, "map-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(entries_capacity.offset, HEAP_MAP_ENTRIES_CAP_OFFSET);
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        let live_count = &layouts[3];
        assert_eq!(live_count.record, "map-record");
        assert_eq!(live_count.name, "live_count");
        assert_eq!(live_count.offset, HEAP_MAP_LIVE_COUNT_OFFSET);
        assert_eq!(live_count.width, 8);
        assert!(!live_count.pointer);

        assert_layout(&layouts, HEAP_MAP_RECORD_SIZE);
    }

    #[test]
    fn map_entry_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_MAP_ENTRY_LAYOUT
            .iter()
            .map(MapEntryHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 5);

        let present = &layouts[0];
        assert_eq!(present.record, "map-entry");
        assert_eq!(present.name, "present");
        assert_eq!(present.offset, HEAP_MAP_ENTRY_PRESENT_OFFSET);
        assert_eq!(present.width, 8);
        assert!(!present.pointer);

        let key_tag = &layouts[1];
        assert_eq!(key_tag.record, "map-entry");
        assert_eq!(key_tag.name, "key_tag");
        assert_eq!(key_tag.offset, HEAP_MAP_ENTRY_KEY_TAG_OFFSET);
        assert_eq!(key_tag.width, 8);
        assert!(!key_tag.pointer);

        let key_payload = &layouts[2];
        assert_eq!(key_payload.record, "map-entry");
        assert_eq!(key_payload.name, "key_payload");
        assert_eq!(key_payload.offset, HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET);
        assert_eq!(key_payload.width, 8);
        assert!(key_payload.pointer);

        let value_tag = &layouts[3];
        assert_eq!(value_tag.record, "map-entry");
        assert_eq!(value_tag.name, "value_tag");
        assert_eq!(value_tag.offset, HEAP_MAP_ENTRY_VALUE_TAG_OFFSET);
        assert_eq!(value_tag.width, 8);
        assert!(!value_tag.pointer);

        let value_payload = &layouts[4];
        assert_eq!(value_payload.record, "map-entry");
        assert_eq!(value_payload.name, "value_payload");
        assert_eq!(value_payload.offset, HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET);
        assert_eq!(value_payload.width, 8);
        assert!(value_payload.pointer);

        assert_layout(&layouts, HEAP_MAP_ENTRY_SIZE);
    }

    #[test]
    fn set_iterator_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_SET_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(SetIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let set_payload = &layouts[0];
        assert_eq!(set_payload.record, "set-iterator-record");
        assert_eq!(set_payload.name, "set_payload");
        assert_eq!(set_payload.offset, HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET);
        assert_eq!(set_payload.width, 8);
        assert!(set_payload.pointer);

        let next_index = &layouts[1];
        assert_eq!(next_index.record, "set-iterator-record");
        assert_eq!(next_index.name, "next_index");
        assert_eq!(next_index.offset, HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET);
        assert_eq!(next_index.width, 8);
        assert!(!next_index.pointer);

        let kind = &layouts[2];
        assert_eq!(kind.record, "set-iterator-record");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_SET_ITERATOR_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let cursor_state = &layouts[3];
        assert_eq!(cursor_state.record, "set-iterator-record");
        assert_eq!(cursor_state.name, "cursor_state");
        assert_eq!(cursor_state.offset, HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET);
        assert_eq!(cursor_state.width, 8);
        assert!(!cursor_state.pointer);

        assert_layout(&layouts, HEAP_SET_ITERATOR_RECORD_SIZE);
    }

    #[test]
    fn set_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_SET_RECORD_LAYOUT
            .iter()
            .map(SetRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let entries_pointer = &layouts[0];
        assert_eq!(entries_pointer.record, "set-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(entries_pointer.offset, HEAP_SET_ENTRIES_PTR_OFFSET);
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[1];
        assert_eq!(entries_length.record, "set-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(entries_length.offset, HEAP_SET_ENTRIES_LEN_OFFSET);
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[2];
        assert_eq!(entries_capacity.record, "set-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(entries_capacity.offset, HEAP_SET_ENTRIES_CAP_OFFSET);
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        let live_count = &layouts[3];
        assert_eq!(live_count.record, "set-record");
        assert_eq!(live_count.name, "live_count");
        assert_eq!(live_count.offset, HEAP_SET_LIVE_COUNT_OFFSET);
        assert_eq!(live_count.width, 8);
        assert!(!live_count.pointer);

        assert_layout(&layouts, HEAP_SET_RECORD_SIZE);
    }

    #[test]
    fn weak_set_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_WEAK_SET_RECORD_LAYOUT
            .iter()
            .map(WeakSetRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let entries_pointer = &layouts[0];
        assert_eq!(entries_pointer.record, "weak-set-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(entries_pointer.offset, HEAP_WEAK_SET_ENTRIES_PTR_OFFSET);
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[1];
        assert_eq!(entries_length.record, "weak-set-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(entries_length.offset, HEAP_WEAK_SET_ENTRIES_LEN_OFFSET);
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[2];
        assert_eq!(entries_capacity.record, "weak-set-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(entries_capacity.offset, HEAP_WEAK_SET_ENTRIES_CAP_OFFSET);
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        let live_count = &layouts[3];
        assert_eq!(live_count.record, "weak-set-record");
        assert_eq!(live_count.name, "live_count");
        assert_eq!(live_count.offset, HEAP_WEAK_SET_LIVE_COUNT_OFFSET);
        assert_eq!(live_count.width, 8);
        assert!(!live_count.pointer);

        assert_layout(&layouts, HEAP_WEAK_SET_RECORD_SIZE);
    }

    #[test]
    fn weak_map_record_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_WEAK_MAP_RECORD_LAYOUT
            .iter()
            .map(WeakMapRecordHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let entries_pointer = &layouts[0];
        assert_eq!(entries_pointer.record, "weak-map-record");
        assert_eq!(entries_pointer.name, "entries_ptr");
        assert_eq!(entries_pointer.offset, HEAP_WEAK_MAP_ENTRIES_PTR_OFFSET);
        assert_eq!(entries_pointer.width, 8);
        assert!(entries_pointer.pointer);

        let entries_length = &layouts[1];
        assert_eq!(entries_length.record, "weak-map-record");
        assert_eq!(entries_length.name, "entries_len");
        assert_eq!(entries_length.offset, HEAP_WEAK_MAP_ENTRIES_LEN_OFFSET);
        assert_eq!(entries_length.width, 8);
        assert!(!entries_length.pointer);

        let entries_capacity = &layouts[2];
        assert_eq!(entries_capacity.record, "weak-map-record");
        assert_eq!(entries_capacity.name, "entries_cap");
        assert_eq!(entries_capacity.offset, HEAP_WEAK_MAP_ENTRIES_CAP_OFFSET);
        assert_eq!(entries_capacity.width, 8);
        assert!(!entries_capacity.pointer);

        let live_count = &layouts[3];
        assert_eq!(live_count.record, "weak-map-record");
        assert_eq!(live_count.name, "live_count");
        assert_eq!(live_count.offset, HEAP_WEAK_MAP_LIVE_COUNT_OFFSET);
        assert_eq!(live_count.width, 8);
        assert!(!live_count.pointer);

        assert_layout(&layouts, HEAP_WEAK_MAP_RECORD_SIZE);
    }

    #[test]
    fn map_iterator_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_MAP_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(MapIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let map_payload = &layouts[0];
        assert_eq!(map_payload.record, "map-iterator-record");
        assert_eq!(map_payload.name, "map_payload");
        assert_eq!(map_payload.offset, HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET);
        assert_eq!(map_payload.width, 8);
        assert!(map_payload.pointer);

        let next_index = &layouts[1];
        assert_eq!(next_index.record, "map-iterator-record");
        assert_eq!(next_index.name, "next_index");
        assert_eq!(next_index.offset, HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET);
        assert_eq!(next_index.width, 8);
        assert!(!next_index.pointer);

        let kind = &layouts[2];
        assert_eq!(kind.record, "map-iterator-record");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_MAP_ITERATOR_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let cursor_state = &layouts[3];
        assert_eq!(cursor_state.record, "map-iterator-record");
        assert_eq!(cursor_state.name, "cursor_state");
        assert_eq!(cursor_state.offset, HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET);
        assert_eq!(cursor_state.width, 8);
        assert!(!cursor_state.pointer);

        assert_layout(&layouts, HEAP_MAP_ITERATOR_RECORD_SIZE);
    }

    #[test]
    fn typed_array_iterator_heap_slot_identities_own_layout_metadata() {
        let layouts = HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT
            .iter()
            .map(TypedArrayIteratorHeapSlot::layout)
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 4);

        let typed_array_payload = &layouts[0];
        assert_eq!(typed_array_payload.record, "typed-array-iterator-record");
        assert_eq!(typed_array_payload.name, "typed_array_payload");
        assert_eq!(
            typed_array_payload.offset,
            HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET
        );
        assert_eq!(typed_array_payload.width, 8);
        assert!(typed_array_payload.pointer);

        let next_index = &layouts[1];
        assert_eq!(next_index.record, "typed-array-iterator-record");
        assert_eq!(next_index.name, "next_index");
        assert_eq!(
            next_index.offset,
            HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET
        );
        assert_eq!(next_index.width, 8);
        assert!(!next_index.pointer);

        let kind = &layouts[2];
        assert_eq!(kind.record, "typed-array-iterator-record");
        assert_eq!(kind.name, "kind");
        assert_eq!(kind.offset, HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET);
        assert_eq!(kind.width, 8);
        assert!(!kind.pointer);

        let done = &layouts[3];
        assert_eq!(done.record, "typed-array-iterator-record");
        assert_eq!(done.name, "done");
        assert_eq!(done.offset, HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET);
        assert_eq!(done.width, 8);
        assert!(!done.pointer);

        assert_layout(&layouts, HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE);
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
