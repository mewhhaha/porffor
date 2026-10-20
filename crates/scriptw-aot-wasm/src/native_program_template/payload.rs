//! Checked per-program packed data; no instance or process selector owns it.
use super::*;
use std::ops::Range;

const MAGIC: &[u8; 8] = b"SCRIPTWNP01";
const VERSION: u32 = 1;

#[derive(Debug)]
pub struct NativeProgramPayload {
    bytes: Arc<[u8]>,
    reusable_instance: bool,
}

impl NativeProgramPayload {
    pub(super) fn from_plan(plan: &plan::Plan) -> Result<Option<Self>, NativeProgramTemplateError> {
        if plan.contexts.is_none() {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        let mut spans = Vec::with_capacity(plan.images.len());
        for image in &plan.images {
            let start = u32::try_from(bytes.len()).map_err(invalid)?;
            let length = u32::try_from(image.bytes.len()).map_err(invalid)?;
            bytes
                .len()
                .checked_add(image.bytes.len())
                .and_then(|length| u32::try_from(length).ok())
                .ok_or_else(|| invalid("native payload image extent overflows"))?;
            spans.push((start, length));
            bytes.extend_from_slice(&image.bytes);
        }
        let images = u32::try_from(bytes.len()).map_err(invalid)?;
        for (start, length) in spans {
            bytes.extend_from_slice(&start.to_le_bytes());
            bytes.extend_from_slice(&length.to_le_bytes());
        }
        let literals = u32::try_from(bytes.len()).map_err(invalid)?;
        for literal in &plan.literal_operands {
            bytes.extend_from_slice(&literal.value().to_le_bytes());
        }
        let literal_count = u32::try_from(plan.literal_operands.len()).map_err(invalid)?;
        let image_count = u32::try_from(plan.images.len()).map_err(invalid)?;
        bytes.extend_from_slice(MAGIC);
        for word in [VERSION, images, image_count, literals, literal_count] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let length = u32::try_from(bytes.len()).map_err(invalid)?;
        let footer = bytes[bytes.len() - NativeProgramPayloadDirectory::FOOTER_BYTES..]
            .try_into()
            .unwrap();
        NativeProgramPayloadDirectory::from_footer(length, footer)?;
        Ok(Some(Self {
            bytes: bytes.into(),
            reusable_instance: plan.reusable,
        }))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn permits_instance_reuse(&self) -> bool {
        self.reusable_instance
    }
}

/// Decoded only from the actual immutable GC payload received by an allocator.
/// Directory bytes never grant access to another program or a mutable image.
pub struct NativeProgramPayloadDirectory {
    image_data_end: u32,
    images: u32,
    image_count: u32,
}

impl NativeProgramPayloadDirectory {
    pub const FOOTER_BYTES: usize = 28;

    pub fn from_footer(
        total: u32,
        footer: &[u8; Self::FOOTER_BYTES],
    ) -> Result<Self, NativeProgramTemplateError> {
        let word = |offset| u32::from_le_bytes(footer[offset..offset + 4].try_into().unwrap());
        if &footer[..8] != MAGIC || word(8) != VERSION {
            return Err(invalid("unknown native program payload header"));
        }
        let (images, image_count, literals, literal_count) =
            (word(12), word(16), word(20), word(24));
        let end = total
            .checked_sub(Self::FOOTER_BYTES as u32)
            .ok_or_else(|| invalid("truncated native program payload"))?;
        if images.checked_add(
            image_count
                .checked_mul(8)
                .ok_or_else(|| invalid("image count overflow"))?,
        ) != Some(literals)
            || literals.checked_add(
                literal_count
                    .checked_mul(4)
                    .ok_or_else(|| invalid("literal count overflow"))?,
            ) != Some(end)
            || images > end
        {
            return Err(invalid(
                "native payload directories exceed their actual image",
            ));
        }
        Ok(Self {
            image_data_end: images,
            images,
            image_count,
        })
    }

    pub fn image_row(&self, ordinal: u32) -> Result<u32, NativeProgramTemplateError> {
        if ordinal >= self.image_count {
            return Err(invalid(
                "native source image ordinal is outside its actual payload",
            ));
        }
        // from_footer proved the entire row table fits the actual array.
        Ok(self.images + ordinal * 8)
    }

    pub fn image_span(&self, row: &[u8; 8]) -> Result<Range<u32>, NativeProgramTemplateError> {
        let start = u32::from_le_bytes(row[..4].try_into().unwrap());
        let length = u32::from_le_bytes(row[4..].try_into().unwrap());
        let end = start
            .checked_add(length)
            .filter(|end| *end <= self.image_data_end)
            .ok_or_else(|| invalid("native source image extends into the payload directories"))?;
        Ok(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn footer() -> [u8; NativeProgramPayloadDirectory::FOOTER_BYTES] {
        let mut bytes = Vec::from(MAGIC.as_slice());
        for word in [VERSION, 4, 1, 12, 1] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.try_into().unwrap()
    }

    #[test]
    fn source_image_bounds_exclude_directories_and_reject_overflow() {
        let directory = NativeProgramPayloadDirectory::from_footer(44, &footer()).unwrap();
        assert_eq!(directory.image_row(0).unwrap(), 4);
        assert!(directory.image_row(1).is_err());
        let row = |start: u32, length: u32| {
            let mut bytes = [0; 8];
            bytes[..4].copy_from_slice(&start.to_le_bytes());
            bytes[4..].copy_from_slice(&length.to_le_bytes());
            bytes
        };
        assert_eq!(directory.image_span(&row(0, 4)).unwrap(), 0..4);
        assert_eq!(directory.image_span(&row(4, 0)).unwrap(), 4..4);
        assert!(directory.image_span(&row(3, 2)).is_err());
        assert!(directory.image_span(&row(5, 0)).is_err());
        assert!(directory.image_span(&row(u32::MAX, 1)).is_err());
    }

    #[test]
    fn payload_footer_requires_exact_lengths_version_and_checked_directories() {
        let original = footer();
        for total in [0, 27, 43, 45, u32::MAX] {
            assert!(NativeProgramPayloadDirectory::from_footer(total, &original).is_err());
        }
        for (offset, value) in [
            (8, 2),
            (12, u32::MAX),
            (16, u32::MAX),
            (20, 4),
            (24, u32::MAX),
        ] {
            let mut changed = original;
            changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(NativeProgramPayloadDirectory::from_footer(44, &changed).is_err());
        }
        let mut changed = original;
        changed[0] ^= 1;
        assert!(NativeProgramPayloadDirectory::from_footer(44, &changed).is_err());
    }
}
