use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/function.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const GC_VALUES: &str = include_str!("../src/gc_types/value.rs");
const CALL_DISPATCH: &str = include_str!("../src/functions/call_dispatch.rs");
const BOUND_RECORD: &str = include_str!("../src/functions/bound_function_record.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/function-prototype-receiver-ownership.md");
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
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

struct RustCode {
    normalized: String,
    identifiers: String,
}

fn rust_code(source: &str) -> RustCode {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut identifiers = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push_str(&source[offset..end]);
            identifiers.push(' ');
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
            assert_eq!(depth, 0, "unterminated block comment");
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
            normalized.push(character);
            identifiers.push(character);
        }
        offset += character.len_utf8();
    }
    RustCode {
        normalized,
        identifiers,
    }
}

// rustfmt may add a trailing comma to a multiline call. Both spellings
// retain the complete, ordered arguments; literals and tuple syntax are untouched.
fn call_count(source: &str, call: &str) -> usize {
    let (arguments, suffix) = if let Some(arguments) = call.strip_suffix(");") {
        (arguments, ";")
    } else {
        (call.strip_suffix(')').expect("a complete call pattern"), "")
    };
    source.matches(call).count() + source.matches(&format!("{arguments},){suffix}")).count()
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
            exact_identifier_count(&rust_code(&source).identifiers, identifier)
        })
        .sum()
}

#[test]
fn receiver_value_is_the_private_whole_non_copy_domain() {
    let lexical_probe = rust_code(
        r###"
        // ValueLocals
        ValueLocals /* nested /* ignored */ comment */;
        "ValueLocals"; b"ValueLocals";
        c"ValueLocals"; r"ValueLocals";
        br##"ValueLocals"##; cr#"ValueLocals"#;
        'V'; b'V'; 'lifetime;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "ValueLocals"),
        1
    );
    assert_eq!(call_count("call(value);call(value,);", "call(value);"), 2);
    assert_eq!(
        call_count("call(value,extra);call((value,));", "call(value);"),
        0
    );

    let value_fields = rust_code(bounded(
        GC_VALUES,
        "pub(crate) struct ValueLocals {",
        "\n}\nimpl ValueLocals {",
    ));
    assert_eq!(
        value_fields.normalized,
        "tag:I32Local,scalar:I64Local,reference:EqRefLocal,"
    );
    let value_header = rust_code(bounded(
        GC_VALUES,
        "pub(crate) enum ScalarValue {",
        "impl ValueLocals {",
    ));
    for forbidden in ["Clone", "Copy", "pubtag:", "pubscalar:", "pubreference:"] {
        assert!(
            !value_header.identifiers.contains(forbidden),
            "found `{forbidden}`"
        );
    }
    let value_code = rust_code(GC_VALUES);
    for forbidden in ["implCloneforValueLocals", "implCopyforValueLocals"] {
        assert!(
            !value_code.normalized.contains(forbidden),
            "found `{forbidden}`"
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for retired in [
        "FunctionPrototypeReceiverLocals",
        "emit_bound_function_invoker_builtin",
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, retired),
            0,
            "retired `{retired}`"
        );
    }
}

#[test]
fn five_prototype_operations_share_the_captured_whole_this_value() {
    let code = rust_code(SOURCE).normalized;
    let owner = bounded(
        &code,
        "fnemit_function_builtin(",
        "self.completion().copy_from(&result,function);",
    );
    let capture = "receiver.copy_from(self.body_entry_locals().expect(\"native Function entry is cached\").this_value(),function);";
    assert_eq!(call_count(owner, capture), 1);
    assert!(owner.contains("letreceiver=schema.reserve_value_local(function);"));
    assert!(
        owner
            .find("returnself.compile_function_constructor_builtin(function);")
            .unwrap()
            < owner.find("receiver.copy_from(").unwrap()
    );
    for variant in [
        "PrototypeSymbolHasInstance",
        "PrototypeCall",
        "PrototypeApply",
        "PrototypeBind",
        "PrototypeToString",
    ] {
        assert_eq!(
            owner
                .matches(&format!("FunctionBuiltin::{variant}=>{{"))
                .count(),
            1
        );
    }
    for forbidden in [
        "this_payload_local",
        "this_tag_local",
        "new_target",
        "receiver_payload_local",
        "receiver_tag_local",
    ] {
        assert!(!owner.contains(forbidden), "found `{forbidden}`");
    }
    assert!(code.contains("self.completion().copy_from(&result,function);result.clear(function);argument.clear(function);receiver.clear(function);"));
}

#[test]
fn operations_and_bound_dispatch_retain_complete_values_and_completions() {
    let code = rust_code(SOURCE).normalized;
    for (start, end, semantic_owner) in [
        (
            "PrototypeSymbolHasInstance",
            "PrototypeCall",
            "emit_ordinary_has_instance_from_locals(&receiver,&argument,&result,function)",
        ),
        (
            "PrototypeCall",
            "PrototypeApply",
            "emit_prepared_tail_call(&receiver,&argument,&arguments,function)",
        ),
        (
            "PrototypeApply",
            "PrototypeBind",
            "emit_prepared_tail_call(&receiver,&argument,&empty,function)",
        ),
        (
            "PrototypeBind",
            "PrototypeToString",
            "emit_alloc_bound_function_for_bind(&receiver,&arguments,&result,function)",
        ),
    ] {
        let branch = bounded(
            &code,
            &format!("FunctionBuiltin::{start}=>{{"),
            &format!("FunctionBuiltin::{end}=>{{"),
        );
        assert_eq!(
            call_count(branch, semantic_owner),
            1,
            "{start} must consume its captured whole receiver"
        );
    }
    let apply = bounded(
        &code,
        "FunctionBuiltin::PrototypeApply=>{",
        "FunctionBuiltin::PrototypeBind=>{",
    );
    assert_eq!(
        call_count(
            apply,
            "emit_prepared_tail_call(&receiver,&argument,arguments,function)"
        ),
        1
    );
    for owner in [
        bounded(
            &code,
            "FunctionBuiltin::PrototypeCall=>{",
            "FunctionBuiltin::PrototypeApply=>{",
        ),
        apply,
    ] {
        assert!(
            owner
                .find("emit_is_callable_i32(&receiver,function)")
                .unwrap()
                < owner.find("emit_prepared_tail_call(").unwrap()
        );
        assert!(!owner.contains("emit_function_or_proxy_call_with_argv("));
    }
    let to_string = bounded(
        &code,
        "FunctionBuiltin::PrototypeToString=>{",
        "self.completion().copy_from(&result,function);",
    );
    assert!(to_string.contains("receiver.cast_reference::<FunctionObject>(schema,function)"));
    assert!(to_string.contains("FunctionObjectSchema::TO_STRING"));
    assert!(to_string.contains("result.set_normal(&argument,function);"));

    let dispatch = rust_code(CALL_DISPATCH).normalized;
    let construct = bounded(
        &dispatch,
        "fnemit_plain_function_construct_dispatch(",
        "fnemit_function_constructor_entry_call(",
    );
    let call = bounded(
        &dispatch,
        "fnemit_plain_function_call_dispatch(",
        "fnemit_proxy_call_dispatch(",
    );
    for owner in [construct, call] {
        for signature in [
            "callee:&ValueLocals",
            "arguments:&GcLocal<ValueArray>",
            "result:&CompletionLocals",
        ] {
            assert!(
                owner.contains(signature),
                "whole dispatch signature `{signature}`"
            );
        }
        for operation in [
            "reference_type::<BoundFunction>(GcNullability::NonNullable)",
            "current.cast_reference::<BoundFunction>(schema,function)",
            "self.emit_load_bound_function_record(&bound,function)",
            "self.emit_concat_argument_vectors(record.arguments(),&list,function)",
            "current.copy_from(record.target(),function);",
            "record.clear(schema,function);",
        ] {
            assert!(
                owner.contains(operation),
                "direct bound dispatch `{operation}`"
            );
        }
    }
    assert!(construct.contains("new_target:&ValueLocals"));
    assert!(construct.contains("Instruction::RefEq"));
    assert!(construct.contains("actual_new_target.copy_from(record.target(),function);"));
    assert!(call.contains("this_value:&ValueLocals"));
    assert!(call.contains("receiver.copy_from(record.this_value(),function);"));
    let record = rust_code(BOUND_RECORD).normalized;
    assert!(record.contains("pub(crate)structBoundFunctionRecordLocals{target:ValueLocals,this_value:ValueLocals,arguments:GcLocal<ValueArray>,constructable:I32Local,}"));
    for field in ["TARGET", "THIS_VALUE", "ARGUMENTS", "CONSTRUCTABLE"] {
        assert!(record.contains(&format!("BoundFunctionSchema::{field}")));
    }
}

#[test]
fn contract_and_fixed_entries_record_the_current_receiver_authority() {
    for marker in [
        "paired Function prototype receiver authority",
        "cannot mix payload and tag sources",
        "function_prototype_receiver_ownership_structure",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "historical contract marker `{marker}`"
        );
        assert!(TASK.contains(marker), "historical task marker `{marker}`");
    }
    for marker in [
        "whole ValueLocals",
        "seven fixed Function entries",
        "direct GC BoundFunction",
        "Historical source checkpoint",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "current contract marker `{marker}`"
        );
    }
    let source = rust_code(SOURCE).normalized;
    // The preceding declaration may re-export more constructor helpers. The
    // semicolon still proves that no attribute, derive or visibility intervenes.
    let domain = bounded(
        &source,
        ";enumFunctionBuiltin{",
        "}implFunctionBuilder<'_>{",
    );
    assert_eq!(domain, "Constructor,Prototype,PrototypeSymbolHasInstance,PrototypeCall,PrototypeApply,PrototypeBind,PrototypeToString,");
    let standard = rust_code(STANDARD).normalized;
    assert!(!standard.contains("FunctionBuiltin"));
    assert!(!standard.contains("emit_function_builtin("));
    assert!(!standard.contains("BoundFunctionInvoker"));
    for (standard_builtin, entry, variant) in [
        (
            "FunctionConstructor",
            "emit_function_constructor_builtin",
            "Constructor",
        ),
        (
            "FunctionPrototype",
            "emit_function_prototype_builtin",
            "Prototype",
        ),
        (
            "FunctionPrototypeSymbolHasInstance",
            "emit_function_prototype_symbol_has_instance_builtin",
            "PrototypeSymbolHasInstance",
        ),
        (
            "FunctionPrototypeCall",
            "emit_function_prototype_call_builtin",
            "PrototypeCall",
        ),
        (
            "FunctionPrototypeApply",
            "emit_function_prototype_apply_builtin",
            "PrototypeApply",
        ),
        (
            "FunctionPrototypeBind",
            "emit_function_prototype_bind_builtin",
            "PrototypeBind",
        ),
        (
            "FunctionPrototypeToString",
            "emit_function_prototype_to_string_builtin",
            "PrototypeToString",
        ),
    ] {
        assert_eq!(
            standard
                .matches(&format!("StandardBuiltinId::{standard_builtin}=>"))
                .count(),
            1,
            "standard route `{standard_builtin}`"
        );
        assert_eq!(
            standard
                .matches(&format!("self.{entry}(function)?"))
                .count(),
            1
        );
        assert_eq!(
            source
                .matches(&format!(
                    "self.emit_function_builtin(FunctionBuiltin::{variant},function)"
                ))
                .count(),
            1,
            "fixed Function producer `{variant}`"
        );
    }
    for evidence in [CONTRACT, T02, TASK] {
        assert!(evidence.contains("private `FunctionBuiltin`"));
        assert!(evidence.contains("fixed Function entries"));
    }
}
