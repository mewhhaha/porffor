const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_get_direct_descriptor_invariants.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn proxy_get_trap_result_roles_have_no_incidental_capabilities() {
    for (role, declaration_start) in [
        (
            "PendingProxyGetTrapResultLocals",
            "/// A Proxy `[[Get]]` trap result whose completion has not yet been consumed.",
        ),
        (
            "NormalProxyGetTrapResultLocals",
            "/// A Proxy `[[Get]]` trap result after abrupt completion has been routed.",
        ),
    ] {
        let declaration = bounded(OBJECTS_SOURCE, declaration_start, &format!("impl {role}"));
        assert!(
            !declaration.contains("#[derive"),
            "{role} derives a capability"
        );

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
                !OBJECTS_SOURCE.contains(&format!("impl {capability} for {role}")),
                "{role} manually implements {capability}"
            );
        }
    }

    assert!(OBJECTS_SOURCE.contains(
        "#[must_use = \"a pending Proxy Get trap result must be normalized before inspection\"]"
    ));
    assert!(OBJECTS_SOURCE.contains(
        "#[must_use = \"a normal Proxy Get trap result must be consumed by its invariant\"]"
    ));
}

#[test]
fn one_transition_routes_completion_before_publishing_the_normal_result() {
    assert_eq!(
        OBJECTS_SOURCE
            .matches("PendingProxyGetTrapResultLocals")
            .count(),
        4
    );
    assert_eq!(
        OBJECTS_SOURCE
            .matches("NormalProxyGetTrapResultLocals")
            .count(),
        6
    );
    assert_eq!(
        OBJECTS_SOURCE
            .matches("PendingProxyGetTrapResultLocals::new(payload_local, tag_local)")
            .count(),
        1
    );

    let transition = bounded(
        OBJECTS_SOURCE,
        "fn emit_normal_proxy_get_trap_result(",
        "fn emit_proxy_get_descriptor_same_value_i32(",
    );
    assert!(transition.contains("pending: PendingProxyGetTrapResultLocals"));
    assert!(transition.contains(") -> NormalProxyGetTrapResultLocals"));
    assert_eq!(
        transition
            .matches("self.emit_return_current_completion_if_throw(function);")
            .count(),
        1
    );
    assert_eq!(
        transition
            .matches("NormalProxyGetTrapResultLocals(pending.0)")
            .count(),
        1
    );
}

#[test]
fn the_normal_result_has_one_consuming_invariant_and_borrowed_observers() {
    let invariant = bounded(
        OBJECTS_SOURCE,
        "fn emit_proxy_get_invariant_check(",
        "fn reserve_proxy_get_descriptor_locals(",
    );
    assert!(invariant.contains("trap_result: NormalProxyGetTrapResultLocals"));
    assert_eq!(
        invariant
            .matches("trap_result.emit_undefined_i32(function);")
            .count(),
        1
    );
    assert_eq!(
        invariant
            .matches("&trap_result,\n            descriptor.data_value,")
            .count(),
        1
    );

    let trap_call = bounded(
        OBJECTS_SOURCE,
        "self.emit_function_handle_call_without_throw_propagation(",
        "function.instruction(&Instruction::Else);",
    );
    let transition_offset = trap_call
        .find("self.emit_normal_proxy_get_trap_result(")
        .expect("trap result must cross the normal-completion transition");
    let invariant_offset = trap_call
        .find("self.emit_proxy_get_invariant_check(")
        .expect("normal trap result must reach the invariant");
    assert!(transition_offset < invariant_offset);
}

#[test]
fn exact_cli_witness_pins_abrupt_result_identity_and_descriptor_invariants() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_proxy_get_direct_descriptor_invariants()"));
    assert!(CLI_TESTS.contains("wasm_proxy_get_direct_descriptor_invariants.js"));
    for marker in [
        "direct thrown trap was replaced by invariant error",
        "Reflect thrown trap was replaced by invariant error",
        "callable Proxy getter is not undefined",
        "symbol object-identity SameValue",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
}
