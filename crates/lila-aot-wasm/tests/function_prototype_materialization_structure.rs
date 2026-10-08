const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/function-prototype-materialization.md");
const TASK: &str = include_str!("../../../tasks/09-functions-classes-private-elements.md");

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn materialization_policy_is_the_exact_non_capability_domain() {
    let declaration_marker = "pub(crate) enum FunctionPrototypeMaterialization {";
    let declaration_offset = FUNCTIONS_SOURCE
        .find(declaration_marker)
        .expect("materialization-policy declaration");
    let preceding_item_end = FUNCTIONS_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("item before materialization-policy declaration");
    let declaration_end = FUNCTIONS_SOURCE[declaration_offset..]
        .find("\n}")
        .map(|offset| declaration_offset + offset + 2)
        .expect("materialization-policy declaration end");
    let declaration_region = &FUNCTIONS_SOURCE[preceding_item_end + 1..declaration_end];
    let expected_declaration = r#"

/// Whether function allocation also creates the default own `prototype`
/// property. This policy is deliberately separate from semantic
/// constructability; realm bootstrap supplies a few intrinsic prototypes.
pub(crate) enum FunctionPrototypeMaterialization {
    Automatic,
    BootstrapSupplied,
}
"#;
    assert_eq!(
        normalized(declaration_region),
        normalized(expected_declaration),
        "the exact adjacent declaration must remain attribute-free"
    );

    for forbidden in [
        "impl FunctionPrototypeMaterialization",
        "for FunctionPrototypeMaterialization",
    ] {
        assert!(
            !FUNCTIONS_SOURCE.contains(forbidden) && !BOOTSTRAP_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn contract_and_t09_record_the_exhaustive_source_equivalence() {
    for marker in [
        "seven producer sites",
        "exhaustive two-arm projection",
        "changes no emitted instruction",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("FunctionPrototypeMaterialization::{Automatic, BootstrapSupplied}"));
    assert!(TASK.contains("function-prototype-materialization.md"));
}
