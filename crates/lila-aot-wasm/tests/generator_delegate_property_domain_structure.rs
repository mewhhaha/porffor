use std::fs;
use std::path::Path;

const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");

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

fn exact_count(source: &str, needle: &str) -> usize {
    source
        .match_indices(needle)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + needle.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn count_in_rust_sources(dir: &Path, needle: &str, identifiers: bool) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle, identifiers);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let normalized = normalize_rust(&source);
            exact_count(
                if identifiers {
                    &normalized.identifiers
                } else {
                    &normalized.routes
                },
                needle,
            )
        })
        .sum()
}

#[test]
fn delegate_property_domains_are_private_capability_free_and_single_owned() {
    let probe = normalize_rust(
        r###"
        GeneratorDelegateProperty /* nested /* ignored */ comment */ :: r#Return;
        GeneratorDelegatePropertyKey::r#OrdinaryString;
        // GeneratorDelegateProperty::Throw
        let normal = "GeneratorDelegateProperty::Done";
        let byte = b"GeneratorDelegatePropertyKey::WellKnownSymbol";
        let c_string = c"GeneratorDelegateProperty";
        let raw = r#"GeneratorDelegatePropertyKey"#;
        let raw_byte = br#"GeneratorDelegateProperty"#;
        let raw_c = cr#"GeneratorDelegatePropertyKey"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###,
    );
    assert_eq!(
        probe.routes,
        concat!(
            "GeneratorDelegateProperty::Return;GeneratorDelegatePropertyKey::OrdinaryString;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;letraw_c=L;",
            "letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_count(&probe.identifiers, "GeneratorDelegateProperty"),
        1
    );
    assert_eq!(
        exact_count(&probe.identifiers, "GeneratorDelegatePropertyKey"),
        1
    );

    let declarations = normalize_rust(bounded(
        DELEGATION_SOURCE,
        "use super::*;",
        "impl GeneratorDelegateProperty {",
    ));
    assert_eq!(declarations.code, concat!("pub(crate)enumAsyncGeneratorDelegationKind{YieldStar,ForAwaitYield,}", "enumGeneratorDelegateProperty{AsyncIterator,Iterator,Next,Return,Throw,Done,Value,}", "enumGeneratorDelegatePropertyKey{WellKnownSymbol(&'staticstr),OrdinaryString(&'staticstr),}"));

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&root, "GeneratorDelegateProperty", true),
        24
    );
    assert_eq!(
        count_in_rust_sources(&root, "GeneratorDelegatePropertyKey", true),
        11
    );
    for (variant, expected) in [
        ("AsyncIterator", 2),
        ("Iterator", 3),
        ("Next", 3),
        ("Return", 4),
        ("Throw", 3),
        ("Done", 3),
        ("Value", 3),
    ] {
        assert_eq!(
            count_in_rust_sources(
                &root,
                &format!("GeneratorDelegateProperty::{variant}"),
                false
            ),
            expected,
            "{variant}"
        );
    }
    for (variant, expected) in [("WellKnownSymbol", 3), ("OrdinaryString", 6)] {
        assert_eq!(
            count_in_rust_sources(
                &root,
                &format!("GeneratorDelegatePropertyKey::{variant}"),
                false
            ),
            expected,
            "{variant}"
        );
    }
}

#[test]
fn delegate_property_projection_and_producers_are_exact_and_ordered() {
    let projection = normalize_rust(bounded(
        DELEGATION_SOURCE,
        "impl GeneratorDelegateProperty {",
        "const ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW",
    ));
    assert_eq!(projection.code, concat!("fnkey(&self)->GeneratorDelegatePropertyKey{matchself{", "GeneratorDelegateProperty::AsyncIterator=>{GeneratorDelegatePropertyKey::WellKnownSymbol(\"Symbol.asyncIterator\")}", "GeneratorDelegateProperty::Iterator=>{GeneratorDelegatePropertyKey::WellKnownSymbol(\"Symbol.iterator\")}", "GeneratorDelegateProperty::Next=>GeneratorDelegatePropertyKey::OrdinaryString(\"next\"),", "GeneratorDelegateProperty::Return=>{GeneratorDelegatePropertyKey::OrdinaryString(\"return\")}", "GeneratorDelegateProperty::Throw=>{GeneratorDelegatePropertyKey::OrdinaryString(\"throw\")}", "GeneratorDelegateProperty::Done=>GeneratorDelegatePropertyKey::OrdinaryString(\"done\"),", "GeneratorDelegateProperty::Value=>{GeneratorDelegatePropertyKey::OrdinaryString(\"value\")}", "}}}"));

    let callers = normalize_rust(bounded(
        DELEGATION_SOURCE,
        "pub(crate) fn compile_async_generator_delegation(",
        "fn emit_generator_delegate_property_read(",
    ));
    let calls = [
        (
            "value_payload_local",
            "value_tag_local",
            "AsyncIterator",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "value_payload_local",
            "value_tag_local",
            "Iterator",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Next",
            "next_payload_local",
            "next_tag_local",
        ),
        (
            "argument_payload_local",
            "argument_tag_local",
            "Done",
            "done_payload_local",
            "done_tag_local",
        ),
        (
            "argument_payload_local",
            "argument_tag_local",
            "Value",
            "value_payload_local",
            "value_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Throw",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Return",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Return",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "value_payload_local",
            "value_tag_local",
            "Iterator",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Next",
            "next_payload_local",
            "next_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Throw",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "iterator_payload_local",
            "iterator_tag_local",
            "Return",
            "method_payload_local",
            "method_tag_local",
        ),
        (
            "result_payload_local",
            "result_tag_local",
            "Done",
            "done_payload_local",
            "done_tag_local",
        ),
        (
            "result_payload_local",
            "result_tag_local",
            "Value",
            "value_payload_local",
            "value_tag_local",
        ),
    ];
    assert_eq!(
        callers
            .code
            .matches("self.emit_generator_delegate_property_read(")
            .count(),
        calls.len()
    );
    let mut remaining = callers.code.as_str();
    for (target_payload, target_tag, variant, value_payload, value_tag) in calls {
        let expected = format!("self.emit_generator_delegate_property_read({target_payload},{target_tag},GeneratorDelegateProperty::{variant},{value_payload},{value_tag},function,)?;");
        let offset = remaining
            .find(&expected)
            .unwrap_or_else(|| panic!("missing ordered producer `{expected}`"));
        remaining = &remaining[offset + expected.len()..];
    }
}

#[test]
fn delegate_property_reader_exhausts_the_exact_key_bodies() {
    let reader = normalize_rust(bounded(
        DELEGATION_SOURCE,
        "fn emit_generator_delegate_property_read(",
        "fn emit_generator_delegate_method_is_missing_i32(",
    ));
    assert_eq!(reader.code, concat!(
        "&mutself,target_payload_local:u32,target_tag_local:u32,property:GeneratorDelegateProperty,value_payload_local:u32,value_tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{matchproperty.key(){",
        "GeneratorDelegatePropertyKey::WellKnownSymbol(key)=>{lettarget=TypedExpr::from_info(ValueInfo{kind:ValueKind::Dynamic,possible_kinds:KindSet::all_runtime_tags().without(ValueKind::Undefined).without(ValueKind::Null),heap_shape:None,function_targets:FunctionTargetKnowledge::unknown(),},ExprIr::Undefined,);",
        "letsymbol_key=TypedExpr::from_info(ValueInfo::new(ValueKind::Symbol),ExprIr::String(key.to_string()),);self.compile_property_read_from_locals(&target,&PropertyKeyIr::StringExpr(Box::new(symbol_key)),target_payload_local,target_tag_local,value_payload_local,value_tag_local,function,)?;self.emit_propagate_throw_from_locals_if_needed(value_payload_local,value_tag_local,function,)}",
        "GeneratorDelegatePropertyKey::OrdinaryString(key)=>{letkey_local=self.reserve_temp_local();function.instruction(&Instruction::I64Const(self.strings.payload(key)));function.instruction(&Instruction::LocalSet(key_local));self.emit_object_read_without_throw_propagation(target_payload_local,target_tag_local,target_payload_local,target_tag_local,key_local,value_payload_local,value_tag_local,function,)?;self.release_temp_local(key_local);self.emit_propagate_throw_from_locals_if_needed(value_payload_local,value_tag_local,function,)}}}"));

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(reader.routes.matches(".key").count(), 1);
    assert_eq!(
        count_in_rust_sources(&root, "GeneratorDelegateProperty::key", false),
        0
    );
}
