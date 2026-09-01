const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn owner(start: &str, end: &str) -> &'static str {
    bounded(ATOMICS_SOURCE, start, end)
}

#[test]
fn atomics_integer_element_kind_local_is_private_and_capability_free() {
    assert!(ATOMICS_SOURCE.contains(
        "#[must_use = \"an Atomics integer element-kind local must be validated\"]\nstruct PendingAtomicsIntegerElementKindLocal(u32);"
    ));
    assert!(ATOMICS_SOURCE.contains(
        "#[must_use = \"a validated Atomics integer element-kind local must be released\"]\nstruct ValidatedAtomicsIntegerElementKindLocal(u32);"
    ));
    for declaration in [
        "#[must_use = \"an Atomics integer element-kind local must be validated\"]",
        "#[must_use = \"a validated Atomics integer element-kind local must be released\"]",
    ] {
        let declaration_start = ATOMICS_SOURCE.find(declaration).unwrap();
        assert!(
            !ATOMICS_SOURCE[..declaration_start]
                .lines()
                .rev()
                .find(|line| !line.trim().is_empty())
                .is_some_and(|line| line.trim().starts_with("#[derive(")),
            "the element-kind authority must not derive capabilities"
        );
    }
    for capability in ["Clone", "Copy", "Default", "Deref", "DerefMut"] {
        assert!(!ATOMICS_SOURCE.contains(&format!(
            "impl {capability} for ValidatedAtomicsIntegerElementKindLocal"
        )));
        assert!(!ATOMICS_SOURCE.contains(&format!(
            "impl {capability} for PendingAtomicsIntegerElementKindLocal"
        )));
    }
}

#[test]
fn one_validation_boundary_mints_all_three_integer_element_kind_authorities() {
    assert_eq!(
        ATOMICS_SOURCE
            .matches("PendingAtomicsIntegerElementKindLocal")
            .count(),
        6,
        "declaration, validation parameter/destructure, and three owners form the pending-local census"
    );
    assert_eq!(
        ATOMICS_SOURCE
            .matches("ValidatedAtomicsIntegerElementKindLocal")
            .count(),
        11,
        "declaration, implementation, validation result/mint, and seven typed consumers form the validated-local census"
    );
    assert_eq!(
        ATOMICS_SOURCE
            .matches("emit_validate_atomics_integer_element_kind")
            .count(),
        4,
        "one boundary and three owner handoffs"
    );
    assert_eq!(
        ATOMICS_SOURCE
            .matches("Ok(ValidatedAtomicsIntegerElementKindLocal(")
            .count(),
        1,
        "only the validation boundary may mint the validated authority"
    );

    let validation = owner(
        "fn emit_validate_atomics_integer_element_kind(",
        "fn emit_atomics_builtin(",
    );
    assert_eq!(
        validation
            .matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET")
            .count(),
        1
    );
    assert!(validation.contains(
        "match requirement {\n            AtomicsIntegerElementKindRequirement::AnyInteger"
    ));
    assert!(validation.contains("AtomicsIntegerElementKindRequirement::Waitable"));
    assert!(!validation.contains("_ =>"));
    assert!(
        validation.rfind("Instruction::End").unwrap()
            < validation
                .rfind("Ok(ValidatedAtomicsIntegerElementKindLocal(")
                .unwrap(),
        "the runtime rejection branch must close before the validated authority is minted"
    );
}

#[test]
fn wait_wait_async_and_integer_operations_own_one_complete_element_kind_lifecycle() {
    for (label, body, requirement, first_use) in [
        (
            "Atomics.waitAsync",
            owner(
                "fn emit_atomics_wait_async(&mut self,",
                "fn emit_atomics_wait_async_timeout_checkpoint(",
            ),
            "AtomicsIntegerElementKindRequirement::Waitable",
            "element_kind.local()",
        ),
        (
            "Atomics.wait",
            owner(
                "fn emit_atomics_wait(&mut self,",
                "fn emit_atomics_integer_operation(",
            ),
            "AtomicsIntegerElementKindRequirement::Waitable",
            "element_kind.local()",
        ),
        (
            "Atomics integer operations",
            owner(
                "fn emit_atomics_integer_operation(",
                "fn emit_atomics_friendly_element_kind_i32(",
            ),
            "AtomicsIntegerElementKindRequirement::AnyInteger",
            "&element_kind",
        ),
    ] {
        assert_eq!(
            body.matches("PendingAtomicsIntegerElementKindLocal(")
                .count(),
            1,
            "{label} pending authority producer"
        );
        assert_eq!(
            body.matches("self.emit_validate_atomics_integer_element_kind(")
                .count(),
            1,
            "{label} validation handoff"
        );
        assert_eq!(
            body.matches("element_kind.into_local()").count(),
            1,
            "{label} final release"
        );
        assert_eq!(body.matches(requirement).count(), 1, "{label} requirement");
        assert_eq!(
            body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
            0,
            "{label} must not bypass the sole validation boundary"
        );

        let pending = body.find("PendingAtomicsIntegerElementKindLocal(").unwrap();
        let validation = body
            .find("self.emit_validate_atomics_integer_element_kind(")
            .unwrap();
        let use_ = body[validation..].find(first_use).unwrap() + validation;
        let release = body.rfind("element_kind.into_local()").unwrap();
        assert!(
            pending < validation && validation < use_ && use_ < release,
            "{label} must reserve, validate, borrow and finally consume one authority"
        );
    }
}

#[test]
fn every_shared_integer_element_kind_consumer_requires_the_validated_local() {
    for function_name in [
        "emit_atomics_normalize_integer_element_i64",
        "emit_validated_atomics_bigint_element_kind_i32",
        "emit_atomics_signed_number_element_kind_i32",
        "emit_atomics_rmw_integer_element_to_i64",
        "emit_atomics_compare_exchange_integer_element_to_i64",
        "emit_atomics_load_integer_element_to_i64",
        "emit_atomics_store_integer_element_from_i64",
    ] {
        let signature_start = format!("fn {function_name}(");
        let signature = ATOMICS_SOURCE
            .split_once(&signature_start)
            .unwrap_or_else(|| panic!("missing `{function_name}`"))
            .1
            .split_once(") {")
            .unwrap_or_else(|| panic!("missing signature end for `{function_name}`"))
            .0;
        assert!(
            signature.contains("element_kind: &ValidatedAtomicsIntegerElementKindLocal"),
            "{function_name} must borrow the validated authority"
        );
        assert!(
            !signature.contains("element_kind_local: u32"),
            "{function_name} must reject an arbitrary Wasm local"
        );
    }

    assert_eq!(
        ATOMICS_SOURCE.matches("&element_kind").count(),
        19,
        "complete external borrow census for the current three owners"
    );
}
