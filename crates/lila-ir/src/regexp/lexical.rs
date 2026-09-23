/// The flags permitted in a scoped modifier group, shared by both parsers.
#[derive(Clone, Copy)]
pub enum RegExpScopedModifier {
    IgnoreCase,
    Multiline,
    DotAll,
}
impl RegExpScopedModifier {
    pub const ALL: [Self; 3] = [Self::IgnoreCase, Self::Multiline, Self::DotAll];

    pub const fn marker(self) -> u8 {
        match self {
            Self::IgnoreCase => b'i',
            Self::Multiline => b'm',
            Self::DotAll => b's',
        }
    }
    pub const fn bit(self) -> u8 {
        match self {
            Self::IgnoreCase => 1,
            Self::Multiline => 2,
            Self::DotAll => 4,
        }
    }
    pub const fn from_marker(marker: u8) -> Option<Self> {
        match marker {
            b'i' => Some(Self::IgnoreCase),
            b'm' => Some(Self::Multiline),
            b's' => Some(Self::DotAll),
            _ => None,
        }
    }
}

/// ECMAScript WhiteSpace and LineTerminator code points used by `\\s`.
pub const REGEXP_WHITESPACE_RANGES: &[(u32, u32)] = &[
    (0x0009, 0x000d),
    (0x0020, 0x0020),
    (0x00a0, 0x00a0),
    (0x1680, 0x1680),
    (0x2000, 0x200a),
    (0x2028, 0x2029),
    (0x202f, 0x202f),
    (0x205f, 0x205f),
    (0x3000, 0x3000),
    (0xfeff, 0xfeff),
];
pub const REGEXP_DIGIT_RANGES: &[(u32, u32)] = &[(0x30, 0x39)];
pub const REGEXP_WORD_RANGES: &[(u32, u32)] =
    &[(0x30, 0x39), (0x41, 0x5a), (0x5f, 0x5f), (0x61, 0x7a)];
/// Fixed CharacterEscape productions shared by the source and emitted parsers.
pub const REGEXP_CHARACTER_ESCAPES: &[(u8, u16)] = &[
    (b'n', 0x0a),
    (b'r', 0x0d),
    (b't', 0x09),
    (b'v', 0x0b),
    (b'f', 0x0c),
];
/// Inclusive digit ranges and the numeric value of their first character.
pub const REGEXP_HEX_DIGIT_RANGES: &[(u16, u16, u16)] = &[
    (b'0' as u16, b'9' as u16, 0),
    (b'A' as u16, b'F' as u16, 10),
    (b'a' as u16, b'f' as u16, 10),
];
pub const REGEXP_LEGACY_THREE_DIGIT_OCTAL_LAST: u8 = b'3';
pub fn regexp_hex_digit_value(unit: u32) -> Option<u32> {
    REGEXP_HEX_DIGIT_RANGES
        .iter()
        .find_map(|&(first, last, value)| {
            (u32::from(first)..=u32::from(last))
                .contains(&unit)
                .then(|| unit - u32::from(first) + u32::from(value))
        })
}
pub fn regexp_character_escape(marker: u8) -> Option<u16> {
    REGEXP_CHARACTER_ESCAPES
        .iter()
        .find_map(|&(escape, unit)| (escape == marker).then_some(unit))
}
