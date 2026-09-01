use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-at-receiver-policy.md");
const T16: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");
const T17: &str = include_str!("../../../tasks/17-typedarrays-binary-data-atomics.md");

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
fn array_at_receiver_policy_is_one_capability_free_two_row_authority() {
    let lexical_probe = r###"
        ArrayAtReceiverPolicy /* nested /* ignored */ comment */ :: r#TypedArray;
        // ArrayAtReceiverPolicy::GenericArrayLike
        let normal = "ArrayAtReceiverPolicy::TypedArray";
        let byte = b"ArrayAtReceiverPolicy::TypedArray";
        let c_string = c"ArrayAtReceiverPolicy::TypedArray";
        let raw = r#"ArrayAtReceiverPolicy::TypedArray"#;
        let raw_byte = br#"ArrayAtReceiverPolicy::TypedArray"#;
        let raw_c = cr#"ArrayAtReceiverPolicy::TypedArray"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "ArrayAtReceiverPolicy"),
        1
    );
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "ArrayAtReceiverPolicy::TypedArray;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );

    let normalized = normalize_rust(ARRAY_SOURCE);
    let declaration = bounded(
        &normalized.code,
        "enumTypedArraySearchKind{Includes,IndexOf,LastIndexOf,}",
        "impl<'a>FunctionBuilder<'a>{",
    );
    assert_eq!(
        declaration,
        "enumArrayAtReceiverPolicy{GenericArrayLike,TypedArray,}"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ArrayAtReceiverPolicy"),
        13
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ArrayAtReceiverPolicy::GenericArrayLike"),
        5
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ArrayAtReceiverPolicy::TypedArray"),
        5
    );
    assert_eq!(
        exact_identifier_count(&normalized.identifiers, "receiver_policy"),
        7
    );
    assert_eq!(
        exact_identifier_count(&normalized.identifiers, "validates_typed_array"),
        0
    );
    assert_eq!(
        exact_identifier_count(&normalized.identifiers, "validate_typed_array"),
        0
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !normalized
                .routes
                .contains(&format!("impl{capability}forArrayAtReceiverPolicy")),
            "ArrayAtReceiverPolicy must not acquire {capability}"
        );
    }
}

#[test]
fn two_fixed_entries_own_the_policy_and_direct_at_selects_the_typed_entry() {
    let standard = normalize_rust(STANDARD_SOURCE);
    let standard_rows = bounded(
        &standard.code,
        "StandardBuiltinId::ArrayPrototypeAt=>{",
        "StandardBuiltinId::ArrayPrototypeToReversed=>{",
    );
    assert_eq!(
        standard_rows,
        concat!(
            "self.compile_array_prototype_at_builtin(function)?;}",
            "StandardBuiltinId::TypedArrayPrototypeAt=>{",
            "self.compile_typed_array_prototype_at_builtin(function)?;}"
        )
    );
    assert!(!standard.identifiers.contains("ArrayAtReceiverPolicy"));

    let array = normalize_rust(ARRAY_SOURCE);
    let fixed_entries = bounded(
        &array.code,
        "pub(super)fncompile_array_prototype_at_builtin(",
        "fncompile_array_like_at_builtin(",
    );
    assert_eq!(
        fixed_entries
            .matches("self.compile_array_like_at_builtin(")
            .count(),
        2
    );
    assert_eq!(
        fixed_entries
            .matches("ArrayAtReceiverPolicy::GenericArrayLike")
            .count(),
        1
    );
    assert_eq!(
        fixed_entries
            .matches("ArrayAtReceiverPolicy::TypedArray")
            .count(),
        1
    );

    let raw_compiler = bounded(
        &array.code,
        "fncompile_array_like_at_builtin(",
        "pub(crate)fncompile_array_prototype_to_reversed_builtin(",
    );
    assert!(raw_compiler.starts_with(
        "&mutself,receiver_policy:ArrayAtReceiverPolicy,function:&mutFunction,)->Result<(),EmitError>{"
    ));
    assert_eq!(
        raw_compiler
            .matches(concat!(
                "self.emit_array_at_from_locals(receiver_payload_local,receiver_tag_local,",
                "index_payload_local,index_tag_local,self.result_local,self.result_tag_local,",
                "receiver_policy,function,)?;"
            ))
            .count(),
        1
    );

    let functions = normalize_rust(FUNCTIONS_SOURCE);
    let direct = bounded(
        &functions.code,
        "ifmatches!(key,PropertyKeyIr::StaticString(name)ifname==\"at\"){",
        "ifmatches!(key,PropertyKeyIr::StaticString(name)ifname==\"includes\"){",
    );
    assert_eq!(
        direct,
        concat!(
            "returnself.emit_array_direct_builtin_method_call(",
            "StandardBuiltinId::TypedArrayPrototypeAt,\"TypedArray.prototype.at\",",
            "receiver,args,payload_local,tag_local,function,);}"
        )
    );
    assert_eq!(
        exact_identifier_count(&array.identifiers, "emit_array_at_from_locals"),
        2
    );
}

#[test]
fn contract_and_tasks_record_the_private_source_equivalent_boundary() {
    for evidence in [CONTRACT, T16, T17] {
        assert!(evidence.contains("private `ArrayAtReceiverPolicy`"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("claims no new Array"));
        assert!(evidence.contains("TypedArray behavior"));
    }
}

#[test]
fn all_four_receiver_decisions_borrow_and_exhaustively_project_the_policy() {
    let array = normalize_rust(ARRAY_SOURCE);
    let consumer = bounded(
        &array.code,
        "#[allow(clippy::too_many_arguments)]fnemit_array_at_from_locals(",
        "#[allow(clippy::too_many_arguments)]pub(crate)fnemit_array_includes_from_locals(",
    );
    assert!(consumer.starts_with(concat!(
        "&mutself,receiver_payload_local:u32,receiver_tag_local:u32,index_payload_local:u32,",
        "index_tag_local:u32,payload_local:u32,tag_local:u32,",
        "receiver_policy:ArrayAtReceiverPolicy,function:&mutFunction,)->Result<(),EmitError>{"
    )));

    let array_or_arguments = concat!(
        "match&receiver_policy{",
        "ArrayAtReceiverPolicy::GenericArrayLike=>{",
        "self.load_i64_to_local_from_offset(receiver_payload_local,HEAP_LEN_OFFSET,len_local,function,);}",
        "ArrayAtReceiverPolicy::TypedArray=>{",
        "self.emit_throw_current_function_realm_type_error(",
        "\"TypedArray.prototype.at called on incompatible receiver\",",
        "payload_local,tag_local,function,)?;self.emit_return_current_completion(function);}}"
    );
    let typed_array_witness = concat!(
        "letwitness_use=match&receiver_policy{",
        "ArrayAtReceiverPolicy::GenericArrayLike=>{",
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot{length_local:len_local,}}",
        "ArrayAtReceiverPolicy::TypedArray=>TypedArrayWitnessUse::ValidatedMethodEntry{",
        "length_local:len_local,},};",
        "self.emit_typed_array_witness(&typed_view,witness_use,function)?;"
    );
    let ordinary_object = concat!(
        "match&receiver_policy{",
        "ArrayAtReceiverPolicy::GenericArrayLike=>{",
        "function.instruction(&Instruction::I64Const(self.strings.payload(\"length\")));",
        "function.instruction(&Instruction::LocalSet(key_local));",
        "self.emit_object_read(receiver_payload_local,receiver_tag_local,receiver_payload_local,",
        "receiver_tag_local,key_local,length_payload_local,length_tag_local,function,)?;",
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_to_length_i64_from_value_locals(length_tag_local,length_payload_local,",
        "len_local,function,)?;}",
        "ArrayAtReceiverPolicy::TypedArray=>{",
        "self.emit_throw_current_function_realm_type_error(",
        "\"TypedArray.prototype.at called on incompatible receiver\",",
        "payload_local,tag_local,function,)?;self.emit_return_current_completion(function);}}"
    );
    let primitive_or_nullish = concat!(
        "match&receiver_policy{",
        "ArrayAtReceiverPolicy::GenericArrayLike=>{",
        "self.emit_array_iteration_nullish_receiver_throw_or_zero_length(",
        "receiver_tag_local,len_local,payload_local,tag_local,",
        "\"Array.prototype.at called on null or undefined\",function,)?;}",
        "ArrayAtReceiverPolicy::TypedArray=>{",
        "self.emit_throw_current_function_realm_type_error(",
        "\"TypedArray.prototype.at called on incompatible receiver\",",
        "payload_local,tag_local,function,)?;self.emit_return_current_completion(function);}}"
    );

    assert_eq!(consumer.matches("match&receiver_policy{").count(), 4);
    for (name, expected) in [
        ("Array/Arguments", array_or_arguments),
        ("TypedArray witness", typed_array_witness),
        ("ordinary Object/Function", ordinary_object),
        ("primitive/nullish", primitive_or_nullish),
    ] {
        assert_eq!(
            consumer.matches(expected).count(),
            1,
            "missing exact {name} policy body"
        );
    }
    let array_or_arguments_position = consumer.find(array_or_arguments).unwrap();
    let witness_position = consumer.find(typed_array_witness).unwrap();
    let ordinary_object_position = consumer.find(ordinary_object).unwrap();
    let primitive_position = consumer.find(primitive_or_nullish).unwrap();
    let index_conversion_position = consumer
        .find("self.emit_value_to_number_payload(index_tag_local,index_payload_local,function)?;")
        .unwrap();
    assert!(array_or_arguments_position < witness_position);
    assert!(witness_position < ordinary_object_position);
    assert!(ordinary_object_position < primitive_position);
    assert!(primitive_position < index_conversion_position);

    for forbidden in [
        "validates_typed_array",
        "validate_typed_array",
        "receiver_policy==",
        "receiver_policy!=",
        "matches!(receiver_policy",
        "ifreceiver_policy",
        "_=>",
        "unreachable!",
        "asbool",
    ] {
        assert!(
            !consumer.contains(forbidden),
            "forbidden policy route `{forbidden}`"
        );
    }
}
