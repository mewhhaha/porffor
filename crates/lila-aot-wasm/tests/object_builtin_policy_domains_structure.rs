const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const ENUMERABLE_OWN_PROPERTIES_SOURCE: &str =
    include_str!("../src/builtins/object/enumerable_own_properties.rs");
const INTEGRITY_TEST_SOURCE: &str = include_str!("../src/builtins/object/integrity_test.rs");
const PROTOTYPE_LOOKUP_SOURCE: &str = include_str!("../src/builtins/object/prototype_lookup.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-builtin-policy-domains.md");
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T10: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn enum_variants(source: &'static str, name: &str) -> Vec<&'static str> {
    let marker = format!("enum {name} {{");
    source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing enum `{name}`"))
        .1
        .split_once('}')
        .unwrap_or_else(|| panic!("missing end of enum `{name}`"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn object_builtin_policy_domains_are_exact_and_capability_free() {
    assert_eq!(
        enum_variants(ENUMERABLE_OWN_PROPERTIES_SOURCE, "EnumerableOwnProperties"),
        ["Entries,", "Values,"]
    );
    assert_eq!(
        enum_variants(INTEGRITY_TEST_SOURCE, "IntegrityTest"),
        ["Sealed,", "Frozen,"]
    );
    assert_eq!(
        enum_variants(PROTOTYPE_LOOKUP_SOURCE, "PrototypeLookup"),
        ["Getter,", "Setter,"]
    );

    for (source, name, declaration) in [
        (
            ENUMERABLE_OWN_PROPERTIES_SOURCE,
            "EnumerableOwnProperties",
            "enum EnumerableOwnProperties {",
        ),
        (
            INTEGRITY_TEST_SOURCE,
            "IntegrityTest",
            "enum IntegrityTest {",
        ),
        (
            PROTOTYPE_LOOKUP_SOURCE,
            "PrototypeLookup",
            "enum PrototypeLookup {",
        ),
    ] {
        assert!(source.contains(declaration), "missing `{name}`");
        let prefix = source
            .split_once(declaration)
            .expect("policy declaration")
            .0
            .rsplit("\n\n")
            .next()
            .unwrap_or_default();
        assert!(!prefix.contains("#[derive"), "{name} derives a capability");
        for capability in [
            "Clone",
            "Copy",
            "Debug",
            "PartialEq",
            "Eq",
            "PartialOrd",
            "Ord",
            "Hash",
            "Default",
        ] {
            assert!(
                !source.contains(&format!("impl {capability} for {name}")),
                "{name} manually implements {capability}"
            );
        }
    }
}

#[test]
fn object_builtin_dispatch_pins_all_six_semantic_policy_routes() {
    assert_eq!(
        OBJECT_SOURCE
            .matches("mod enumerable_own_properties;")
            .count(),
        1
    );
    assert!(!OBJECT_SOURCE.contains("pub mod enumerable_own_properties;"));
    assert_eq!(OBJECT_SOURCE.matches("EnumerableOwnProperties").count(), 0);
    for escaped_raw_name in [
        "EnumerableOwnProperties::",
        ": EnumerableOwnProperties",
        "EnumerableOwnProperties,",
        "EnumerableOwnProperties;",
        "EnumerableOwnProperties as ",
        "compile_object_enumerable_own_properties_builtin(",
        "enumerable_own_properties::",
    ] {
        assert_eq!(OBJECT_SOURCE.matches(escaped_raw_name).count(), 0);
        assert_eq!(STANDARD_SOURCE.matches(escaped_raw_name).count(), 0);
    }
    assert_eq!(
        ENUMERABLE_OWN_PROPERTIES_SOURCE
            .matches("EnumerableOwnProperties")
            .count(),
        8
    );
    assert_eq!(
        ENUMERABLE_OWN_PROPERTIES_SOURCE
            .matches("EnumerableOwnProperties::Entries")
            .count(),
        3
    );
    assert_eq!(
        ENUMERABLE_OWN_PROPERTIES_SOURCE
            .matches("EnumerableOwnProperties::Values")
            .count(),
        3
    );
    assert_eq!(
        ENUMERABLE_OWN_PROPERTIES_SOURCE
            .matches("compile_object_enumerable_own_properties_builtin(")
            .count(),
        3
    );
    for wrapper in [
        "compile_object_entries_builtin(",
        "compile_object_values_builtin(",
    ] {
        assert_eq!(ENUMERABLE_OWN_PROPERTIES_SOURCE.matches(wrapper).count(), 1);
        assert_eq!(STANDARD_SOURCE.matches(wrapper).count(), 1);
    }
    assert_eq!(
        ENUMERABLE_OWN_PROPERTIES_SOURCE
            .matches("pub(in crate::builtins) fn compile_object_")
            .count(),
        2
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches(
                "StandardBuiltinId::ObjectEntries => self.compile_object_entries_builtin(function)?",
            )
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches(
                "StandardBuiltinId::ObjectValues => self.compile_object_values_builtin(function)?",
            )
            .count(),
        1
    );

    assert_eq!(OBJECT_SOURCE.matches("mod integrity_test;").count(), 1);
    assert!(!OBJECT_SOURCE.contains("pub mod integrity_test;"));
    assert_eq!(OBJECT_SOURCE.matches("IntegrityTest").count(), 0);
    for escaped_raw_name in [
        "IntegrityTest::",
        ": IntegrityTest",
        "IntegrityTest,",
        "IntegrityTest;",
        "IntegrityTest as ",
        "compile_object_integrity_test_builtin(",
        "integrity_test::",
    ] {
        assert_eq!(OBJECT_SOURCE.matches(escaped_raw_name).count(), 0);
        assert_eq!(STANDARD_SOURCE.matches(escaped_raw_name).count(), 0);
    }
    assert_eq!(INTEGRITY_TEST_SOURCE.matches("IntegrityTest").count(), 6);
    assert_eq!(
        INTEGRITY_TEST_SOURCE
            .matches("IntegrityTest::Sealed")
            .count(),
        2
    );
    assert_eq!(
        INTEGRITY_TEST_SOURCE
            .matches("IntegrityTest::Frozen")
            .count(),
        2
    );
    assert_eq!(
        INTEGRITY_TEST_SOURCE
            .matches("compile_object_integrity_test_builtin(")
            .count(),
        3
    );
    for wrapper in [
        "compile_object_is_sealed_builtin(",
        "compile_object_is_frozen_builtin(",
    ] {
        assert_eq!(INTEGRITY_TEST_SOURCE.matches(wrapper).count(), 1);
        assert_eq!(STANDARD_SOURCE.matches(wrapper).count(), 1);
    }
    assert_eq!(
        INTEGRITY_TEST_SOURCE
            .matches("pub(in crate::builtins) fn compile_object_is_")
            .count(),
        2
    );

    assert_eq!(
        STANDARD_SOURCE
            .matches(
                "StandardBuiltinId::ObjectIsSealed => self.compile_object_is_sealed_builtin(function)?",
            )
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches(
                "StandardBuiltinId::ObjectIsFrozen => self.compile_object_is_frozen_builtin(function)?",
            )
            .count(),
        1
    );

    assert_eq!(OBJECT_SOURCE.matches("mod prototype_lookup;").count(), 1);
    assert!(!OBJECT_SOURCE.contains("pub mod prototype_lookup;"));
    assert_eq!(OBJECT_SOURCE.matches("PrototypeLookup").count(), 0);
    for escaped_raw_name in [
        "PrototypeLookup::",
        ": PrototypeLookup",
        "PrototypeLookup,",
        "PrototypeLookup;",
        "PrototypeLookup as ",
        "compile_object_prototype_lookup_builtin(",
        "prototype_lookup::",
    ] {
        assert_eq!(OBJECT_SOURCE.matches(escaped_raw_name).count(), 0);
        assert_eq!(STANDARD_SOURCE.matches(escaped_raw_name).count(), 0);
    }
    assert_eq!(
        PROTOTYPE_LOOKUP_SOURCE.matches("PrototypeLookup").count(),
        6
    );
    assert_eq!(
        PROTOTYPE_LOOKUP_SOURCE
            .matches("PrototypeLookup::Getter")
            .count(),
        2
    );
    assert_eq!(
        PROTOTYPE_LOOKUP_SOURCE
            .matches("PrototypeLookup::Setter")
            .count(),
        2
    );
    assert_eq!(
        PROTOTYPE_LOOKUP_SOURCE
            .matches("compile_object_prototype_lookup_builtin(")
            .count(),
        3
    );
    for wrapper in [
        "compile_object_prototype_lookup_getter_builtin(",
        "compile_object_prototype_lookup_setter_builtin(",
    ] {
        assert_eq!(PROTOTYPE_LOOKUP_SOURCE.matches(wrapper).count(), 1);
        assert_eq!(STANDARD_SOURCE.matches(wrapper).count(), 1);
    }
    assert_eq!(
        PROTOTYPE_LOOKUP_SOURCE
            .matches("pub(in crate::builtins) fn compile_object_prototype_lookup_")
            .count(),
        2
    );

    let getter_dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ObjectPrototypeLookupGetter => {",
        "            StandardBuiltinId::ObjectPrototypeLookupSetter => {",
    );
    assert!(getter_dispatch.contains("compile_object_prototype_lookup_getter_builtin(function)"));
    assert!(!getter_dispatch.contains("compile_object_prototype_lookup_setter_builtin(function)"));
    let setter_dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ObjectPrototypeLookupSetter => {",
        "            StandardBuiltinId::ObjectPrototypePropertyIsEnumerable => {",
    );
    assert!(setter_dispatch.contains("compile_object_prototype_lookup_setter_builtin(function)"));
    assert!(!setter_dispatch.contains("compile_object_prototype_lookup_getter_builtin(function)"));
}

#[test]
fn enumerable_own_properties_projects_each_policy_exhaustively() {
    let body = bounded(
        ENUMERABLE_OWN_PROPERTIES_SOURCE,
        "    fn compile_object_enumerable_own_properties_builtin(",
        "    pub(in crate::builtins) fn compile_object_entries_builtin(",
    );

    assert!(body.contains("mode: EnumerableOwnProperties,"));
    assert_eq!(body.matches("match &mode").count(), 2);
    assert_eq!(body.matches("match mode").count(), 0);
    assert_eq!(body.matches("EnumerableOwnProperties::Entries").count(), 2);
    assert_eq!(body.matches("EnumerableOwnProperties::Values").count(), 2);
    assert!(body.contains(
        "EnumerableOwnProperties::Entries => \"Object.entries called on null or undefined\""
    ));
    assert!(body.contains(
        "EnumerableOwnProperties::Values => \"Object.values called on null or undefined\""
    ));

    let result_policy = body
        .rsplit_once("        match &mode {")
        .expect("Enumerable result policy")
        .1
        .split_once("        function.instruction(&Instruction::LocalGet(write_index_local));")
        .expect("Enumerable result policy end")
        .0;
    let (entries_policy, values_policy) = result_policy
        .split_once("EnumerableOwnProperties::Values =>")
        .expect("Enumerable Values policy");
    assert_eq!(
        entries_policy
            .matches("emit_alloc_array_payload_with_length(")
            .count(),
        1,
        "only Entries materializes a key-value pair"
    );
    assert!(!values_policy.contains("emit_alloc_array_payload_with_length("));
    assert_eq!(result_policy.matches("self.emit_array_write(").count(), 4);

    for forbidden in [
        "include_keys",
        "mode ==",
        "mode !=",
        "=> true",
        "=> false",
        "_ =>",
        "unreachable!",
        "debug_assert!",
    ] {
        assert!(
            !body.contains(forbidden),
            "Enumerable policy contains `{forbidden}`"
        );
    }
}

#[test]
fn integrity_and_prototype_lookup_policies_are_exhaustive() {
    let integrity = bounded(
        INTEGRITY_TEST_SOURCE,
        "    fn compile_object_integrity_test_builtin(",
        "    pub(in crate::builtins) fn compile_object_is_sealed_builtin(",
    );
    assert!(integrity.contains("mode: IntegrityTest,"));
    assert_eq!(integrity.matches("match &mode").count(), 1);
    assert_eq!(integrity.matches("match mode").count(), 0);
    assert_eq!(integrity.matches("IntegrityTest::Sealed").count(), 1);
    assert_eq!(integrity.matches("IntegrityTest::Frozen").count(), 1);
    assert_eq!(
        integrity
            .matches("self.strings.payload(\"writable\")")
            .count(),
        1
    );
    let (before_integrity_policy, integrity_policy) = integrity
        .split_once("        match &mode {")
        .expect("Integrity policy");
    assert!(!before_integrity_policy.contains("self.strings.payload(\"writable\")"));
    let (sealed_policy, frozen_policy) = integrity_policy
        .split_once("IntegrityTest::Frozen =>")
        .expect("Frozen policy");
    assert!(!sealed_policy.contains("self.strings.payload(\"writable\")"));
    assert!(frozen_policy.contains("self.strings.payload(\"writable\")"));

    let prototype_lookup = bounded(
        PROTOTYPE_LOOKUP_SOURCE,
        "    fn compile_object_prototype_lookup_builtin(",
        "    pub(in crate::builtins) fn compile_object_prototype_lookup_getter_builtin(",
    );
    assert!(prototype_lookup.contains("mode: PrototypeLookup,"));
    assert_eq!(prototype_lookup.matches("match &mode").count(), 1);
    assert_eq!(prototype_lookup.matches("match mode").count(), 0);
    assert_eq!(
        prototype_lookup.matches("PrototypeLookup::Getter").count(),
        1
    );
    assert_eq!(
        prototype_lookup.matches("PrototypeLookup::Setter").count(),
        1
    );
    assert!(prototype_lookup.contains("PrototypeLookup::Getter => \"get\""));
    assert!(prototype_lookup.contains("PrototypeLookup::Setter => \"set\""));

    for (name, body) in [
        ("Integrity", integrity),
        ("Prototype lookup", prototype_lookup),
    ] {
        for forbidden in [
            "check_writable",
            "mode ==",
            "mode !=",
            "=> true",
            "=> false",
            "_ =>",
            "unreachable!",
            "debug_assert!",
        ] {
            assert!(
                !body.contains(forbidden),
                "{name} policy contains `{forbidden}`"
            );
        }
    }

    for evidence in [CONTRACT, T02, T10] {
        let words = evidence.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("Batch AL"));
        assert!(words.contains("Batch AM"));
        assert!(words.contains("Batch AN"));
        assert!(words.contains("EnumerableOwnProperties"));
        assert!(words.contains("PrototypeLookup"));
        assert!(words.contains("IntegrityTest"));
    }

    for task in [T02, T10] {
        let words = task.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("private `builtins/object/prototype_lookup.rs`"));
        assert!(words.contains("private `builtins/object/integrity_test.rs`"));
        assert!(words.contains("private `builtins/object/enumerable_own_properties.rs`"));
    }
}
