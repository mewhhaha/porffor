use std::fs;
use std::path::Path;

const INSTALLER_SOURCE: &str = include_str!("../src/intrinsics/function.rs");
const FUNCTION_BODY_SOURCE: &str = include_str!("../src/builtins/function.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const DEFINE_PROPERTY_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const HAS_INSTANCE_SOURCE: &str = include_str!("../src/operations/has_instance.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_function_prototype_symbol_has_instance.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/function-prototype-symbol-has-instance.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "{earlier} must precede {later}");
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

#[test]
fn intrinsic_is_one_rooted_nonconstructable_catalog_identity() {
    let catalog = bounded(
        CATALOG_SOURCE,
        "    FunctionPrototypeSymbolHasInstance {",
        "\n}\n\nimpl StandardBuiltinId {",
    );
    assert!(catalog.contains("=> BUILTIN_FUNCTION_PROTOTYPE_SYMBOL_HAS_INSTANCE_FUNCTION_ID"));
    assert!(catalog.contains("debug: \"Function.prototype[Symbol.hasInstance]\""));
    assert!(catalog.contains("flags: []"));
    assert!(catalog.contains("installer: None"));
    assert!(catalog.contains("native: \"[Symbol.hasInstance]\""));

    let roots = bounded(
        PLANNING_SOURCE,
        "        if builtin == StandardBuiltinId::FunctionConstructor {",
        "        if builtin == StandardBuiltinId::DisposableStackConstructor {",
    );
    assert_eq!(
        roots
            .matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance,")
            .count(),
        1
    );
    assert!(roots.contains("self.require_standard_builtin(dependency)"));

    let length = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn standard_builtin_length(builtin: StandardBuiltinId) -> u64 {",
        "pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {",
    );
    assert!(length.contains("StandardBuiltinId::FunctionPrototypeSymbolHasInstance"));
    assert!(length.contains("=> 1,"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::FunctionPrototypeSymbolHasInstance => {",
        "            StandardBuiltinId::FunctionPrototypeCall => {",
    );
    assert_eq!(
        dispatch
            .matches("self.emit_function_prototype_symbol_has_instance_builtin(function)?")
            .count(),
        1
    );

    let body = bounded(
        FUNCTION_BODY_SOURCE,
        "            FunctionBuiltin::PrototypeSymbolHasInstance => {",
        "            FunctionBuiltin::PrototypeCall => {",
    );
    assert_eq!(body.matches("self.emit_builtin_arg_to_locals(").count(), 1);
    assert_eq!(
        body.matches("self.emit_ordinary_has_instance_from_locals(")
            .count(),
        1
    );
    assert!(body.contains("ValueKind::Boolean.tag()"));
    assert!(!body.contains("emit_instanceof_operator_from_locals"));
}

#[test]
fn entry_realm_installs_the_symbol_property_with_all_false_attributes() {
    let install = bounded(
        INSTALLER_SOURCE,
        "        let has_instance_meta = self",
        "        self.release_temp_local(prototype_object_local);",
    );
    assert_eq!(
        install
            .matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance.function_id()")
            .count(),
        1
    );
    assert!(install.contains("lila_ir::WellKnownSymbol::HasInstance.description()"));
    assert!(install.contains("property_key_symbol_payload"));
    assert!(install.contains("self.emit_function_value_payload(&has_instance_meta, function)?"));
    assert!(install.contains("ValueKind::Function.tag()"));
    assert!(install.contains(
        "prototype_object_local,\n            key_local,\n            payload_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert_before(
        install,
        "property_key_symbol_payload",
        "self.emit_object_append_data_property_with_flags(",
    );
}

#[test]
fn created_realm_installs_a_fresh_realm_local_function_through_the_typed_context() {
    let meta = bounded(
        HOST_SOURCE,
        "        let function_prototype_has_instance_meta = self",
        "        let object_meta = self",
    );
    assert_eq!(
        meta.matches("StandardBuiltinId::FunctionPrototypeSymbolHasInstance.function_id()")
            .count(),
        1
    );

    let install = bounded(
        HOST_SOURCE,
        "        let has_instance_payload_local = self.reserve_temp_local();",
        "        for (_, prototype_local) in &typed_array_prototype_locals {",
    );
    for operation in [
        "self.emit_function_value_payload_in_realm(",
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "lila_ir::WellKnownSymbol::HasInstance",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    ] {
        assert!(
            install.contains(operation),
            "missing created-realm step: {operation}"
        );
    }
    assert!(install.contains(
        "has_instance_payload_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert_before(
        install,
        "self.emit_function_value_payload_in_realm(",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    );
    assert_before(
        install,
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "self.emit_define_realm_function_prototype_symbol_data_with_flags(",
    );

    let helper = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_define_realm_function_prototype_symbol_data_with_flags(",
        "    pub(crate) fn emit_bind_realm_function_constructor_prototype(",
    );
    assert!(helper.contains("context: &RealmFunctionMaterializationContext"));
    assert!(helper.contains("symbol: lila_ir::WellKnownSymbol"));
    assert!(helper.contains("property_key_symbol_payload(symbol.description())"));
    assert!(helper.contains("context.function_prototype_local"));
    assert!(!helper.contains("FUNCTION_PROTOTYPE_GLOBAL_INDEX"));
}

#[test]
fn has_instance_dispatch_is_a_closed_noncopyable_two_entry_domain() {
    assert_eq!(
        OPERATIONS_SOURCE.matches("\nmod has_instance;\n").count(),
        1
    );
    assert!(!OPERATIONS_SOURCE.contains("pub mod has_instance;"));
    assert!(!OPERATIONS_SOURCE.contains("pub(crate) mod has_instance;"));
    assert!(HAS_INSTANCE_SOURCE.starts_with("use super::*;\n\n"));

    for declaration in [
        "struct HasInstanceValueLocals {",
        "enum HasInstanceRequestLocals {",
        "enum HasInstanceRuntimeState {",
    ] {
        assert_eq!(
            HAS_INSTANCE_SOURCE.matches(declaration).count(),
            1,
            "child must own exactly one {declaration}"
        );
        assert_eq!(
            OPERATIONS_SOURCE.matches(declaration).count(),
            0,
            "parent must not own {declaration}"
        );
    }

    let builder = bounded(
        HAS_INSTANCE_SOURCE,
        "impl<'a> FunctionBuilder<'a> {",
        "\n}\n",
    );
    assert_eq!(builder.matches("    pub(crate) fn ").count(), 3);
    assert_eq!(builder.matches("    fn ").count(), 1);
    for method in [
        "emit_instanceof_i32",
        "emit_instanceof_operator_from_locals",
        "emit_ordinary_has_instance_from_locals",
    ] {
        let declaration = format!("    pub(crate) fn {method}(");
        assert_eq!(builder.matches(&declaration).count(), 1);
        assert_eq!(OPERATIONS_SOURCE.matches(&declaration).count(), 0);
    }
    assert_eq!(
        builder.matches("    fn emit_has_instance_request(").count(),
        1
    );
    assert_eq!(
        OPERATIONS_SOURCE
            .matches("    fn emit_has_instance_request(")
            .count(),
        0
    );

    let value = bounded(
        HAS_INSTANCE_SOURCE,
        "struct HasInstanceValueLocals {",
        "\n}",
    );
    assert_eq!(
        value.split_whitespace().collect::<String>(),
        "payload:u32,tag:u32,"
    );
    let request = bounded(
        HAS_INSTANCE_SOURCE,
        "enum HasInstanceRequestLocals {",
        "\n}",
    );
    assert_eq!(
        request.split_whitespace().collect::<String>(),
        "InstanceofOperator{object:HasInstanceValueLocals,constructor:HasInstanceValueLocals,},OrdinaryHasInstance{constructor:HasInstanceValueLocals,object:HasInstanceValueLocals,},"
    );
    assert!(!request.contains("bool"));
    assert!(!request.contains("_ =>"));
    assert!(HAS_INSTANCE_SOURCE.contains("#[must_use]\nenum HasInstanceRequestLocals"));
    assert!(!HAS_INSTANCE_SOURCE.contains("impl Copy for HasInstanceRequestLocals"));

    let lexical_probe = r###"
        // HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code()
        "HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code()";
        r#"HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code()"#;
        struct r#HasInstanceRuntimeState;
        value./* split route */r#runtime_code();
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "HasInstanceRuntimeState"),
        1
    );
    assert_eq!(lexical_probe.routes.matches(".runtime_code()").count(), 1);

    let runtime_authority = bounded(
        HAS_INSTANCE_SOURCE,
        "        object: HasInstanceValueLocals,\n    },\n}\n\n",
        "\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(
        normalize_rust(runtime_authority).code,
        concat!(
            "enumHasInstanceRuntimeState{InstanceofOperator,OrdinaryHasInstance,}",
            "implHasInstanceRuntimeState{constfnruntime_code(&self)->i64{",
            "matchself{Self::InstanceofOperator=>0,Self::OrdinaryHasInstance=>1,}}}"
        )
    );
    let normalized_source = normalize_rust(HAS_INSTANCE_SOURCE);
    assert!(!normalized_source
        .code
        .contains("forHasInstanceRuntimeState"));
    assert!(!normalized_source.code.contains("#[repr"));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "HasInstanceRuntimeState"),
        7,
        "the declaration, projection, two request mappings and three runtime-state selections must be the complete source census"
    );
    assert_eq!(
        normalized_source
            .routes
            .matches("HasInstanceRuntimeState::InstanceofOperator")
            .count(),
        3
    );
    assert_eq!(
        normalized_source
            .routes
            .matches("HasInstanceRuntimeState::OrdinaryHasInstance")
            .count(),
        2
    );
    assert_eq!(
        normalized_source.routes.matches(".runtime_code()").count(),
        4,
        "the selected initial state and all three gate/transition constants must use the exhaustive projection"
    );
    assert!(!normalized_source.routes.contains("stateasi64"));
    assert!(!normalized_source
        .routes
        .contains("HasInstanceRuntimeState::InstanceofOperatorasi64"));
    assert!(!normalized_source
        .routes
        .contains("HasInstanceRuntimeState::OrdinaryHasInstanceasi64"));

    let operator_wrapper = bounded(
        HAS_INSTANCE_SOURCE,
        "    pub(crate) fn emit_instanceof_operator_from_locals(",
        "    pub(crate) fn emit_ordinary_has_instance_from_locals(",
    );
    assert!(operator_wrapper.contains("HasInstanceRequestLocals::InstanceofOperator"));
    assert!(operator_wrapper.contains("object: HasInstanceValueLocals::new("));
    assert!(operator_wrapper.contains("constructor: HasInstanceValueLocals::new("));

    let ordinary_wrapper = bounded(
        HAS_INSTANCE_SOURCE,
        "    pub(crate) fn emit_ordinary_has_instance_from_locals(",
        "    fn emit_has_instance_request(",
    );
    assert!(ordinary_wrapper.contains("HasInstanceRequestLocals::OrdinaryHasInstance"));
    assert!(ordinary_wrapper.contains("constructor: HasInstanceValueLocals::new("));
    assert!(ordinary_wrapper.contains("object: HasInstanceValueLocals::new("));

    let emitter = bounded(
        HAS_INSTANCE_SOURCE,
        "    fn emit_has_instance_request(",
        "\n    }\n}",
    );
    assert!(emitter.contains("let (state, constructor, object) = match request"));
    assert!(!emitter.contains("_ =>"));
    assert!(emitter.contains("property_key_symbol_payload(\"Symbol.hasInstance\")"));
    assert!(emitter.contains("self.emit_indirect_call_from_locals("));
    assert!(emitter.contains("self.emit_to_boolean_payload_from_tagged_locals("));
    assert!(emitter.contains("FUNCTION_FLAG_BOUND"));
    assert!(emitter.contains("self.strings.payload(\"prototype\")"));
    assert!(emitter.contains("self.emit_object_read("));
    assert!(emitter.contains("self.emit_object_get_prototype_of("));

    let normalized_emitter = normalize_rust(emitter);
    assert_eq!(
        exact_identifier_count(&normalized_emitter.identifiers, "state_local"),
        6,
        "the reservation, three writes, one comparison read and final release must be the complete state-local lifecycle"
    );
    let local_reservations = bounded(
        emitter,
        ") -> Result<(), EmitError> {",
        "\n\n        let (state, constructor, object) = match request",
    );
    assert_eq!(
        normalize_rust(local_reservations).code,
        concat!(
            "letstate_local=self.reserve_temp_local();",
            "letconstructor_payload_local=self.reserve_temp_local();",
            "letconstructor_tag_local=self.reserve_temp_local();",
            "letobject_payload_local=self.reserve_temp_local();",
            "letobject_tag_local=self.reserve_temp_local();",
            "letkey_local=self.reserve_temp_local();",
            "lethandler_payload_local=self.reserve_temp_local();",
            "lethandler_tag_local=self.reserve_temp_local();",
            "letcall_result_payload_local=self.reserve_temp_local();",
            "letcall_result_tag_local=self.reserve_temp_local();",
            "letflags_local=self.reserve_temp_local();",
            "letbound_record_local=self.reserve_temp_local();",
            "letprototype_payload_local=self.reserve_temp_local();",
            "letprototype_tag_local=self.reserve_temp_local();",
            "letsearch_payload_local=self.reserve_temp_local();",
            "letsearch_tag_local=self.reserve_temp_local();",
            "letnext_prototype_payload_local=self.reserve_temp_local();",
            "letnext_prototype_tag_local=self.reserve_temp_local();"
        )
    );
    let normalized_emitter = &normalized_emitter.code;
    assert!(normalized_emitter.contains(concat!(
        "let(state,constructor,object)=matchrequest{",
        "HasInstanceRequestLocals::InstanceofOperator{object,constructor,}=>",
        "(HasInstanceRuntimeState::InstanceofOperator,constructor,object,),",
        "HasInstanceRequestLocals::OrdinaryHasInstance{constructor,object,}=>",
        "(HasInstanceRuntimeState::OrdinaryHasInstance,constructor,object,),};",
        "function.instruction(&Instruction::I64Const(state.runtime_code()));",
        "function.instruction(&Instruction::LocalSet(state_local));",
        "function.instruction(&Instruction::LocalGet(constructor.payload));"
    )));
    assert!(normalized_emitter.contains(concat!(
        "function.instruction(&Instruction::LocalGet(state_local));",
        "function.instruction(&Instruction::I64Const(",
        "HasInstanceRuntimeState::InstanceofOperator.runtime_code(),));",
        "function.instruction(&Instruction::I64Eq);",
        "self.open_frame(ControlFrameKind::If,function);"
    )));
    assert!(normalized_emitter.contains(concat!(
        "function.instruction(&Instruction::I64Const(",
        "HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code(),));",
        "function.instruction(&Instruction::LocalSet(state_local));",
        "self.emit_branch_to_target(dispatch,function);"
    )));
    assert!(normalized_emitter.contains(concat!(
        "self.load_i64_to_local_from_offset(bound_record_local,",
        "HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,constructor_tag_local,function,);",
        "function.instruction(&Instruction::I64Const(",
        "HasInstanceRuntimeState::InstanceofOperator.runtime_code(),));",
        "function.instruction(&Instruction::LocalSet(state_local));",
        "self.emit_branch_to_target(dispatch,function);"
    )));
    assert_eq!(
        normalized_emitter
            .matches("Instruction::LocalSet(state_local)")
            .count(),
        3
    );
    assert_eq!(
        normalized_emitter
            .matches("Instruction::LocalGet(state_local)")
            .count(),
        1
    );
    assert!(normalized_emitter.ends_with(concat!(
        "self.release_temp_local(next_prototype_tag_local);",
        "self.release_temp_local(next_prototype_payload_local);",
        "self.release_temp_local(search_tag_local);",
        "self.release_temp_local(search_payload_local);",
        "self.release_temp_local(prototype_tag_local);",
        "self.release_temp_local(prototype_payload_local);",
        "self.release_temp_local(bound_record_local);",
        "self.release_temp_local(flags_local);",
        "self.release_temp_local(call_result_tag_local);",
        "self.release_temp_local(call_result_payload_local);",
        "self.release_temp_local(handler_tag_local);",
        "self.release_temp_local(handler_payload_local);",
        "self.release_temp_local(key_local);",
        "self.release_temp_local(object_tag_local);",
        "self.release_temp_local(object_payload_local);",
        "self.release_temp_local(constructor_tag_local);",
        "self.release_temp_local(constructor_payload_local);",
        "self.release_temp_local(state_local);",
        "Ok(())"
    )));

    let absent_handler = bounded(
        emitter,
        "function.instruction(&Instruction::I32Or);",
        "self.emit_is_callable_i32(handler_tag_local, handler_payload_local, function)?;",
    );
    assert!(absent_handler
        .contains("self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local"));
    assert!(absent_handler.contains("Right-hand side of 'instanceof' is not callable"));
    assert_before(
        absent_handler,
        "self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local",
        "HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code()",
    );
    assert_before(
        emitter,
        "FUNCTION_FLAG_BOUND",
        "self.strings.payload(\"prototype\")",
    );

    let define_property = bounded(
        DEFINE_PROPERTY_SOURCE,
        "    pub(in crate::builtins) fn compile_object_define_property_builtin(",
        "\n}",
    );
    assert!(define_property.contains("self.emit_object_define_entry_validated("));
    assert!(!define_property.contains("FUNCTION_FLAG_BOUND | FUNCTION_FLAG_IS_HTMLDDA"));
}

#[test]
fn consumer_fixture_covers_the_complete_nondynamic_runtime_boundary() {
    for witness in [
        "realm-local intrinsic identity",
        "label + \" writable\"",
        "label + \" enumerable\"",
        "label + \" configurable\"",
        "undefined receiver",
        "number candidate",
        "positive chain",
        "negative chain",
        "bound target recursion",
        "bound custom handler result",
        "call-only function starts without prototype",
        "poisoned prototype abrupt",
        "default prototype stays non-configurable",
        "configurable prototype changes kind",
        "non-object prototype TypeError",
        "Proxy GetPrototypeOf abrupt",
        "Proxy chain abrupt",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing fixture witness: {witness}"
        );
    }
    assert!(FIXTURE.contains("__lilaCreateRealm"));
    assert!(CONTRACT.contains("The focused Test262 directory contains eleven files."));
    assert!(CONTRACT.contains("Dynamic Function source"));
    assert!(CONTRACT.contains("generation remains a non-claim"));
}
