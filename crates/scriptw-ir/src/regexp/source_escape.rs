//! EscapeRegExpPattern display rules shared by finite discovery and Wasm Get.

/// The empty OriginalSource is displayed as a noncapturing empty pattern.
pub const REGEXP_EMPTY_SOURCE: &str = "(?:)";
/// Escape a delimiter outside a character class and an odd backslash run.
pub const REGEXP_SOURCE_SLASH_ESCAPE: &str = "\\/";
/// Each LineTerminator is escaped without changing its matched code unit.
/// After an odd backslash run, reuse its final backslash by appending only the
/// suffix of this spelling; adding a second backslash changes the matcher.
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
    let mut in_class = false;
    for &unit in pattern {
        if let Some(replacement) = REGEXP_SOURCE_LINE_ESCAPES
            .iter()
            .find_map(|&(code, text)| (unit == code).then_some(text))
        {
            // The preceding backslash already belongs to this escape. Keep it
            // once, including when the original source has a longer odd run.
            let replacement = if escaped {
                &replacement[1..]
            } else {
                replacement
            };
            result.extend(replacement.encode_utf16());
        } else if unit == 47 && !escaped && !in_class {
            result.extend(REGEXP_SOURCE_SLASH_ESCAPE.encode_utf16());
        } else {
            result.push(unit);
        }
        // Track the lexical RegularExpressionClass, not Pattern nesting. The
        // literal grammar permits a raw slash inside this class. In v mode a
        // valid Pattern already escapes every class-set slash.
        if !escaped {
            if unit == 91 {
                in_class = true;
            } else if unit == 93 {
                in_class = false;
            }
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

#[cfg(test)]
mod source_context_tests {
    use super::*;

    #[test]
    fn line_terminator_escapes_reuse_an_existing_odd_backslash() {
        for (unit, replacement) in REGEXP_SOURCE_LINE_ESCAPES {
            for prefix_length in 0..=4 {
                for class in [false, true] {
                    let mut pattern = vec![92; prefix_length];
                    pattern.push(unit);
                    let mut expected = vec![92; prefix_length - prefix_length % 2];
                    expected.extend(replacement.encode_utf16());
                    if class {
                        pattern.insert(0, 91);
                        pattern.push(93);
                        expected.insert(0, 91);
                        expected.push(93);
                    }
                    assert_eq!(escape_regexp_pattern_utf16(&pattern), expected);
                }
            }
        }
    }

    #[test]
    fn slashes_follow_lexical_class_and_backslash_context() {
        for (pattern, expected) in [
            ("[/]", "[/]"),
            (r"[\/]", r"[\/]"),
            (r"[\\/]", r"[\\/]"),
            (r"[\\\/]", r"[\\\/]"),
            (r"\[/\]", r"\[\/\]"),
            (r"[\]]/", r"[\]]\/"),
            (r"[[]/", r"[[]\/"),
            (r"[\\]/", r"[\\]\/"),
            (r"[/]\[/", r"[/]\[\/"),
            (r"[[a]--[b]]\/", r"[[a]--[b]]\/"),
            (r"[\q{a\/b}]", r"[\q{a\/b}]"),
        ] {
            assert_eq!(
                escape_regexp_pattern_utf16(&pattern.encode_utf16().collect::<Vec<_>>()),
                expected.encode_utf16().collect::<Vec<_>>(),
                "{pattern}"
            );
        }
    }
}
