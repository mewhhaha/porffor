use std::fs;
use std::path::Path;

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/conversion-error-realm-source-lifecycle.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

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

#[test]
fn conversion_error_realm_domains_are_closed_non_capability_authorities() {
    let lexical_probe = r###"
        // ConversionErrorRealm::MainRealm
        /* ConversionErrorRealmSource /* nested */ :: Fixed */
        "ConversionErrorRealm"; b"ConversionErrorRealmSource";
        c"ConversionErrorRealm"; r"ConversionErrorRealmSource";
        br##"ConversionErrorRealm::CurrentFunctionRealm"##;
        cr#"ConversionErrorRealmSource::RuntimeHelperArgument"#;
        'C'; b'S'; 'lifetime;
        r#ConversionErrorRealm::r#MainRealm;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "ConversionErrorRealm"),
        1
    );
    assert_eq!(
        lexical_probe
            .routes
            .matches("ConversionErrorRealm::MainRealm")
            .count(),
        1
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "lifetime"),
        1
    );

    let declarations = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "impl PrimitiveToNumberThrowRouting {",
        "/// A tagged `ToPrimitive` result whose possible throw still needs an owner.",
    ));
    assert_eq!(
        declarations.code,
        concat!(
            "fnemit(&self,builder:&mutFunctionBuilder<'_>,function:&mutFunction){",
            "matchself{Self::ReturnCurrentFunction=>",
            "builder.emit_return_current_completion(function),Self::LeaveInCompletion=>{}}}",
            "}enumOrdinaryToPrimitiveReceiverKind{Object,Function,}",
            "implOrdinaryToPrimitiveReceiverKind{",
            "constfnvalue_kind(&self)->ValueKind{matchself{",
            "Self::Object=>ValueKind::Object,Self::Function=>ValueKind::Function,}}",
            "constfnhas_boxed_primitive_slot(&self)->bool{matchself{",
            "Self::Object=>true,Self::Function=>false,}}}",
            "enumConversionErrorRealm{MainRealm,CurrentFunctionRealm,}",
            "implConversionErrorRealm{constfnabi_word(&self)->i64{matchself{",
            "Self::MainRealm=>0,Self::CurrentFunctionRealm=>1,}}}",
            "enumConversionErrorRealmSource{",
            "Fixed(ConversionErrorRealm),RuntimeHelperArgument,}",
            "#[must_use=\"a current-function-realm primitive must be consumed by its matching ToString wrapper\"]",
            "pub(crate)structCurrentFunctionRealmPrimitiveLocals{",
            "payload_local:u32,tag_local:u32,}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ConversionErrorRealm"),
        16
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ConversionErrorRealmSource"),
        23
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "CurrentFunctionRealmPrimitiveLocals",),
        5
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ConversionErrorRealm::MainRealm"),
        9
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ConversionErrorRealm::CurrentFunctionRealm",),
        4
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ConversionErrorRealmSource::Fixed"),
        12
    );
    assert_eq!(
        count_route_in_rust_sources(
            &source_root,
            "ConversionErrorRealmSource::RuntimeHelperArgument",
        ),
        3
    );

    let all_source = normalize_rust(OPERATIONS_SOURCE);
    for domain in [
        "ConversionErrorRealm",
        "ConversionErrorRealmSource",
        "CurrentFunctionRealmPrimitiveLocals",
    ] {
        for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
            assert!(!all_source
                .code
                .contains(&format!("impl{capability}for{domain}")));
        }
    }
    for forbidden in ["#[derive", "#[repr", "=0", "=1"] {
        assert!(
            !declarations.code.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn conversion_error_realm_serialization_and_decoder_are_exact() {
    let argument = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_conversion_error_realm_argument(",
        "    fn emit_conversion_type_error(",
    ));
    assert_eq!(
        argument.code,
        concat!(
            "&self,error_realm:&ConversionErrorRealmSource,function:&mutFunction,){",
            "matcherror_realm{",
            "ConversionErrorRealmSource::Fixed(error_realm)=>{",
            "function.instruction(&Instruction::I64Const(error_realm.abi_word()));}",
            "ConversionErrorRealmSource::RuntimeHelperArgument=>{",
            "function.instruction(&Instruction::LocalGet(2));}}}"
        )
    );

    let type_error = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_conversion_type_error(",
        "    /// Emit helper ABI parameter 6 for outlined `ToNumeric`/`ToNumber` calls.",
    ));
    assert_eq!(
        type_error.code,
        concat!(
            "&mutself,error_realm:&ConversionErrorRealmSource,message:&str,",
            "payload_local:u32,tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{",
            "matcherror_realm{",
            "ConversionErrorRealmSource::Fixed(ConversionErrorRealm::MainRealm)=>self.",
            "emit_throw_runtime_error(TYPE_ERROR_NAME,message,payload_local,tag_local,function,),",
            "ConversionErrorRealmSource::Fixed(ConversionErrorRealm::CurrentFunctionRealm)=>self.",
            "emit_throw_current_function_realm_type_error(",
            "message,payload_local,tag_local,function,),",
            "ConversionErrorRealmSource::RuntimeHelperArgument=>{",
            "function.instruction(&Instruction::LocalGet(2));",
            "function.instruction(&Instruction::I64Const(",
            "ConversionErrorRealm::MainRealm.abi_word(),));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_throw_runtime_error(",
            "TYPE_ERROR_NAME,message,payload_local,tag_local,function,)?;",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(2));",
            "function.instruction(&Instruction::I64Const(",
            "ConversionErrorRealm::CurrentFunctionRealm.abi_word(),));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_throw_current_function_realm_type_error(",
            "message,payload_local,tag_local,function,)?;",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::Unreachable);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);Ok(())}}}"
        )
    );
    for forbidden in [
        "_=>",
        "==",
        "!=",
        "matches!(",
        "unwrap_or",
        "Default::default",
    ] {
        assert!(
            !argument.code.contains(forbidden),
            "argument contains `{forbidden}`"
        );
        assert!(
            !type_error.code.contains(forbidden),
            "decoder contains `{forbidden}`"
        );
    }
}

#[test]
fn all_source_producers_and_the_current_realm_phase_lifecycle_are_exact() {
    let operations = normalize_rust(OPERATIONS_SOURCE);
    assert_eq!(
        operations
            .code
            .matches("error_realm:ConversionErrorRealmSource")
            .count(),
        0,
        "the typed phase token must not own a freely shaped source policy"
    );

    let borrowed_seams = [
        (
            "    fn emit_conversion_error_realm_argument(",
            "    fn emit_conversion_type_error(",
            "&self,error_realm:&ConversionErrorRealmSource,function:&mutFunction,){",
        ),
        (
            "    fn emit_conversion_type_error(",
            "    /// Emit helper ABI parameter 6 for outlined `ToNumeric`/`ToNumber` calls.",
            concat!(
                "&mutself,error_realm:&ConversionErrorRealmSource,message:&str,",
                "payload_local:u32,tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{"
            ),
        ),
        (
            "    fn emit_value_to_primitive_via_helper_if_outlined(",
            "    pub(crate) fn emit_tagged_to_primitive_locals(",
            concat!(
                "&mutself,hint:ToPrimitiveHint,input_payload_local:u32,input_tag_local:u32,",
                "payload_local:u32,tag_local:u32,error_realm:&ConversionErrorRealmSource,",
                "function:&mutFunction,)->bool{"
            ),
        ),
        (
            "    fn emit_tagged_to_primitive_locals_pending(",
            "    /// Emit a complete ToPrimitive runtime-helper result tuple.",
            concat!(
                "&mutself,hint:ToPrimitiveHint,input_payload_local:u32,input_tag_local:u32,",
                "payload_local:u32,tag_local:u32,error_realm:&ConversionErrorRealmSource,",
                "function:&mutFunction,)->Result<PendingToPrimitiveCompletion,EmitError>{"
            ),
        ),
        (
            "    fn emit_object_to_primitive_locals_pending(",
            "    fn emit_object_to_primitive_locals_inner(",
            concat!(
                "&mutself,hint:ToPrimitiveHint,object_local:u32,payload_local:u32,tag_local:u32,",
                "error_realm:&ConversionErrorRealmSource,function:&mutFunction,",
                ")->Result<PendingToPrimitiveCompletion,EmitError>{"
            ),
        ),
        (
            "    fn emit_object_to_primitive_locals_inner(",
            "    pub(crate) fn emit_ordinary_object_default_to_string_applies_i32(",
            concat!(
                "&mutself,hint:ToPrimitiveHint,object_local:u32,",
                "receiver_kind:OrdinaryToPrimitiveReceiverKind,",
                "payload_local:u32,tag_local:u32,error_realm:&ConversionErrorRealmSource,",
                "function:&mutFunction,)->Result<(),EmitError>{"
            ),
        ),
        (
            "    fn emit_primitive_to_string_payload_with_error_realm(",
            "    pub(crate) fn emit_bigint_value_to_string_payload(",
            concat!(
                "&mutself,payload_local:u32,tag_local:u32,",
                "abrupt_route:PrimitiveToStringAbruptRoute,",
                "error_realm:&ConversionErrorRealmSource,function:&mutFunction,",
                ")->Result<(),EmitError>{"
            ),
        ),
    ];
    for (start, end, signature) in borrowed_seams {
        let seam = normalize_rust(bounded(OPERATIONS_SOURCE, start, end));
        assert!(
            seam.code.starts_with(signature),
            "borrowed source signature drifted for `{start}`"
        );
        assert_eq!(
            seam.code
                .matches("error_realm:&ConversionErrorRealmSource")
                .count(),
            1,
            "borrowed source ownership drifted for `{start}`"
        );
    }

    let outlined = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_value_to_primitive_via_helper_if_outlined(",
        "    pub(crate) fn emit_tagged_to_primitive_locals(",
    ));
    assert_eq!(
        outlined
            .code
            .matches("self.emit_conversion_error_realm_argument(error_realm,function);")
            .count(),
        1
    );
    let tagged_pending = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_tagged_to_primitive_locals_pending(",
        "    /// Emit a complete ToPrimitive runtime-helper result tuple.",
    ));
    for forwarding in [
        concat!(
            "self.emit_value_to_primitive_via_helper_if_outlined(",
            "hint,input_payload_local,input_tag_local,payload_local,tag_local,",
            "error_realm,function,)"
        ),
        concat!(
            "self.emit_object_to_primitive_locals_inner(",
            "hint,input_payload_local,OrdinaryToPrimitiveReceiverKind::Object,",
            "payload_local,tag_local,",
            "error_realm,function,)?;"
        ),
        concat!(
            "self.emit_object_to_primitive_locals_inner(",
            "hint,input_payload_local,OrdinaryToPrimitiveReceiverKind::Function,",
            "payload_local,tag_local,",
            "error_realm,function,)?;"
        ),
    ] {
        assert_eq!(
            tagged_pending.code.matches(forwarding).count(),
            1,
            "{forwarding}"
        );
    }
    let object_pending = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_object_to_primitive_locals_pending(",
        "    fn emit_object_to_primitive_locals_inner(",
    ));
    assert_eq!(
        object_pending
            .code
            .matches(concat!(
                "self.emit_object_to_primitive_locals_inner(hint,object_local,",
                "OrdinaryToPrimitiveReceiverKind::Object,payload_local,tag_local,",
                "error_realm,function,)?;"
            ))
            .count(),
        1,
    );
    let object_inner = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_object_to_primitive_locals_inner(",
        "    pub(crate) fn emit_ordinary_object_default_to_string_applies_i32(",
    ));
    assert_eq!(
        object_inner
            .code
            .matches("self.emit_conversion_type_error(error_realm,")
            .count(),
        3
    );
    let primitive_to_string = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_primitive_to_string_payload_with_error_realm(",
        "    pub(crate) fn emit_bigint_value_to_string_payload(",
    ));
    assert_eq!(
        primitive_to_string
            .code
            .matches(concat!(
                "self.emit_conversion_type_error(error_realm,",
                "\"Cannot convert a Symbol value to a string\",",
                "self.result_local,self.result_tag_local,function,)?;"
            ))
            .count(),
        1
    );

    let main_producers = [
        concat!(
            "self.emit_tagged_to_primitive_locals_pending(hint,input_payload_local,",
            "input_tag_local,payload_local,tag_local,&ConversionErrorRealmSource::Fixed(",
            "ConversionErrorRealm::MainRealm),function,)?.route(self,route,function)"
        ),
        concat!(
            "self.emit_object_to_primitive_locals_pending(hint,object_local,payload_local,",
            "tag_local,&ConversionErrorRealmSource::Fixed(",
            "ConversionErrorRealm::MainRealm),function,)?.route(self,route,function)"
        ),
        concat!(
            "letpending=self.emit_object_to_primitive_locals_pending(",
            "ToPrimitiveHint::Number,payload_local,primitive_payload_local,primitive_tag_local,",
            "&ConversionErrorRealmSource::Fixed(ConversionErrorRealm::MainRealm),function,)?;",
            "pending.emit_number_payload(self,function)?;"
        ),
        concat!(
            "letpending=self.emit_object_to_primitive_locals_pending(",
            "ToPrimitiveHint::Number,payload_local,primitive_payload_local,primitive_tag_local,",
            "&ConversionErrorRealmSource::Fixed(ConversionErrorRealm::MainRealm),function,)?;",
            "pending.emit_number_payload_without_return(self,function)?;"
        ),
        concat!(
            "letpending=self.emit_object_to_primitive_locals_pending(",
            "ToPrimitiveHint::Number,payload_local,primitive_payload_local,primitive_tag_local,",
            "&ConversionErrorRealmSource::Fixed(ConversionErrorRealm::MainRealm),function,)?;",
            "pending.emit_number_payload_allow_bigint(self,function)?;"
        ),
        concat!(
            "letpending=self.emit_object_to_primitive_locals_pending(",
            "ToPrimitiveHint::String,payload_local,primitive_payload_local,primitive_tag_local,",
            "&ConversionErrorRealmSource::Fixed(ConversionErrorRealm::MainRealm),function,)?;",
            "pending.emit_string_payload(self,function)?;"
        ),
        concat!(
            "self.emit_primitive_to_string_payload_with_error_realm(",
            "payload_local,tag_local,abrupt_route,&ConversionErrorRealmSource::Fixed(",
            "ConversionErrorRealm::MainRealm),function,)"
        ),
    ];
    for producer in main_producers {
        assert_eq!(operations.code.matches(producer).count(), 1, "{producer}");
    }

    let current_lifecycle = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn emit_tagged_to_primitive_locals_in_current_function_realm(",
        "    fn emit_tagged_to_primitive_locals_pending(",
    ));
    let expected_current_lifecycle = r#"
        &mut self,
        hint: ToPrimitiveHint,
        input_payload_local: u32,
        input_tag_local: u32,
        function: &mut Function,
    ) -> Result<CurrentFunctionRealmPrimitiveLocals, EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_tagged_to_primitive_locals_pending(
            hint,
            input_payload_local,
            input_tag_local,
            payload_local,
            tag_local,
            &ConversionErrorRealmSource::Fixed(ConversionErrorRealm::CurrentFunctionRealm),
            function,
        )?
        .route(
            self,
            ToPrimitiveAbruptRoute::ReturnCurrentFunction,
            function,
        )?;

        Ok(CurrentFunctionRealmPrimitiveLocals {
            payload_local,
            tag_local,
        })
    }

    pub(crate) fn emit_current_function_realm_primitive_to_string_local(
        &mut self,
        primitive: CurrentFunctionRealmPrimitiveLocals,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CurrentFunctionRealmPrimitiveLocals {
            payload_local,
            tag_local,
        } = primitive;

        self.emit_primitive_to_string_payload_with_error_realm(
            payload_local,
            tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            &ConversionErrorRealmSource::Fixed(ConversionErrorRealm::CurrentFunctionRealm),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(string_payload_local));

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

"#;
    assert_eq!(
        current_lifecycle.code,
        normalize_rust(expected_current_lifecycle).code,
        "the typed token must couple both fixed current-function-Realm boundaries and release its locals exactly"
    );
    assert_eq!(
        current_lifecycle
            .routes
            .matches("ConversionErrorRealmSource::Fixed")
            .count(),
        2
    );

    let helper_body = normalize_rust(bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn emit_to_primitive_runtime_helper_result_tuple(",
        "    pub(crate) fn emit_object_to_primitive_locals(",
    ));
    assert_eq!(
        helper_body
            .routes
            .matches("ConversionErrorRealmSource::RuntimeHelperArgument")
            .count(),
        1
    );
    assert!(helper_body.code.contains(concat!(
        "self.emit_tagged_to_primitive_locals_pending(",
        "hint,input_payload_local,input_tag_local,payload_local,tag_local,",
        "&ConversionErrorRealmSource::RuntimeHelperArgument,function,)?;"
    )));

    let compiled_helper = normalize_rust(bounded(
        EMIT_SOURCE,
        "    fn compile_value_to_primitive_helper(",
        "    fn compile_object_get_prototype_of_helper(",
    ));
    assert!(compiled_helper.code.contains(concat!(
        "function.instruction(&Instruction::LocalGet(6));",
        "function.instruction(&Instruction::LocalSet(self.current_env_local));",
        "self.set_completion_kind(CompletionKind::Normal,&mutfunction);",
        "self.emit_statement_result(&mutfunction,ValueKind::Undefined);",
        "self.emit_to_primitive_runtime_helper_result_tuple(hint,0,1,&mutfunction)?;"
    )));
}

#[test]
fn contract_and_t04_record_the_non_copy_phase_authority() {
    for marker in [
        "type-owned current-function Realm proof",
        "payload and tag locals only",
        "two fixed boundary selections",
        "helper ABI parameter 2",
        "does not claim a conversion-semantics change",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("conversion-error-realm-source-lifecycle.md"));
}
