use std::fs;
use std::path::Path;

const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");

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

fn exact_route_count(source: &str, route: &str) -> usize {
    source
        .match_indices(route)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + route.len()..].chars().next();
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
            exact_route_count(&normalize_rust(&source).routes, route)
        })
        .sum()
}

fn integer_operation_consumer() -> &'static str {
    bounded(
        ATOMICS_SOURCE,
        "    fn emit_atomics_integer_operation(",
        "    fn emit_atomics_rmw_integer_element_to_i64(",
    )
}

#[test]
fn atomics_integer_operation_is_one_private_capability_free_authority() {
    let lexical_probe = r###"
        AtomicsIntegerOperation /* nested /* ignored route */ comment */ :: r#Store;
        // AtomicsIntegerOperation::Load
        let normal = "AtomicsIntegerOperation::Add";
        let byte = b"AtomicsIntegerOperation::And";
        let c_string = c"AtomicsIntegerOperation::CompareExchange";
        let raw = r#"AtomicsIntegerOperation::Exchange"#;
        let raw_byte = br#"AtomicsIntegerOperation::Or"#;
        let raw_c = cr#"AtomicsIntegerOperation::Sub"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "AtomicsIntegerOperation::Store;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;",
            "letraw_byte=L;letraw_c=L;letcharacter=L;letbyte_character=L;",
            "letborrowed:&'astr=value;"
        )
    );
    assert!(normalized_probe
        .code
        .contains("letraw_c=cr#\"AtomicsIntegerOperation::Sub\"#;"));
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "AtomicsIntegerOperation"),
        1
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "Store"),
        1
    );
    assert_eq!(
        exact_route_count(&normalized_probe.routes, "AtomicsIntegerOperation::Store"),
        1
    );

    let declaration_start = ATOMICS_SOURCE
        .find("enum AtomicsIntegerOperation {")
        .expect("integer-operation declaration");
    assert_eq!(
        ATOMICS_SOURCE[..declaration_start]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("];")
    );
    let declaration = bounded(
        ATOMICS_SOURCE,
        "enum AtomicsIntegerOperation {",
        "\n}\n\nimpl AtomicsIntegerOperation {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "Load,",
            "Add,",
            "And,",
            "CompareExchange,",
            "Exchange,",
            "Or,",
            "Store,",
            "Sub,",
            "Xor,",
        ]
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!ATOMICS_SOURCE.contains(&format!("impl {capability} for AtomicsIntegerOperation")));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "AtomicsIntegerOperation"),
        48,
        "declaration, impl, consumer, nine producers and four nine-row policy tables own every mention"
    );
    let normalized_source = normalize_rust(ATOMICS_SOURCE).code;
    for (variant, method) in [
        ("Load", "load"),
        ("Add", "add"),
        ("And", "and"),
        ("CompareExchange", "compare_exchange"),
        ("Exchange", "exchange"),
        ("Or", "or"),
        ("Store", "store"),
        ("Sub", "sub"),
        ("Xor", "xor"),
    ] {
        assert_eq!(
            count_route_in_rust_sources(
                &source_root,
                &format!("AtomicsIntegerOperation::{variant}"),
            ),
            5,
            "{variant} must occur once as a producer and once in each fully qualified table"
        );
        let producer = format!(
            "fnemit_atomics_{method}(&mutself,function:&mutFunction)->Result<(),EmitError>{{self.emit_atomics_integer_operation(AtomicsIntegerOperation::{variant},function)}}"
        );
        assert_eq!(
            normalized_source.matches(&producer).count(),
            1,
            "{variant} must have exactly one direct builtin producer"
        );
    }
}

#[test]
fn atomics_integer_operation_borrows_exact_arity_and_diagnostic_tables() {
    let authority = bounded(
        ATOMICS_SOURCE,
        "impl AtomicsIntegerOperation {",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum AtomicsRmwOperation",
    );
    assert_eq!(
        normalize_rust(authority).code,
        concat!(
            "fnvalue_arg_count(&self)->u8{matchself{",
            "Self::Load=>0,Self::CompareExchange=>2,",
            "Self::Add|Self::And|Self::Exchange|Self::Or|Self::Store|Self::Sub|Self::Xor=>1,",
            "}}}"
        )
    );

    let diagnostic_tables = bounded(
        integer_operation_consumer(),
        "&mut self,",
        "        let typed_array_payload_local = self.reserve_temp_local();",
    );
    assert_eq!(
        normalize_rust(diagnostic_tables).code,
        concat!(
            "operation:AtomicsIntegerOperation,function:&mutFunction,)->Result<(),EmitError>{",
            "lettype_error_message=match&operation{",
            "AtomicsIntegerOperation::Add=>\"Atomics.add requires an integer typed array\",",
            "AtomicsIntegerOperation::And=>\"Atomics.and requires an integer typed array\",",
            "AtomicsIntegerOperation::CompareExchange=>{\"Atomics.compareExchange requires an integer typed array\"}",
            "AtomicsIntegerOperation::Exchange=>\"Atomics.exchange requires an integer typed array\",",
            "AtomicsIntegerOperation::Load=>\"Atomics.load requires an integer typed array\",",
            "AtomicsIntegerOperation::Or=>\"Atomics.or requires an integer typed array\",",
            "AtomicsIntegerOperation::Store=>\"Atomics.store requires an integer typed array\",",
            "AtomicsIntegerOperation::Sub=>\"Atomics.sub requires an integer typed array\",",
            "AtomicsIntegerOperation::Xor=>\"Atomics.xor requires an integer typed array\",",
            "};",
            "letrange_error_message=match&operation{",
            "AtomicsIntegerOperation::Add=>\"Atomics.add index out of range\",",
            "AtomicsIntegerOperation::And=>\"Atomics.and index out of range\",",
            "AtomicsIntegerOperation::CompareExchange=>{\"Atomics.compareExchange index out of range\"}",
            "AtomicsIntegerOperation::Exchange=>\"Atomics.exchange index out of range\",",
            "AtomicsIntegerOperation::Load=>\"Atomics.load index out of range\",",
            "AtomicsIntegerOperation::Or=>\"Atomics.or index out of range\",",
            "AtomicsIntegerOperation::Store=>\"Atomics.store index out of range\",",
            "AtomicsIntegerOperation::Sub=>\"Atomics.sub index out of range\",",
            "AtomicsIntegerOperation::Xor=>\"Atomics.xor index out of range\",",
            "};letvalue_arg_count=operation.value_arg_count();"
        )
    );
}

#[test]
fn atomics_integer_operation_exhausts_emission_and_result_publication() {
    let consumer = integer_operation_consumer();
    let normalized_consumer = normalize_rust(consumer);
    assert_eq!(
        normalized_consumer
            .routes
            .matches("match&operation{")
            .count(),
        4
    );
    for forbidden in [
        "matchoperation{",
        "operation!=AtomicsIntegerOperation::Store",
        "operation==AtomicsIntegerOperation::Store",
        "matches!(operation",
        "_=>",
    ] {
        assert!(
            !normalized_consumer.routes.contains(forbidden),
            "forbidden operation-policy escape `{forbidden}`"
        );
    }

    let address_start = consumer
        .rfind("        function.instruction(&Instruction::LocalGet(data_ptr_local));")
        .expect("integer-operation address calculation");
    let emission_end = consumer
        .rfind("\n\n        Ok(())")
        .expect("integer-operation return after local release");
    let emission_and_release = &consumer[address_start..emission_end];
    assert_eq!(
        normalize_rust(emission_and_release).code,
        concat!(
            "function.instruction(&Instruction::LocalGet(data_ptr_local));",
            "function.instruction(&Instruction::LocalGet(byte_offset_local));",
            "function.instruction(&Instruction::I64Add);",
            "function.instruction(&Instruction::LocalGet(index_local));",
            "function.instruction(&Instruction::LocalGet(bytes_per_element_local));",
            "function.instruction(&Instruction::I64Mul);",
            "function.instruction(&Instruction::I64Add);",
            "function.instruction(&Instruction::LocalSet(address_local));",
            "match&operation{",
            "AtomicsIntegerOperation::Store=>{",
            "self.emit_atomics_store_integer_element_from_i64(address_local,&element_kind,value_raw_local,function,);",
            "self.emit_validated_atomics_bigint_element_kind_i32(&element_kind,function);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "function.instruction(&Instruction::LocalGet(value_bigint_payload_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::LocalGet(value_bigint_tag_local));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(value_payload_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::Number.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::End);}",
            "AtomicsIntegerOperation::Load=>{",
            "self.emit_atomics_load_integer_element_to_i64(address_local,&element_kind,old_raw_local,function,);}",
            "AtomicsIntegerOperation::CompareExchange=>{",
            "self.emit_atomics_normalize_integer_element_i64(value_raw_local,&element_kind,function,);",
            "function.instruction(&Instruction::LocalSet(value_raw_local));",
            "self.emit_atomics_compare_exchange_integer_element_to_i64(address_local,&element_kind,value_raw_local,replacement_raw_local,old_raw_local,function,);}",
            "AtomicsIntegerOperation::Add=>{",
            "self.emit_atomics_rmw_integer_element_to_i64(address_local,&element_kind,value_raw_local,AtomicsRmwOperation::Add,old_raw_local,function,);}",
            "AtomicsIntegerOperation::And=>self.emit_atomics_rmw_integer_element_to_i64(",
            "address_local,&element_kind,value_raw_local,AtomicsRmwOperation::And,old_raw_local,function,),",
            "AtomicsIntegerOperation::Exchange=>self.emit_atomics_rmw_integer_element_to_i64(",
            "address_local,&element_kind,value_raw_local,AtomicsRmwOperation::Exchange,old_raw_local,function,),",
            "AtomicsIntegerOperation::Or=>self.emit_atomics_rmw_integer_element_to_i64(",
            "address_local,&element_kind,value_raw_local,AtomicsRmwOperation::Or,old_raw_local,function,),",
            "AtomicsIntegerOperation::Sub=>self.emit_atomics_rmw_integer_element_to_i64(",
            "address_local,&element_kind,value_raw_local,AtomicsRmwOperation::Sub,old_raw_local,function,),",
            "AtomicsIntegerOperation::Xor=>self.emit_atomics_rmw_integer_element_to_i64(",
            "address_local,&element_kind,value_raw_local,AtomicsRmwOperation::Xor,old_raw_local,function,),}",
            "match&operation{",
            "AtomicsIntegerOperation::Store=>{}",
            "AtomicsIntegerOperation::Load|AtomicsIntegerOperation::Add|AtomicsIntegerOperation::And|",
            "AtomicsIntegerOperation::CompareExchange|AtomicsIntegerOperation::Exchange|",
            "AtomicsIntegerOperation::Or|AtomicsIntegerOperation::Sub|AtomicsIntegerOperation::Xor=>{",
            "self.emit_validated_atomics_bigint_element_kind_i32(&element_kind,function);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "function.instruction(&Instruction::LocalGet(element_kind.local()));",
            "function.instruction(&Instruction::I64Const(11));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::LocalGet(old_raw_local));",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64LtS);",
            "function.instruction(&Instruction::I32And);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_alloc_one_limb_bigint(1,old_raw_local,function)?;",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(old_raw_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::Else);",
            "self.emit_atomics_signed_number_element_kind_i32(&element_kind,function);",
            "function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));",
            "function.instruction(&Instruction::LocalGet(old_raw_local));",
            "function.instruction(&Instruction::F64ConvertI64S);",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(old_raw_local));",
            "function.instruction(&Instruction::F64ConvertI64U);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::I64ReinterpretF64);",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::Number.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::End);",
            "}}",
            "self.release_temp_local(value_bigint_tag_local);",
            "self.release_temp_local(value_bigint_payload_local);",
            "self.release_temp_local(replacement_raw_local);",
            "self.release_temp_local(value_raw_local);",
            "self.release_temp_local(old_raw_local);",
            "self.release_temp_local(address_local);",
            "self.release_temp_local(index_local);",
            "self.release_temp_local(element_kind.into_local());",
            "self.release_temp_local(element_length_local);",
            "self.release_temp_local(bytes_per_element_local);",
            "self.release_temp_local(stored_byte_length_local);",
            "self.release_temp_local(byte_offset_local);",
            "self.release_temp_local(data_ptr_local);",
            "self.release_temp_local(buffer_tag_local);",
            "self.release_temp_local(buffer_payload_local);",
            "self.release_temp_local(typed_array_brand_local);",
            "self.release_temp_local(replacement_tag_local);",
            "self.release_temp_local(replacement_payload_local);",
            "self.release_temp_local(value_tag_local);",
            "self.release_temp_local(value_payload_local);",
            "self.release_temp_local(index_tag_local);",
            "self.release_temp_local(index_payload_local);",
            "self.release_temp_local(typed_array_tag_local);",
            "self.release_temp_local(typed_array_payload_local);"
        )
    );
}
