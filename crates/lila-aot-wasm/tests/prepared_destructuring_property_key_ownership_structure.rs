use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/prepared-destructuring-property-key-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/08-environments-control-flow.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn rust_code(source: &str, retain_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'"' {
            let start = offset;
            offset += 1;
            let mut escaped = false;
            while offset < bytes.len() {
                let byte = bytes[offset];
                offset += 1;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    break;
                }
            }
            if retain_literals {
                code.push_str(&source[start..offset]);
            } else {
                code.push(' ');
            }
            continue;
        }
        if bytes[offset] == b'r' {
            let start = offset;
            let mut quote = offset + 1;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let hashes = quote - start - 1;
                offset = quote + 1;
                while offset < bytes.len() {
                    if bytes[offset] == b'"'
                        && bytes
                            .get(offset + 1..offset + 1 + hashes)
                            .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
                    {
                        offset += 1 + hashes;
                        break;
                    }
                    offset += 1;
                }
                if retain_literals {
                    code.push_str(&source[start..offset]);
                } else {
                    code.push(' ');
                }
                continue;
            }
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
            assert_eq!(depth, 0, "unterminated block comment");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#") {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if character.is_whitespace() {
            if !retain_literals {
                code.push(' ');
            }
        } else {
            code.push(character);
        }
        offset += character.len_utf8();
    }
    code
}

fn normalized_rust(source: &str) -> String {
    rust_code(source, true)
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
            exact_identifier_count(&rust_code(&source, false), identifier)
        })
        .sum()
}

#[test]
fn prepared_key_is_the_exact_private_no_capability_domain() {
    let declaration = normalized_rust(bounded(
        SOURCE,
        "enum PreparedDestructuringPropertyKey {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        declaration,
        concat!(
            "Static(String),Computed{raw_key:TypedExpr,payload_local:u32,",
            "tag_local:u32,},}"
        )
    );
    assert!(SOURCE.contains(
        "#[must_use = \"a prepared destructuring property key must be consumed by its write\"]"
    ));
    for forbidden in [
        "impl Clone for PreparedDestructuringPropertyKey",
        "impl Copy for PreparedDestructuringPropertyKey",
        "impl Default for PreparedDestructuringPropertyKey",
        "key_payload: Option<u32>",
        "key_tag: Option<u32>",
    ] {
        assert!(!SOURCE.contains(forbidden), "found `{forbidden}`");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PreparedDestructuringPropertyKey"),
        9
    );
}

#[test]
fn producer_constructs_computed_authority_after_both_locals_are_populated() {
    let producer = normalized_rust(bounded(
        SOURCE,
        "fn prepare_destructuring_target(",
        "fn put_destructuring_target(",
    ));
    let reserve_payload = producer
        .find("letpayload_local=self.reserve_temp_local();")
        .expect("computed key must reserve its payload local");
    let reserve_tag = producer
        .find("lettag_local=self.reserve_temp_local();")
        .expect("computed key must reserve its tag local");
    let populate = producer
        .find("self.compile_expr_to_locals(key,payload_local,tag_local,function)?;")
        .expect("computed key must populate both locals together");
    let propagate = producer
        .find(
            "self.emit_propagate_throw_from_locals_if_needed(payload_local,tag_local,function,)?;",
        )
        .expect("computed-key abrupt completion must precede authority construction");
    let construct = producer
        .find(concat!(
            "PreparedDestructuringPropertyKey::Computed{raw_key:key.clone(),",
            "payload_local,tag_local,}"
        ))
        .expect("computed authority must own the populated local pair");
    assert!(reserve_payload < reserve_tag);
    assert!(reserve_tag < populate);
    assert!(populate < propagate);
    assert!(propagate < construct);
    assert_eq!(
        producer
            .matches("PreparedDestructuringPropertyKey::Static(name.clone())")
            .count(),
        1
    );
}

#[test]
fn write_installs_and_releases_computed_key_locals_exhaustively() {
    let consumer = normalized_rust(bounded(
        SOURCE,
        "DestructuringTargetIr::AssignmentProperty { .. } => {",
        "DestructuringTargetIr::AssignmentPrivate { .. } => {",
    ));
    let install = consumer
        .find(concat!(
            "ifletPreparedDestructuringPropertyKey::Computed{payload_local,tag_local,..}=",
            "&key{scope.insert("
        ))
        .expect("only a computed key may install the prepared key binding");
    let project = consumer
        .find("letproperty_key=match&key{")
        .expect("property-key projection must be exhaustive");
    let write = consumer
        .find("emitter.compile_property_write_to_locals(")
        .expect("prepared key must reach the property write");
    let release = consumer
        .find("matchkey{PreparedDestructuringPropertyKey::Static(_)=>{}")
        .expect("release must exhaustively distinguish static and computed keys");
    assert!(install < project);
    assert!(project < write);
    assert!(write < release);
    assert!(consumer[release..].contains(concat!(
        "PreparedDestructuringPropertyKey::Computed{payload_local,tag_local,..}=>{",
        "self.release_temp_local(tag_local);self.release_temp_local(payload_local);}}"
    )));
    assert!(!consumer.contains("_=>"));
}

#[test]
fn contract_task_and_existing_semantic_witness_name_the_boundary() {
    for text in [CONTRACT, TASK] {
        assert!(text.contains("PreparedDestructuringPropertyKey"));
        assert!(text.contains("prepared_destructuring_property_key_ownership_structure"));
    }
    assert!(CONTRACT
        .contains("run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions"));
    assert!(CONTRACT.contains("source-equivalent"));
}
