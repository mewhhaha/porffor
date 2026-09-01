use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/sync-iterator-protocol-error-authority.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
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
        if !character.is_whitespace() {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
        } else {
            identifiers.push(' ');
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

fn count_route_in_rust_sources(dir: &Path, route: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_route_in_rust_sources(&path, route);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            normalize_rust(&source).routes.matches(route).count()
        })
        .sum()
}

#[test]
fn protocol_error_is_one_private_capability_free_four_variant_authority() {
    let probe = normalize_rust(
        r###"
        SyncIteratorProtocolError /* nested /* ignored */ comment */ :: r#NotIterable;
        // SyncIteratorProtocolError::NextNotCallable
        let normal = "SyncIteratorProtocolError::NextResultNotObject";
        let byte = b"SyncIteratorProtocolError";
        let c_string = c"SyncIteratorProtocolError";
        let raw = r#"SyncIteratorProtocolError"#;
        let raw_byte = br#"SyncIteratorProtocolError"#;
        let raw_c = cr#"SyncIteratorProtocolError"#;
        let character = '\x7b';
        let byte_character = b'}';
        let borrowed: &'a str = value;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&probe.identifiers, "SyncIteratorProtocolError"),
        1
    );
    assert_eq!(
        probe.routes,
        concat!(
            "SyncIteratorProtocolError::NotIterable;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );

    let declaration = normalize_rust(bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) enum IteratorFlatMapInnerState {\n    NotInstalled,\n    Active,\n}\n\n",
        "struct DestructuringIteratorLocals",
    ));
    assert_eq!(
        declaration.code,
        "enumSyncIteratorProtocolError{NotIterable,MethodResultNotObject,NextNotCallable,NextResultNotObject,}"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "SyncIteratorProtocolError"),
        35,
        "one declaration, one typed consumer, seventeen producers and sixteen mapping arms must own the authority"
    );
    for (variant, count) in [
        ("NotIterable", 10),
        ("MethodResultNotObject", 7),
        ("NextNotCallable", 8),
        ("NextResultNotObject", 8),
    ] {
        assert_eq!(
            count_route_in_rust_sources(
                &source_root,
                &format!("SyncIteratorProtocolError::{variant}")
            ),
            count
        );
    }
    let routes = &normalize_rust(CONTROL_FLOW_SOURCE).routes;
    for forbidden in [
        "implCloneforSyncIteratorProtocolError",
        "implCopyforSyncIteratorProtocolError",
        "implDebugforSyncIteratorProtocolError",
        "implDefaultforSyncIteratorProtocolError",
        "implPartialEqforSyncIteratorProtocolError",
        "implEqforSyncIteratorProtocolError",
        "typeSyncIteratorProtocolError=",
        "asSyncIteratorProtocolError",
    ] {
        assert!(!routes.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn shared_helpers_bind_five_protocol_failures_to_their_exact_checks_in_order() {
    let acquisition = normalize_rust(bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_iterator_from_value_locals(",
        "    fn emit_sync_iterator_protocol_type_error(",
    ));
    let step = normalize_rust(bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    ));
    let rows = [
        concat!(
            "self.compile_nullish_tagged_i32(source_tag,function)?;",
            "self.open_frame(ControlFrameKind::If,function);",
            "self.emit_sync_iterator_protocol_type_error(consumer,SyncIteratorProtocolError::NotIterable,function,)?;",
            "self.emit_propagate_current_throw(function);self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::End);"
        ),
        concat!(
            "self.emit_is_callable_i32(method_tag,method_payload,function)?;",
            "function.instruction(&Instruction::I32Eqz);self.open_frame(ControlFrameKind::If,function);",
            "self.emit_sync_iterator_protocol_type_error(consumer,SyncIteratorProtocolError::NotIterable,function,)?;",
            "self.emit_propagate_current_throw(function);self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::End);"
        ),
        concat!(
            "self.emit_is_heap_object_like_tag_i32(locals.iterator_tag,function);",
            "function.instruction(&Instruction::I32Eqz);self.open_frame(ControlFrameKind::If,function);",
            "self.emit_sync_iterator_protocol_type_error(consumer,SyncIteratorProtocolError::MethodResultNotObject,function,)?;",
            "self.emit_propagate_current_throw(function);self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::End);"
        ),
        concat!(
            "self.emit_is_callable_i32(locals.next_tag,locals.next_payload,function)?;",
            "function.instruction(&Instruction::I32Eqz);self.open_frame(ControlFrameKind::If,function);",
            "self.emit_sync_iterator_protocol_type_error(consumer,SyncIteratorProtocolError::NextNotCallable,function,)?;",
            "self.emit_propagate_current_throw(function);self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::End);"
        ),
        concat!(
            "self.emit_is_heap_object_like_tag_i32(locals.result_tag,function);",
            "function.instruction(&Instruction::I32Eqz);self.open_frame(ControlFrameKind::If,function);",
            "self.emit_sync_iterator_protocol_type_error(consumer,SyncIteratorProtocolError::NextResultNotObject,function,)?;",
            "self.emit_propagate_current_throw(function);self.pop_control(ControlFrameKind::If);",
            "function.instruction(&Instruction::End);"
        ),
    ];
    for (source, expected_rows) in [
        (acquisition.code.as_str(), &rows[..3]),
        (step.code.as_str(), &rows[3..]),
    ] {
        let mut prior = 0;
        for &row in expected_rows {
            assert_eq!(
                source.matches(row).count(),
                1,
                "missing exact producer `{row}`"
            );
            let offset = source.find(row).unwrap();
            assert!(offset >= prior, "protocol-error producer order changed");
            prior = offset + row.len();
        }
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_route_in_rust_sources(&source_root, ".emit_sync_iterator_protocol_type_error("),
        17
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "::emit_sync_iterator_protocol_type_error"),
        0
    );
}

#[test]
fn sole_consumer_exhaustively_maps_diagnostics_and_body_realm_sources() {
    let consumer = normalize_rust(bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    fn compile_array_destructuring_element(",
    ));
    assert!(consumer.code.starts_with(concat!(
        "&mutself,consumer:&SyncIteratorConsumer,error:SyncIteratorProtocolError,",
        "function:&mutFunction,)->Result<(),EmitError>{letmessage=match(consumer,error){"
    )));
    let canonical_rows = consumer
        .code
        .chars()
        .filter(|character| !matches!(*character, '{' | '}' | ','))
        .collect::<String>();
    for (semantic_consumer, error, message) in [
        (
            "ArrayDestructuring",
            "NotIterable",
            "destructuring value is not iterable",
        ),
        (
            "ArrayDestructuring",
            "MethodResultNotObject",
            "destructuring iterator method must return object",
        ),
        (
            "ArrayDestructuring",
            "NextNotCallable",
            "destructuring iterator next must be callable",
        ),
        (
            "ArrayDestructuring",
            "NextResultNotObject",
            "destructuring iterator next result must be object",
        ),
        (
            "ArrayAccumulation",
            "NotIterable",
            "array spread value is not iterable",
        ),
        (
            "ArrayAccumulation",
            "MethodResultNotObject",
            "array spread iterator method must return object",
        ),
        (
            "ArrayAccumulation",
            "NextNotCallable",
            "array spread iterator next must be callable",
        ),
        (
            "ArrayAccumulation",
            "NextResultNotObject",
            "array spread iterator next result must be object",
        ),
        ("ForOf", "NotIterable", "for-of target is not iterable"),
        (
            "ForOf",
            "MethodResultNotObject",
            "for-of iterator method must return object",
        ),
        (
            "ForOf",
            "NextNotCallable",
            "for-of iterator next must be callable",
        ),
        (
            "ForOf",
            "NextResultNotObject",
            "for-of iterator next result must be object",
        ),
        (
            "MathSumPrecise",
            "NotIterable",
            "Math.sumPrecise input is not iterable",
        ),
        (
            "MathSumPrecise",
            "MethodResultNotObject",
            "Math.sumPrecise iterator method must return an object",
        ),
        (
            "MathSumPrecise",
            "NextNotCallable",
            "Math.sumPrecise iterator next method is not callable",
        ),
        (
            "MathSumPrecise",
            "NextResultNotObject",
            "Math.sumPrecise iterator next result must be an object",
        ),
    ] {
        let row = format!(
            "(SyncIteratorConsumer::{semantic_consumer}SyncIteratorProtocolError::{error})=>\"{message}\""
        );
        assert_eq!(
            canonical_rows.matches(&row).count(),
            1,
            "diagnostic row `{row}`"
        );
    }
    assert_eq!(consumer.code.matches("match(consumer,error){").count(), 1);
    for semantic_consumer in [
        "ArrayDestructuring",
        "ArrayAccumulation",
        "ForOf",
        "MathSumPrecise",
    ] {
        assert_eq!(
            consumer
                .code
                .matches(&format!("SyncIteratorConsumer::{semantic_consumer}"))
                .count(),
            4,
            "diagnostic row count for {semantic_consumer}"
        );
    }
    for error in [
        "NotIterable",
        "MethodResultNotObject",
        "NextNotCallable",
        "NextResultNotObject",
    ] {
        assert_eq!(
            consumer
                .code
                .matches(&format!("SyncIteratorProtocolError::{error}"))
                .count(),
            4,
            "diagnostic row count for {error}"
        );
    }
    assert_eq!(
        consumer
            .code
            .matches("self.emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .code
            .matches("self.emit_throw_runtime_error(")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .code
            .matches("matchself.numeric_error_realm_source(){")
            .count(),
        1
    );
    for source in [
        "NumericErrorRealmSource::StandardBuiltinEnvironment",
        "NumericErrorRealmSource::GlobalFallback",
        "NumericErrorRealmSource::NumericConversionHelperArgument",
    ] {
        assert_eq!(
            consumer.code.matches(source).count(),
            1,
            "Realm source {source}"
        );
    }
    for forbidden in [
        "matcherror{",
        "matchconsumer{",
        "&error",
        "error==",
        "error!=",
        "matches!(error",
        "_=>",
    ] {
        assert!(!consumer.code.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_task_record_the_focused_invariant_without_conformance_overclaim() {
    let contract = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for phrase in [
        "SyncIteratorProtocolError",
        "Exactly seventeen typed projector calls",
        "confirmed census is 35 identifiers",
        "Central verification for the seventeen-producer boundary passed",
    ] {
        assert!(contract.contains(phrase), "contract missing `{phrase}`");
    }
    assert!(task.contains("sync iterator protocol-error authority"));
    assert!(task.contains("35"));
    assert!(task.contains("`SyncIteratorProtocolError`"));
    assert!(task.contains("No complete Test262 directory"));
}
