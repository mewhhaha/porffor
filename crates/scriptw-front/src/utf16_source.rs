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
        match MarkerTail::read(&mut chars) {
            MarkerTail::Escaped => units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2])),
            MarkerTail::Surrogate(unit) => units.push(unit),
            MarkerTail::Literal { consumed, len } => {
                units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
                for ch in &consumed[..len] {
                    units.extend_from_slice(ch.encode_utf16(&mut [0u16; 2]));
                }
            }
        }
    }
    units
}

// An escaped sentinel takes precedence over a four-hex surrogate marker.
enum MarkerTail {
    Escaped,
    Surrogate(u16),
    Literal { consumed: [char; 4], len: usize },
}
impl MarkerTail {
    fn read(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Self {
        if chars.peek().copied() == Some(JS_STRING_SURROGATE_SENTINEL) {
            chars.next();
            return Self::Escaped;
        }
        let mut consumed = ['\0'; 4];
        let mut len = 0;
        let mut unit = 0u16;
        while len < consumed.len() {
            let Some(ch) = chars.next() else { break };
            consumed[len] = ch;
            len += 1;
            let Some(digit) = ch.to_digit(16) else {
                return Self::Literal { consumed, len };
            };
            unit = (unit << 4) | digit as u16;
        }
        if len == 4 && (0xD800..=0xDFFF).contains(&unit) {
            Self::Surrogate(unit)
        } else {
            Self::Literal { consumed, len }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_unit_round_trips_alone_and_beside_marker_spellings() {
        for unit in 0..=u16::MAX {
            let units = [unit];
            assert_eq!(
                decode_js_string_utf16(&encode_js_string_utf16(&units)),
                units
            );
            let mut adjacent: Vec<u16> = format!("{JS_STRING_SURROGATE_SENTINEL}D800")
                .encode_utf16()
                .collect();
            adjacent.extend([unit, 0xD800, 0xDC00, 0xDFFF]);
            assert_eq!(
                decode_js_string_utf16(&encode_js_string_utf16(&adjacent)),
                adjacent
            );
        }
    }

    #[test]
    fn malformed_markers_retain_consumed_characters_and_precedence() {
        for tail in [
            "", "D", "D80", "D80X", "D8X0", "0041", "abcd", "😀D80", "D80\0",
        ] {
            let value = format!("{JS_STRING_SURROGATE_SENTINEL}{tail}");
            assert_eq!(
                decode_js_string_utf16(&value),
                value.encode_utf16().collect::<Vec<_>>()
            );
        }
        let value = format!("{JS_STRING_SURROGATE_SENTINEL}{JS_STRING_SURROGATE_SENTINEL}D800");
        assert_eq!(
            decode_js_string_utf16(&value),
            format!("{JS_STRING_SURROGATE_SENTINEL}D800")
                .encode_utf16()
                .collect::<Vec<_>>()
        );
    }
}
