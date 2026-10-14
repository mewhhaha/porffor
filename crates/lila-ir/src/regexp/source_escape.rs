//! EscapeRegExpPattern display rules shared by finite discovery and Wasm Get.

/// The empty OriginalSource is displayed as a noncapturing empty pattern.
pub const REGEXP_EMPTY_SOURCE: &str = "(?:)";
/// Escape a delimiter not already protected by an odd backslash run.
pub const REGEXP_SOURCE_SLASH_ESCAPE: &str = "\\/";
/// Each LineTerminator is escaped without changing its matched code unit.
pub const REGEXP_SOURCE_LINE_ESCAPES: [(u16, &str); 4] = [
    (10, "\\n"),
    (13, "\\r"),
    (0x2028, "\\u2028"),
    (0x2029, "\\u2029"),
];

/// Formats semantic UTF-16 OriginalSource; this does not validate a Pattern,
/// prove a RegExp receiver, perform a property Get, or authorize source eval.
pub fn escape_regexp_pattern_utf16(pattern: &[u16]) -> Vec<u16> {
    if pattern.is_empty() {
        return REGEXP_EMPTY_SOURCE.encode_utf16().collect();
    }
    let mut result = Vec::with_capacity(pattern.len());
    let mut escaped = false;
    for &unit in pattern {
        let replacement = REGEXP_SOURCE_LINE_ESCAPES
            .iter()
            .find_map(|&(code, text)| (unit == code).then_some(text))
            .or_else(|| (unit == 47 && !escaped).then_some(REGEXP_SOURCE_SLASH_ESCAPE));
        if let Some(replacement) = replacement {
            result.extend(replacement.encode_utf16());
        } else {
            result.push(unit);
        }
        escaped = unit == 92 && !escaped;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_escaping_preserves_backslash_parity_and_utf16_units() {
        for (pattern, expected) in [
            (vec![], vec![40, 63, 58, 41]),
            (vec![47], vec![92, 47]),
            (vec![92, 47], vec![92, 47]),
            (vec![92, 92, 47], vec![92, 92, 92, 47]),
            (vec![92, 92, 92, 47], vec![92, 92, 92, 47]),
            (vec![10, 13], vec![92, 110, 92, 114]),
            (vec![0x2028], "\\u2028".encode_utf16().collect()),
            (vec![0x2029], "\\u2029".encode_utf16().collect()),
            (vec![0xd800, 0x2f, 0xdc00], vec![0xd800, 92, 47, 0xdc00]),
            (vec![0xd83d, 0xde00], vec![0xd83d, 0xde00]),
        ] {
            assert_eq!(escape_regexp_pattern_utf16(&pattern), expected);
        }
    }
}
