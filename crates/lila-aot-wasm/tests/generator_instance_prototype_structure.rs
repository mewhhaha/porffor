//! Generator and async-generator instances take their `[[Prototype]]` from
//! `GetPrototypeFromConstructor(functionObject, default)` after parameter
//! initialization (EvaluateGeneratorBody / EvaluateAsyncGeneratorBody), not
//! from the function header's creation-time prototype snapshot or from an
//! entry-realm global.

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OWNER_SOURCE: &str = include_str!("../src/functions/generator_instance_prototype.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn generator_instance_prototype_has_one_private_owner() {
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod generator_instance_prototype;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("pub mod generator_instance_prototype;"));
    assert!(!FUNCTIONS_SOURCE.contains("pub(crate) mod generator_instance_prototype;"));
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("use generator_instance_prototype::GeneratorInstanceFamily;")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) fn emit_install_generator_instance_prototype(")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("fn emit_install_generator_instance_prototype("));
    assert!(!FUNCTIONS_SOURCE.contains("enum GeneratorInstanceFamily"));
}

#[test]
fn generator_instance_family_selects_its_own_realm_intrinsic_exhaustively() {
    let domain = bounded(
        OWNER_SOURCE,
        "pub(super) enum GeneratorInstanceFamily {",
        "\n}\n",
    );
    assert_eq!(
        domain
            .lines()
            .filter(|line| line.trim_end().ends_with(','))
            .count(),
        2
    );
    let projection = normalized(bounded(
        OWNER_SOURCE,
        "const fn default_prototype(self) -> OrdinaryDefaultPrototype {",
        "\n    }\n",
    ));
    assert!(projection.contains("Self::Generator=>OrdinaryDefaultPrototype::Generator,"));
    assert!(projection.contains("Self::AsyncGenerator=>OrdinaryDefaultPrototype::AsyncGenerator,"));
    assert!(!projection.contains("_=>"));
}

#[test]
fn generator_instance_prototype_is_an_observable_get_with_function_realm_fallback() {
    let installer = bounded(
        OWNER_SOURCE,
        "pub(super) fn emit_install_generator_instance_prototype(",
        "\n    }\n}",
    );
    let ordered = [
        "self.emit_object_read(",
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.emit_is_heap_object_like_tag_i32(prototype_tag_local, function);",
        "self.emit_required_new_target_realm_ordinary_prototype(",
        "family.default_prototype(),",
        "HEAP_PROTOTYPE_OFFSET,",
        "HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,",
    ];
    let mut cursor = 0;
    for marker in ordered {
        assert_eq!(installer.matches(marker).count(), 1, "{marker}");
        let position = installer
            .find(marker)
            .unwrap_or_else(|| panic!("missing installer step: {marker}"));
        assert!(position >= cursor, "installer step out of order: {marker}");
        cursor = position;
    }
    for forbidden in [
        "GlobalGet",
        "GLOBAL_INDEX",
        "HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET",
        "HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET",
    ] {
        assert!(
            !installer.contains(forbidden),
            "the generator instance prototype must not come from {forbidden}"
        );
    }
}

#[test]
fn generator_calls_install_the_prototype_after_parameter_initialization() {
    let call_path = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_handle_call_with_argv_inner(",
        "    pub(crate) fn emit_prepare_super_construct_to_locals(",
    );
    for forbidden in [
        "HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET",
        "HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET",
        "GENERATOR_PROTOTYPE_GLOBAL_INDEX",
    ] {
        assert!(
            !call_path.contains(forbidden),
            "the generator call path must not select an instance prototype from {forbidden}"
        );
    }

    for (branch_start, branch_end, family) in [
        (
            "        if can_call_generator {",
            "        if can_call_async_generator {",
            "GeneratorInstanceFamily::Generator,",
        ),
        (
            "        if can_call_async_generator {",
            "        if can_call_async {",
            "GeneratorInstanceFamily::AsyncGenerator,",
        ),
    ] {
        let branch = bounded(call_path, branch_start, branch_end);
        assert!(
            branch.contains("self.emit_alloc_plain_object_with_prototype(None, None, function)?;"),
            "{family} allocates its unobservable instance without a placeholder prototype"
        );
        let initialization = branch
            .find("self.emit_propagate_throw_from_locals_if_needed(\n                initialization_payload_local,")
            .unwrap_or_else(|| panic!("{family} must propagate abrupt parameter initialization"));
        assert_eq!(
            branch
                .matches("self.emit_install_generator_instance_prototype(")
                .count(),
            1,
            "{family}"
        );
        let install = branch
            .find("self.emit_install_generator_instance_prototype(")
            .expect("generator instance prototype installer call");
        assert!(
            install > initialization,
            "{family} must read functionObject.prototype only after parameter initialization"
        );
        assert_eq!(branch.matches(family).count(), 1, "{family}");
    }
}
