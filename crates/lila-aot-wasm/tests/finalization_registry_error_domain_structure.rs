const FINALIZATION_REGISTRY_SOURCE: &str = include_str!("../src/builtins/finalization_registry.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/finalization-registry-error-domain.md");
const TASK: &str = include_str!("../../../tasks/21-symbols-collections-weakrefs.md");

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

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn finalization_registry_type_error_is_an_exact_capability_free_domain() {
    let declaration = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "enum FinalizationRegistryTypeError {",
        "impl FinalizationRegistryTypeError {",
    );
    assert_eq!(
        normalized(declaration),
        concat!(
            "ConstructorRequiresNew,",
            "CleanupCallbackNotCallable,",
            "TargetCannotBeHeldWeakly,",
            "TargetMatchesHoldings,",
            "UnregisterTokenCannotBeHeldWeakly,",
            "ReceiverMissingCells,",
            "}"
        )
    );
    let declaration_offset = FINALIZATION_REGISTRY_SOURCE
        .find("enum FinalizationRegistryTypeError {")
        .expect("FinalizationRegistry TypeError declaration");
    let preceding_item = &FINALIZATION_REGISTRY_SOURCE[..declaration_offset];
    assert!(preceding_item.ends_with("use crate::functions::NewTargetPrototypeFallback;\n\n"));
    for forbidden in [
        "pub enum FinalizationRegistryTypeError",
        "#[derive",
        "Default",
    ] {
        assert!(!declaration.contains(forbidden), "found `{forbidden}`");
    }

    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("FinalizationRegistryTypeError")
            .count(),
        11,
        "one declaration, one implementation, one typed emitter input and eight producers"
    );
}

#[test]
fn exhaustive_projection_is_the_only_message_authority() {
    let projection = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    fn message(self) -> &'static str {",
        "\n    }\n}\n\nimpl<'a> FunctionBuilder<'a>",
    );
    for (variant, message, expected_mentions) in [
        (
            "ConstructorRequiresNew",
            "FinalizationRegistry constructor requires new",
            1,
        ),
        (
            "CleanupCallbackNotCallable",
            "FinalizationRegistry cleanup callback is not callable",
            1,
        ),
        (
            "TargetCannotBeHeldWeakly",
            "FinalizationRegistry target cannot be held weakly",
            1,
        ),
        (
            "TargetMatchesHoldings",
            "FinalizationRegistry target and holdings must not be the same value",
            1,
        ),
        (
            "UnregisterTokenCannotBeHeldWeakly",
            "FinalizationRegistry unregister token cannot be held weakly",
            2,
        ),
        (
            "ReceiverMissingCells",
            "FinalizationRegistry method receiver does not have [[Cells]]",
            2,
        ),
    ] {
        assert_eq!(
            FINALIZATION_REGISTRY_SOURCE
                .matches(&format!("FinalizationRegistryTypeError::{variant}"))
                .count(),
            expected_mentions,
            "variant `{variant}` must have its exact producer set"
        );
        assert_eq!(
            FINALIZATION_REGISTRY_SOURCE
                .matches(&format!("\"{message}\""))
                .count(),
            1,
            "message `{message}` must exist only in the exhaustive projection"
        );
        assert!(projection.contains(&format!("Self::{variant}")));
    }
    for forbidden in ["_ =>", "if self", "matches!(self", "==", "!="] {
        assert!(!projection.contains(forbidden), "found `{forbidden}`");
    }

    let emitter = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    fn emit_finalization_registry_type_error(",
        "\n    }\n}",
    );
    assert!(emitter.contains("error: FinalizationRegistryTypeError,"));
    assert!(emitter.contains("error.message(),"));
    assert!(!emitter.contains("message: &'static str"));
    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("self.emit_finalization_registry_type_error(")
            .count(),
        8
    );
}

#[test]
fn typed_failures_preserve_the_register_and_receiver_ordering() {
    let register = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    pub(crate) fn emit_finalization_registry_register(",
        "    pub(crate) fn emit_finalization_registry_unregister(",
    );
    assert_before(
        register,
        "self.emit_finalization_registry_record_from_receiver(registry_record_local, function)?;",
        "self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);",
    );
    assert_before(
        register,
        "FinalizationRegistryTypeError::TargetCannotBeHeldWeakly",
        "self.emit_builtin_arg_to_locals(1, holdings_payload_local, holdings_tag_local, function);",
    );
    assert_before(
        register,
        "FinalizationRegistryTypeError::TargetMatchesHoldings",
        "self.emit_builtin_arg_to_locals(2, token_payload_local, token_tag_local, function);",
    );
    assert_before(
        register,
        "FinalizationRegistryTypeError::UnregisterTokenCannotBeHeldWeakly",
        "HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET",
    );

    let receiver = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    fn emit_finalization_registry_record_from_receiver(",
        "    fn emit_finalization_registry_type_error(",
    );
    assert_eq!(
        receiver
            .matches("FinalizationRegistryTypeError::ReceiverMissingCells")
            .count(),
        2,
        "non-object and brand-mismatch failures share the typed receiver state"
    );
    assert_before(
        receiver,
        "self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;",
        "FinalizationRegistryTypeError::ReceiverMissingCells",
    );
    assert_before(
        receiver,
        "OBJECT_INTERNAL_BRAND_FINALIZATION_REGISTRY as i64",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
    );
}

#[test]
fn contract_and_task_record_the_invariant_and_non_claim() {
    let normalized_contract = normalized(CONTRACT);
    let normalized_task = normalized(TASK);
    for evidence in [
        "FinalizationRegistryTypeError",
        "arbitrary diagnostic string",
        "weak reachability",
        "does not change emitted Wasm",
    ] {
        let normalized_evidence = normalized(evidence);
        assert!(
            normalized_contract.contains(&normalized_evidence),
            "contract evidence `{evidence}`"
        );
        assert!(
            normalized_task.contains(&normalized_evidence),
            "task evidence `{evidence}`"
        );
    }
}
