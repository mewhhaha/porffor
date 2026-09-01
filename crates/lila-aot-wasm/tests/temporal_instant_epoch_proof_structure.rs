use std::fs;
use std::path::Path;

const INSTANT_SOURCE: &str = include_str!("../src/builtins/temporal_instant.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-instant-epoch-proof.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

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
    let value_end = if bytes.get(value_start) == Some(&b'\\') {
        let mut offset = value_start + 2;
        if bytes.get(value_start + 1) == Some(&b'u') && bytes.get(offset) == Some(&b'{') {
            offset += 1;
            while bytes.get(offset).is_some_and(|byte| *byte != b'}') {
                offset += 1;
            }
            offset + 1
        } else if bytes.get(value_start + 1) == Some(&b'x') {
            offset + 2
        } else {
            offset
        }
    } else {
        value_start + source.get(value_start..)?.chars().next()?.len_utf8()
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

fn rust_code(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push(' ');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            code.push(' ');
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            code.push(' ');
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
        code.push(character);
        offset += character.len_utf8();
    }
    code
}

fn compact_rust(source: &str) -> String {
    rust_code(source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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

fn rust_sources(dir: &Path) -> Vec<String> {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .flat_map(|path| {
            if path.is_dir() {
                return rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return Vec::new();
            }
            vec![fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))]
        })
        .collect()
}

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn validated_epoch_is_one_private_non_copy_proof() {
    let lexical_probe = rust_code(
        r###"
        // EpochNanoseconds
        EpochNanoseconds /* EpochNanoseconds */;
        "EpochNanoseconds"; b"EpochNanoseconds"; c"EpochNanoseconds";
        r"EpochNanoseconds"; br##"EpochNanoseconds"##;
        cr#"EpochNanoseconds"#; 'E'; b'E'; 'lifetime;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "EpochNanoseconds"),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mentions = rust_sources(&source_root)
        .iter()
        .map(|source| exact_identifier_count(&rust_code(source), "EpochNanoseconds"))
        .sum::<usize>();
    assert_eq!(mentions, 5, "review every new validated-epoch observer");

    let declaration_prefix = bounded(
        INSTANT_SOURCE,
        "/// The same pair, after `emit_temporal_instant_validate_range` has accepted it.",
        "struct EpochNanoseconds(",
    );
    assert!(!declaration_prefix.contains("#[derive"));
    assert!(!declaration_prefix.contains("pub "));
    assert!(!INSTANT_SOURCE.contains("impl Clone for EpochNanoseconds"));
    assert!(!INSTANT_SOURCE.contains("impl Copy for EpochNanoseconds"));
}

#[test]
fn range_validation_is_the_only_proof_constructor() {
    let source = compact_rust(INSTANT_SOURCE);
    assert_eq!(source.matches("Ok(EpochNanoseconds(epoch))").count(), 1);

    let constructor = compact_rust(bounded(
        INSTANT_SOURCE,
        "    fn emit_temporal_instant_validated_epoch(",
        "    /// `CreateTemporalInstant(epochNanoseconds)`",
    ));
    let validation = constructor
        .find("self.emit_temporal_instant_validate_range(epoch.payload_local,epoch.tag_local,function)?;")
        .expect("validated-epoch range check");
    let proof = constructor
        .find("Ok(EpochNanoseconds(epoch))")
        .expect("validated-epoch proof construction");
    assert!(validation < proof);
}

#[test]
fn allocation_exhaustively_consumes_the_validated_epoch() {
    let source = compact_rust(INSTANT_SOURCE);
    assert_eq!(source.matches("epoch:EpochNanoseconds").count(), 1);
    assert!(!source.contains("epoch.0"));

    let consumer = compact_rust(bounded(
        INSTANT_SOURCE,
        "    fn emit_alloc_validated_temporal_instant(",
        "    /// `ℤ(epochMilliseconds) × 10^6`",
    ));
    let consume = consumer
        .find("letEpochNanoseconds(UnvalidatedEpochNanoseconds{payload_local,tag_local,})=epoch;")
        .expect("validated-epoch exhaustive destructuring");
    let allocation = consumer
        .find("self.emit_alloc_temporal_instant(payload_local,tag_local,prototype_payload_local,function,)?;")
        .expect("validated Temporal.Instant allocation");
    assert!(consume < allocation);
}

#[test]
fn both_epoch_builtins_follow_validate_then_allocate() {
    for (start, end) in [
        (
            "    pub(crate) fn emit_temporal_instant_from_epoch_nanoseconds(",
            "    /// Temporal proposal 8.2.2 `Temporal.Instant.fromEpochMilliseconds`.",
        ),
        (
            "    pub(crate) fn emit_temporal_instant_from_epoch_milliseconds(",
            "    /// Temporal proposal 8.3.12 `Temporal.Instant.prototype.valueOf`.",
        ),
    ] {
        let builtin = compact_rust(bounded(INSTANT_SOURCE, start, end));
        let validate = builtin
            .find("self.emit_temporal_instant_validated_epoch(")
            .expect("validated-epoch constructor call");
        let allocate = builtin
            .find("self.emit_alloc_validated_temporal_instant(epoch,function)?;")
            .expect("validated-epoch allocation call");
        assert!(validate < allocate);
    }
}

#[test]
fn contract_and_task_record_the_epoch_proof() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("EpochNanoseconds"));
        assert!(evidence.contains("non-`Copy`"));
        assert!(evidence.contains("emit_alloc_validated_temporal_instant"));
        assert!(evidence.contains("5"));
    }
}
