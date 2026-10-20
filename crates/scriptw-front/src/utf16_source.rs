//! Lossless compiler transport of ECMAScript UTF-16 source and string data.

/// The spelling domain of source metadata. Host UTF-8 files retain their bytes;
/// source supplied by an ECMAScript string uses the explicit lossless codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTextEncoding {
    Utf8,
    EncodedUtf16,
}

pub const JS_STRING_SURROGATE_SENTINEL: char = '\u{F0000}';

/// Lossless internal StringPool spelling of an ECMAScript UTF-16 string.
pub fn encode_js_string_utf16(units: &[u16]) -> String {
    char::decode_utf16(units.iter().copied())
        .map(|decoded| match decoded {
            Ok(ch) if ch == JS_STRING_SURROGATE_SENTINEL => {
                format!("{JS_STRING_SURROGATE_SENTINEL}{JS_STRING_SURROGATE_SENTINEL}")
            }
            Ok(ch) => ch.to_string(),
            Err(error) => format!(
                "{JS_STRING_SURROGATE_SENTINEL}{:04X}",
                error.unpaired_surrogate()
            ),
        })
        .collect()
}
/// Decodes the lossless internal StringPool spelling into semantic UTF-16.
/// This codec does not interpret ECMAScript or RegExp escape syntax.
pub fn decode_js_string_utf16(value: &str) -> Vec<u16> {
    let mut units = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != JS_STRING_SURROGATE_SENTINEL {
            units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
            continue;
        }
        if chars.peek().copied() == Some(JS_STRING_SURROGATE_SENTINEL) {
            chars.next();
            units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
            continue;
        }
        let mut consumed = Vec::new();
        let mut unit = 0u16;
        let mut marker = true;
        for _ in 0..4 {
            let Some(hex) = chars.next() else {
                marker = false;
                break;
            };
            consumed.push(hex);
            let Some(value) = hex.to_digit(16) else {
                marker = false;
                break;
            };
            unit = (unit << 4) | value as u16;
        }
        if marker && (0xD800..=0xDFFF).contains(&unit) {
            units.push(unit);
        } else {
            units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
            for ch in consumed {
                units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
            }
        }
    }
    units
}
