const FINALIZATION_REGISTRY_SOURCE: &str = include_str!("../src/builtins/finalization_registry.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const CELL_LAYOUT_SOURCE: &str = include_str!("../src/heap_finalization_registry_cell_layout.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/finalization-registry-cell-state.md");
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

#[test]
fn finalization_registry_cell_state_is_an_exact_capability_free_wire_domain() {
    let declaration = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "finalization_registry_cell_state_domain! {",
        "\n}\n",
    );
    assert_eq!(normalized(declaration), "Vacant=0,Occupied=1,");

    let domain = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "macro_rules! finalization_registry_cell_state_domain {",
        "finalization_registry_cell_state_domain! {",
    );
    for evidence in [
        "enum FinalizationRegistryCellState",
        "const ALL: &'static [Self]",
        "const fn word(&self) -> u64",
        "match self",
    ] {
        assert!(domain.contains(evidence), "domain evidence `{evidence}`");
    }
    for forbidden in ["#[derive", "Default", "pub enum", "PartialEq", "Eq"] {
        assert!(!domain.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn typed_serialization_is_the_only_cell_state_write_authority() {
    let serializer = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    fn store_finalization_registry_cell_state(",
        "    fn emit_finalization_registry_record_from_receiver(",
    );
    assert!(serializer.contains("state: &FinalizationRegistryCellState,"));
    assert!(serializer.contains("HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,"));
    assert!(serializer.contains("state.word(),"));

    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("store_finalization_registry_cell_state(")
            .count(),
        4,
        "one serializer plus occupied registration, vacant unregistration and typed relocation"
    );
    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET")
            .count(),
        3,
        "the state word is read by relocation and unregistration and written only by the serializer"
    );

    let layout = bounded(
        CELL_LAYOUT_SOURCE,
        "impl FinalizationRegistryCellHeapSlot {",
        "pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT",
    );
    assert!(layout.contains("name: \"state\","));
    assert!(layout.contains("offset: HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,"));
    assert!(!HEAP_SOURCE.contains("HEAP_FINALIZATION_REGISTRY_CELL_PRESENT_OFFSET"));
}

#[test]
fn every_cell_state_load_routes_exact_words_and_traps_corruption() {
    let register = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    pub(crate) fn emit_finalization_registry_register(",
        "    pub(crate) fn emit_finalization_registry_unregister(",
    );
    let unregister = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    pub(crate) fn emit_finalization_registry_unregister(",
        "    fn store_finalization_registry_cell_state(",
    );

    for route in [register, unregister] {
        assert!(route.contains("for cell_state in FinalizationRegistryCellState::ALL"));
        assert!(route.contains("Instruction::I64Eq"));
        assert!(route.contains("match cell_state"));
        assert!(route.contains("FinalizationRegistryCellState::Vacant"));
        assert!(route.contains("FinalizationRegistryCellState::Occupied"));
        assert!(route.contains("Instruction::Unreachable"));
    }
    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("for cell_state in FinalizationRegistryCellState::ALL")
            .count(),
        2
    );
    assert_eq!(
        FINALIZATION_REGISTRY_SOURCE
            .matches("Instruction::Unreachable")
            .count(),
        2
    );
    assert!(!unregister.contains("Instruction::I64Ne"));
}

#[test]
fn contract_and_task_record_the_lifecycle_invariant_and_non_claim() {
    let normalized_contract = normalized(CONTRACT);
    let normalized_task = normalized(TASK);
    for evidence in [
        "FinalizationRegistryCellState",
        "Vacant",
        "Occupied",
        "invalid persisted word",
        "weak reachability",
        "cleanup jobs",
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
