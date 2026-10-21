/// Normalize JSONC in one lexical pass. Preserve byte offsets and line breaks
/// so serde's diagnostics still refer to the user's source. A comment separates
/// tokens: `1/*...*/2` must never become the valid number `12`.
pub(crate) fn normalize(source: &str) -> Result<String, String> {
    let input = source.as_bytes();
    let mut output = input.to_vec();
    let mut cursor = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut pending_comma = None;
    let mut previous = None;
    while let Some(&byte) = input.get(cursor) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            cursor += 1;
            continue;
        }
        if byte == b'/' && input.get(cursor + 1) == Some(&b'/') {
            while cursor < input.len() && !matches!(input[cursor], b'\r' | b'\n') {
                output[cursor] = b' ';
                cursor += 1;
            }
            continue;
        }
        if byte == b'/' && input.get(cursor + 1) == Some(&b'*') {
            let start = cursor;
            output[cursor..cursor + 2].fill(b' ');
            cursor += 2;
            loop {
                if cursor == input.len() {
                    return Err(format!("unterminated JSONC comment at byte {start}"));
                }
                if input[cursor] == b'*' && input.get(cursor + 1) == Some(&b'/') {
                    output[cursor..cursor + 2].fill(b' ');
                    cursor += 2;
                    break;
                }
                if !matches!(input[cursor], b'\r' | b'\n') {
                    output[cursor] = b' ';
                }
                cursor += 1;
            }
            continue;
        }
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            cursor += 1;
            continue;
        }
        if matches!(byte, b']' | b'}') {
            if let Some(comma) = pending_comma.take() {
                output[comma] = b' ';
            }
        }
        pending_comma = (byte == b','
            && !matches!(previous, None | Some(b'[' | b'{' | b':' | b',')))
        .then_some(cursor);
        previous = Some(byte);
        in_string = byte == b'"';
        cursor += 1;
    }
    // Only comment bytes (including all bytes of each non-ASCII character) and
    // ASCII commas were replaced. All remaining UTF-8 comes from the input.
    Ok(String::from_utf8(output).expect("JSONC normalization preserves UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn comments_and_commas_preserve_strings_and_source_coordinates() {
        let input = "{\r\n\"text\":\"//é\\\"/*,]*/\", /* 😀\n */ \"list\":[1,//尾\n],}\n";
        let output = normalize(input).unwrap();
        assert_eq!(output.len(), input.len());
        assert_eq!(
            output
                .match_indices('\n')
                .map(|(i, _)| i)
                .collect::<Vec<_>>(),
            input
                .match_indices('\n')
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        );
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["text"], "//é\"/*,]*/");
        assert_eq!(value["list"], serde_json::json!([1]));
    }

    #[test]
    fn malformed_json_remains_rejected() {
        for input in [
            "[1/*gap*/2]",
            "[1,,]",
            "[,]",
            "{,}",
            "{'x':1}",
            "[true false]",
            "[1,\u{a0}]",
        ] {
            assert!(
                serde_json::from_str::<serde_json::Value>(&normalize(input).unwrap()).is_err(),
                "{input}"
            );
        }
        for input in ["{}/*", "{}/*x", "{}/*x*"] {
            assert!(normalize(input).unwrap_err().contains("unterminated"));
        }
        assert!(
            serde_json::from_str::<serde_json::Value>(&normalize("\"unterminated").unwrap())
                .is_err()
        );
    }
}
