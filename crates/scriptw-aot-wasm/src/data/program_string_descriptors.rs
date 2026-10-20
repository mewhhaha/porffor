use super::*;
use crate::gc_types::ProgramLiteralIndex;

#[derive(Clone, Copy)]
pub(crate) enum ProgramStringDescriptorWord {
    LiteralIndex,
    Utf16ByteOffset,
    UnitLength,
}

impl ProgramStringDescriptorWord {
    pub(crate) const ROW_BYTES: u32 = Self::UnitLength.byte_offset() + 4;
    pub(crate) const fn byte_offset(self) -> u32 {
        match self {
            Self::LiteralIndex => 0,
            Self::Utf16ByteOffset => 4,
            Self::UnitLength => 8,
        }
    }
}

/// Complete checked descriptors for the program half of the string pool.
/// All target indexes and UTF-16 spans are P-relative. A runtime ordinal or
/// runtime UTF-16 offset cannot enter this image. Only this owner serializes
/// the three validated u32 words.
pub(crate) struct ProgramStringDescriptorImage {
    bytes: Vec<u8>,
    count: u32,
}

impl ProgramStringDescriptorImage {
    pub(crate) const DATA_SEGMENT: u32 = 2;

    pub(crate) fn count(&self) -> u32 {
        self.count
    }

    pub(crate) fn emit_bounds(&self, function: &mut Function) {
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Const(self.bytes.len() as i32));
    }

    pub(super) fn append_passive(&self, section: &mut wasm_encoder::DataSection) {
        assert_eq!(section.len(), Self::DATA_SEGMENT);
        section.passive(self.bytes.iter().copied());
    }
}

impl StringPool {
    pub(crate) fn program_string_descriptor_image(&self) -> ProgramStringDescriptorImage {
        let boundary = self.compiler_owned;
        let count = self
            .pooled_string_count()
            .checked_sub(boundary.strings)
            .expect("program pool retains the complete runtime prefix");
        let byte_length = count
            .checked_mul(ProgramStringDescriptorWord::ROW_BYTES)
            .expect("program descriptor bytes fit the Wasm data domain");
        let utf16_bytes =
            u32::try_from(self.pooled_code_unit_bytes.len() - boundary.code_unit_bytes)
                .expect("program UTF-16 image fits the Wasm data domain");
        let mut bytes = Vec::with_capacity(byte_length as usize);
        for (ordinal, literal) in self.program_pooled_string_initializers() {
            let index = ProgramLiteralIndex::for_pool(self, ordinal)
                .expect("program initializer retains a checked P-local slot");
            let end = literal
                .length
                .checked_mul(2)
                .and_then(|length| literal.byte_offset.checked_add(length))
                .expect("descriptor UTF-16 slice fits the Wasm data domain");
            assert_eq!(literal.byte_offset % 2, 0);
            assert!(end <= utf16_bytes);
            for word in [index.ordinal(), literal.byte_offset, literal.length] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        assert_eq!(bytes.len(), byte_length as usize);
        ProgramStringDescriptorImage { bytes, count }
    }
}

#[cfg(test)]
mod tests;
