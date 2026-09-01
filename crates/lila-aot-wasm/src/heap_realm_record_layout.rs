#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_REALM_AGENT_ID_OFFSET, HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
    HEAP_REALM_GLOBAL_OBJECT_OFFSET, HEAP_REALM_GLOBAL_THIS_OFFSET, HEAP_REALM_HOST_HOOKS_OFFSET,
    HEAP_REALM_ID_OFFSET, HEAP_REALM_INTRINSICS_OFFSET, HEAP_REALM_MODULE_REGISTRY_OFFSET,
    HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
};

pub(crate) enum RealmRecordHeapSlot {
    RealmId,
    AgentId,
    GlobalObject,
    GlobalThis,
    GlobalEnvironment,
    Intrinsics,
    HostHooks,
    ModuleRegistry,
    PrivateElements,
}

struct RealmRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl RealmRecordHeapSlot {
    const fn metadata(&self) -> RealmRecordHeapSlotMetadata {
        match self {
            Self::RealmId => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "realm_id",
                offset: HEAP_REALM_ID_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::AgentId => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "agent_id",
                offset: HEAP_REALM_AGENT_ID_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::GlobalObject => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "global_object",
                offset: HEAP_REALM_GLOBAL_OBJECT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::GlobalThis => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "global_this",
                offset: HEAP_REALM_GLOBAL_THIS_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::GlobalEnvironment => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "global_environment",
                offset: HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Intrinsics => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "intrinsics",
                offset: HEAP_REALM_INTRINSICS_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HostHooks => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "host_hooks",
                offset: HEAP_REALM_HOST_HOOKS_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ModuleRegistry => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "module_registry",
                offset: HEAP_REALM_MODULE_REGISTRY_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::PrivateElements => RealmRecordHeapSlotMetadata {
                record: "realm-record",
                name: "private_elements",
                offset: HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
                width: 8,
                pointer: true,
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

pub(crate) const HEAP_REALM_RECORD_LAYOUT: &[RealmRecordHeapSlot] = &[
    RealmRecordHeapSlot::RealmId,
    RealmRecordHeapSlot::AgentId,
    RealmRecordHeapSlot::GlobalObject,
    RealmRecordHeapSlot::GlobalThis,
    RealmRecordHeapSlot::GlobalEnvironment,
    RealmRecordHeapSlot::Intrinsics,
    RealmRecordHeapSlot::HostHooks,
    RealmRecordHeapSlot::ModuleRegistry,
    RealmRecordHeapSlot::PrivateElements,
];
