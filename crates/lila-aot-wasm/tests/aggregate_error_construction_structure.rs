use std::fs;
use std::path::Path;

const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const AGGREGATE_ERROR_PREPARATION_SOURCE: &str =
    include_str!("../src/builtins/errors/aggregate_error_preparation.rs");
const MESSAGE_CONSTRUCTOR_SOURCE: &str = include_str!("../src/builtins/errors/constructor.rs");
const PROMISE_ANY_ERROR_SOURCE: &str = include_str!("../src/builtins/errors/promise_any.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
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

fn count_in_rust_sources(root: &Path, fragment: &str) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let path = entry.expect("failed to read Rust source entry").path();
        if path.is_dir() {
            count += count_in_rust_sources(&path, fragment);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            count += source.matches(fragment).count();
        }
    }
    count
}

#[test]
fn aggregate_error_construction_has_a_closed_cause_options_role() {
    let lexical_probe = r###"
        ErrorCauseOptionsArgument /* nested /* ignored */ comment */ :: r#MessageError;
        let r#options_argument = ErrorCauseOptionsArgument::AggregateError;
        // ErrorCauseOptionsArgument options_argument
        let normal = "ErrorCauseOptionsArgument options_argument";
        let byte = b"ErrorCauseOptionsArgument";
        let c_string = c"options_argument";
        let raw = r#"ErrorCauseOptionsArgument"#;
        let raw_byte = br#"options_argument"#;
        let raw_c = cr#"ErrorCauseOptionsArgument"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "ErrorCauseOptionsArgument::MessageError;",
            "letoptions_argument=ErrorCauseOptionsArgument::AggregateError;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "ErrorCauseOptionsArgument"),
        2
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "options_argument"),
        1
    );

    let errors = normalize_rust(ERRORS_SOURCE);
    let domain = bounded(
        &errors.code,
        "fnnative_error_kind(",
        "#[must_use=\"Promise.any AggregateError allocation context must be consumed\"]pub(super)structPromiseAnyAggregateErrorAllocationContext",
    );
    assert_eq!(
        domain,
        concat!(
            "name:&str)->Result<NativeErrorKind,EmitError>{",
            "NativeErrorKind::from_str(name).ok_or_else(||{EmitError::unsupported(format!(",
            "\"internal wasm-aot error emitter received unknown native error name `{name}`\"))})}",
            "enumErrorCauseOptionsArgument{MessageError,AggregateError,}",
            "implErrorCauseOptionsArgument{fnindex(self)->usize{matchself{",
            "Self::MessageError=>1,Self::AggregateError=>2,}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ErrorCauseOptionsArgument"),
        6,
        "the declaration, impl, typed parameter and three producers own every type use"
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "options_argument"),
        2,
        "the typed parameter and its sole consuming index projection own every local use"
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ErrorCauseOptionsArgument::MessageError"),
        2
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "ErrorCauseOptionsArgument::AggregateError"),
        1
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "emit_install_error_cause_from_arg"),
        4,
        "one definition and three exact producer calls own the complete installer route"
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !errors
                .routes
                .contains(&format!("impl{capability}forErrorCauseOptionsArgument")),
            "ErrorCauseOptionsArgument must not acquire {capability}"
        );
    }

    let constructor = normalize_rust(MESSAGE_CONSTRUCTOR_SOURCE);
    let message_error_call = concat!(
        "self.emit_install_error_cause_from_arg(self.result_local,",
        "ErrorCauseOptionsArgument::MessageError,function,)?;"
    );
    assert_eq!(constructor.code.matches(message_error_call).count(), 2);
    let absent_message_cause = concat!(
        "self.emit_alloc_error_instance_from_locals(&prototype,None,self.result_local,",
        "self.result_tag_local,function,)?;",
        "self.emit_install_error_cause_from_arg(self.result_local,",
        "ErrorCauseOptionsArgument::MessageError,function,)?;",
        "function.instruction(&Instruction::Else);"
    );
    assert_eq!(constructor.code.matches(absent_message_cause).count(), 1);
    let present_message_cause = concat!(
        "self.emit_alloc_error_instance_from_locals(&prototype,Some(message_payload_local),",
        "self.result_local,self.result_tag_local,function,)?;",
        "self.emit_install_error_cause_from_arg(self.result_local,",
        "ErrorCauseOptionsArgument::MessageError,function,)?;",
        "function.instruction(&Instruction::End);",
        "self.release_error_constructor_prototype(prototype);"
    );
    assert_eq!(constructor.code.matches(present_message_cause).count(), 1);
    let aggregate_error_call = concat!(
        "self.emit_install_error_cause_from_arg(object_local,",
        "ErrorCauseOptionsArgument::AggregateError,function,)?;"
    );
    let preparation = normalize_rust(AGGREGATE_ERROR_PREPARATION_SOURCE);
    assert_eq!(preparation.code.matches(aggregate_error_call).count(), 1);

    let installer = bounded(
        &errors.code,
        "fnemit_install_error_cause_from_arg(",
        "pub(crate)fnemit_alloc_suppressed_error_instance_from_locals(",
    );
    assert_eq!(
        installer,
        concat!(
            "&mutself,error_object_local:u32,options_argument:ErrorCauseOptionsArgument,",
            "function:&mutFunction,)->Result<(),EmitError>{",
            "letoptions_payload_local=self.reserve_temp_local();",
            "letoptions_tag_local=self.reserve_temp_local();",
            "letcause_key_local=self.reserve_temp_local();",
            "lethas_cause_local=self.reserve_temp_local();",
            "letcause_payload_local=self.reserve_temp_local();",
            "letcause_tag_local=self.reserve_temp_local();",
            "self.emit_builtin_arg_to_locals(options_argument.index(),options_payload_local,",
            "options_tag_local,function,);",
            "self.emit_is_heap_object_like_tag_i32(options_tag_local,function);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "function.instruction(&Instruction::I64Const(self.strings.payload(\"cause\")));",
            "function.instruction(&Instruction::LocalSet(cause_key_local));",
            "self.emit_object_has_property_i32(options_payload_local,options_tag_local,",
            "cause_key_local,has_cause_local,function,)?;",
            "function.instruction(&Instruction::LocalGet(has_cause_local));",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64Ne);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_object_read(options_payload_local,options_tag_local,options_payload_local,",
            "options_tag_local,cause_key_local,cause_payload_local,cause_tag_local,function,)?;",
            "self.emit_propagate_throw_from_locals_if_needed(cause_payload_local,",
            "cause_tag_local,function,)?;",
            "self.emit_object_define_data(error_object_local,cause_key_local,cause_payload_local,",
            "cause_tag_local,function,)?;",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "self.release_temp_local(cause_tag_local);",
            "self.release_temp_local(cause_payload_local);",
            "self.release_temp_local(has_cause_local);",
            "self.release_temp_local(cause_key_local);",
            "self.release_temp_local(options_tag_local);",
            "self.release_temp_local(options_payload_local);Ok(())}"
        )
    );
    for forbidden in [
        "options_arg_index",
        "options_argument.clone(",
        "&options_argument",
        "options_argument==",
        "options_argument!=",
        "matches!(options_argument",
        "_=>",
        "unreachable!",
        "options_argumentas",
    ] {
        assert!(
            !installer.contains(forbidden),
            "forbidden role route `{forbidden}`"
        );
    }
}

#[test]
fn aggregate_error_construction_requires_a_prepared_object_before_errors() {
    assert_eq!(
        ERRORS_SOURCE
            .matches("\nmod aggregate_error_preparation;\n")
            .count(),
        1
    );
    assert!(!ERRORS_SOURCE.contains("pub mod aggregate_error_preparation;"));
    assert!(!ERRORS_SOURCE.contains("aggregate_error_preparation::"));
    assert!(!ERRORS_SOURCE.contains("PreparedAggregateErrorLocal"));
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("PreparedAggregateErrorLocal")
            .count(),
        7
    );
    assert!(!MESSAGE_CONSTRUCTOR_SOURCE.contains("PreparedAggregateErrorLocal"));
    assert!(!PROMISE_SOURCE.contains("PreparedAggregateErrorLocal"));

    let declaration = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("pub(super) struct PreparedAggregateErrorLocal {")
        .expect("prepared AggregateError token")
        .0
        .rsplit_once("\n\n")
        .expect("prepared token attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    let fields = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("pub(super) struct PreparedAggregateErrorLocal {")
        .expect("prepared AggregateError token fields")
        .1
        .split_once('}')
        .expect("prepared AggregateError token fields end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(fields, ["object: u32,"]);
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !AGGREGATE_ERROR_PREPARATION_SOURCE.contains(&format!(
                "impl {capability} for PreparedAggregateErrorLocal"
            )),
            "PreparedAggregateErrorLocal must not acquire manual {capability}"
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PreparedAggregateErrorLocal"),
        7,
        "the private child must own every prepared AggregateError type use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "aggregate_error_preparation::"),
        0,
        "the prepared AggregateError owner must have no import or re-export",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_alloc_aggregate_error_instance_from_locals",
        ),
        0,
        "the old untyped combined allocator must have no definition or caller",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_promise_any_aggregate_error_from_context(",
        ),
        3,
        "the Promise.any-only wrapper must have one definition and two callers",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_prepare_aggregate_error_instance("),
        2,
        "the constructor producer must have one definition and only its constructor caller",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_prepare_promise_any_aggregate_error_instance(",
        ),
        2,
        "the private Promise.any producer must have one definition and only its wrapper caller",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_finish_aggregate_error_instance("),
        3,
        "the finalizer must have one definition, one constructor caller and one Promise.any caller",
    );
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("\n    pub(super) fn emit_prepare_promise_any_aggregate_error_instance(")
            .count(),
        1,
        "only the private child may expose the Promise.any token producer to its parent",
    );
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("\n    pub(super) fn emit_finish_aggregate_error_instance(")
            .count(),
        1,
        "only the private child may expose the shared token consumer to its parent",
    );
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("\n    pub(super) fn emit_prepare_aggregate_error_instance(")
            .count(),
        1,
        "only the private child may expose the constructor token producer to its parent",
    );
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("Ok(PreparedAggregateErrorLocal {")
            .count(),
        2,
    );
    assert_eq!(
        AGGREGATE_ERROR_PREPARATION_SOURCE
            .matches("let PreparedAggregateErrorLocal {")
            .count(),
        1,
    );

    let prepare = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("fn emit_prepare_aggregate_error_instance(")
        .expect("AggregateError preparation phase")
        .1
        .split_once("pub(super) fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("AggregateError preparation phase end")
        .0;
    assert!(prepare.contains(") -> Result<PreparedAggregateErrorLocal, EmitError> {"));
    assert_eq!(
        prepare.matches("Ok(PreparedAggregateErrorLocal {").count(),
        1
    );

    let promise_prepare = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("Promise.any AggregateError preparation phase")
        .1
        .split_once("pub(super) fn emit_finish_aggregate_error_instance(")
        .expect("Promise.any AggregateError preparation phase end")
        .0;
    assert!(promise_prepare.contains(") -> Result<PreparedAggregateErrorLocal, EmitError> {"));
    assert_eq!(
        promise_prepare
            .matches("Ok(PreparedAggregateErrorLocal {")
            .count(),
        1
    );
    assert_eq!(
        promise_prepare
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1
    );
    assert_eq!(
        promise_prepare
            .matches("OBJECT_INTERNAL_BRAND_ERROR")
            .count(),
        1
    );
    for forbidden in [
        "emit_value_to_string_payload(",
        "emit_install_error_cause_from_arg(",
        "emit_object_define_data(",
        "strings.payload(\"message\")",
        "strings.payload(\"errors\")",
    ] {
        assert!(
            !promise_prepare.contains(forbidden),
            "Promise.any preparation must not run constructor-only phase {forbidden}",
        );
    }
    assert_eq!(
        without_whitespace(promise_prepare),
        without_whitespace(
            r#"
            &mut self,
                prototype_payload_local: u32,
                function: &mut Function,
            ) -> Result<PreparedAggregateErrorLocal, EmitError> {
                let object_local = self.reserve_temp_local();
                self.emit_alloc_plain_object_with_prototype(
                    Some(prototype_payload_local),
                    None,
                    function
                )?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    OBJECT_INTERNAL_BRAND_ERROR,
                    function,
                );
                Ok(PreparedAggregateErrorLocal {
                    object: object_local,
                })
            }
            "#,
        ),
        "Promise.any may prepare only a fresh branded AggregateError object",
    );

    let finish = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("fn emit_finish_aggregate_error_instance(")
        .expect("AggregateError finalization phase")
        .1
        .rsplit_once("\n    }\n}")
        .expect("AggregateError finalization phase end")
        .0;
    assert_eq!(
        finish
            .matches("prepared: PreparedAggregateErrorLocal")
            .count(),
        1
    );
    assert_eq!(
        finish.matches("let PreparedAggregateErrorLocal {").count(),
        1
    );

    let promise_wrapper = PROMISE_ANY_ERROR_SOURCE
        .split_once("pub(in crate::builtins) fn emit_promise_any_aggregate_error_from_context(")
        .expect("Promise.any AggregateError wrapper")
        .1
        .split_once("\n    }\n}")
        .expect("Promise.any AggregateError wrapper end")
        .0;
    assert_eq!(
        promise_wrapper
            .matches("emit_prepare_promise_any_aggregate_error_instance(")
            .count(),
        1
    );
    assert_eq!(
        promise_wrapper
            .matches("emit_finish_aggregate_error_instance(")
            .count(),
        1
    );
    assert_before(
        promise_wrapper,
        "emit_prepare_promise_any_aggregate_error_instance(",
        "emit_finish_aggregate_error_instance(",
    );
    assert_eq!(
        normalize_rust(promise_wrapper).code,
        normalize_rust(
            r#"
            &mut self,
                errors_payload_local: u32,
                context: PromiseAnyAggregateErrorAllocationContext,
                payload_local: u32,
                tag_local: u32,
                function: &mut Function,
            ) -> Result<(), EmitError> {
                let prepared = self
                    .emit_prepare_promise_any_aggregate_error_instance(
                        context.prototype_local,
                        function
                    )?;
                self.emit_finish_aggregate_error_instance(
                    prepared,
                    errors_payload_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(context.prototype_local);
                Ok(())
            "#,
        )
        .code,
        "the builtins-visible Promise.any boundary must consume its context and private token",
    );

    let reject_element = PROMISE_SOURCE
        .split_once("pub(crate) fn emit_promise_any_reject_element(")
        .expect("Promise.any reject-element body")
        .1
        .split_once("pub(crate) fn emit_promise_race(")
        .expect("Promise.any reject-element body end")
        .0;
    assert_eq!(
        reject_element
            .matches("emit_promise_any_aggregate_error_from_context(")
            .count(),
        1
    );
    assert!(
        without_whitespace(reject_element).contains(&without_whitespace(
            r#"
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        let allocation_context =
            self.emit_self_backed_promise_any_aggregate_error_allocation_context(function);
        self.emit_promise_any_aggregate_error_from_context(
            errors_payload_local,
            allocation_context,
            aggregate_payload_local,
            aggregate_tag_local,
            function,
        )?;
        "#,
        ))
    );

    let combinator = PROMISE_SOURCE
        .split_once("fn emit_promise_combinator(")
        .expect("Promise combinator body")
        .1
        .split_once("pub(crate) fn emit_promise_resolving_function(")
        .expect("Promise combinator body end")
        .0;
    assert_eq!(
        combinator
            .matches("emit_promise_any_aggregate_error_from_context(")
            .count(),
        1
    );
    assert!(without_whitespace(combinator).contains(&without_whitespace(
        r#"
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        match mode {
            PromiseCombinatorMode::Values | PromiseCombinatorMode::SettledRecords => {
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(next_value_tag_local));
                self.emit_function_or_proxy_call_leave_throw_completion(
                    resolve_payload_local,
                    resolve_tag_local,
                    undefined_payload_local,
                    undefined_tag_local,
                    &[(values_payload_local, next_value_tag_local)],
                    call_payload_local,
                    call_tag_local,
                    function,
                )?;
            }
            PromiseCombinatorMode::FirstFulfillment => {
                let allocation_context =
                    self.emit_promise_combinator_aggregate_error_allocation_context(function);
                self.emit_promise_any_aggregate_error_from_context(
                values_payload_local,
                allocation_context,
                next_value_payload_local,
                next_value_tag_local,
                function,
            )?;
        "#,
    )));

    let arm = ERRORS_SOURCE
        .split_once("NativeErrorKind::AggregateError => {")
        .expect("AggregateError constructor arm")
        .1
        .split_once("NativeErrorKind::SuppressedError => {")
        .expect("AggregateError constructor arm end")
        .0;
    assert_eq!(
        arm.matches("emit_prepare_aggregate_error_instance(")
            .count(),
        1
    );
    assert_eq!(
        arm.matches("emit_aggregate_error_iterable_to_list_payload(")
            .count(),
        1
    );
    assert_eq!(
        arm.matches("emit_finish_aggregate_error_instance(").count(),
        1
    );
    assert_before(
        arm,
        "emit_prepare_aggregate_error_instance(",
        "emit_aggregate_error_iterable_to_list_payload(",
    );
    assert_before(
        arm,
        "emit_aggregate_error_iterable_to_list_payload(",
        "emit_finish_aggregate_error_instance(",
    );
}

#[test]
fn aggregate_error_construction_emits_message_cause_then_errors() {
    let prepare = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("fn emit_prepare_aggregate_error_instance(")
        .expect("AggregateError preparation phase")
        .1
        .split_once("pub(super) fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("AggregateError preparation phase end")
        .0;
    assert_eq!(
        prepare
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1
    );
    assert_eq!(prepare.matches("OBJECT_INTERNAL_BRAND_ERROR").count(), 1);
    assert_eq!(prepare.matches("emit_value_to_string_payload(").count(), 1);
    assert_eq!(prepare.matches("strings.payload(\"message\")").count(), 1);
    assert_eq!(prepare.matches("emit_object_define_data(").count(), 1);
    assert_eq!(
        prepare
            .matches("emit_install_error_cause_from_arg(")
            .count(),
        1
    );
    assert_before(
        prepare,
        "emit_alloc_plain_object_with_prototype(",
        "OBJECT_INTERNAL_BRAND_ERROR",
    );
    assert_before(
        prepare,
        "OBJECT_INTERNAL_BRAND_ERROR",
        "emit_value_to_string_payload(",
    );
    let compact_prepare = without_whitespace(prepare);
    let compact_message_then_cause = without_whitespace(
        r#"
        function.instruction(&Instruction::LocalGet(message_arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(
            message_arg_payload_local,
            message_arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(message_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            message_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_install_error_cause_from_arg(
            object_local,
            ErrorCauseOptionsArgument::AggregateError,
            function,
        )?;
        "#,
    );
    assert!(
        compact_prepare.contains(&compact_message_then_cause),
        "the optional message conversion and definition must complete before cause installation"
    );

    let finish = AGGREGATE_ERROR_PREPARATION_SOURCE
        .split_once("fn emit_finish_aggregate_error_instance(")
        .expect("AggregateError finalization phase")
        .1
        .rsplit_once("\n    }\n}")
        .expect("AggregateError finalization phase end")
        .0;
    assert!(finish.contains("prepared: PreparedAggregateErrorLocal"));
    assert!(finish.contains("let PreparedAggregateErrorLocal"));
    assert_eq!(finish.matches("strings.payload(\"errors\")").count(), 1);
    assert_eq!(finish.matches("emit_object_define_data(").count(), 1);
    let compact_finish = without_whitespace(finish);
    let compact_publication = without_whitespace(
        r#"
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
        "#,
    );
    assert!(
        compact_finish.contains(&compact_publication),
        "errors definition must publish the object/tag pair before reverse local release"
    );
}
