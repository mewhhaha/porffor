use std::fs;
use std::path::Path;

const PRIVATE_ELEMENTS_SOURCE: &str = include_str!("../src/objects/private_elements.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/private-element-entry-protocol.md");
const TASK: &str = include_str!("../../../tasks/09-functions-classes-private-elements.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let tail = &source[start_offset..];
    tail.split_once(end)
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
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
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

fn count_code_fragment_in_rust_sources(dir: &Path, fragment: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_code_fragment_in_rust_sources(&path, fragment);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            normalize_rust(&source).routes.matches(fragment).count()
        })
        .sum()
}

#[test]
fn private_element_entry_rows_are_one_non_capability_authority() {
    let lexical_probe = r###"
        self.r#emit_private_element_entry_add();
        FunctionBuilder::emit_private_element_entry_add();
        FunctionBuilder::<T>::r#emit_private_element_entry_add();
        // PrivateElementEntryLocals::Brand
        /* PrivateElementEntryLocals /* nested */ :: Field */
        "PrivateElementEntryLocals"; b"PrivateElementEntryLocals";
        c"PrivateElementEntryLocals"; r"PrivateElementEntryLocals";
        br#"PrivateElementEntryLocals"#; cr#"PrivateElementEntryLocals"#;
        'P'; b'E'; 'lifetime;
        r#PrivateElementEntryLocals::r#Brand;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "emit_private_element_entry_add"),
        3
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "PrivateElementEntryLocals"),
        1
    );
    assert_eq!(
        exact_route_count(&lexical_probe.routes, "self.emit_private_element_entry_add"),
        1
    );
    assert_eq!(
        lexical_probe
            .routes
            .matches("::emit_private_element_entry_add")
            .count(),
        2
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "lifetime"),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PrivateElementEntryLocals"),
        13
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "emit_private_element_entry_add"),
        6
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "self.emit_private_element_entry_add"),
        5
    );
    assert_eq!(
        count_code_fragment_in_rust_sources(&source_root, "::emit_private_element_entry_add"),
        0
    );

    let source = normalize_rust(PRIVATE_ELEMENTS_SOURCE);
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !source
                .code
                .contains(&format!("{capability}forPrivateElementEntryLocals")),
            "manual {capability} capability escaped the closed row authority"
        );
    }
}

#[test]
fn private_element_entry_declaration_and_projections_are_exact_and_exhaustive() {
    let authority = normalize_rust(bounded(
        PRIVATE_ELEMENTS_SOURCE,
        "use super::*;",
        "#[cfg(test)]",
    ));
    assert_eq!(
        authority.code,
        concat!(
            "enumPrivateElementEntryLocals{",
            "Brand{receiver:(u32,u32),},",
            "Field{receiver:(u32,u32),value:(u32,u32),},",
            "SetterDefinition{value:(u32,u32),},",
            "MethodDefinition{value:(u32,u32),},",
            "GetterDefinition{value:(u32,u32),},}",
            "implPrivateElementEntryLocals{",
            "constfnkind(&self)->PrivateElementHeapKind{matchself{",
            "Self::Brand{..}=>PrivateElementHeapKind::Brand,",
            "Self::Field{..}=>PrivateElementHeapKind::Field,",
            "Self::SetterDefinition{..}=>PrivateElementHeapKind::SetterDefinition,",
            "Self::MethodDefinition{..}=>PrivateElementHeapKind::MethodDefinition,",
            "Self::GetterDefinition{..}=>PrivateElementHeapKind::GetterDefinition,}}",
            "constfnreceiver(&self)->Option<(u32,u32)>{matchself{",
            "Self::Brand{receiver}|Self::Field{receiver,..}=>Some(*receiver),",
            "Self::SetterDefinition{..}|Self::MethodDefinition{..}|",
            "Self::GetterDefinition{..}=>None,}}",
            "constfnvalue(&self)->Option<(u32,u32)>{matchself{",
            "Self::Brand{..}=>None,",
            "Self::Field{value,..}|Self::SetterDefinition{value}|",
            "Self::MethodDefinition{value}|Self::GetterDefinition{value}=>Some(*value),}}}"
        )
    );

    let row_unit = normalize_rust(bounded_inclusive(
        PRIVATE_ELEMENTS_SOURCE,
        "    fn private_element_rows_fix_wire_and_storage_projections() {",
        "}\n}\n\nimpl<'a> FunctionBuilder<'a>",
    ));
    let rows = normalize_rust(bounded(
        PRIVATE_ELEMENTS_SOURCE,
        "        let rows = [",
        "        ];\n\n        for (entry, kind, expected_receiver, expected_value) in &rows",
    ));
    assert_eq!(
        rows.code,
        concat!(
            "(PrivateElementEntryLocals::Brand{receiver},PrivateElementHeapKind::Brand,",
            "Some(receiver),None,),",
            "(PrivateElementEntryLocals::Field{receiver,value},PrivateElementHeapKind::Field,",
            "Some(receiver),Some(value),),",
            "(PrivateElementEntryLocals::SetterDefinition{value},",
            "PrivateElementHeapKind::SetterDefinition,None,Some(value),),",
            "(PrivateElementEntryLocals::MethodDefinition{value},",
            "PrivateElementHeapKind::MethodDefinition,None,Some(value),),",
            "(PrivateElementEntryLocals::GetterDefinition{value},",
            "PrivateElementHeapKind::GetterDefinition,None,Some(value),),"
        )
    );
    assert!(row_unit.code.contains(concat!(
        "for(entry,kind,expected_receiver,expected_value)in&rows{",
        "assert_eq!(entry.kind(),*kind);",
        "assert_eq!(entry.receiver(),*expected_receiver);",
        "assert_eq!(entry.value(),*expected_value);",
        "assert_eq!(kind.has_receiver(),expected_receiver.is_some());",
        "assert_eq!(kind.has_value(),expected_value.is_some());}"
    )));
}

#[test]
fn private_element_entry_producers_fix_all_five_product_rows() {
    let producers = [
        (
            "    pub(crate) fn emit_private_brand_add(",
            "    pub(crate) fn emit_private_field_add(",
            r#"
                pub(crate) fn emit_private_brand_add(
                    &mut self,
                    receiver_payload_local: u32,
                    receiver_tag_local: u32,
                    token_local: u32,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_private_element_entry_add(
                        token_local,
                        PrivateElementEntryLocals::Brand {
                            receiver: (receiver_payload_local, receiver_tag_local),
                        },
                        function,
                    )
                }
            "#,
        ),
        (
            "    pub(crate) fn emit_private_field_add(",
            "    pub(crate) fn emit_private_setter_definition_add(",
            r#"
                pub(crate) fn emit_private_field_add(
                    &mut self,
                    receiver_payload_local: u32,
                    receiver_tag_local: u32,
                    token_local: u32,
                    value_payload_local: u32,
                    value_tag_local: u32,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_private_element_entry_add(
                        token_local,
                        PrivateElementEntryLocals::Field {
                            receiver: (receiver_payload_local, receiver_tag_local),
                            value: (value_payload_local, value_tag_local),
                        },
                        function,
                    )
                }
            "#,
        ),
        (
            "    pub(crate) fn emit_private_setter_definition_add(",
            "    pub(crate) fn emit_private_method_definition_add(",
            r#"
                pub(crate) fn emit_private_setter_definition_add(
                    &mut self,
                    token_local: u32,
                    setter_payload_local: u32,
                    setter_tag_local: u32,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_private_element_entry_add(
                        token_local,
                        PrivateElementEntryLocals::SetterDefinition {
                            value: (setter_payload_local, setter_tag_local),
                        },
                        function,
                    )
                }
            "#,
        ),
        (
            "    pub(crate) fn emit_private_method_definition_add(",
            "    pub(crate) fn emit_private_getter_definition_add(",
            r#"
                pub(crate) fn emit_private_method_definition_add(
                    &mut self,
                    token_local: u32,
                    method_payload_local: u32,
                    method_tag_local: u32,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_private_element_entry_add(
                        token_local,
                        PrivateElementEntryLocals::MethodDefinition {
                            value: (method_payload_local, method_tag_local),
                        },
                        function,
                    )
                }
            "#,
        ),
        (
            "    pub(crate) fn emit_private_getter_definition_add(",
            "    fn emit_private_element_entry_add(",
            r#"
                pub(crate) fn emit_private_getter_definition_add(
                    &mut self,
                    token_local: u32,
                    getter_payload_local: u32,
                    getter_tag_local: u32,
                    function: &mut Function,
                ) -> Result<(), EmitError> {
                    self.emit_private_element_entry_add(
                        token_local,
                        PrivateElementEntryLocals::GetterDefinition {
                            value: (getter_payload_local, getter_tag_local),
                        },
                        function,
                    )
                }
            "#,
        ),
    ];
    for (start, end, expected_owner) in producers {
        let owner = normalize_rust(bounded_inclusive(PRIVATE_ELEMENTS_SOURCE, start, end));
        assert_eq!(owner.code, normalize_rust(expected_owner).code, "{start}");
    }
}

#[test]
fn private_element_entry_consumer_projects_once_before_sole_realm_publication() {
    let consumer = normalize_rust(bounded_inclusive(
        PRIVATE_ELEMENTS_SOURCE,
        "    fn emit_private_element_entry_add(",
        "    fn emit_private_receiver_kind_guard(",
    ));
    let expected_consumer = r#"
        fn emit_private_element_entry_add(
            &mut self,
            token_local: u32,
            entry: PrivateElementEntryLocals,
            function: &mut Function,
        ) -> Result<(), EmitError> {
            let realm_local = self.reserve_temp_local();
            let previous_local = self.reserve_temp_local();
            let entry_local = self.reserve_temp_local();
            let kind = entry.kind();
            let receiver_locals = entry.receiver();
            let value_locals = entry.value();

            debug_assert_eq!(kind.has_receiver(), receiver_locals.is_some());
            debug_assert_eq!(kind.has_value(), value_locals.is_some());

            if let Some((receiver_payload_local, receiver_tag_local)) = receiver_locals {
                let extensible_local = self.reserve_temp_local();
                self.emit_object_is_extensible_i32(
                    receiver_payload_local,
                    receiver_tag_local,
                    extensible_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(extensible_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error_to_active_handler(
                    TYPE_ERROR_NAME,
                    "private element cannot be installed on non-extensible object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.release_temp_local(extensible_local);

                let existing_entry_local = self.reserve_temp_local();
                self.emit_private_element_find(
                    receiver_payload_local,
                    token_local,
                    existing_entry_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(existing_entry_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error_to_active_handler(
                    TYPE_ERROR_NAME,
                    "private element already installed on object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.release_temp_local(existing_entry_local);
            }

            function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(realm_local));
            self.load_i64_to_local_from_offset(
                realm_local,
                HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
                previous_local,
                function,
            );
            self.emit_heap_alloc_const(HEAP_PRIVATE_ELEMENT_ENTRY_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
                previous_local,
                function,
            );
            if let Some((receiver_payload_local, _)) = receiver_locals {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                    receiver_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                    0,
                    function,
                );
            }
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
                token_local,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
                kind.wire_word(),
                function,
            );
            if let Some((value_payload_local, value_tag_local)) = value_locals {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                    value_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                    value_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                    0,
                    function,
                );
            }
            self.store_i64_local_at_offset(
                realm_local,
                HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
                entry_local,
                function,
            );

            self.release_temp_local(entry_local);
            self.release_temp_local(previous_local);
            self.release_temp_local(realm_local);
            Ok(())
        }
    "#;
    assert_eq!(consumer.code, normalize_rust(expected_consumer).code);

    let publication = concat!(
        "self.store_i64_local_at_offset(realm_local,",
        "HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,entry_local,function,);"
    );
    assert_eq!(consumer.code.matches(publication).count(), 1);
    assert_eq!(
        normalize_rust(PRIVATE_ELEMENTS_SOURCE)
            .code
            .matches(publication)
            .count(),
        1
    );
}

#[test]
fn private_element_entry_contract_and_t09_checkpoint_name_the_closed_row_owner() {
    for marker in [
        "PrivateElementEntryLocals",
        "one owned row",
        "borrowed exhaustive projections",
        "13 lexical mentions",
        "five product producers",
        "Realm-list publication",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(TASK.contains(marker), "missing T09 marker: {marker}");
    }
}
