use std::fs;
use std::path::Path;

const DATA_SOURCE: &str = include_str!("../src/data.rs");
const OWNER_SOURCE: &str = include_str!("../src/data/runtime_regexp_entry_kind.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");

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
fn runtime_regexp_entry_kind_has_one_private_capability_free_owner() {
    let lexical_probe = r###"
        RuntimeRegExpEntryKind /* nested /* ignored */ comment */ :: r#Program;
        // RuntimeRegExpEntryKind::Rejected
        let normal = "RuntimeRegExpEntryKind::Unsupported";
        let byte = b"RuntimeRegExpEntryKind::Program";
        let c_string = c"RuntimeRegExpEntryKind::Rejected";
        let raw = r#"RuntimeRegExpEntryKind::Unsupported"#;
        let raw_byte = br#"RuntimeRegExpEntryKind::Program"#;
        let raw_c = cr#"RuntimeRegExpEntryKind::Rejected"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "RuntimeRegExpEntryKind::Program;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "RuntimeRegExpEntryKind"),
        1
    );

    assert_eq!(
        DATA_SOURCE
            .matches("\nmod runtime_regexp_entry_kind;\n")
            .count(),
        1
    );
    assert_eq!(
        DATA_SOURCE
            .matches("pub(crate) use runtime_regexp_entry_kind::RuntimeRegExpEntryKind;")
            .count(),
        1
    );
    assert!(!DATA_SOURCE.contains("\npub mod runtime_regexp_entry_kind;\n"));
    assert_eq!(
        normalize_rust(OWNER_SOURCE).routes,
        concat!(
            "usesuper::*;pub(crate)enumRuntimeRegExpEntryKind{Program,Rejected,Unsupported,}",
            "implRuntimeRegExpEntryKind{",
            "pub(crate)constALL:[Self;3]=[Self::Program,Self::Rejected,Self::Unsupported];",
            "pub(crate)constfnword(&self)->u64{matchself{",
            "Self::Program=>RUNTIME_REGEXP_ENTRY_KIND_PROGRAM,",
            "Self::Rejected=>RUNTIME_REGEXP_ENTRY_KIND_REJECTED,",
            "Self::Unsupported=>RUNTIME_REGEXP_ENTRY_KIND_UNSUPPORTED,}}",
            "pub(crate)constfnthrows_syntax_error(&self)->bool{matchself{",
            "Self::Program|Self::Unsupported=>false,Self::Rejected=>true,}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "RuntimeRegExpEntryKind"),
        10,
        "the owner, reexport, three writers and four reader routes must be the complete source census"
    );
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!("impl{capability}forRuntimeRegExpEntryKind")));
    }
    for forbidden in [
        "RuntimeRegExpEntryKindas",
        "RuntimeRegExpEntryKind::Programas",
        "RuntimeRegExpEntryKind::Rejectedas",
        "RuntimeRegExpEntryKind::Unsupportedas",
    ] {
        assert!(!all_routes.contains(forbidden));
    }
}

#[test]
fn runtime_regexp_entry_kind_preserves_exact_writer_and_wire_policies() {
    let normalized_data = normalize_rust(DATA_SOURCE).routes;
    for (constant, word) in [
        ("RUNTIME_REGEXP_ENTRY_KIND_PROGRAM", 0),
        ("RUNTIME_REGEXP_ENTRY_KIND_REJECTED", 1),
        ("RUNTIME_REGEXP_ENTRY_KIND_UNSUPPORTED", 2),
    ] {
        assert_eq!(
            normalized_data
                .matches(&format!("pub(crate)const{constant}:u64={word};"))
                .count(),
            1,
            "{constant} declaration"
        );
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, constant),
            2,
            "{constant} must occur only at its declaration and typed word projection"
        );
    }

    let writer = bounded(
        DATA_SOURCE,
        "    fn append_runtime_regexp_program_table(&mut self) {",
        "    fn append_regexp_programs(&mut self) {",
    );
    let normalized_writer = normalize_rust(writer).routes;
    for forbidden in ["_=>", "==", "!=", "default()", "unwrap_or"] {
        assert!(!normalized_writer.contains(forbidden), "{forbidden}");
    }
    let typed_assignment =
        "            record[RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD] = match entry {";
    assert_eq!(writer.matches(typed_assignment).count(), 1);
    let writer_tail = writer
        .split_once(typed_assignment)
        .expect("typed entry-kind assignment")
        .1;
    let expected_writer_tail = r#"
                RuntimeRegExpEntry::Program(program) => {
                    record[RUNTIME_REGEXP_RECORD_PROGRAM_PTR_WORD] = program.ptr as u64;
                    record[RUNTIME_REGEXP_RECORD_INSTRUCTION_COUNT_WORD] =
                        program.instruction_count as u64;
                    record[RUNTIME_REGEXP_RECORD_CAPTURE_COUNT_WORD] = program.capture_count as u64;
                    record[RUNTIME_REGEXP_RECORD_SPLIT_COUNT_WORD] = program.split_count as u64;
                    record[RUNTIME_REGEXP_RECORD_REPEATABLE_SPLIT_COUNT_WORD] =
                        program.repeatable_split_count as u64;
                    record[RUNTIME_REGEXP_RECORD_NAMED_GROUP_TABLE_PTR_WORD] =
                        program.named_group_table_ptr as u64;
                    RuntimeRegExpEntryKind::Program.word()
                }
                RuntimeRegExpEntry::Rejected => RuntimeRegExpEntryKind::Rejected.word(),
                RuntimeRegExpEntry::Unsupported => RuntimeRegExpEntryKind::Unsupported.word(),
            };
            for value in record {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

"#;
    assert_eq!(
        normalize_rust(writer_tail).routes,
        normalize_rust(expected_writer_tail).routes,
        "the typed entry-kind assignment must flow directly into exact record serialization"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    for (route, expected) in [
        ("RuntimeRegExpEntryKind::Program.word()", 3),
        ("RuntimeRegExpEntryKind::Rejected.word()", 1),
        ("RuntimeRegExpEntryKind::Unsupported.word()", 1),
    ] {
        assert_eq!(all_routes.matches(route).count(), expected, "{route}");
    }
    assert_eq!(
        all_routes
            .matches(".map(RuntimeRegExpEntryKind::word)")
            .count(),
        1
    );
    assert_eq!(all_routes.matches(".throws_syntax_error()").count(), 1);
    assert_eq!(
        all_routes
            .matches("RuntimeRegExpEntryKind::ALL.iter()")
            .count(),
        1
    );
    assert_eq!(
        all_routes.matches("RuntimeRegExpEntryKind::word").count(),
        1
    );
    assert!(!all_routes.contains("<RuntimeRegExpEntryKind>::word"));
    assert!(!all_routes.contains("RuntimeRegExpEntryKind::ALL.into_iter()"));
}

#[test]
fn runtime_regexp_entry_kind_reader_is_borrowed_and_ordered() {
    let reader = bounded(
        EXPRESSIONS_SOURCE,
        "    pub(crate) fn emit_runtime_regexp_program_slots(",
        "    fn compile_regexp_literal_payload(",
    );
    let normalized_reader = normalize_rust(reader).routes;
    assert_eq!(
        normalized_reader
            .matches(concat!(
                "function.instruction(&Instruction::I64Const(",
                "RuntimeRegExpEntryKind::Program.word()asi64,));",
                "function.instruction(&Instruction::LocalSet(entry_kind_local));"
            ))
            .count(),
        1
    );
    assert_eq!(
        normalized_reader
            .matches(concat!(
                "function.instruction(&Instruction::LocalGet(entry_kind_local));",
                "function.instruction(&Instruction::I64Const(",
                "RuntimeRegExpEntryKind::Program.word()asi64,));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "for(record_word,heap_offset)in["
            ))
            .count(),
        1
    );
    assert_eq!(
        normalized_reader
            .matches(concat!(
                "letthrowing_kind_words=RuntimeRegExpEntryKind::ALL.iter()",
                ".filter(|kind|kind.throws_syntax_error())",
                ".map(RuntimeRegExpEntryKind::word).collect::<Vec<_>>();"
            ))
            .count(),
        1
    );
    let tail_start = reader
        .find("        let throwing_kind_words = RuntimeRegExpEntryKind::ALL")
        .expect("throwing-kind reader pipeline");
    let reader_tail = &reader[tail_start..];
    let expected_tail = r#"
        let throwing_kind_words = RuntimeRegExpEntryKind::ALL
            .iter()
            .filter(|kind| kind.throws_syntax_error())
            .map(RuntimeRegExpEntryKind::word)
            .collect::<Vec<_>>();
        let mut thrown = Ok(());
        if !throwing_kind_words.is_empty() {
            for (position, word) in throwing_kind_words.iter().enumerate() {
                function.instruction(&Instruction::LocalGet(entry_kind_local));
                function.instruction(&Instruction::I64Const(*word as i64));
                function.instruction(&Instruction::I64Eq);
                if position > 0 {
                    function.instruction(&Instruction::I32Or);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            thrown = self.emit_throw_runtime_error_to_active_handler(
                SYNTAX_ERROR_NAME,
                "Invalid regular expression pattern",
                self.result_local,
                self.result_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(entry_kind_local);
        self.release_temp_local(candidate_payload_local);
        self.release_temp_local(record_ptr_local);
        self.release_temp_local(index_local);
        thrown
    }

"#;
    assert_eq!(
        normalize_rust(reader_tail).routes,
        normalize_rust(expected_tail).routes,
        "the throwing comparison, SyntaxError emission and local-unwind tail must remain exact"
    );
    assert_eq!(normalized_reader.matches(".copied()").count(), 0);
    assert!(!normalized_reader.contains("RuntimeRegExpEntryKind::ALL.into_iter()"));
    assert!(!normalized_reader.contains("_=>"));
    for forbidden in [
        "kind==",
        "kind!=",
        "RuntimeRegExpEntryKind::default",
        "RuntimeRegExpEntryKind::ALL.into_iter()",
    ] {
        assert!(!normalized_reader.contains(forbidden), "{forbidden}");
    }
}
