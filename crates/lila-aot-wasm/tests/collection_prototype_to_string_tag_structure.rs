use std::fs;
use std::path::Path;

const COLLECTION_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/collections.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_ascii_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn normalized_preserving_string_contents(source: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for character in source.chars() {
        if in_string {
            result.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            result.push(character);
        } else if !character.is_whitespace() {
            result.push(character);
        }
    }

    assert!(!in_string, "unterminated string literal in bounded source");
    result
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn assert_after_last(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.rfind(earlier).expect("last earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{later}` must follow the last `{earlier}`"
    );
}

fn assert_after_containing_loop(source: &str, loop_body_operation: &str, later: &str) {
    let operation_offset = source
        .rfind(loop_body_operation)
        .expect("loop-body operation");
    let loop_end_offset = operation_offset
        + source[operation_offset..]
            .find("\n        }\n")
            .expect("containing loop end");
    let later_offset = source.find(later).expect("operation after loop");
    assert!(
        loop_end_offset < later_offset,
        "`{later}` must be outside and after the loop containing `{loop_body_operation}`"
    );
}

#[test]
fn authority_normalizer_preserves_string_literal_spaces() {
    assert_eq!(
        normalized_preserving_string_contents("Self::WeakMap => \"Weak Map\","),
        "Self::WeakMap=>\"Weak Map\","
    );
}

#[test]
fn collection_prototype_to_string_tags_have_one_closed_descriptor_authority() {
    let declaration_start = COLLECTION_INTRINSICS_SOURCE
        .find("enum CollectionPrototypeIntrinsic {")
        .expect("collection prototype authority declaration");
    assert_eq!(
        COLLECTION_INTRINSICS_SOURCE[..declaration_start]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("pub(crate)")
    );
    let declaration = between(
        COLLECTION_INTRINSICS_SOURCE,
        "enum CollectionPrototypeIntrinsic {",
        "}\n\nimpl CollectionPrototypeIntrinsic {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Map,", "Set,", "WeakMap,", "WeakSet,"]);
    let declaration_region = between(
        COLLECTION_INTRINSICS_SOURCE,
        "use crate::functions::NonArrayRealmIntrinsicSlot;",
        "impl CollectionPrototypeIntrinsic {",
    );
    assert!(!declaration_region.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!COLLECTION_INTRINSICS_SOURCE.contains(&format!(
            "impl {capability} for CollectionPrototypeIntrinsic"
        )));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "CollectionPrototypeIntrinsic"),
        10,
        "the import, declaration, inherent impl, owned emitter parameter and six producers own every mention"
    );
    for (variant, expected_producers) in [("Map", 1), ("Set", 1), ("WeakMap", 2), ("WeakSet", 2)] {
        assert_eq!(
            count_in_rust_sources(
                &source_root,
                &format!("CollectionPrototypeIntrinsic::{variant}")
            ),
            expected_producers,
            "{variant} must have the expected entry- and created-Realm producers"
        );
    }

    let authority = between(
        COLLECTION_INTRINSICS_SOURCE,
        "impl CollectionPrototypeIntrinsic {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(
        normalized_preserving_string_contents(authority),
        concat!(
            "constfnprototype_global_index(&self)->u32{matchself{",
            "Self::Map=>MAP_PROTOTYPE_GLOBAL_INDEX,",
            "Self::Set=>SET_PROTOTYPE_GLOBAL_INDEX,",
            "Self::WeakMap=>WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,",
            "Self::WeakSet=>WEAK_SET_PROTOTYPE_GLOBAL_INDEX,",
            "}}",
            "constfnto_string_tag(&self)->&'staticstr{matchself{",
            "Self::Map=>\"Map\",",
            "Self::Set=>\"Set\",",
            "Self::WeakMap=>\"WeakMap\",",
            "Self::WeakSet=>\"WeakSet\",",
            "}}",
            "pub(crate)constfnrealm_slot(&self)->NonArrayRealmIntrinsicSlot{matchself{",
            "Self::Map=>NonArrayRealmIntrinsicSlot::MapPrototype,",
            "Self::Set=>NonArrayRealmIntrinsicSlot::SetPrototype,",
            "Self::WeakMap=>NonArrayRealmIntrinsicSlot::WeakMapPrototype,",
            "Self::WeakSet=>NonArrayRealmIntrinsicSlot::WeakSetPrototype,",
            "}}",
        )
    );
    for global in [
        "MAP_PROTOTYPE_GLOBAL_INDEX",
        "SET_PROTOTYPE_GLOBAL_INDEX",
        "WEAK_MAP_PROTOTYPE_GLOBAL_INDEX",
        "WEAK_SET_PROTOTYPE_GLOBAL_INDEX",
    ] {
        assert_eq!(
            exact_identifier_count(COLLECTION_INTRINSICS_SOURCE, global),
            1,
            "{global} must be selected only by the closed authority"
        );
    }

    let emitter = between(
        COLLECTION_INTRINSICS_SOURCE,
        "    pub(crate) fn emit_collection_prototype_to_string_tag(",
        "    pub(crate) fn install_map_constructor_intrinsics(",
    );
    assert!(normalized(emitter).starts_with(concat!(
        "&mutself,intrinsic:CollectionPrototypeIntrinsic,prototype_local:u32,",
        "function:&mutFunction,)->Result<(),EmitError>{"
    )));
    assert_eq!(
        emitter
            .matches("property_key_symbol_payload(\"Symbol.toStringTag\")")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("self.strings.payload(intrinsic.to_string_tag())")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("emit_object_append_data_property_with_flags(")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches(
                "            false,\n            false,\n            true,\n            function,"
            )
            .count(),
        1,
        "the shared descriptor must be non-writable, non-enumerable and configurable"
    );
    assert_eq!(exact_identifier_count(emitter, "reserve_temp_local"), 3);
    assert_eq!(exact_identifier_count(emitter, "release_temp_local"), 3);
    for local in ["key_local", "payload_local", "tag_local"] {
        assert_eq!(
            emitter
                .matches(&format!("let {local} = self.reserve_temp_local();"))
                .count(),
            1,
            "{local} must be reserved exactly once"
        );
        assert_eq!(
            emitter
                .matches(&format!("self.release_temp_local({local});"))
                .count(),
            1,
            "{local} must be released exactly once"
        );
    }
    assert_before(
        emitter,
        "let key_local = self.reserve_temp_local();",
        "let payload_local = self.reserve_temp_local();",
    );
    assert_before(
        emitter,
        "let payload_local = self.reserve_temp_local();",
        "let tag_local = self.reserve_temp_local();",
    );
    assert_before(
        emitter,
        "self.release_temp_local(tag_local);",
        "self.release_temp_local(payload_local);",
    );
    assert_before(
        emitter,
        "self.release_temp_local(payload_local);",
        "self.release_temp_local(key_local);",
    );
    assert!(normalized(emitter).ends_with(concat!(
        "self.release_temp_local(tag_local);",
        "self.release_temp_local(payload_local);",
        "self.release_temp_local(key_local);",
        "Ok(())}",
    )));

    let installers = [
        (
            "Map",
            "    pub(crate) fn install_map_constructor_intrinsics(",
            "    pub(crate) fn install_weak_map_constructor_intrinsics(",
            "map_prototype_local",
            "emit_object_append_accessor_property_with_flags(",
            false,
        ),
        (
            "WeakMap",
            "    pub(crate) fn install_weak_map_constructor_intrinsics(",
            "    pub(crate) fn install_weak_set_constructor_intrinsics(",
            "weak_map_prototype_local",
            "emit_object_define_function_data(weak_map_prototype_local, name, &meta, function)?;",
            true,
        ),
        (
            "WeakSet",
            "    pub(crate) fn install_weak_set_constructor_intrinsics(",
            "    pub(crate) fn install_weak_ref_constructor_intrinsics(",
            "prototype_local",
            "emit_object_define_function_data(prototype_local, name, &meta, function)?;",
            true,
        ),
        (
            "Set",
            "    pub(crate) fn install_set_constructor_intrinsics(",
            "\n}\n",
            "set_prototype_local",
            "emit_object_append_accessor_property_with_flags(",
            false,
        ),
    ];

    for (
        variant,
        start,
        end,
        prototype_local,
        last_existing_property_emitter,
        last_existing_property_is_in_loop,
    ) in installers
    {
        let installer = between(COLLECTION_INTRINSICS_SOURCE, start, end);
        assert_eq!(
            installer
                .matches(&format!(
                    "let intrinsic = CollectionPrototypeIntrinsic::{variant};"
                ))
                .count(),
            1,
            "{variant} family selection"
        );
        assert_before(
            installer,
            "Instruction::GlobalGet(intrinsic.prototype_global_index())",
            "emit_collection_prototype_to_string_tag(",
        );
        let normalized_installer = normalized(installer).replace(",)?;", ")?;");
        let consuming_call = format!(
            "self.emit_collection_prototype_to_string_tag(intrinsic,{prototype_local},function)?;"
        );
        assert_eq!(
            normalized_installer.matches(&consuming_call).count(),
            1,
            "{variant} must move its authority once after the borrowed prototype projection"
        );
        assert!(!normalized_installer
            .contains("self.emit_collection_prototype_to_string_tag(&intrinsic,"));
        assert_eq!(
            installer
                .matches("Instruction::GlobalGet(intrinsic.prototype_global_index())")
                .count(),
            1,
            "{variant} prototype selection"
        );
        assert_eq!(
            installer
                .matches("emit_collection_prototype_to_string_tag(")
                .count(),
            1,
            "{variant} descriptor installation"
        );
        assert!(!installer.contains("property_key_symbol_payload(\"Symbol.toStringTag\")"));
        assert!(!installer.contains("emit_object_append_data_property_with_flags("));
        assert_after_last(
            installer,
            last_existing_property_emitter,
            "emit_collection_prototype_to_string_tag(",
        );
        if last_existing_property_is_in_loop {
            assert_after_containing_loop(
                installer,
                last_existing_property_emitter,
                "emit_collection_prototype_to_string_tag(",
            );
        }
        assert_before(
            installer,
            "emit_collection_prototype_to_string_tag(",
            &format!("release_temp_local({prototype_local})"),
        );
    }

    assert_eq!(
        COLLECTION_INTRINSICS_SOURCE
            .matches("let intrinsic = CollectionPrototypeIntrinsic::")
            .count(),
        4
    );
    assert_eq!(
        COLLECTION_INTRINSICS_SOURCE
            .matches("emit_collection_prototype_to_string_tag(")
            .count(),
        5,
        "one definition and four family calls"
    );
}
