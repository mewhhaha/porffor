use std::fs;
use std::path::Path;

const TEMPORAL_SOURCE: &str = include_str!("../src/builtins/temporal.rs");

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

fn count_identifier_in_rust_sources(root: &Path, identifier: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
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

fn count_route_in_rust_sources(root: &Path, route: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
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
            exact_identifier_count(&normalize_rust(&source).routes, route)
        })
        .sum()
}

#[test]
fn zdt_field_result_is_private_and_capability_free() {
    let lexical_probe = r###"
        ZdtFieldResult /* nested /* ignored */ comment */ :: r#NumberOnStack;
        let r#delivery = ZdtFieldResult::WrittenByCallee;
        // ZdtFieldResult delivery
        let normal = "ZdtFieldResult delivery";
        let byte = b"ZdtFieldResult";
        let c_string = c"delivery";
        let raw = r#"ZdtFieldResult"#;
        let raw_byte = br#"delivery"#;
        let raw_c = cr#"ZdtFieldResult"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "ZdtFieldResult::NumberOnStack;",
            "letdelivery=ZdtFieldResult::WrittenByCallee;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "ZdtFieldResult"),
        2
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "delivery"),
        1
    );

    let temporal = normalize_rust(TEMPORAL_SOURCE);
    let declaration = bounded(
        &temporal.code,
        concat!(
            "pub(crate)enumZonedDateTimeField{Era,EraYear,Year,Month,MonthCode,Day,",
            "Hour,Minute,Second,Millisecond,Microsecond,Nanosecond,}"
        ),
        "#[derive(Clone,Copy)]enumZonedDateTimeOptionKey{",
    );
    assert_eq!(
        declaration,
        "enumZdtFieldResult{NumberOnStack,WrittenByCallee,}"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ZdtFieldResult"),
        15,
        "one declaration, twelve producers and two consumer arms own the domain"
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "delivery"),
        2,
        "the inferred binding and consuming match own the delivery local"
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ZdtFieldResult::NumberOnStack"),
        10
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ZdtFieldResult::WrittenByCallee"),
        4
    );

    for forbidden in [
        "implCloneforZdtFieldResult",
        "implCopyforZdtFieldResult",
        "implDebugforZdtFieldResult",
        "implPartialEqforZdtFieldResult",
        "implEqforZdtFieldResult",
        "implDefaultforZdtFieldResult",
        "typeZdtFieldResult=",
        "ZdtFieldResultas",
    ] {
        assert!(
            !temporal.routes.contains(forbidden),
            "forbidden ZdtFieldResult capability or escape `{forbidden}`"
        );
    }
}

#[test]
fn zdt_field_result_binds_every_field_to_its_complete_delivery_body() {
    let temporal = normalize_rust(TEMPORAL_SOURCE);
    assert_eq!(temporal.code.matches("letdelivery=matchfield{").count(), 1);
    let delivery = bounded(&temporal.code, "letdelivery=matchfield{", "forlocalin[");
    let expected = normalize_rust(
        r###"
            ZonedDateTimeField::Year => {
                function.instruction(&Instruction::LocalGet(year_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Month => {
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Day => {
                function.instruction(&Instruction::LocalGet(day_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Hour => {
                function.instruction(&Instruction::LocalGet(hour_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Minute => {
                function.instruction(&Instruction::LocalGet(minute_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Second => {
                function.instruction(&Instruction::LocalGet(second_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Millisecond => {
                function.instruction(&Instruction::LocalGet(millisecond_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Microsecond => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64DivU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Nanosecond => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64RemU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::MonthCode => {
                let month_number_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(month_number_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 2..=12 {
                    function.instruction(&Instruction::LocalGet(month_number_local));
                    function.instruction(&Instruction::I64Const(month));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(&format!("M{month:02}")),
                    ));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(month_number_local);
                ZdtFieldResult::WrittenByCallee
            }
            ZonedDateTimeField::Era => {
                self.emit_temporal_zoned_date_time_era_field(
                    record_local,
                    year_payload_local,
                    TemporalEraField::Era,
                    function,
                );
                ZdtFieldResult::WrittenByCallee
            }
            ZonedDateTimeField::EraYear => {
                self.emit_temporal_zoned_date_time_era_field(
                    record_local,
                    year_payload_local,
                    TemporalEraField::EraYear,
                    function,
                );
                ZdtFieldResult::WrittenByCallee
            }
        };
        match delivery {
            ZdtFieldResult::NumberOnStack => {
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ZdtFieldResult::WrittenByCallee => {}
        }

        "###,
    );
    assert_eq!(delivery, expected.code);
}

#[test]
fn zdt_field_result_has_one_consuming_projection_before_release() {
    let temporal = normalize_rust(TEMPORAL_SOURCE);
    let emitter = bounded(
        &temporal.code,
        "pub(crate)fnemit_temporal_zoned_date_time_iso_field(",
        "fnemit_temporal_zoned_date_time_era_field(",
    );
    assert_eq!(emitter.matches("letdelivery=matchfield{").count(), 1);
    assert_eq!(emitter.matches("matchdelivery{").count(), 1);
    assert!(
        emitter.find("letdelivery=matchfield{").unwrap() < emitter.find("matchdelivery{").unwrap()
    );
    assert!(emitter.find("matchdelivery{").unwrap() < emitter.find("forlocalin[").unwrap());
    for forbidden in [
        "&delivery",
        "delivery.clone(",
        "delivery==",
        "delivery!=",
        "matches!(delivery",
        "discriminant(&delivery)",
        "_=>",
        "unreachable!",
        "transmute",
    ] {
        assert!(
            !emitter.contains(forbidden),
            "forbidden secondary delivery observation `{forbidden}`"
        );
    }
}
