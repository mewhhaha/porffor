use crate::LocaleTextDirection;

pub const LOCALE_TEXT_WIRE_BYTES: usize = 8;

/// Encodes only closed native direction results, including an unknown direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleTextWire(u64);

impl LocaleTextWire {
    #[must_use]
    pub const fn from_info(direction: Option<LocaleTextDirection>) -> Self {
        Self(match direction {
            None => 0,
            Some(LocaleTextDirection::LeftToRight) => 1,
            Some(LocaleTextDirection::RightToLeft) => 2,
        })
    }

    #[must_use]
    pub const fn encode(self) -> [u8; LOCALE_TEXT_WIRE_BYTES] {
        self.0.to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_directions_and_unknown_have_distinct_exact_little_endian_words() {
        for (direction, expected) in [
            (None, [0, 0, 0, 0, 0, 0, 0, 0]),
            (
                Some(LocaleTextDirection::LeftToRight),
                [1, 0, 0, 0, 0, 0, 0, 0],
            ),
            (
                Some(LocaleTextDirection::RightToLeft),
                [2, 0, 0, 0, 0, 0, 0, 0],
            ),
        ] {
            assert_eq!(LocaleTextWire::from_info(direction).encode(), expected);
        }
    }
}
