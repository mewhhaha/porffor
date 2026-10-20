const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");
const PROMISE_FINALLY_COMPLETION_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_finally_completion.rs");
const STANDARD_SOURCE: &str = include_str!("../../src/builtins/standard.rs");

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    source[start_offset..]
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn quoted_literal_end(source: &str, quote_start: usize, quote: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return Some(offset + 1);
        }
        offset += 1;
    }
    None
}

fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let value_start = start + 1;
    if value_start >= bytes.len() {
        return None;
    }
    let value_end = if bytes[value_start] == b'\\' {
        let mut offset = value_start + 1;
        if offset >= bytes.len() {
            return None;
        }
        if bytes[offset] == b'u' && bytes.get(offset + 1) == Some(&b'{') {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'}') {
                offset += 1;
            }
            if bytes.get(offset) != Some(&b'}') {
                return None;
            }
            offset + 1
        } else if bytes[offset] == b'x'
            && bytes
                .get(offset + 1..offset + 3)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
        {
            offset + 3
        } else {
            offset + 1
        }
    } else {
        value_start + source[value_start..].chars().next()?.len_utf8()
    };
    (bytes.get(value_end) == Some(&b'\'')).then_some(value_end + 1)
}

fn raw_literal_end(source: &str, start: usize, prefix_len: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote_start = start + prefix_len;
    while bytes.get(quote_start) == Some(&b'#') {
        quote_start += 1;
    }
    if bytes.get(quote_start) != Some(&b'"') {
        return None;
    }
    let hashes = quote_start - start - prefix_len;
    let mut offset = quote_start + 1;
    while offset < bytes.len() {
        if bytes[offset] == b'"'
            && bytes
                .get(offset + 1..offset + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(offset + 1 + hashes);
        }
        offset += 1;
    }
    None
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    match bytes.get(start).copied()? {
        b'"' => quoted_literal_end(source, start, b'"'),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => {
            quoted_literal_end(source, start + 1, b'"')
        }
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

struct NormalizedRust {
    code: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push_str(&source[offset..end]);
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            offset += 2;
            let mut depth = 1;
            while offset < bytes.len() && depth != 0 {
                if bytes.get(offset..offset + 2) == Some(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if bytes.get(offset..offset + 2) == Some(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
            assert_eq!(depth, 0, "unterminated block comment in Rust source");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#")
            && source[offset + 2..]
                .chars()
                .next()
                .is_some_and(|character| character == '_' || character.is_alphabetic())
        {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            code.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust { code }
}

#[test]
fn finally_completion_is_one_private_closed_domain() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_finally_completion;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_finally_completion;"));
    assert!(!PROMISE_SOURCE.contains("promise_finally_completion::"));
    assert!(!PROMISE_SOURCE.contains("PromiseFinallyCompletion"));
    assert!(!PROMISE_FINALLY_COMPLETION_SOURCE.contains("pub enum PromiseFinallyCompletion"));
    assert!(!PROMISE_FINALLY_COMPLETION_SOURCE.contains("#[derive"));
    let declaration_marker = "enum PromiseFinallyCompletion {";
    let declaration_offset = PROMISE_FINALLY_COMPLETION_SOURCE
        .find(declaration_marker)
        .expect("PromiseFinallyCompletion declaration");
    let following_impl = PROMISE_FINALLY_COMPLETION_SOURCE[declaration_offset..]
        .find("impl FunctionBuilder<'_> {")
        .map(|offset| declaration_offset + offset)
        .expect("PromiseFinallyCompletion impl");
    assert_eq!(
        normalize_rust(&PROMISE_FINALLY_COMPLETION_SOURCE[declaration_offset..following_impl]).code,
        "enumPromiseFinallyCompletion{Fulfill,Reject,}",
        "the private domain must remain exact and attribute-free"
    );
}

#[test]
fn named_wrappers_own_the_four_spec_mappings() {
    for (start, end, expected) in [
        (
            "pub(crate) fn emit_promise_then_finally(",
            "pub(crate) fn emit_promise_catch_finally(",
            r#"
                pub(crate) fn emit_promise_then_finally(
                    &mut self,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_promise_finally_continuation(
                        PromiseFinallyCompletion::Fulfill,
                        function
                    )
                }
            "#,
        ),
        (
            "pub(crate) fn emit_promise_catch_finally(",
            "fn emit_promise_finally_continuation(",
            r#"
                pub(crate) fn emit_promise_catch_finally(
                    &mut self,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_promise_finally_continuation(
                        PromiseFinallyCompletion::Reject,
                        function
                    )
                }
            "#,
        ),
        (
            "pub(crate) fn emit_promise_value_thunk(",
            "pub(crate) fn emit_promise_thrower(",
            r#"
                pub(crate) fn emit_promise_value_thunk(
                    &mut self,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_promise_finally_value_thunk(
                        PromiseFinallyCompletion::Fulfill,
                        function
                    )
                }
            "#,
        ),
        (
            "pub(crate) fn emit_promise_thrower(",
            "fn emit_promise_finally_value_thunk(",
            r#"
                pub(crate) fn emit_promise_thrower(
                    &mut self,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_promise_finally_value_thunk(
                        PromiseFinallyCompletion::Reject,
                        function
                    )
                }
            "#,
        ),
    ] {
        assert_eq!(
            normalize_rust(bounded_inclusive(
                PROMISE_FINALLY_COMPLETION_SOURCE,
                start,
                end,
            ))
            .code,
            normalize_rust(expected).code,
            "wrapper `{start}`"
        );
    }

    for retired in [
        "emit_promise_finally_continuation(\n        &mut self,\n        rejected: bool,",
        "emit_promise_finally_value_thunk(\n        &mut self,\n        throws: bool,",
        "emit_promise_finally_continuation(false, function)",
        "emit_promise_finally_continuation(true, function)",
        "emit_promise_finally_value_thunk(false, function)",
        "emit_promise_finally_value_thunk(true, function)",
    ] {
        assert!(
            !PROMISE_FINALLY_COMPLETION_SOURCE.contains(retired),
            "retired `{retired}`"
        );
        assert!(!PROMISE_SOURCE.contains(retired), "retired `{retired}`");
        assert!(!STANDARD_SOURCE.contains(retired), "retired `{retired}`");
    }
}

#[test]
fn standard_dispatch_has_no_finally_direction_choice() {
    let dispatch = normalize_rust(bounded_inclusive(
        STANDARD_SOURCE,
        "StandardBuiltinId::PromiseThenFinally => {",
        "StandardBuiltinId::PromiseResolve => {",
    ));
    assert_eq!(
        dispatch.code,
        concat!(
            "StandardBuiltinId::PromiseThenFinally=>{",
            "self.emit_promise_then_finally(function)?;}",
            "StandardBuiltinId::PromiseCatchFinally=>{",
            "self.emit_promise_catch_finally(function)?;}",
            "StandardBuiltinId::PromiseValueThunk=>{",
            "self.emit_promise_value_thunk(function)?;}",
            "StandardBuiltinId::PromiseThrower=>{",
            "self.emit_promise_thrower(function)?;}"
        ),
        "each standard builtin must retain its exact named completion wrapper"
    );
    assert!(!dispatch.code.contains("PromiseFinallyCompletion"));
    assert!(!dispatch.code.contains("true"));
    assert!(!dispatch.code.contains("false"));
}
