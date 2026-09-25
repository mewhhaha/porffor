const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");
const ACCESS_SOURCE: &str = include_str!("../src/builtins/atomics/access.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/binary_data.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_atomics_revalidate_after_coercion.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn position(body: &str, needle: &str) -> usize {
    body.find(needle)
        .unwrap_or_else(|| panic!("missing `{needle}`"))
}

#[test]
fn only_the_access_module_mints_an_atomics_element_address() {
    // The tuple field is private to `atomics/access.rs`, so the parent module
    // cannot wrap a raw pointer local; the compiler rejects it.
    assert!(ACCESS_SOURCE.contains("pub(super) struct AtomicsElementAddress(u32);"));
    assert_eq!(
        ACCESS_SOURCE
            .matches("AtomicsElementAddress(address_local)")
            .count(),
        1
    );
    assert!(!ATOMICS_SOURCE.contains("AtomicsElementAddress("));
    for mint in [
        "pub(super) fn emit_revalidate_atomic_access(",
        "pub(super) fn emit_shared_atomics_element_address(",
    ] {
        assert_eq!(ACCESS_SOURCE.matches(mint).count(), 1, "{mint}");
    }
    assert_eq!(
        ATOMICS_SOURCE
            .matches("self.emit_revalidate_atomic_access(")
            .count(),
        1,
        "the integer-operation owner is the sole RevalidateAtomicAccess caller"
    );
    assert_eq!(
        ATOMICS_SOURCE
            .matches("self.emit_shared_atomics_element_address(")
            .count(),
        3,
        "notify, wait and waitAsync address a buffer already proven shared"
    );
}

#[test]
fn every_atomic_element_access_borrows_a_minted_address() {
    for function_name in [
        "emit_atomics_rmw_integer_element_to_i64",
        "emit_atomics_compare_exchange_integer_element_to_i64",
        "emit_atomics_load_integer_element_to_i64",
        "emit_atomics_store_integer_element_from_i64",
    ] {
        let signature = bounded(ATOMICS_SOURCE, &format!("fn {function_name}("), ") {");
        assert!(
            signature.contains("address: &AtomicsElementAddress"),
            "{function_name} must borrow a minted element address"
        );
        assert!(
            !signature.contains("address_local: u32"),
            "{function_name} must reject a raw address local"
        );
    }
}

#[test]
fn integer_operations_revalidate_after_their_last_coercion() {
    let body = bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_integer_operation(",
        "fn emit_atomics_friendly_element_kind_i32(",
    );
    let mint_index = position(
        body,
        "ValidatedAtomicsIndex::after_range_check(index_local)",
    );
    let range_error = position(body, "emit_throw_current_function_realm_range_error(");
    let last_coercion = body
        .rfind("emit_integer_typed_array_value_i64(replacement_payload_local, function)")
        .expect("compareExchange replacement coercion");
    let revalidation = position(body, "let address = self.emit_revalidate_atomic_access(");
    let first_access = position(body, "self.emit_atomics_store_integer_element_from_i64(");
    assert!(
        range_error < mint_index
            && mint_index < last_coercion
            && last_coercion < revalidation
            && revalidation < first_access,
        "ValidateAtomicAccess, every coercion, RevalidateAtomicAccess, then the access"
    );
    assert!(
        !body.contains("LocalSet(address_local)"),
        "the owner must not compute an address from its pre-coercion pointer"
    );
}

#[test]
fn revalidation_bounds_the_index_before_rereading_the_backing_pointer() {
    let body = bounded(
        ACCESS_SOURCE,
        "pub(super) fn emit_revalidate_atomic_access(",
        "pub(super) fn emit_shared_atomics_element_address(",
    );
    let witness = position(
        body,
        "TypedArrayWitnessUse::ValidatedMethodEntry {\n                length_local: current_length_local,\n                access: TypedArrayAccessMode::Read,",
    );
    let bound = position(body, "Instruction::LocalGet(current_length_local)");
    let range_error = position(body, "emit_throw_current_function_realm_range_error(");
    let address = position(body, "self.emit_atomics_element_address(");
    assert!(witness < bound && bound < range_error && range_error < address);

    let address_body = bounded(
        ACCESS_SOURCE,
        "    fn emit_atomics_element_address(",
        "AtomicsElementAddress(address_local)",
    );
    assert!(
        address_body.contains(
            "self.emit_load_array_buffer_data(buffer_payload_local, address_local, function);"
        ),
        "every minted address reads the backing pointer at mint time"
    );
}

#[test]
fn notify_answers_zero_for_a_non_shared_buffer_before_addressing_it() {
    let body = bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_notify(",
        "fn emit_atomics_require_agent_can_suspend(",
    );
    let count_coercion = position(
        body,
        "emit_value_to_number_payload(count_tag_local, count_payload_local, function)",
    );
    let shared_check = position(body, "OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64");
    let address = position(body, "self.emit_shared_atomics_element_address(");
    assert!(count_coercion < shared_check && shared_check < address);
}

#[test]
fn focused_cli_fixture_pins_revalidation_behavior() {
    let test = bounded(
        CLI_TESTS,
        "fn run_wasm_backend_revalidates_atomics_access_after_argument_coercion()",
        "\n#[test]",
    );
    assert!(test.contains("wasm_atomics_revalidate_after_coercion.js"));
    assert!(test.contains("number(67"));
    for marker in [
        "load index detach",
        "store value detach",
        "compareExchange replacement detach",
        "BigInt64Array add value detach",
        "compareExchange coercion order",
        "tracking view shrink",
        "tracking view partial element",
        "fixed view out of bounds",
        "grow writes visible",
        "non-shared notify after count detach",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
