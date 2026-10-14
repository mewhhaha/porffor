use super::*;

#[derive(Clone, Copy)]
pub(crate) enum ProgramStringDescriptorWord {
    Ordinal,
    Utf16ByteOffset,
    UnitLength,
}

impl ProgramStringDescriptorWord {
    pub(crate) const ROW_BYTES: u32 = Self::UnitLength.byte_offset() + 4;
    pub(crate) const fn byte_offset(self) -> u32 {
        match self {
            Self::Ordinal => 0,
            Self::Utf16ByteOffset => 4,
            Self::UnitLength => 8,
        }
    }
}

/// Complete checked descriptors for the program half of the string pool.
/// No runtime-owned ordinal or offset into the runtime's UTF-16 segment can
/// enter this image. Only its constructor serializes the three u32 words.
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
            assert!(ordinal.ordinal() >= boundary.strings);
            assert!(ordinal.ordinal() < self.pooled_string_count());
            let end = literal
                .length
                .checked_mul(2)
                .and_then(|length| literal.byte_offset.checked_add(length))
                .expect("descriptor UTF-16 slice fits the Wasm data domain");
            assert_eq!(literal.byte_offset % 2, 0);
            assert!(end <= utf16_bytes);
            for word in [ordinal.ordinal(), literal.byte_offset, literal.length] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        assert_eq!(bytes.len(), byte_length as usize);
        ProgramStringDescriptorImage { bytes, count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_rows_keep_absolute_slots_and_program_relative_utf16_spans() {
        let mut pool = StringPool::default();
        pool.intern_string("runtime\0\u{10000}");
        pool.intern_string("name");
        pool.compiler_owned = pool.boundary();
        let runtime = pool.pooled_code_unit_bytes.clone();
        let lone = format!("{JS_STRING_SURROGATE_SENTINEL}D800X{JS_STRING_SURROGATE_SENTINEL}DC00");
        let sources = [
            "".to_owned(),
            "\0program".to_owned(),
            "Ā\u{10000}".to_owned(),
            "zz-long-text".to_owned(),
            lone,
        ];
        for source in &sources {
            pool.intern_string(source);
        }
        pool.intern_string("name");
        let image = pool.program_string_descriptor_image();
        assert_eq!(image.count(), 5);
        assert_eq!(&pool.pooled_code_unit_bytes[..runtime.len()], runtime);
        let program = &pool.pooled_code_unit_bytes[runtime.len()..];
        let mut decoded = BTreeMap::new();
        for row in image
            .bytes
            .chunks_exact(ProgramStringDescriptorWord::ROW_BYTES as usize)
        {
            let read = |word: ProgramStringDescriptorWord| {
                let start = word.byte_offset() as usize;
                u32::from_le_bytes(row[start..start + 4].try_into().unwrap())
            };
            let ordinal = read(ProgramStringDescriptorWord::Ordinal);
            let start = read(ProgramStringDescriptorWord::Utf16ByteOffset) as usize;
            let end = start + read(ProgramStringDescriptorWord::UnitLength) as usize * 2;
            assert!(decoded.insert(ordinal, &program[start..end]).is_none());
        }
        for source in &sources {
            let expected = StringPool::runtime_code_units_for_string(source)
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>();
            assert_eq!(
                decoded[&pool.pooled_string_index(source).unwrap().ordinal()],
                expected
            );
        }
        assert!(!decoded.contains_key(&pool.pooled_string_index("name").unwrap().ordinal()));
    }
}
