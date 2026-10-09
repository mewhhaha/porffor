//! Generator and async-generator instances take their `[[Prototype]]` from
//! `GetPrototypeFromConstructor(functionObject, default)` after parameter
//! initialization (EvaluateGeneratorBody / EvaluateAsyncGeneratorBody), not
//! from the function header's creation-time prototype snapshot or from an
//! entry-realm global.

const FUNCTIONS_SOURCE: &str = include_str!("../../src/functions.rs");
const OWNER_SOURCE: &str = include_str!("../../src/functions/generator_instance_prototype.rs");

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
            .matches("pub(super) fn emit_generator_instance_prototype(")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("fn emit_generator_instance_prototype("));
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
