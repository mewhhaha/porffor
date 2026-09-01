use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OWNER_SOURCE: &str = include_str!("../src/functions/function_realm.rs");
const REQUIRED_ORDINARY_PROTOTYPE_SOURCE: &str =
    include_str!("../src/functions/required_resolved_realm_ordinary_prototype.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
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
fn function_realm_outcome_is_one_closed_runtime_code_authority() {
    assert_eq!(
        FUNCTIONS_SOURCE.matches("\nmod function_realm;\n").count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("\npub mod function_realm;\n"));
    assert!(FUNCTIONS_SOURCE.contains("pub(crate) use function_realm::FunctionRealmRevokedRoute;"));
    assert!(FUNCTIONS_SOURCE.contains("use function_realm::ResolvedFunctionRealmLocal;"));
    for private_state in ["FunctionRealmOutcome", "FunctionRealmResultLocals"] {
        assert!(!FUNCTIONS_SOURCE.contains(&format!("enum {private_state}")));
        assert!(!FUNCTIONS_SOURCE.contains(&format!("struct {private_state}")));
    }
    for owner_method in [
        "pub(crate) fn emit_get_function_realm(",
        "pub(crate) fn emit_route_function_realm_result(",
        "pub(crate) fn release_resolved_function_realm_local(",
    ] {
        assert_eq!(
            OWNER_SOURCE.matches(owner_method).count(),
            1,
            "{owner_method}"
        );
        assert!(!FUNCTIONS_SOURCE.contains(owner_method), "{owner_method}");
    }

    let lexical_probe = r###"
        // FunctionRealmOutcome::Invalid.runtime_code()
        "FunctionRealmOutcome::Invalid.runtime_code()";
        r#"FunctionRealmOutcome::Invalid.runtime_code()"#;
        struct r#FunctionRealmOutcome;
        value./* split route */r#runtime_code();
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "FunctionRealmOutcome"),
        1
    );
    assert_eq!(lexical_probe.routes.matches(".runtime_code()").count(), 1);

    let declaration_start = OWNER_SOURCE
        .find("enum FunctionRealmOutcome {")
        .expect("FunctionRealmOutcome owner");
    let declaration_end = OWNER_SOURCE[declaration_start..]
        .find("/// The raw run-time result of `GetFunctionRealm`")
        .map(|offset| declaration_start + offset)
        .expect("following raw-result declaration");
    let declaration = normalize_rust(&OWNER_SOURCE[declaration_start..declaration_end]);
    assert_eq!(
        declaration.code,
        concat!(
            "enumFunctionRealmOutcome{Resolved,Revoked,Invalid,}",
            "implFunctionRealmOutcome{constfnruntime_code(&self)->i64{matchself{",
            "Self::Resolved=>0,Self::Revoked=>1,Self::Invalid=>2,}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "FunctionRealmOutcome"),
        7,
        "the declaration, projection and five runtime constants must be the complete authority census"
    );
    assert_eq!(
        [
            "FunctionRealmOutcome::Resolved.runtime_code()",
            "FunctionRealmOutcome::Revoked.runtime_code()",
            "FunctionRealmOutcome::Invalid.runtime_code()",
        ]
        .into_iter()
        .map(|route| count_route_in_rust_sources(&source_root, route))
        .sum::<usize>(),
        5
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "FunctionRealmOutcome::Resolved"),
        1
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "FunctionRealmOutcome::Revoked"),
        2
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "FunctionRealmOutcome::Invalid"),
        2
    );
    let source = normalize_rust(OWNER_SOURCE);
    assert!(!source.code.contains("forFunctionRealmOutcome"));
    assert!(!source
        .routes
        .contains("FunctionRealmOutcome::Resolvedasi64"));
    assert!(!source.routes.contains("FunctionRealmOutcome::Revokedasi64"));
    assert!(!source.routes.contains("FunctionRealmOutcome::Invalidasi64"));
}

#[test]
fn get_function_realm_writes_all_three_outcomes_in_the_existing_order() {
    let emitter = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn emit_get_function_realm(",
        "    /// Consume a raw GetFunctionRealm result",
    );
    let emitter = normalize_rust(emitter);
    assert_eq!(
        exact_identifier_count(&emitter.identifiers, "outcome_local"),
        5,
        "one reservation, three writes and the opaque result field must be the complete producer lifecycle"
    );
    assert_eq!(
        emitter
            .code
            .matches("Instruction::LocalSet(outcome_local)")
            .count(),
        3
    );
    assert!(emitter.code.contains(concat!(
        "function.instruction(&Instruction::I64Const(0));",
        "function.instruction(&Instruction::LocalSet(realm_local));",
        "function.instruction(&Instruction::I64Const(",
        "FunctionRealmOutcome::Invalid.runtime_code(),));",
        "function.instruction(&Instruction::LocalSet(outcome_local));",
        "function.instruction(&Instruction::Block(BlockType::Empty));"
    )));
    assert!(emitter.code.contains(concat!(
        "self.load_i64_to_local_from_offset(current_payload_local,",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET,realm_local,function,);",
        "function.instruction(&Instruction::LocalGet(realm_local));",
        "function.instruction(&Instruction::I64Eqz);",
        "function.instruction(&Instruction::If(BlockType::Empty));",
        "function.instruction(&Instruction::Else);",
        "function.instruction(&Instruction::I64Const(",
        "FunctionRealmOutcome::Resolved.runtime_code(),));",
        "function.instruction(&Instruction::LocalSet(outcome_local));",
        "function.instruction(&Instruction::End);",
        "function.instruction(&Instruction::Br(2));"
    )));
    assert!(emitter.code.contains(concat!(
        "function.instruction(&Instruction::LocalGet(proxy_handler_local));",
        "function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MINasi64));",
        "function.instruction(&Instruction::I64Eq);",
        "function.instruction(&Instruction::If(BlockType::Empty));",
        "function.instruction(&Instruction::I64Const(",
        "FunctionRealmOutcome::Revoked.runtime_code(),));",
        "function.instruction(&Instruction::LocalSet(outcome_local));",
        "function.instruction(&Instruction::Br(4));"
    )));
    assert!(emitter.code.ends_with(concat!(
        "self.release_temp_local(proxy_handler_local);",
        "self.release_temp_local(record_local);",
        "self.release_temp_local(flags_local);",
        "self.release_temp_local(current_tag_local);",
        "self.release_temp_local(current_payload_local);",
        "FunctionRealmResultLocals{realm_local,outcome_local,}}"
    )));
}

#[test]
fn function_realm_router_handles_revoked_then_invalid_before_resolving() {
    let router = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn emit_route_function_realm_result(",
        "    pub(crate) fn release_resolved_function_realm_local(",
    );
    let router = normalize_rust(router);
    assert_eq!(
        exact_identifier_count(&router.identifiers, "outcome_local"),
        3,
        "two reads and the final release must be the complete consumer lifecycle"
    );
    let revoked = concat!(
        "function.instruction(&Instruction::LocalGet(result.outcome_local));",
        "function.instruction(&Instruction::I64Const(",
        "FunctionRealmOutcome::Revoked.runtime_code(),));",
        "function.instruction(&Instruction::I64Eq);",
        "function.instruction(&Instruction::If(BlockType::Empty));"
    );
    let routes = concat!(
        "matchrevoked_route{",
        "FunctionRealmRevokedRoute::UseCurrentRealm=>{",
        "function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));",
        "function.instruction(&Instruction::LocalSet(result.realm_local));}",
        "FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn{payload_local,tag_local,}=>{",
        "self.emit_throw_runtime_error(TYPE_ERROR_NAME,",
        "\"cannot get function realm from a revoked Proxy\",payload_local,tag_local,function,)?;",
        "self.emit_return_current_completion(function);}",
        "FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch{payload_local,tag_local,relative_depth,}=>{",
        "self.emit_throw_runtime_error(TYPE_ERROR_NAME,",
        "\"cannot get function realm from a revoked Proxy\",payload_local,tag_local,function,)?;",
        "function.instruction(&Instruction::Br(relative_depth));}}"
    );
    let invalid = concat!(
        "function.instruction(&Instruction::LocalGet(result.outcome_local));",
        "function.instruction(&Instruction::I64Const(",
        "FunctionRealmOutcome::Invalid.runtime_code(),));",
        "function.instruction(&Instruction::I64Eq);",
        "function.instruction(&Instruction::If(BlockType::Empty));",
        "function.instruction(&Instruction::Unreachable);",
        "function.instruction(&Instruction::End);"
    );
    let release = concat!(
        "self.release_temp_local(result.outcome_local);",
        "Ok(ResolvedFunctionRealmLocal(result.realm_local))}"
    );
    assert_eq!(
        router.code.matches("ResolvedFunctionRealmLocal(").count(),
        1
    );
    let exact_tail =
        format!("{revoked}{routes}function.instruction(&Instruction::End);{invalid}{release}");
    assert!(router.code.ends_with(&exact_tail));
    assert!(!router.code.contains("_=>"));
    assert!(!router.routes.contains("FunctionRealmOutcome::Resolved"));
}

#[test]
fn all_five_get_function_realm_results_are_immediately_routed() {
    let generic_construct = concat!(
        "letprototype_realm_result=self.emit_get_function_realm(",
        "new_target_payload_local,new_target_tag_local,function);",
        "letprototype_realm=self.emit_route_function_realm_result(",
        "prototype_realm_result,FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch{",
        "payload_local,tag_local,relative_depth:2,},function,)?;"
    );
    let required_ordinary = concat!(
        "letrealm_result=self.emit_get_function_realm(",
        "new_target_payload_local,new_target_tag_local,function);",
        "letrealm=self.emit_route_function_realm_result(",
        "realm_result,FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn{",
        "payload_local:self.result_local,tag_local:self.result_tag_local,},function,)?;"
    );
    let required_error = concat!(
        "letprototype_realm_result=self.emit_get_function_realm(",
        "new_target_payload_local,new_target_tag_local,function,);",
        "letprototype_realm=self.emit_route_function_realm_result(",
        "prototype_realm_result,FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn{",
        "payload_local:self.result_local,tag_local:self.result_tag_local,},function,)?;"
    );
    let promise_current = concat!(
        "letrealm_result=self.emit_get_function_realm(",
        "callback_payload_local,callback_tag_local,function);",
        "letresolved_realm=self.emit_route_function_realm_result(",
        "realm_result,FunctionRealmRevokedRoute::UseCurrentRealm,function,)?;"
    );
    let typed_array = concat!(
        "letprototype_realm_result=self.emit_get_function_realm(",
        "self.new_target_payload_local().unwrap(),",
        "self.new_target_tag_local().unwrap(),function,);",
        "letprototype_realm=self.emit_route_function_realm_result(",
        "prototype_realm_result,FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn{",
        "payload_local:self.result_local,tag_local:self.result_tag_local,},function,)?;"
    );
    let generic_construct_owner = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_handle_construct_with_argv(",
        "    pub(crate) fn copy_function_realm_typed_array_prototypes(",
    );
    let required_ordinary_owner = bounded(
        REQUIRED_ORDINARY_PROTOTYPE_SOURCE,
        "    pub(crate) fn emit_required_new_target_realm_ordinary_prototype(",
        "    pub(super) fn emit_install_resolved_realm_ordinary_prototype(",
    );
    let required_error_owner = bounded(
        ERRORS_SOURCE,
        "    pub(crate) fn emit_new_target_prototype_to_locals(",
        "    pub(crate) fn emit_aggregate_error_new_target_prototype_to_local(",
    );
    let promise_current_owner = bounded(
        PROMISE_SOURCE,
        "    fn emit_promise_job_callback_realm_to_local(",
        "    fn emit_promise_reaction_job_realm_to_local(",
    );
    let standard_owner = STANDARD_SOURCE
        .split_once("    pub(crate) fn compile_standard_builtin(")
        .expect("compile_standard_builtin owner")
        .1
        .rsplit_once("\n}")
        .expect("compile_standard_builtin impl end")
        .0;
    for (owner, pair) in [
        (generic_construct_owner, generic_construct),
        (required_ordinary_owner, required_ordinary),
        (required_error_owner, required_error),
        (promise_current_owner, promise_current),
        (standard_owner, typed_array),
    ] {
        let owner = normalize_rust(owner);
        assert_eq!(owner.code.matches(pair).count(), 1, "{pair}");
        assert_eq!(owner.routes.matches(".emit_get_function_realm(").count(), 1);
        assert_eq!(
            owner
                .routes
                .matches(".emit_route_function_realm_result(")
                .count(),
            1
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_route_in_rust_sources(&source_root, ".emit_get_function_realm("),
        5
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, ".emit_route_function_realm_result("),
        5
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "FunctionBuilder::emit_get_function_realm"),
        0
    );
    assert_eq!(
        count_route_in_rust_sources(
            &source_root,
            "FunctionBuilder::emit_route_function_realm_result",
        ),
        0
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "FunctionRealmRevokedRoute::UseCurrentRealm",),
        2
    );
    assert_eq!(
        count_route_in_rust_sources(
            &source_root,
            "FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn",
        ),
        4
    );
    assert_eq!(
        count_route_in_rust_sources(
            &source_root,
            "FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch",
        ),
        2
    );
}
