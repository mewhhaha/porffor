use std::fs;
use std::path::Path;

const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-buffer-slice-source-reobservation.md");
const TASK: &str = include_str!("../../../tasks/17-typedarrays-binary-data-atomics.md");

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

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn copy_policy_is_one_closed_non_copy_authority() {
    let lexical_probe = r###"
        // ArrayBufferSliceCopyPolicy::DetachableBounded
        ArrayBufferSliceCopyPolicy /* nested /* ignored */ comment */ :: r#SharedBounded;
        "ArrayBufferSliceCopyPolicy"; b"ArrayBufferSliceCopyPolicy";
        c"ArrayBufferSliceCopyPolicy"; r"ArrayBufferSliceCopyPolicy";
        br##"ArrayBufferSliceCopyPolicy"##; cr#"ArrayBufferSliceCopyPolicy"#;
        'A'; b'A'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "ArrayBufferSliceCopyPolicy"),
        1
    );
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.routes,
            "ArrayBufferSliceCopyPolicy::SharedBounded"
        ),
        1
    );

    let declaration_marker = "pub(super) enum ArrayBufferSliceCopyPolicy {";
    let declaration_offset = BINARY_DATA_SOURCE
        .find(declaration_marker)
        .expect("ArrayBufferSliceCopyPolicy declaration");
    let preceding_item_end = BINARY_DATA_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("preceding ArrayBufferSliceBound impl");
    let following_item = BINARY_DATA_SOURCE[declaration_offset..]
        .find("/// Locals needed to re-observe")
        .map(|offset| declaration_offset + offset)
        .expect("following ArrayBufferSliceCopyLocals item");
    assert_eq!(
        normalize_rust(&BINARY_DATA_SOURCE[preceding_item_end + 1..following_item]).code,
        concat!(
            "pub(super)enumArrayBufferSliceCopyPolicy{",
            "DetachableBounded{target_data_local:u32,},",
            "SharedBounded{target_data_local:u32,},",
            "DetachableExactFinal{target_data_local:u32,",
            "target_object_local:u32,target_tag_local:u32,},}"
        ),
        "the payload authority must remain exact and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let sources = rust_sources(&source_root);
    let normalized = sources
        .iter()
        .map(|source| normalize_rust(source))
        .collect::<Vec<_>>();
    assert_eq!(
        normalized
            .iter()
            .map(|source| exact_identifier_count(&source.identifiers, "ArrayBufferSliceCopyPolicy"))
            .sum::<usize>(),
        31
    );
    for variant in ["DetachableBounded", "SharedBounded", "DetachableExactFinal"] {
        assert_eq!(
            normalized
                .iter()
                .map(|source| {
                    exact_identifier_count(
                        &source.routes,
                        &format!("ArrayBufferSliceCopyPolicy::{variant}"),
                    )
                })
                .sum::<usize>(),
            9,
            "variant `{variant}` must retain its exact producer and consumer census"
        );
    }
    let all_routes = normalized
        .iter()
        .map(|source| format!("{}\n", source.routes))
        .collect::<String>();
    assert_eq!(
        normalized
            .iter()
            .map(|source| {
                exact_identifier_count(&source.identifiers, "emit_array_buffer_slice_copy")
            })
            .sum::<usize>(),
        2,
        "one definition and one call must be the complete copy-writer route census"
    );
    assert_eq!(
        all_routes.matches(".emit_array_buffer_slice_copy(").count(),
        1
    );
    assert_eq!(
        all_routes.matches("::emit_array_buffer_slice_copy").count(),
        0
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!("impl{capability}forArrayBufferSliceCopyPolicy")));
    }
    for forbidden in [
        "ArrayBufferSliceCopyPolicyas",
        "typeArrayBufferSliceCopyPolicy",
        "matchpolicy{_=>",
        "match&policy{_=>",
        "matchcopy_policy{_=>",
        "match&copy_policy{_=>",
    ] {
        assert!(!all_routes.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn slice_kind_is_one_capability_free_borrowed_authority() {
    let declaration_marker = "enum ArrayBufferSliceKind {";
    let declaration_offset = STANDARD_SOURCE
        .find(declaration_marker)
        .expect("ArrayBufferSliceKind declaration");
    let preceding_item_end = STANDARD_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("preceding ActiveStandardBuiltinFunction impl");
    let following_item = STANDARD_SOURCE[declaration_offset..]
        .find("impl ArrayBufferSliceKind {")
        .map(|offset| declaration_offset + offset)
        .expect("ArrayBufferSliceKind implementation");
    assert_eq!(
        normalize_rust(&STANDARD_SOURCE[preceding_item_end + 1..following_item]).code,
        "enumArrayBufferSliceKind{Ordinary,Shared,ToImmutable,}",
        "the slice-kind authority must remain exact and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let normalized = rust_sources(&source_root)
        .iter()
        .map(|source| normalize_rust(source))
        .collect::<Vec<_>>();
    assert_eq!(
        normalized
            .iter()
            .map(|source| exact_identifier_count(&source.identifiers, "ArrayBufferSliceKind"))
            .sum::<usize>(),
        5,
        "the declaration, impl and three grouped producers must be the complete census"
    );
    let all_routes = normalized
        .iter()
        .map(|source| format!("{}\n", source.routes))
        .collect::<String>();
    for variant in ["Ordinary", "Shared", "ToImmutable"] {
        assert_eq!(
            all_routes
                .matches(&format!("ArrayBufferSliceKind::{variant}"))
                .count(),
            1,
            "slice kind `{variant}` must retain one producer"
        );
    }
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "Hash",
        "PartialOrd",
        "Ord",
    ] {
        assert!(!all_routes.contains(&format!("impl{capability}forArrayBufferSliceKind")));
    }
    for forbidden in [
        "ArrayBufferSliceKindas",
        "typeArrayBufferSliceKind",
        "ArrayBufferSliceKind::clone(",
    ] {
        assert!(!all_routes.contains(forbidden), "found `{forbidden}`");
    }

    let implementation = normalize_rust(bounded_inclusive(
        STANDARD_SOURCE,
        "impl ArrayBufferSliceKind {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(implementation.code.matches("&self").count(), 6);
    assert_eq!(implementation.code.matches("matchself{").count(), 6);
    assert!(!implementation.code.contains("matchself{_=>"));
    let pre_hardening_semantics = implementation.code.replace("&self", "self");
    assert_eq!(
        (
            pre_hardening_semantics.len(),
            fnv1a(&pre_hardening_semantics)
        ),
        (1179, 0x21c8_12f9_ad84_ac3e)
    );

    let owner = normalize_rust(bounded_inclusive(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayBufferPrototypeSlice\n",
        "            StandardBuiltinId::ArrayBufferPrototypeTransfer\n",
    ));
    assert_eq!(exact_identifier_count(&owner.identifiers, "slice_kind"), 7);
    for projection in [
        "copy_policy",
        "uses_species",
        "default_result_prototype",
        "default_result_brand",
        "default_result_flags",
        "rejects_immutable_species_result",
    ] {
        assert_eq!(
            owner
                .routes
                .matches(&format!("slice_kind.{projection}("))
                .count(),
            1,
            "slice-kind projection `{projection}` must have one consumer"
        );
    }
    for forbidden in ["slice_kind.clone(", "&mutslice_kind", "*slice_kind"] {
        assert!(!owner.routes.contains(forbidden), "found `{forbidden}`");
    }
    assert_eq!(
        (owner.code.len(), fnv1a(&owner.code)),
        (14341, 0xd07f_66f9_6448_5b66)
    );
}

#[test]
fn slice_kind_projects_exactly_three_copy_policies() {
    let projection = normalize_rust(bounded_inclusive(
        STANDARD_SOURCE,
        "    const fn copy_policy(",
        "    const fn uses_species(",
    ));
    assert_eq!(
        projection.code,
        concat!(
            "constfncopy_policy(&self,target_data_local:u32,target_object_local:u32,",
            "target_tag_local:u32,)->ArrayBufferSliceCopyPolicy{matchself{",
            "Self::Ordinary=>ArrayBufferSliceCopyPolicy::DetachableBounded{target_data_local},",
            "Self::Shared=>ArrayBufferSliceCopyPolicy::SharedBounded{target_data_local},",
            "Self::ToImmutable=>ArrayBufferSliceCopyPolicy::DetachableExactFinal{",
            "target_data_local,target_object_local,target_tag_local,},}}"
        )
    );
}

#[test]
fn slice_copy_locals_are_one_move_only_handoff() {
    let declaration_marker = "pub(super) struct ArrayBufferSliceCopyLocals {";
    let declaration_offset = BINARY_DATA_SOURCE
        .find(declaration_marker)
        .expect("ArrayBufferSliceCopyLocals declaration");
    let preceding_item_end = BINARY_DATA_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("preceding ArrayBufferSliceCopyPolicy declaration");
    let following_item = BINARY_DATA_SOURCE[declaration_offset..]
        .find("impl ArrayBufferSliceCopyLocals {")
        .map(|offset| declaration_offset + offset)
        .expect("ArrayBufferSliceCopyLocals constructor");
    assert_eq!(
        normalize_rust(&BINARY_DATA_SOURCE[preceding_item_end + 1..following_item]).code,
        concat!(
            "pub(super)structArrayBufferSliceCopyLocals{",
            "source_object_local:u32,source_start_local:u32,",
            "source_final_local:u32,requested_len_local:u32,}"
        ),
        "the source-local carrier must remain exact and attribute-free"
    );

    let constructor = normalize_rust(bounded_inclusive(
        BINARY_DATA_SOURCE,
        "impl ArrayBufferSliceCopyLocals {",
        "/// The immutable private slots needed to observe a TypedArray view.",
    ));
    assert_eq!(
        constructor.code,
        concat!(
            "implArrayBufferSliceCopyLocals{pub(super)constfnnew(",
            "source_object_local:u32,source_start_local:u32,source_final_local:u32,",
            "requested_len_local:u32,)->Self{Self{source_object_local,",
            "source_start_local,source_final_local,requested_len_local,}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let normalized = rust_sources(&source_root)
        .iter()
        .map(|source| normalize_rust(source))
        .collect::<Vec<_>>();
    assert_eq!(
        normalized
            .iter()
            .map(|source| {
                exact_identifier_count(&source.identifiers, "ArrayBufferSliceCopyLocals")
            })
            .sum::<usize>(),
        5,
        "the declaration, impl, import, producer and owned writer must be the complete census"
    );
    let all_routes = normalized
        .iter()
        .map(|source| format!("{}\n", source.routes))
        .collect::<String>();
    assert_eq!(
        all_routes
            .matches("ArrayBufferSliceCopyLocals::new(")
            .count(),
        1,
        "the grouped slice owner must remain the sole producer"
    );
    assert_eq!(
        all_routes
            .matches("locals:ArrayBufferSliceCopyLocals")
            .count(),
        1,
        "the copy writer must remain the sole owned handoff"
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "Hash",
        "PartialOrd",
        "Ord",
    ] {
        assert!(!all_routes.contains(&format!("impl{capability}forArrayBufferSliceCopyLocals")));
    }
    for forbidden in [
        "ArrayBufferSliceCopyLocalsas",
        "typeArrayBufferSliceCopyLocals",
        "&ArrayBufferSliceCopyLocals",
        "&mutArrayBufferSliceCopyLocals",
        "ArrayBufferSliceCopyLocals::clone(",
    ] {
        assert!(!all_routes.contains(forbidden), "found `{forbidden}`");
    }

    let writer = normalize_rust(bounded_inclusive(
        BINARY_DATA_SOURCE,
        "    pub(super) fn emit_array_buffer_slice_copy(",
        "    pub(crate) fn emit_initialize_typed_array_from_array_buffer(",
    ));
    assert_eq!(exact_identifier_count(&writer.identifiers, "locals"), 14);
    for (field, count) in [
        ("source_object_local", 4),
        ("source_start_local", 3),
        ("source_final_local", 1),
        ("requested_len_local", 5),
    ] {
        assert_eq!(
            writer.routes.matches(&format!("locals.{field}")).count(),
            count,
            "source-local projection `{field}` changed"
        );
    }
    for forbidden in ["&locals", "&mutlocals", "locals.clone(", "=locals"] {
        assert!(!writer.routes.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn grouped_slice_body_borrows_then_hands_off_the_policy_once() {
    let body = normalize_rust(bounded_inclusive(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayBufferPrototypeSlice\n",
        "            StandardBuiltinId::ArrayBufferPrototypeTransfer\n",
    ));
    assert_eq!(exact_identifier_count(&body.identifiers, "copy_policy"), 8);
    assert_eq!(body.routes.matches(".copy_policy(").count(), 1);
    assert_eq!(body.code.matches("match&copy_policy{").count(), 5);
    assert_eq!(body.code.matches("matchcopy_policy{").count(), 0);
    assert_eq!(
        body.routes
            .matches(".emit_array_buffer_slice_copy(")
            .count(),
        1
    );
    let handoff_route = "self.emit_array_buffer_slice_copy(copy_policy,";
    let handoff = body
        .code
        .find(handoff_route)
        .expect("owned copy-policy handoff");
    assert!(!body.code[handoff + handoff_route.len()..].contains("copy_policy"));
    assert_eq!(
        (body.code.len(), fnv1a(&body.code)),
        (14341, 0xd07f_66f9_6448_5b66)
    );
}

#[test]
fn copy_writer_borrows_twice_then_consumes_the_policy() {
    let writer = normalize_rust(bounded_inclusive(
        BINARY_DATA_SOURCE,
        "    pub(super) fn emit_array_buffer_slice_copy(",
        "    pub(crate) fn emit_initialize_typed_array_from_array_buffer(",
    ));
    assert!(writer.code.starts_with(concat!(
        "pub(super)fnemit_array_buffer_slice_copy(&mutself,",
        "policy:ArrayBufferSliceCopyPolicy,locals:ArrayBufferSliceCopyLocals,",
        "function:&mutFunction,)->Result<(),EmitError>{"
    )));
    assert_eq!(exact_identifier_count(&writer.identifiers, "policy"), 4);
    assert_eq!(writer.code.matches("match&policy{").count(), 2);
    assert_eq!(writer.code.matches("matchpolicy{").count(), 1);
    let first_borrow = writer.code.find("match&policy{").unwrap();
    let second_borrow = writer.code[first_borrow + 1..]
        .find("match&policy{")
        .map(|offset| first_borrow + 1 + offset)
        .unwrap();
    let consumption = writer.code.find("matchpolicy{").unwrap();
    assert!(first_borrow < second_borrow && second_borrow < consumption);
    assert!(!writer.code[consumption + "matchpolicy{".len()..].contains("policy"));
    assert!(writer.code.ends_with(concat!(
        "self.release_temp_local(source_data_local);",
        "self.release_temp_local(copy_len_local);",
        "self.release_temp_local(available_local);",
        "self.release_temp_local(source_byte_length_local);",
        "self.release_temp_local(source_flags_local);Ok(())}"
    )));
    assert_eq!(
        (writer.code.len(), fnv1a(&writer.code)),
        (7153, 0x3229_1bb0_8809_c608)
    );
}

#[test]
fn contract_and_t17_record_the_single_handoff_boundary() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "non-`Clone`, non-`Copy`",
        "31 lexical mentions",
        "five borrowed pre-handoff decisions",
        "two borrowed writer decisions",
        "final consuming source-selection decision",
        "five production mentions",
        "thirteen field projections",
        "single move-only carrier",
        "five production type mentions",
        "six borrowed projections",
        "single capability-free slice-kind authority",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T17 marker: {marker}");
    }
}
