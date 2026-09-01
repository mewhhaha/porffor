const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const OBJECT_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const FUNCTION_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const TYPED_ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/accessor-descriptor-local-roles.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn accessor_descriptor_roles_form_one_exact_nonempty_domain() {
    let declarations = normalized(bounded(
        OBJECTS_SOURCE,
        "pub(crate) struct AccessorGetterLocals(TaggedLocals);",
        "/// Allocation-free stored fields consumed by descriptor compatibility.",
    ));
    assert!(declarations.contains(
        "implAccessorGetterLocals{pub(crate)constfnnew(value:TaggedLocals)->Self{Self(value)}}"
    ));
    assert!(declarations.contains("pub(crate)structAccessorSetterLocals(TaggedLocals);"));
    assert!(declarations.contains(
        "implAccessorSetterLocals{pub(crate)constfnnew(value:TaggedLocals)->Self{Self(value)}}"
    ));
    assert!(declarations.contains(concat!(
        "pub(crate)enumAccessorDescriptorLocals{",
        "Getter(AccessorGetterLocals),Setter(AccessorSetterLocals),",
        "GetterAndSetter{getter:AccessorGetterLocals,setter:AccessorSetterLocals,},}"
    )));
    assert!(!declarations.contains("Option<"));
    assert!(!declarations.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !OBJECTS_SOURCE.contains(&format!("impl {capability} for AccessorDescriptorLocals"))
        );
    }
}

#[test]
fn three_definition_boundaries_consume_the_typed_descriptor() {
    let definitions = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_define_accessor(",
        "/// Positional adapter for the two `Object.defineProperty` call sites",
    );
    assert_eq!(
        definitions
            .matches("accessors: AccessorDescriptorLocals,")
            .count(),
        3
    );
    assert!(!definitions.contains("getter: Option<(u32, u32)>,"));
    assert!(!definitions.contains("setter: Option<(u32, u32)>,"));

    let projection = bounded(
        definitions,
        "let (get, set) = match accessors {",
        "let descriptor = WasmPartialDescriptor {",
    );
    for marker in [
        "AccessorDescriptorLocals::Getter(AccessorGetterLocals(getter))",
        "AccessorDescriptorLocals::Setter(AccessorSetterLocals(setter))",
        "AccessorDescriptorLocals::GetterAndSetter {",
        "getter: AccessorGetterLocals(getter),",
        "setter: AccessorSetterLocals(setter),",
    ] {
        assert!(
            projection.contains(marker),
            "missing projection marker `{marker}`"
        );
    }
    assert_eq!(projection.matches("AccessorDescriptorLocals::").count(), 3);
    assert!(!projection.contains("_ =>"));
    assert!(!OBJECTS_SOURCE.contains("fn presence_of_accessor_locals("));
}

#[test]
fn every_definition_producer_names_getter_and_setter_roles() {
    let production_sources = [OBJECTS_SOURCE, FUNCTIONS_SOURCE, HOST_SOURCE].join("\n");
    assert_eq!(
        production_sources
            .matches("AccessorGetterLocals::new(")
            .count(),
        17
    );
    assert_eq!(
        production_sources
            .matches("AccessorSetterLocals::new(")
            .count(),
        8
    );
    assert_eq!(
        production_sources
            .matches("AccessorDescriptorLocals::")
            .count(),
        23
    );
    assert_eq!(
        HOST_SOURCE
            .matches("self.emit_object_define_accessor(")
            .count(),
        11
    );
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("self.emit_object_define_accessor(")
            .count(),
        3
    );
    assert_eq!(
        OBJECTS_SOURCE
            .matches("self.emit_object_define_enumerable_accessor(")
            .count(),
        4
    );
    for source in [FUNCTIONS_SOURCE, HOST_SOURCE] {
        assert!(source.contains("AccessorDescriptorLocals,"));
        assert!(source.contains("AccessorGetterLocals,"));
        assert!(source.contains("AccessorSetterLocals,"));
        assert!(source.contains("TaggedLocals,"));
    }
}

#[test]
fn focused_accessor_behavior_and_evidence_remain_in_inventory() {
    assert!(OBJECT_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_object_form_fixture()"));
    assert!(FUNCTION_CLI_TESTS.contains("fn run_wasm_class_auto_accessor_fixture()"));
    assert!(TYPED_ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_typedarray_accessors_fixture()"));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("`AccessorDescriptorLocals::{Getter, Setter, GetterAndSetter}`"));
        assert!(evidence.contains("`AccessorGetterLocals`"));
        assert!(evidence.contains("`AccessorSetterLocals`"));
    }
}
