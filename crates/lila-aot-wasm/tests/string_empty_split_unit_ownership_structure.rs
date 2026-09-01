const STRING: &str = include_str!("../src/builtins/string.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/string-empty-split-code-unit-walk.md");
const TASK: &str = include_str!("../../../tasks/18-strings-unicode.md");

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

fn lexically_normalized(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push_str(&source[offset..end]);
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
            assert_eq!(depth, 0, "unterminated block comment in String emitter");
            continue;
        }
        let character = source[offset..]
            .chars()
            .next()
            .expect("valid UTF-8 character boundary");
        if !character.is_whitespace() {
            normalized.push(character);
        }
        offset += character.len_utf8();
    }
    normalized
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

fn coordinator() -> String {
    let source = lexically_normalized(STRING);
    bounded(
        &source,
        "modempty_string_split_units{",
        "modstring_code_unit_access{",
    )
    .to_string()
}

#[test]
fn empty_split_locals_are_the_exact_private_no_capability_domains() {
    let coordinator = coordinator();
    assert!(coordinator.starts_with(
        "usesuper::*;structUnitIndexLocal(u32);structUnitLengthLocal(u32);structOneUnitLocal(u32);"
    ));
    assert!(!coordinator.contains("derive("));
    assert!(!coordinator.contains("pubstruct"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        for local in ["UnitIndexLocal", "UnitLengthLocal", "OneUnitLocal"] {
            assert!(!coordinator.contains(&format!("impl{capability}for{local}")));
        }
    }
    assert_eq!(coordinator.matches("UnitIndexLocal").count(), 3);
    assert_eq!(coordinator.matches("UnitLengthLocal").count(), 2);
    assert_eq!(coordinator.matches("OneUnitLocal").count(), 3);
}

#[test]
fn one_unit_materializer_borrows_the_owned_index_and_width() {
    let coordinator = coordinator();
    let materializer = bounded(
        &coordinator,
        "fnemit_one_unit_payload(",
        "#[allow(clippy::too_many_arguments)]",
    );
    assert!(materializer.starts_with(
        "builder:&mutFunctionBuilder<'_>,string_local:u32,index:&UnitIndexLocal,one:&OneUnitLocal,function:&mutFunction,)->Result<(),EmitError>{"
    ));
    assert_eq!(
        materializer
            .matches("emit_utf16_code_unit_range_payload_from_locals(")
            .count(),
        1
    );
    assert!(materializer.contains("string_local,index.0,one.0,function,"));
    assert!(!materializer.contains("UnitLengthLocal"));
    assert!(!materializer.contains("emit_string_slice_payload_from_locals("));
    assert!(!materializer.contains("emit_decode_utf8_scalar_at_index("));
}

#[test]
fn loop_owner_constructs_borrows_reuses_and_releases_each_local() {
    let coordinator = coordinator();
    for construction in [
        "letindex=UnitIndexLocal(builder.reserve_temp_local());",
        "letlength=UnitLengthLocal(builder.reserve_temp_local());",
        "letone=OneUnitLocal(builder.reserve_temp_local());",
    ] {
        assert_eq!(
            coordinator.matches(construction).count(),
            1,
            "`{construction}`"
        );
    }
    let materialize = coordinator
        .find("emit_one_unit_payload(builder,string_local,&index,&one,function)?;")
        .expect("borrowed one-unit materialization");
    let advance = coordinator[materialize..]
        .find("forlocalin[write_index_local,index.0]")
        .expect("post-materialization index advancement")
        + materialize;
    let release_one = coordinator[advance..]
        .find("builder.release_temp_local(one.0);")
        .expect("one-unit local release")
        + advance;
    let release_length = coordinator[release_one..]
        .find("builder.release_temp_local(length.0);")
        .expect("length local release")
        + release_one;
    let release_index = coordinator[release_length..]
        .find("builder.release_temp_local(index.0);")
        .expect("index local release")
        + release_length;
    assert!(materialize < advance);
    assert!(advance < release_one);
    assert!(release_one < release_length);
    assert!(release_length < release_index);
    assert_eq!(
        coordinator
            .matches("emit_one_unit_payload(builder,string_local,&index,&one,function)?;")
            .count(),
        1
    );
}

#[test]
fn contract_and_task_record_the_ownership_boundary_without_a_runtime_claim() {
    for marker in [
        "None of the three local domains implements `Clone` or `Copy`.",
        "`emit_one_unit_payload` borrows the index and one-unit width",
        "source-equivalent and the existing empty-separator fixture was not rerun",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
    }
    for marker in [
        "private index, length and one-unit local domains now derive",
        "no incidental capabilities",
        "source-equivalent ownership closure",
        "no runtime or conformance claim",
    ] {
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
