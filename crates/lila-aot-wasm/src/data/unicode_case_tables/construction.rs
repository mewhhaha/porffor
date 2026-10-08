//! Original scalar walks, executed by Cargo when building the compiler.

pub(crate) fn lowercase() -> Vec<u8> {
    let mut mappings = Vec::new();
    for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
        let mut lowercase = codepoint.to_lowercase();
        let first = lowercase.next().expect("lowercase mapping is never empty");
        let second = lowercase.next();
        if first == codepoint && second.is_none() {
            continue;
        }
        let mut lowercase_bytes = Vec::with_capacity(4);
        for lowercase_codepoint in std::iter::once(first).chain(second).chain(lowercase) {
            let mut encoded = [0; 4];
            lowercase_bytes
                .extend_from_slice(lowercase_codepoint.encode_utf8(&mut encoded).as_bytes());
        }
        assert!(lowercase_bytes.len() <= 4, "lowercase row payload overflow");
        mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
        mappings.extend_from_slice(&(lowercase_bytes.len() as u32).to_le_bytes());
        mappings.extend_from_slice(&lowercase_bytes);
        mappings.resize(mappings.len() + 4 - lowercase_bytes.len(), 0);
        mappings.extend_from_slice(&0_u32.to_le_bytes());
    }
    mappings
}

pub(crate) fn uppercase() -> Vec<u8> {
    let mut mappings = Vec::new();
    for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
        let mut uppercase = codepoint.to_uppercase();
        let first = uppercase.next().expect("uppercase mapping is never empty");
        let second = uppercase.next();
        if first == codepoint && second.is_none() {
            continue;
        }
        let mut uppercase_bytes = Vec::with_capacity(8);
        for uppercase_codepoint in std::iter::once(first).chain(second).chain(uppercase) {
            let mut encoded = [0; 4];
            uppercase_bytes
                .extend_from_slice(uppercase_codepoint.encode_utf8(&mut encoded).as_bytes());
        }
        assert!(uppercase_bytes.len() <= 8, "uppercase row payload overflow");
        mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
        mappings.extend_from_slice(&(uppercase_bytes.len() as u32).to_le_bytes());
        mappings.extend_from_slice(&uppercase_bytes);
        mappings.resize(mappings.len() + 8 - uppercase_bytes.len(), 0);
    }
    mappings
}
