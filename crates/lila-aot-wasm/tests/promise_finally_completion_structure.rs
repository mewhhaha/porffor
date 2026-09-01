use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_FINALLY_COMPLETION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_finally_completion.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/promise-finally-completion.md");
const TASK: &str = include_str!("../../../tasks/14-promises-jobs-async.md");

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
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push_str(&source[offset..end]);
            identifiers.push(' ');
            routes.push('L');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            identifiers.push(' ');
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            identifiers.push(' ');
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
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
        code,
        identifiers,
        routes,
    }
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn exact_route_count(source: &str, route: &str) -> usize {
    exact_identifier_count(source, route)
}

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&normalize_rust(&source).identifiers, identifier)
        })
        .sum()
}

fn normalized_routes_in_rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return normalized_routes_in_rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let mut routes = normalize_rust(&source).routes;
            routes.push('\n');
            routes
        })
        .collect()
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: `{marker}`"));
        cursor += offset + marker.len();
    }
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
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
    assert!(PROMISE_FINALLY_COMPLETION_SOURCE.lines().count() <= 270);

    let lexical_probe = r###"
        // PromiseFinallyCompletion::Fulfill
        PromiseFinallyCompletion /* nested /* ignored */ comment */ :: r#Reject;
        "PromiseFinallyCompletion"; b"PromiseFinallyCompletion::Fulfill";
        c"PromiseFinallyCompletion"; r"PromiseFinallyCompletion::Reject";
        br##"PromiseFinallyCompletion"##; cr#"PromiseFinallyCompletion"#;
        'P'; b'F'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "PromiseFinallyCompletion"),
        1
    );
    assert_eq!(
        exact_route_count(&lexical_probe.routes, "PromiseFinallyCompletion::Reject"),
        1
    );

    let declaration_marker = "enum PromiseFinallyCompletion {";
    let declaration_offset = PROMISE_FINALLY_COMPLETION_SOURCE
        .find(declaration_marker)
        .expect("PromiseFinallyCompletion declaration");
    let following_impl = PROMISE_FINALLY_COMPLETION_SOURCE[declaration_offset..]
        .find("impl PromiseFinallyCompletion {")
        .map(|offset| declaration_offset + offset)
        .expect("PromiseFinallyCompletion impl");
    assert_eq!(
        normalize_rust(&PROMISE_FINALLY_COMPLETION_SOURCE[declaration_offset..following_impl]).code,
        "enumPromiseFinallyCompletion{Fulfill,Reject,}",
        "the private domain must remain exact and attribute-free"
    );

    let policy = normalize_rust(bounded_inclusive(
        PROMISE_FINALLY_COMPLETION_SOURCE,
        "impl PromiseFinallyCompletion {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        policy.code,
        concat!(
            "implPromiseFinallyCompletion{",
            "constfncontinuation_builtin(self)->StandardBuiltinId{matchself{",
            "Self::Fulfill=>StandardBuiltinId::PromiseValueThunk,",
            "Self::Reject=>StandardBuiltinId::PromiseThrower,}}",
            "constfncompletion_kind(self)->CompletionKind{matchself{",
            "Self::Fulfill=>CompletionKind::Normal,",
            "Self::Reject=>CompletionKind::Throw,}}}"
        ),
        "both semantic projections must consume one exact closed choice"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PromiseFinallyCompletion"),
        8,
        "declaration, impl, four producers and two consumer signatures own every mention"
    );
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    assert!(!all_routes.contains("promise_finally_completion::"));
    let child_routes = normalize_rust(PROMISE_FINALLY_COMPLETION_SOURCE).routes;
    assert_eq!(
        exact_route_count(&child_routes, "PromiseFinallyCompletion::Fulfill"),
        2
    );
    assert_eq!(
        exact_route_count(&child_routes, "PromiseFinallyCompletion::Reject"),
        2
    );
    assert_eq!(
        exact_route_count(&child_routes, "completion.continuation_builtin"),
        1
    );
    assert_eq!(
        exact_route_count(&child_routes, "completion.completion_kind"),
        1
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!("impl{capability}forPromiseFinallyCompletion")));
    }
    for forbidden in [
        "PromiseFinallyCompletionas",
        "PromiseFinallyCompletion::Fulfillas",
        "PromiseFinallyCompletion::Rejectas",
        "typePromiseFinallyCompletion",
    ] {
        assert!(!all_routes.contains(forbidden), "found `{forbidden}`");
    }
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

    let continuation = normalize_rust(bounded_inclusive(
        PROMISE_FINALLY_COMPLETION_SOURCE,
        "fn emit_promise_finally_continuation(",
        "pub(crate) fn emit_promise_value_thunk(",
    ));
    assert!(continuation.code.starts_with(concat!(
        "fnemit_promise_finally_continuation(&mutself,",
        "completion:PromiseFinallyCompletion,function:&mutFunction,)",
        "->Result<(),EmitError>{"
    )));
    assert_eq!(
        exact_identifier_count(&continuation.identifiers, "completion"),
        2,
        "the continuation choice must be declared and consumed exactly once"
    );
    assert_eq!(
        exact_route_count(&continuation.routes, "completion.continuation_builtin"),
        1
    );
    assert_eq!(
        exact_route_count(&continuation.routes, "completion.completion_kind"),
        0
    );
    positions_in_order(
        &continuation.code,
        &[
            "letcontinuation_builtin=completion.continuation_builtin();",
            "letcontinuation_meta=self.functions.get(&continuation_builtin.function_id())",
            "self.emit_load_promise_internal_function_context",
        ],
    );

    let restoration = normalize_rust(bounded_inclusive(
        PROMISE_FINALLY_COMPLETION_SOURCE,
        "fn emit_promise_finally_value_thunk(",
        "\n}",
    ));
    assert!(restoration.code.starts_with(concat!(
        "fnemit_promise_finally_value_thunk(&mutself,",
        "completion:PromiseFinallyCompletion,function:&mutFunction,)",
        "->Result<(),EmitError>{"
    )));
    assert_eq!(
        exact_identifier_count(&restoration.identifiers, "completion"),
        2,
        "the restoration choice must be declared and consumed exactly once"
    );
    assert_eq!(
        exact_route_count(&restoration.routes, "completion.completion_kind"),
        1
    );
    assert_eq!(
        exact_route_count(&restoration.routes, "completion.continuation_builtin"),
        0
    );
    positions_in_order(
        &restoration.code,
        &[
            "self.emit_load_promise_internal_function_context",
            "HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET",
            "HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET",
            "self.set_completion_kind(completion.completion_kind(),function);",
            "self.release_temp_local(context_local);",
            "Ok(())",
        ],
    );

    for consumer in [&continuation.code, &restoration.code] {
        for forbidden in [
            "&completion",
            "completion.clone",
            "discriminant(&completion)",
            "matchcompletion",
            "matches!(completion",
            "completion==",
            "completion!=",
            "completionas",
        ] {
            assert!(!consumer.contains(forbidden), "found `{forbidden}`");
        }
    }

    let consumer_fingerprints =
        [&continuation.code, &restoration.code].map(|body| (body.len(), fnv1a(body)));
    assert_eq!(
        consumer_fingerprints,
        [(5417, 0x0e0f_e7ee_e097_5928), (592, 0x00d4_c1a6_bcfc_3c62),],
        "both consuming emitters must retain their complete normalized bodies"
    );

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

#[test]
fn contract_and_t14_record_the_one_shot_completion_authority() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "non-`Clone`, non-`Copy`",
        "eight lexical mentions",
        "four exact wrapper producers",
        "two consuming projections",
        "runtime stages remain independently constructed",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T14 marker: {marker}");
    }
}
