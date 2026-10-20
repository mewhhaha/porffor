//! Exact Rust Unicode case mappings built once, retaining the emitted row ABI.

use super::{StringPool, STATIC_DATA_OFFSET};
use icu_properties::{props, CodePointSetData};
use std::sync::OnceLock;

// Host and target std must describe the same Unicode mapping authority when
// cross-compiling. The generated file contains only the three version bytes.
const BUILD_UNICODE_VERSION: &[u8; 3] =
    include_bytes!(concat!(env!("OUT_DIR"), "/unicode-case-version.bin"));
const _: () = assert!(
    BUILD_UNICODE_VERSION[0] == char::UNICODE_VERSION.0
        && BUILD_UNICODE_VERSION[1] == char::UNICODE_VERSION.1
        && BUILD_UNICODE_VERSION[2] == char::UNICODE_VERSION.2
);

struct CaseMappingImage(&'static [u8]);

impl CaseMappingImage {
    const ROW_BYTES: usize = 16;

    const fn new(bytes: &'static [u8]) -> Self {
        assert!(bytes.len() % Self::ROW_BYTES == 0);
        assert!(bytes.len() / Self::ROW_BYTES <= u32::MAX as usize);
        Self(bytes)
    }

    fn bytes(&self) -> &'static [u8] {
        self.0
    }

    fn count(&self) -> u32 {
        (self.0.len() / Self::ROW_BYTES) as u32
    }
}

const LOWERCASE: CaseMappingImage = CaseMappingImage::new(include_bytes!(concat!(
    env!("OUT_DIR"),
    "/unicode-lowercase.bin"
)));
const UPPERCASE: CaseMappingImage = CaseMappingImage::new(include_bytes!(concat!(
    env!("OUT_DIR"),
    "/unicode-uppercase.bin"
)));

struct CaseContextRanges {
    cased: Vec<std::ops::RangeInclusive<u32>>,
    ignorable: Vec<std::ops::RangeInclusive<u32>>,
}

impl StringPool {
    pub(super) fn append_lowercase_tables(&mut self) {
        static RANGES: OnceLock<CaseContextRanges> = OnceLock::new();
        let ranges = RANGES.get_or_init(|| CaseContextRanges {
            cased: CodePointSetData::new::<props::Cased>()
                .iter_ranges()
                .collect(),
            ignorable: CodePointSetData::new::<props::CaseIgnorable>()
                .iter_ranges()
                .collect(),
        });
        self.align_bytes(8);
        self.lowercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(LOWERCASE.bytes());
        self.lowercase_mapping_count = LOWERCASE.count();
        self.cased_range_table_ptr = self.append_codepoint_ranges(&ranges.cased);
        self.cased_range_count = ranges.cased.len() as u32;
        self.case_ignorable_range_table_ptr = self.append_codepoint_ranges(&ranges.ignorable);
        self.case_ignorable_range_count = ranges.ignorable.len() as u32;
    }

    pub(super) fn append_uppercase_tables(&mut self) {
        self.align_bytes(8);
        self.uppercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(UPPERCASE.bytes());
        self.uppercase_mapping_count = UPPERCASE.count();
    }
}

#[cfg(test)]
mod tests;
