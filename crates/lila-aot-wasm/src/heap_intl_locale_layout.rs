#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_INTL_LOCALE_BASE_NAME_OFFSET, HEAP_INTL_LOCALE_LANGUAGE_OFFSET,
    HEAP_INTL_LOCALE_REGION_OFFSET, HEAP_INTL_LOCALE_SCRIPT_OFFSET, HEAP_INTL_LOCALE_TAG_OFFSET,
};

pub(crate) enum IntlLocaleHeapSlot {
    TagPayload,
    LanguagePayload,
    ScriptPayload,
    RegionPayload,
    BaseNamePayload,
}

struct IntlLocaleHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl IntlLocaleHeapSlot {
    const fn metadata(&self) -> IntlLocaleHeapSlotMetadata {
        match self {
            Self::TagPayload => IntlLocaleHeapSlotMetadata {
                record: "intl-locale-record",
                name: "tag_payload",
                offset: HEAP_INTL_LOCALE_TAG_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::LanguagePayload => IntlLocaleHeapSlotMetadata {
                record: "intl-locale-record",
                name: "language_payload",
                offset: HEAP_INTL_LOCALE_LANGUAGE_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ScriptPayload => IntlLocaleHeapSlotMetadata {
                record: "intl-locale-record",
                name: "script_payload",
                offset: HEAP_INTL_LOCALE_SCRIPT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::RegionPayload => IntlLocaleHeapSlotMetadata {
                record: "intl-locale-record",
                name: "region_payload",
                offset: HEAP_INTL_LOCALE_REGION_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::BaseNamePayload => IntlLocaleHeapSlotMetadata {
                record: "intl-locale-record",
                name: "base_name_payload",
                offset: HEAP_INTL_LOCALE_BASE_NAME_OFFSET,
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

pub(crate) const HEAP_INTL_LOCALE_RECORD_LAYOUT: &[IntlLocaleHeapSlot] = &[
    IntlLocaleHeapSlot::TagPayload,
    IntlLocaleHeapSlot::LanguagePayload,
    IntlLocaleHeapSlot::ScriptPayload,
    IntlLocaleHeapSlot::RegionPayload,
    IntlLocaleHeapSlot::BaseNamePayload,
];
