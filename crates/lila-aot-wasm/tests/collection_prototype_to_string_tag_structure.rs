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
fn collection_prototype_to_string_tags_have_one_closed_descriptor_authority() {
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

    let authority = between(
        COLLECTION_INTRINSICS_SOURCE,
        "impl CollectionPrototypeIntrinsic {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    assert!(!authority.contains("_ =>"));
    for (variant, global, tag) in [
        ("Map", "MAP_PROTOTYPE_GLOBAL_INDEX", "Map"),
        ("Set", "SET_PROTOTYPE_GLOBAL_INDEX", "Set"),
        ("WeakMap", "WEAK_MAP_PROTOTYPE_GLOBAL_INDEX", "WeakMap"),
        ("WeakSet", "WEAK_SET_PROTOTYPE_GLOBAL_INDEX", "WeakSet"),
    ] {
        let global_mapping = format!("Self::{variant} => {global},");
        let tag_mapping = format!("Self::{variant} => \"{tag}\",");
        assert_eq!(authority.matches(&global_mapping).count(), 1, "{variant}");
        assert_eq!(authority.matches(&tag_mapping).count(), 1, "{variant}");
        assert_eq!(
            exact_identifier_count(COLLECTION_INTRINSICS_SOURCE, global),
            1,
            "{global} must be selected only by the closed authority"
        );
    }

    let emitter = between(
        COLLECTION_INTRINSICS_SOURCE,
        "    fn emit_collection_prototype_to_string_tag(",
        "    pub(crate) fn install_map_constructor_intrinsics(",
    );
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
