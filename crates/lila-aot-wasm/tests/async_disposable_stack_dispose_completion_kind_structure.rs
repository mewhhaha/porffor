const SOURCE: &str = include_str!("../src/builtins/async_disposable_stack.rs");
const DISPOSAL: &str = include_str!("../src/builtins/async_disposable_stack/disposal.rs");
const LAYOUTS: &str = include_str!("../src/gc_types/layouts.rs");
const VALUES: &str = include_str!("../src/gc_types/value.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}

#[test]
fn disposal_completion_kind_has_one_closed_typed_gc_domain() {
    let domain = bounded(
        SOURCE,
        "pub(crate) enum AsyncDisposableStackDisposeCompletionKind {",
        "impl AsyncDisposableStackDisposeCompletionKind {",
    );
    assert_eq!(
        domain
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        ["Normal,", "Throw,"]
    );
    assert!(!SOURCE.contains("pub enum AsyncDisposableStackDisposeCompletionKind"));
    assert!(LAYOUTS.contains("COMPLETION_KIND: crate::builtins::AsyncDisposableStackDisposeCompletionKind, Mutable, NonNullable;"));
    // Unlike the retired untyped heap word, the private GC field can only
    // receive the closed enum or a local read from that same field type.
    let local = bounded(
        VALUES,
        "pub(crate) struct GcI32DomainLocal<V: GcI32Constant> {",
        "/// A validated native I64 domain",
    );
    assert!(local.contains("pub(crate) fn set_constant(&self, value: V,"));
    assert!(local.contains("pub(crate) fn copy_from(&self, source: &Self,"));
    assert!(local.contains("destination: &GcI32DomainLocal<V>"));
    assert!(!local.contains("pub(crate) fn store("));
}

#[test]
fn completion_kind_serialization_is_exhaustive_and_exact() {
    let projection = bounded(
        SOURCE,
        "pub(crate) const fn wire_code(self) -> i32 {",
        "enum AdsReceiverPolicy {",
    );
    assert_eq!(
        projection
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("Self::"))
            .collect::<Vec<_>>(),
        ["Self::Normal => 0,", "Self::Throw => 1,"]
    );
    assert!(!projection.contains("_ =>"));
    let codec = bounded(
        VALUES,
        "impl GcI32Constant for crate::builtins::AsyncDisposableStackDisposeCompletionKind {",
        "macro_rules! intl_number_i32_codec {",
    );
    assert!(codec.contains("self.wire_code()"));
}

#[test]
fn completion_kind_has_one_typed_store_and_two_typed_reads() {
    assert_eq!(
        DISPOSAL
            .matches("AsyncDisposableStackDisposalSchema::COMPLETION_KIND")
            .count(),
        3
    );
    assert_eq!(DISPOSAL.matches(".store_domain(&kind, f)").count(), 2);
    assert_eq!(
        DISPOSAL
            .matches("GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Throw)")
            .count(),
        1
    );
    assert_eq!(
        DISPOSAL
            .matches("GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Normal)")
            .count(),
        1
    );
    assert!(!DISPOSAL.contains("DISPOSE_COMPLETION_KIND_OFFSET"));
    assert!(!DISPOSAL.contains("has_error_local"));
    assert!(VALUES.contains("pub(crate) fn operand(&self) -> GcOperand<'_, V, NonNullable>"));
}

#[test]
fn initialization_suppression_and_settlement_name_their_completion_routes() {
    let initialization = bounded(
        DISPOSAL,
        "pub(crate) fn emit_async_disposable_stack_dispose_async(",
        "pub(crate) fn emit_async_disposable_stack_dispose_async_fulfilled(",
    );
    assert!(initialization.contains("s.struct_type::<AsyncDisposableStackDisposal>().construct("));
    assert!(initialization
        .contains("GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Normal)"));
    let suppression = bounded(DISPOSAL, "fn emit_ads_record_error(", "fn emit_ads_await(");
    assert!(suppression.contains("AsyncDisposableStackDisposeCompletionKind::Throw.wire_code()"));
    assert!(suppression.contains("self.emit_alloc_suppressed_error_instance("));
    assert!(suppression.contains("NonArrayRealmIntrinsicSlot::SuppressedErrorPrototype"));
    assert!(suppression
        .contains("GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Throw)"));
    let settlement = bounded(DISPOSAL, "fn emit_ads_finish(", "fn emit_ads_dispose_step(");
    assert!(settlement.contains("AsyncDisposableStackDisposeCompletionKind::Throw.wire_code()"));
    let reject = settlement.find("PromiseSettlement::Reject").unwrap();
    let alternative = settlement.find("Instruction::Else").unwrap();
    let fulfill = settlement.find("PromiseSettlement::Fulfill").unwrap();
    assert!(reject < alternative && alternative < fulfill);
}
