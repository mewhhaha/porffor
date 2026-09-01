const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const OWN_DESCRIPTOR_PREDICATE_SOURCE: &str =
    include_str!("../src/builtins/object/own_descriptor_predicate.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-own-descriptor-predicate-kind.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn own_descriptor_predicate_body() -> &'static str {
    bounded(
        OWN_DESCRIPTOR_PREDICATE_SOURCE,
        "    fn compile_object_own_descriptor_predicate_builtin(",
        "    pub(in crate::builtins) fn compile_object_has_own_builtin(",
    )
}

#[test]
fn own_descriptor_predicate_kind_is_exact_and_capability_free() {
    let declaration = bounded(
        OWN_DESCRIPTOR_PREDICATE_SOURCE,
        "enum OwnDescriptorPredicateBuiltin {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    let variants = declaration
        .split_once('}')
        .expect("own-descriptor predicate kind end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "ObjectHasOwn,",
            "PrototypeHasOwnProperty,",
            "PrototypePropertyIsEnumerable,",
        ]
    );

    let declaration_prefix = OWN_DESCRIPTOR_PREDICATE_SOURCE
        .split_once("enum OwnDescriptorPredicateBuiltin {")
        .expect("own-descriptor predicate kind declaration")
        .0
        .rsplit("\n\n")
        .next()
        .unwrap_or_default();
    assert!(!declaration_prefix.contains("#[derive"));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !OWN_DESCRIPTOR_PREDICATE_SOURCE.contains(&format!(
                "impl {capability} for OwnDescriptorPredicateBuiltin"
            )),
            "own-descriptor predicate kind implements {capability}"
        );
    }
    assert!(!OWN_DESCRIPTOR_PREDICATE_SOURCE.contains("pub enum OwnDescriptorPredicateBuiltin"));
    assert!(
        !OWN_DESCRIPTOR_PREDICATE_SOURCE.contains("pub(crate) enum OwnDescriptorPredicateBuiltin")
    );
    assert!(
        !OWN_DESCRIPTOR_PREDICATE_SOURCE.contains("pub(super) enum OwnDescriptorPredicateBuiltin")
    );
    assert_eq!(
        OBJECT_SOURCE
            .matches("mod own_descriptor_predicate;")
            .count(),
        1
    );
    assert!(!OBJECT_SOURCE.contains("mod own_descriptor_predicate {"));
    assert!(!OBJECT_SOURCE.contains("own_descriptor_predicate::"));
    assert!(!OBJECT_SOURCE.contains("OwnDescriptorPredicateBuiltin"));
    assert!(!OBJECT_SOURCE.contains("compile_object_own_descriptor_predicate_builtin"));
    assert_eq!(
        OWN_DESCRIPTOR_PREDICATE_SOURCE
            .matches("OwnDescriptorPredicateBuiltin")
            .count(),
        14
    );
    for variant in [
        "ObjectHasOwn",
        "PrototypeHasOwnProperty",
        "PrototypePropertyIsEnumerable",
    ] {
        assert_eq!(OWN_DESCRIPTOR_PREDICATE_SOURCE.matches(variant).count(), 5);
    }
}

#[test]
fn each_own_descriptor_predicate_wrapper_produces_one_kind() {
    for (start, end, variant) in [
        (
            "    pub(in crate::builtins) fn compile_object_has_own_builtin(",
            "    pub(in crate::builtins) fn compile_object_prototype_has_own_property_builtin(",
            "ObjectHasOwn",
        ),
        (
            "    pub(in crate::builtins) fn compile_object_prototype_has_own_property_builtin(",
            "    pub(in crate::builtins) fn compile_object_prototype_property_is_enumerable_builtin(",
            "PrototypeHasOwnProperty",
        ),
        (
            "    pub(in crate::builtins) fn compile_object_prototype_property_is_enumerable_builtin(",
            "\n}",
            "PrototypePropertyIsEnumerable",
        ),
    ] {
        let wrapper = bounded(OWN_DESCRIPTOR_PREDICATE_SOURCE, start, end);
        assert_eq!(
            wrapper
                .matches("compile_object_own_descriptor_predicate_builtin(")
                .count(),
            1
        );
        assert_eq!(
            wrapper
                .matches(&format!("OwnDescriptorPredicateBuiltin::{variant}"))
                .count(),
            1
        );
    }

    assert_eq!(
        OWN_DESCRIPTOR_PREDICATE_SOURCE
            .matches("compile_object_own_descriptor_predicate_builtin(")
            .count(),
        4
    );

    for compiler in [
        "compile_object_has_own_builtin(function)",
        "compile_object_prototype_has_own_property_builtin(function)",
        "compile_object_prototype_property_is_enumerable_builtin(function)",
    ] {
        assert_eq!(STANDARD_SOURCE.matches(compiler).count(), 1, "{compiler}");
    }
}

#[test]
fn one_owned_kind_controls_all_three_semantic_decisions() {
    let body = own_descriptor_predicate_body();
    assert!(body.contains("builtin: OwnDescriptorPredicateBuiltin,"));
    assert_eq!(body.matches("match &builtin").count(), 3);
    assert_eq!(body.matches("match builtin").count(), 0);
    for variant in [
        "ObjectHasOwn",
        "PrototypeHasOwnProperty",
        "PrototypePropertyIsEnumerable",
    ] {
        assert_eq!(
            body.matches(&format!("OwnDescriptorPredicateBuiltin::{variant}"))
                .count(),
            3,
            "semantic decision census for {variant}"
        );
    }

    for forbidden in [
        "builtin.clone()",
        "builtin ==",
        "builtin !=",
        "=> true",
        "=> false",
        "_ =>",
        "unreachable!",
        "debug_assert!",
    ] {
        assert!(
            !body.contains(forbidden),
            "kind escapes through `{forbidden}`"
        );
    }
}

#[test]
fn evidence_records_the_borrowed_own_descriptor_predicate_authority() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("OwnDescriptorPredicateBuiltin"));
        assert!(evidence.contains("borrowed exhaustive"));
    }
}
