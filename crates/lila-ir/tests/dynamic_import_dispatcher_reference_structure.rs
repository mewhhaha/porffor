use std::fs;
use std::path::Path;

const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/dynamic-import-dispatcher-reference-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/12-modules-linking-loading.md");

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    source[start_offset..]
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

fn rust_sources(dir: &Path) -> Vec<String> {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .flat_map(|path| {
            if path.is_dir() {
                return rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return Vec::new();
            }
            vec![fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))]
        })
        .collect()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn dispatcher_reference_is_the_exact_non_copy_domain() {
    let lexical_probe = r###"
        // DynamicImportDispatcherReference::ModuleLocal
        DynamicImportDispatcherReference /* nested /* ignored */ comment */ :: r#ScriptEntryExport;
        "DynamicImportDispatcherReference"; b"DynamicImportDispatcherReference";
        c"DynamicImportDispatcherReference"; r"DynamicImportDispatcherReference";
        br##"DynamicImportDispatcherReference"##; cr#"DynamicImportDispatcherReference"#;
        'D'; b'D'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.identifiers,
            "DynamicImportDispatcherReference"
        ),
        1
    );
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.routes,
            "DynamicImportDispatcherReference::ScriptEntryExport"
        ),
        1
    );

    let declaration_marker = "enum DynamicImportDispatcherReference {";
    let declaration_offset = DYNAMIC_SOURCE
        .find(declaration_marker)
        .expect("dispatcher-reference declaration");
    let preceding_item_end = DYNAMIC_SOURCE[..declaration_offset]
        .rfind(';')
        .expect("linker-name constant before declaration");
    let following_item = DYNAMIC_SOURCE[declaration_offset..]
        .find("/// Merged-scope name of the `import()` dispatcher")
        .map(|offset| declaration_offset + offset)
        .expect("dispatcher-name item after declaration");
    assert_eq!(
        normalize_rust(&DYNAMIC_SOURCE[preceding_item_end + 1..following_item]).code,
        "enumDynamicImportDispatcherReference{ModuleLocal,ScriptEntryExport,}",
        "the dispatcher-reference authority must remain private and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let normalized = rust_sources(&source_root)
        .iter()
        .map(|source| normalize_rust(source))
        .collect::<Vec<_>>();
    assert_eq!(
        normalized
            .iter()
            .map(|source| {
                exact_identifier_count(&source.identifiers, "DynamicImportDispatcherReference")
            })
            .sum::<usize>(),
        6,
        "the declaration, parameter, two producers and two consumer arms own every mention"
    );
    for variant in ["ModuleLocal", "ScriptEntryExport"] {
        assert_eq!(
            normalized
                .iter()
                .map(|source| {
                    exact_identifier_count(
                        &source.routes,
                        &format!("DynamicImportDispatcherReference::{variant}"),
                    )
                })
                .sum::<usize>(),
            2,
            "variant `{variant}` must retain one producer and one consumer"
        );
    }
    let all_routes = normalized
        .iter()
        .map(|source| format!("{}\n", source.routes))
        .collect::<String>();
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!(
            "impl{capability}forDynamicImportDispatcherReference"
        )));
    }
    for forbidden in [
        "typeDynamicImportDispatcherReference",
        "DynamicImportDispatcherReferenceas",
        "DynamicImportDispatcherReference::clone",
        "DynamicImportDispatcherReference::eq",
    ] {
        assert!(!all_routes.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn the_two_public_rewriters_construct_distinct_references() {
    let module_rewriter = normalize_rust(bounded_inclusive(
        DYNAMIC_SOURCE,
        "    pub fn rewrite_dynamic_import_calls(",
        "    /// [`Self::rewrite_dynamic_import_calls`] for the Script entry",
    ));
    assert_eq!(
        module_rewriter.code,
        concat!(
            "pubfnrewrite_dynamic_import_calls(&self,unit:ModuleUnitId,source:&str,)",
            "->Result<String,String>{self.rewrite_calls(unit,",
            "DynamicImportDispatcherReference::ModuleLocal,source)}"
        )
    );

    let script_rewriter = normalize_rust(bounded_inclusive(
        DYNAMIC_SOURCE,
        "    pub fn rewrite_script_entry_import_calls(",
        "    /// `(exported name, dispatcher name)`",
    ));
    assert_eq!(
        script_rewriter.code,
        concat!(
            "pubfnrewrite_script_entry_import_calls(&self,source:&str)->Result<String,String>{",
            "self.rewrite_calls(self.entry,",
            "DynamicImportDispatcherReference::ScriptEntryExport,source,)}"
        )
    );
}

#[test]
fn rewrite_calls_consumes_the_reference_in_one_exhaustive_projection() {
    let rewrite_calls = normalize_rust(bounded_inclusive(
        DYNAMIC_SOURCE,
        "    fn rewrite_calls(",
        "    /// Every reason this graph's `import()` usage cannot be desugared.",
    ));
    assert!(rewrite_calls.code.starts_with(concat!(
        "fnrewrite_calls(&self,unit:ModuleUnitId,",
        "dispatcher_reference:DynamicImportDispatcherReference,source:&str,)",
        "->Result<String,String>{"
    )));
    assert_eq!(
        exact_identifier_count(&rewrite_calls.identifiers, "dispatcher_reference"),
        2,
        "the owned parameter must be observed only by its consuming match"
    );
    let projection = concat!(
        "letname=matchdispatcher_reference{",
        "DynamicImportDispatcherReference::ModuleLocal=>dispatcher_name(unit,site.phase),",
        "DynamicImportDispatcherReference::ScriptEntryExport=>{",
        "exported_dispatcher_name(unit,site.phase)}};"
    );
    assert_eq!(rewrite_calls.code.matches(projection).count(), 1);
    assert_eq!(
        rewrite_calls
            .code
            .matches("matchdispatcher_reference{")
            .count(),
        1
    );
    assert!(!rewrite_calls.code.contains("match&dispatcher_reference{"));
    assert!(!rewrite_calls.code.contains("_=>"));
    let handoff = rewrite_calls
        .code
        .find(projection)
        .expect("consuming dispatcher-reference projection");
    assert!(!rewrite_calls.code[handoff + projection.len()..].contains("dispatcher_reference"));
    assert_eq!(
        (rewrite_calls.code.len(), fnv1a(&rewrite_calls.code)),
        (1210, 0x9ac0_c98b_f0eb_84da),
        "the complete rewrite body changed; review ordering and refresh deliberately"
    );
}

#[test]
fn contract_and_t12_record_the_single_projection_boundary() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "non-`Clone`, non-`Copy`",
        "six lexical mentions",
        "two producers",
        "one exhaustive consuming projection",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T12 marker: {marker}");
    }
}
