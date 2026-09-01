#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET,
    HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET, HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
    HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET, HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
    HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
};

pub(crate) enum AtomicsAsyncWaiterHeapSlot {
    State,
    Address,
    PromiseRecord,
    DeadlineNanos,
    Next,
    HostId,
}

struct AtomicsAsyncWaiterHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl AtomicsAsyncWaiterHeapSlot {
    const fn metadata(&self) -> AtomicsAsyncWaiterHeapSlotMetadata {
        match self {
            Self::State => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "state",
                offset: HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Address => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "address",
                offset: HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::PromiseRecord => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "promise_record",
                offset: HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::DeadlineNanos => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "deadline_nanos",
                offset: HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Next => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "next",
                offset: HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HostId => AtomicsAsyncWaiterHeapSlotMetadata {
                record: "atomics-async-waiter",
                name: "host_id",
                offset: HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
                width: 8,
                pointer: false,
            },
        }
    }

    pub(crate) const fn layout(&self) -> HeapLayoutSlot {
        let metadata = self.metadata();
        HeapLayoutSlot {
            record: metadata.record,
            name: metadata.name,
            offset: metadata.offset,
            width: metadata.width,
            pointer: metadata.pointer,
        }
    }
}

pub(crate) const HEAP_ATOMICS_ASYNC_WAITER_LAYOUT: &[AtomicsAsyncWaiterHeapSlot] = &[
    AtomicsAsyncWaiterHeapSlot::State,
    AtomicsAsyncWaiterHeapSlot::Address,
    AtomicsAsyncWaiterHeapSlot::PromiseRecord,
    AtomicsAsyncWaiterHeapSlot::DeadlineNanos,
    AtomicsAsyncWaiterHeapSlot::Next,
    AtomicsAsyncWaiterHeapSlot::HostId,
];
