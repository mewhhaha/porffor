const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");

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
fn completion_abi_slot_has_one_private_source_of_truth() {
    assert!(OPERATIONS_SOURCE.contains("pub struct CompletionAbiSlot(CompletionKindIr);"));
    let descriptor = bounded(
        OPERATIONS_SOURCE,
        "pub struct CompletionAbiSlot",
        "impl CompletionAbiSlot {",
    );
    assert!(!descriptor.contains("pub kind: CompletionKindIr"));
    for redundant_field in [
        "pub name:",
        "pub code:",
        "pub carries_value:",
        "pub carries_target:",
    ] {
        assert!(!descriptor.contains(redundant_field));
    }

    let projections = bounded(
        OPERATIONS_SOURCE,
        "impl CompletionAbiSlot {",
        "pub const COMPLETION_ABI_SLOTS",
    );
    for projection in ["kind", "name", "code", "carries_value", "carries_target"] {
        assert!(
            projections.contains(&format!("pub const fn {projection}(self)")),
            "missing {projection} projection"
        );
    }
}

#[test]
fn completion_kind_inventory_is_the_only_slot_inventory() {
    let declaration = bounded(
        OPERATIONS_SOURCE,
        "macro_rules! completion_kinds",
        "completion_kinds! {",
    );
    assert!(declaration.contains("pub enum CompletionKindIr"));
    assert!(declaration.contains("pub const ALL: &'static [Self] = &[ $( Self::$variant, )+ ];"));
    assert!(declaration.contains("$( completion_abi_slot(CompletionKindIr::$variant), )+"));
    for projection in ["name", "abi_code", "carries_value", "carries_target"] {
        assert!(declaration.contains(&format!("pub const fn {projection}(self)")));
    }
    assert!(declaration.contains("while index < CompletionKindIr::ALL.len()"));
    assert!(declaration.contains("CompletionKindIr::ALL[index].abi_code() == index as i64"));

    let rows = bounded(
        OPERATIONS_SOURCE,
        "completion_kinds! {",
        "#[derive(Debug, Clone, PartialEq, Eq)]",
    );
    assert_eq!(rows.matches("=> {").count(), 6);
    for variant in ["Normal", "Throw", "Return", "Break", "Continue", "Empty"] {
        assert!(rows.contains(&format!("{variant} => {{")));
    }

    assert!(LIB_SOURCE.contains("completion_abi_slot, completion_abi_slots,"));
    assert!(!LIB_SOURCE.contains("find_completion_abi_slot"));
}

#[test]
fn abrupt_completion_inventory_is_emitted_with_its_closed_domain() {
    let declaration = bounded(
        OPERATIONS_SOURCE,
        "macro_rules! completion_abrupt_kinds",
        "completion_abrupt_kinds! {",
    );
    assert!(declaration.contains("pub enum CompletionAbruptKind"));
    assert!(declaration.contains("pub const ALL: &'static [Self] = &[ $( Self::$variant, )+ ];"));

    let rows = bounded(
        OPERATIONS_SOURCE,
        "completion_abrupt_kinds! {",
        "macro_rules! completion_kinds",
    );
    assert!(rows.contains("Throw, Return, Break, Continue"));
    assert!(!OPERATIONS_SOURCE.contains("mask == 0b1111"));
    assert!(OPERATIONS_SOURCE.contains(
        "const CONTROL_COMPLETIONS: &[CompletionAbruptKind] = CompletionAbruptKind::ALL;"
    ));
}

#[test]
fn completion_abi_slot_construction_is_total_and_closed() {
    let constructor = bounded(
        OPERATIONS_SOURCE,
        "pub const fn completion_abi_slot(",
        "#[derive(Debug, Clone, PartialEq, Eq)]",
    );
    assert!(constructor.contains("CompletionAbiSlot::new(kind)"));
    assert!(!constructor.contains("Option"));
    assert!(!OPERATIONS_SOURCE.contains("find_completion_abi_slot"));
}
