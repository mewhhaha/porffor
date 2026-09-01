const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const OWNER_SOURCE: &str = include_str!("../src/lowering/invocation_effects.rs");
const BUILTIN_CALL_INFO_SOURCE: &str = include_str!("../src/lowering/builtin_call_info.rs");
const CALL_CANDIDATE_SOURCE: &str = include_str!("../src/lowering/call_candidate_analysis.rs");
const CALL_EXPRESSION_SOURCE: &str = include_str!("../src/lowering/call_expression.rs");
const NON_PROPERTY_CALL_SOURCE: &str =
    include_str!("../src/lowering/call_expression/non_property_call.rs");
const NEW_EXPRESSION_SOURCE: &str = include_str!("../src/lowering/new_expression.rs");

#[test]
fn invocation_effects_have_one_private_nonduplicable_owner() {
    assert_eq!(
        LOWERING_SOURCE
            .matches("\nmod invocation_effects;\n")
            .count(),
        1
    );
    assert!(!LOWERING_SOURCE.contains("\npub mod invocation_effects;\n"));
    assert!(!LOWERING_SOURCE.contains("pub use invocation_effects::"));

    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) struct AccountedInvocationEffects")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) enum StandardBuiltinCallAnalysis")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) enum AnalyzedInvocationEffects")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("impl Drop for AccountedInvocationEffects")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("attached_to_emitted_call: false")
            .count(),
        1,
        "only the recorded constructor may create an unattached proof"
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("effects: AccountedInvocationEffects::recorded()")
            .count(),
        1
    );
    for source in [
        OWNER_SOURCE,
        LOWERING_SOURCE,
        CALL_CANDIDATE_SOURCE,
        CALL_EXPRESSION_SOURCE,
        NON_PROPERTY_CALL_SOURCE,
        NEW_EXPRESSION_SOURCE,
    ] {
        assert!(
            !source.contains("Option<AccountedInvocationEffects>"),
            "analyzed invocation effects must not regain an ambiguous optional carrier"
        );
        assert!(
            !source.contains("accounted_invocation_effects.is_none()"),
            "emitters must not infer effect analysis from an empty option"
        );
    }
    let mut derived_traits = Vec::new();
    let mut remaining_owner_source = OWNER_SOURCE;
    while let Some((_, after_derive)) = remaining_owner_source.split_once("#[derive(") {
        let (traits, remaining) = after_derive
            .split_once(")]")
            .expect("derive attribute should terminate");
        derived_traits.extend(traits.split(',').map(str::trim));
        remaining_owner_source = remaining;
    }
    for forbidden_trait in ["Clone", "Copy"] {
        assert!(
            !derived_traits.contains(&forbidden_trait),
            "the linear invocation-effects proof must not derive {forbidden_trait}"
        );
    }
    for forbidden in [
        "impl Clone for AccountedInvocationEffects",
        "impl Copy for AccountedInvocationEffects",
    ] {
        assert!(
            !OWNER_SOURCE.contains(forbidden),
            "the linear invocation-effects proof must not gain {forbidden}"
        );
    }

    assert!(!BUILTIN_CALL_INFO_SOURCE.contains("struct AccountedInvocationEffects"));
    assert!(!BUILTIN_CALL_INFO_SOURCE.contains("enum StandardBuiltinCallAnalysis"));
    for forbidden_reexport in [
        "pub use super::invocation_effects",
        "pub(super) use super::invocation_effects",
        "pub(crate) use super::invocation_effects",
    ] {
        assert!(!BUILTIN_CALL_INFO_SOURCE.contains(forbidden_reexport));
    }
}
