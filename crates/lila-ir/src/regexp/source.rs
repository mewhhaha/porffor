use super::{RegExpCompileError, NON_BOUNDARY_SOURCE};

/// Pattern bytes are UTF-8 scalars plus WTF-8 lone surrogate units. This owner
/// can only be built from semantic UTF-16; marker strings and diagnostic escape
/// spellings cannot enter the parser through its constructor.
pub(super) struct Utf16PatternSource {
    bytes: Vec<u8>,
}

impl Utf16PatternSource {
    pub(super) fn new(units: &[u16]) -> Self {
        let mut bytes = Vec::with_capacity(units.len());
        for decoded in char::decode_utf16(units.iter().copied()) {
            match decoded {
                Ok(character) => {
                    let mut buffer = [0; 4];
                    bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                }
                Err(error) => {
                    let unit = error.unpaired_surrogate();
                    bytes.extend_from_slice(&[
                        0xe0 | (unit >> 12) as u8,
                        0x80 | ((unit >> 6) & 0x3f) as u8,
                        0x80 | (unit & 0x3f) as u8,
                    ]);
                }
            }
        }
        Self { bytes }
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Decodes exactly one source character. Checking the remainder as UTF-8 would
/// wrongly reject a valid scalar whenever a later atom is a raw lone surrogate.
pub(super) fn pattern_code_point_at(
    bytes: &[u8],
    offset: usize,
) -> Result<(u32, usize), RegExpCompileError> {
    let failure = || RegExpCompileError::unsupported_feature(offset, NON_BOUNDARY_SOURCE);
    let first = *bytes.get(offset).ok_or_else(failure)?;
    let width = match first {
        0..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Err(failure()),
    };
    let encoded = bytes.get(offset..offset + width).ok_or_else(failure)?;
    if width == 3 && first == 0xed && (0xa0..=0xbf).contains(&encoded[1]) {
        if !(0x80..=0xbf).contains(&encoded[2]) {
            return Err(failure());
        }
        let unit = (u32::from(first & 0x0f) << 12)
            | (u32::from(encoded[1] & 0x3f) << 6)
            | u32::from(encoded[2] & 0x3f);
        return Ok((unit, offset + width));
    }
    let source = std::str::from_utf8(encoded).map_err(|_| failure())?;
    let character = source.chars().next().ok_or_else(failure)?;
    Ok((u32::from(character), offset + width))
}
