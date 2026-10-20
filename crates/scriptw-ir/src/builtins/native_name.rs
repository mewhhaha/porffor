/// Validates the closed native-name surface used by the builtin catalog:
/// anonymous, ASCII IdentifierName, or a computed well-known Symbol name,
/// optionally preceded by an accessor prefix. This is not a JavaScript parser.
pub(super) const fn is_catalog_native_function_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    let mut start = 0;
    let mut end = bytes.len();
    if end >= 4
        && ((bytes[0] == b'g' && bytes[1] == b'e' && bytes[2] == b't')
            || (bytes[0] == b's' && bytes[1] == b'e' && bytes[2] == b't'))
        && bytes[3] == b' '
    {
        start = 4;
    }
    if start < end && bytes[start] == b'[' {
        let symbol_prefix = b"[Symbol.";
        if end - start <= symbol_prefix.len() || bytes[end - 1] != b']' {
            return false;
        }
        let mut index = 0;
        while index < symbol_prefix.len() {
            if bytes[start + index] != symbol_prefix[index] {
                return false;
            }
            index += 1;
        }
        start += symbol_prefix.len();
        end -= 1;
    }
    if start == end || !is_ascii_identifier_start(bytes[start]) {
        return false;
    }
    start += 1;
    while start < end {
        if !is_ascii_identifier_start(bytes[start]) && !bytes[start].is_ascii_digit() {
            return false;
        }
        start += 1;
    }
    true
}

const fn is_ascii_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::super::callable_to_string::CallableToStringRepresentation;
    use super::*;

    #[test]
    fn native_catalog_names_keep_accessor_and_symbol_property_syntax() {
        for name in [
            "",
            "Array",
            "get input",
            "set input",
            "[Symbol.iterator]",
            "get [Symbol.species]",
        ] {
            assert!(is_catalog_native_function_name(name), "{name}");
        }
        for name in [
            "get RegExp legacy static",
            "set RegExp legacy static",
            "get ",
            "get set input",
            "[Symbol.]",
            "[Symbol.iterator",
            "Symbol.iterator",
        ] {
            assert!(!is_catalog_native_function_name(name), "{name}");
        }
        assert_eq!(
            CallableToStringRepresentation::NativeNamed("get input".to_string()).materialize(),
            "function get input() { [native code] }"
        );
    }
}
