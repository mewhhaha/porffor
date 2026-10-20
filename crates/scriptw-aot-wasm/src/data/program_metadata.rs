//! Checked P-relative byte spans and tagged literal operands in private images.

use super::*;

/// A collected span in one P metadata image, never a linear-memory address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProgramMetadataSpan {
    start: u32,
    length: u32,
}

impl ProgramMetadataSpan {
    pub(crate) fn for_pool(
        strings: &StringPool,
        address: u32,
        length: u32,
    ) -> Result<Self, EmitError> {
        let prefix = u32::try_from(strings.compiler_owned_boundary().static_len())
            .ok()
            .and_then(|length| STATIC_DATA_OFFSET.checked_add(length))
            .ok_or_else(|| {
                EmitError::unsupported("runtime static prefix exceeds its address domain")
            })?;
        let start = address.checked_sub(prefix).ok_or_else(|| {
            EmitError::unsupported("P metadata span cannot select the shared runtime prefix")
        })?;
        let total = strings
            .bytes
            .len()
            .checked_sub(strings.compiler_owned_boundary().static_len())
            .and_then(|length| u32::try_from(length).ok())
            .ok_or_else(|| {
                EmitError::unsupported("P metadata image exceeds the GC array domain")
            })?;
        if start > total || length > total - start {
            return Err(EmitError::unsupported(
                "P metadata span exceeds its actual image",
            ));
        }
        Ok(Self { start, length })
    }

    pub(crate) const fn start(self) -> u32 {
        self.start
    }
    pub(crate) const fn length(self) -> u32 {
        self.length
    }

    /// The candidate format explicitly declares the Program domain separately.
    pub(crate) const fn packed(self) -> u64 {
        ((self.start as u64) << 32) | self.length as u64
    }
}

pub(crate) const POOLED_LITERAL_RUNTIME_DOMAIN: u32 = 0;
pub(crate) const POOLED_LITERAL_PROGRAM_DOMAIN: u32 = 1;

/// High word is a closed R/P domain; low word is relative to that table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PooledLiteralOperand(u64);

impl PooledLiteralOperand {
    pub(crate) fn for_pool(
        strings: &StringPool,
        index: PooledStringIndex,
    ) -> Result<Self, EmitError> {
        Self::for_collected(
            index,
            strings.compiler_owned_boundary().strings(),
            strings.pooled_string_count(),
        )
    }

    pub(super) fn for_collected(
        index: PooledStringIndex,
        runtime: u32,
        total: u32,
    ) -> Result<Self, EmitError> {
        let ordinal = index.ordinal();
        if runtime > total || ordinal >= total {
            return Err(EmitError::unsupported(
                "literal operand is outside its collected pool",
            ));
        }
        let (domain, ordinal) = if ordinal < runtime {
            (POOLED_LITERAL_RUNTIME_DOMAIN, ordinal)
        } else {
            (POOLED_LITERAL_PROGRAM_DOMAIN, ordinal - runtime)
        };
        Ok(Self((u64::from(domain) << 32) | u64::from(ordinal)))
    }

    pub(crate) const fn word(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_spans_reject_runtime_prefix_crossing_and_overflow() {
        let mut pool = StringPool::default();
        pool.bytes.extend_from_slice(&[1, 2, 3, 4]);
        pool.compiler_owned = pool.boundary();
        pool.bytes.extend_from_slice(&[5, 6, 7]);
        let base = STATIC_DATA_OFFSET + 4;
        let complete = ProgramMetadataSpan::for_pool(&pool, base, 3).unwrap();
        assert_eq!(
            (complete.start(), complete.length(), complete.packed()),
            (0, 3, 3)
        );
        let last = ProgramMetadataSpan::for_pool(&pool, base + 2, 1).unwrap();
        assert_eq!((last.start(), last.length()), (2, 1));
        assert!(ProgramMetadataSpan::for_pool(&pool, base - 1, 1).is_err());
        assert!(ProgramMetadataSpan::for_pool(&pool, base + 2, 2).is_err());
        assert!(ProgramMetadataSpan::for_pool(&pool, base, u32::MAX).is_err());
        assert!(ProgramMetadataSpan::for_pool(&pool, u32::MAX, u32::MAX).is_err());
        assert!(ProgramMetadataSpan::for_pool(&pool, base + 3, 0).is_ok());
        assert!(ProgramMetadataSpan::for_pool(&pool, base + 4, 0).is_err());
    }

    #[test]
    fn literal_operands_keep_runtime_dedup_and_colliding_program_slots_distinct() {
        let mut first = StringPool::default();
        first.intern_string("runtime");
        first.compiler_owned = first.boundary();
        first.intern_string("first P");
        let mut second = StringPool::default();
        second.intern_string("runtime");
        second.compiler_owned = second.boundary();
        second.intern_string("second P");
        let runtime =
            PooledLiteralOperand::for_pool(&first, first.pooled_string_index("runtime").unwrap())
                .unwrap();
        let first_p =
            PooledLiteralOperand::for_pool(&first, first.pooled_string_index("first P").unwrap())
                .unwrap();
        let second_p = PooledLiteralOperand::for_pool(
            &second,
            second.pooled_string_index("second P").unwrap(),
        )
        .unwrap();
        assert_eq!(runtime.word(), 0);
        assert_eq!(first_p.word(), 1u64 << 32);
        assert_eq!(
            first_p, second_p,
            "identical relative slots require the actual P owner at runtime"
        );
        assert_ne!(runtime, first_p);
        assert!(PooledLiteralOperand::for_collected(PooledStringIndex(2), 1, 2).is_err());
        assert!(PooledLiteralOperand::for_collected(PooledStringIndex(0), 2, 1).is_err());
    }
}
