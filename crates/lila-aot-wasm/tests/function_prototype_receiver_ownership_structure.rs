use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/function.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
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
fn receiver_carrier_is_the_exact_private_non_copy_domain() {
    let lexical_probe = rust_code(
        r###"
        // FunctionPrototypeReceiverLocals
        FunctionPrototypeReceiverLocals /* nested /* ignored */ comment */;
        "FunctionPrototypeReceiverLocals"; b"FunctionPrototypeReceiverLocals";
        c"FunctionPrototypeReceiverLocals"; r"FunctionPrototypeReceiverLocals";
        br##"FunctionPrototypeReceiverLocals"##; cr#"FunctionPrototypeReceiverLocals"#;
        'F'; b'F'; 'lifetime;
        "###,
    );
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.identifiers,
            "FunctionPrototypeReceiverLocals",
        ),
        1
    );

    let receiver_module = rust_code(bounded(
        SOURCE,
        "mod function_prototype_receiver {",
        "\n}\n\nuse self::function_prototype_receiver::FunctionPrototypeReceiverLocals;",
    ));
    assert!(receiver_module.normalized.starts_with(concat!(
        "usesuper::*;",
        "pub(super)structFunctionPrototypeReceiverLocals{",
        "payload_local:u32,tag_local:u32,}",
        "implFunctionPrototypeReceiverLocals{"
    )));
    assert_eq!(
        receiver_module
            .normalized
            .matches("(builder.this_payload_local,builder.this_tag_local)")
            .count(),
        1
    );
    for forbidden in [
        "new_target",
        "implCloneforFunctionPrototypeReceiverLocals",
        "implCopyforFunctionPrototypeReceiverLocals",
        "derive(",
    ] {
        assert!(
            !receiver_module.normalized.contains(forbidden),
            "found `{forbidden}`"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "FunctionPrototypeReceiverLocals"),
        8
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "from_this"),
        6
    );
}

#[test]
fn five_prototype_operations_can_only_construct_from_this() {
    let operations = bounded(
        SOURCE,
        "            FunctionBuiltin::PrototypeSymbolHasInstance => {",
        "            FunctionBuiltin::BoundFunctionInvoker => {",
    );
    assert_eq!(
        operations
            .matches("FunctionPrototypeReceiverLocals::from_this(")
            .count(),
        5
    );
    for builtin_name in [
        "Function.prototype[Symbol.hasInstance]",
        "Function.prototype.call",
        "Function.prototype.apply",
        "Function.prototype.bind",
        "Function.prototype.toString",
    ] {
        assert_eq!(
            operations.matches(&format!("\"{builtin_name}\"")).count(),
            1
        );
    }
    for forbidden in [
        "this_payload_local",
        "this_tag_local",
        "new_target_payload_local",
        "new_target_tag_local",
        "receiver_payload_local",
        "receiver_tag_local",
        "constructor_payload_local",
        "constructor_tag_local",
    ] {
        assert!(!operations.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn each_operation_keeps_payload_and_tag_on_the_same_carrier() {
    let operations = bounded(
        SOURCE,
        "            FunctionBuiltin::PrototypeSymbolHasInstance => {",
        "            FunctionBuiltin::BoundFunctionInvoker => {",
    );
    for (start, end, payload_reads, tag_reads) in [
        ("PrototypeSymbolHasInstance", "PrototypeCall", 1, 1),
        ("PrototypeCall", "PrototypeApply", 1, 1),
        ("PrototypeApply", "PrototypeBind", 2, 2),
        ("PrototypeBind", "PrototypeToString", 1, 2),
        ("PrototypeToString", "BoundFunctionInvoker", 2, 2),
    ] {
        let branch = bounded(
            SOURCE,
            &format!("FunctionBuiltin::{start} => {{"),
            &format!("FunctionBuiltin::{end} => {{"),
        );
        assert_eq!(
            branch.matches("receiver.payload_local()").count(),
            payload_reads
        );
        assert_eq!(branch.matches("receiver.tag_local()").count(), tag_reads);
        assert_eq!(
            branch
                .matches("FunctionPrototypeReceiverLocals::from_this(")
                .count(),
            1
        );
    }
    assert_eq!(operations.matches("receiver.payload_local()").count(), 7);
    assert_eq!(operations.matches("receiver.tag_local()").count(), 8);
}

#[test]
fn contract_and_task_record_the_receiver_authority() {
    for marker in [
        "paired Function prototype receiver authority",
        "cannot mix payload and tag sources",
        "function_prototype_receiver_ownership_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }

    let identifiers = rust_code(SOURCE).identifiers;
    assert_eq!(exact_identifier_count(&identifiers, "FunctionBuiltin"), 18);
    assert!(SOURCE.contains("enum FunctionBuiltin {"));
    assert!(!SOURCE.contains("pub(super) enum FunctionBuiltin"));
    assert!(!SOURCE.contains("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum FunctionBuiltin"));
    assert!(!STANDARD.contains("FunctionBuiltin"));
    assert!(!STANDARD.contains("emit_function_builtin("));
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
        (
            "BoundFunctionInvoker",
            "emit_bound_function_invoker_builtin",
            "BoundFunctionInvoker",
        ),
    ] {
        assert_eq!(
            STANDARD
                .matches(&format!("StandardBuiltinId::{standard_builtin} =>"))
                .count(),
            1,
            "standard route `{standard_builtin}`"
        );
        assert_eq!(
            STANDARD
                .matches(&format!("self.{entry}(function)?"))
                .count(),
            1
        );
        assert_eq!(
            SOURCE
                .matches(&format!(
                    "self.emit_function_builtin(FunctionBuiltin::{variant}, function)"
                ))
                .count(),
            1,
            "fixed Function producer `{variant}`"
        );
    }
    for evidence in [CONTRACT, T02, TASK] {
        assert!(evidence.contains("private `FunctionBuiltin`"));
        assert!(evidence.contains("fixed Function entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new Function behavior"));
    }
}
