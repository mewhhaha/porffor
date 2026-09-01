use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_JOB_TO_ENQUEUE_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_job_to_enqueue.rs");

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
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
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
            identifiers.push(character);
            routes.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
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

fn exact_prefixed_identifier_count(source: &str, prefix: &str, identifier: &str) -> usize {
    let needle = format!("{prefix}{identifier}");
    source
        .match_indices(&needle)
        .filter(|(offset, _)| {
            source[*offset + needle.len()..]
                .chars()
                .next()
                .map(|character| !character.is_alphanumeric() && character != '_')
                .unwrap_or(true)
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

#[test]
fn promise_job_to_enqueue_is_one_private_non_copy_payload_authority() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_job_to_enqueue;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_job_to_enqueue;"));
    assert!(!PROMISE_SOURCE.contains("promise_job_to_enqueue::"));
    assert!(!PROMISE_SOURCE.contains("PromiseJobToEnqueue"));
    assert!(!PROMISE_JOB_TO_ENQUEUE_SOURCE.contains("pub enum PromiseJobToEnqueue"));
    assert!(PROMISE_JOB_TO_ENQUEUE_SOURCE.lines().count() <= 220);

    let lexical_probe = r###"
        PromiseJobToEnqueue /* nested /* ignored */ comment */ :: r#Reaction;
        // PromiseJobToEnqueue::ResolveThenable
        let normal = "PromiseJobToEnqueue";
        let byte = b"PromiseJobToEnqueue";
        let c_string = c"PromiseJobToEnqueue";
        let raw = r#"PromiseJobToEnqueue"#;
        let raw_byte = br#"PromiseJobToEnqueue"#;
        let raw_c = cr#"PromiseJobToEnqueue"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "PromiseJobToEnqueue::Reaction;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "PromiseJobToEnqueue"),
        1
    );

    let declaration_scope = bounded(
        PROMISE_JOB_TO_ENQUEUE_SOURCE,
        "use super::*;",
        "impl<'a> FunctionBuilder<'a> {",
    );
    let declaration_routes = normalize_rust(declaration_scope).routes;
    assert!(!declaration_scope.contains("#[derive"));
    let declaration_end = declaration_routes
        .rfind('}')
        .expect("PromiseJobToEnqueue declaration must close")
        + 1;
    assert_eq!(
        &declaration_routes[..declaration_end],
        concat!(
            "enumPromiseJobToEnqueue{",
            "Reaction{reaction_record_local:u32,argument_payload_local:u32,argument_tag_local:u32,},",
            "ResolveThenable{thenable_job_local:u32,then_payload_local:u32,then_tag_local:u32,},}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PromiseJobToEnqueue"),
        6,
        "one declaration, two producers, one owned parameter and two arms must be complete"
    );
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    assert!(!all_routes.contains("promise_job_to_enqueue::"));
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!("impl{capability}forPromiseJobToEnqueue")));
    }
    for forbidden in [
        "PromiseJobToEnqueueas",
        "PromiseJobToEnqueue::Reactionas",
        "PromiseJobToEnqueue::ResolveThenableas",
    ] {
        assert!(!all_routes.contains(forbidden));
    }
}

#[test]
fn both_complete_payload_producers_immediately_consume_their_job() {
    let reaction = format!(
        "fn emit_enqueue_promise_reaction_job({}",
        bounded(
            PROMISE_JOB_TO_ENQUEUE_SOURCE,
            "    pub(super) fn emit_enqueue_promise_reaction_job(",
            "    pub(super) fn emit_enqueue_promise_thenable_job(",
        )
    );
    let expected_reaction = r#"fn emit_enqueue_promise_reaction_job(
        &mut self,
        reaction_record_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_enqueue_promise_job(
            PromiseJobToEnqueue::Reaction {
                reaction_record_local,
                argument_payload_local,
                argument_tag_local,
            },
            function,
        )
    }

"#;
    assert_eq!(
        normalize_rust(&reaction).routes,
        normalize_rust(expected_reaction).routes,
        "the reaction producer must remain one complete owned selection and enqueue"
    );

    let thenable = format!(
        "fn emit_enqueue_promise_thenable_job({}",
        bounded(
            PROMISE_JOB_TO_ENQUEUE_SOURCE,
            "    pub(super) fn emit_enqueue_promise_thenable_job(",
            "    fn emit_enqueue_promise_job(",
        )
    );
    let expected_thenable = r#"fn emit_enqueue_promise_thenable_job(
        &mut self,
        promise_payload_local: u32,
        promise_record_local: u32,
        thenable_payload_local: u32,
        thenable_tag_local: u32,
        then_payload_local: u32,
        then_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thenable_job_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_PROMISE_THENABLE_JOB_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(thenable_job_local));
        for (offset, value_local) in [
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_RECORD_OFFSET,
                promise_record_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_PAYLOAD_OFFSET,
                promise_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_PAYLOAD_OFFSET,
                thenable_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_TAG_OFFSET,
                thenable_tag_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THEN_PAYLOAD_OFFSET,
                then_payload_local,
            ),
            (HEAP_PROMISE_THENABLE_JOB_THEN_TAG_OFFSET, then_tag_local),
        ] {
            self.store_i64_local_at_offset(thenable_job_local, offset, value_local, function);
        }

        let result = self.emit_enqueue_promise_job(
            PromiseJobToEnqueue::ResolveThenable {
                thenable_job_local,
                then_payload_local,
                then_tag_local,
            },
            function,
        );
        self.release_temp_local(thenable_job_local);
        result
    }

"#;
    assert_eq!(
        normalize_rust(&thenable).routes,
        normalize_rust(expected_thenable).routes,
        "the thenable producer must populate all six words before its sole owned enqueue and release"
    );

    let route_probe = r#"
        receiver /* call boundary */ . r#emit_enqueue_promise_job :: <Job>(job);
        let selected = receiver . emit_enqueue_promise_job;
        let alternate = FunctionBuilder /* UFCS */ :: r#emit_enqueue_promise_job :: <'a>;
        let inert = ".emit_enqueue_promise_job :: <Ignored>";
    "#;
    let normalized_probe = normalize_rust(route_probe).routes;
    assert_eq!(
        exact_prefixed_identifier_count(&normalized_probe, ".", "emit_enqueue_promise_job"),
        2
    );
    assert_eq!(
        exact_prefixed_identifier_count(&normalized_probe, "::", "emit_enqueue_promise_job"),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    assert_eq!(
        exact_prefixed_identifier_count(&all_routes, ".", "emit_enqueue_promise_job"),
        2,
        "only the two exact producer calls may select the owned consumer"
    );
    assert_eq!(
        exact_prefixed_identifier_count(&all_routes, "::", "emit_enqueue_promise_job"),
        0,
        "method items, turbofish and UFCS routes must not create an alternate owner"
    );
    assert_eq!(all_routes.matches("fnemit_enqueue_promise_job(").count(), 1);
}

#[test]
fn sole_consumer_exhaustively_routes_payload_before_one_fifo_append() {
    let consumer = bounded(
        PROMISE_JOB_TO_ENQUEUE_SOURCE,
        "    fn emit_enqueue_promise_job(",
        "\n}",
    );
    let expected = r#"
        &mut self,
        job: PromiseJobToEnqueue,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let job_record_local = self.reserve_temp_local();
        let queue_tail_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_PENDING_JOB_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(job_record_local));
        self.store_i64_const_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        let kind = match job {
            PromiseJobToEnqueue::Reaction {
                reaction_record_local,
                argument_payload_local,
                argument_tag_local,
            } => {
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                    reaction_record_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                    argument_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                    argument_payload_local,
                    function,
                );
                self.emit_promise_reaction_job_realm_to_local(
                    reaction_record_local,
                    realm_local,
                    function,
                )?;
                PromiseJobKind::Reaction
            }
            PromiseJobToEnqueue::ResolveThenable {
                thenable_job_local,
                then_payload_local,
                then_tag_local,
            } => {
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                    thenable_job_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                    function,
                );
                self.store_i64_const_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                    0,
                    function,
                );
                self.emit_promise_job_callback_realm_to_local(
                    then_payload_local,
                    then_tag_local,
                    realm_local,
                    function,
                )?;
                PromiseJobKind::ResolveThenable
            }
        };
        self.store_i64_local_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_REALM_OFFSET,
            realm_local,
            function,
        );
        self.store_i64_const_at_offset(job_record_local, HEAP_PENDING_JOB_NEXT_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_KIND_OFFSET,
            kind.word(),
            function,
        );
        function.instruction(&Instruction::GlobalGet(PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(queue_tail_local));
        function.instruction(&Instruction::LocalGet(queue_tail_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(job_record_local));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            queue_tail_local,
            HEAP_PENDING_JOB_NEXT_OFFSET,
            job_record_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(job_record_local));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX));
        self.release_temp_local(realm_local);
        self.release_temp_local(queue_tail_local);
        self.release_temp_local(job_record_local);
        Ok(())
    }

"#;
    assert_eq!(
        normalize_rust(consumer).routes,
        normalize_rust(expected).routes,
        "payload selection and the sole FIFO append must remain one exact owned operation"
    );
}
