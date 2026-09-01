use super::RegExpInstruction;

#[derive(Clone, Copy)]
pub(super) struct LegacyUtf16Pair {
    lead: u32,
    trail: u32,
}

impl LegacyUtf16Pair {
    pub(super) fn from_scalar(scalar: char) -> Option<Self> {
        let supplementary = u32::from(scalar).checked_sub(0x1_0000)?;
        Some(Self {
            lead: 0xD800 + (supplementary >> 10),
            trail: 0xDC00 + (supplementary & 0x3ff),
        })
    }

    pub(super) fn lead_instruction(self) -> RegExpInstruction {
        RegExpInstruction::literal_code_point(self.lead)
    }

    pub(super) fn trail_instruction(self) -> RegExpInstruction {
        RegExpInstruction::literal_code_point(self.trail)
    }
}
