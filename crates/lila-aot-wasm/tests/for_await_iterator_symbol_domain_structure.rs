const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const SYMBOL_OWNER_SOURCE: &str = include_str!("../src/control_flow/for_await_iterator_symbol.rs");

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
fn for_await_iterator_symbols_have_one_exhaustive_name_projection() {
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("mod for_await_iterator_symbol;")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("use for_await_iterator_symbol::ForAwaitIteratorSymbol;")
            .count(),
        1
    );
    assert!(!CONTROL_FLOW_SOURCE.contains("pub mod for_await_iterator_symbol;"));
    assert!(!CONTROL_FLOW_SOURCE.contains("pub(crate) mod for_await_iterator_symbol;"));
    assert!(!CONTROL_FLOW_SOURCE.contains("enum ForAwaitIteratorSymbol"));
    assert!(!CONTROL_FLOW_SOURCE.contains("fn emit_for_await_well_known_symbol_read("));

    let symbols = bounded(
        SYMBOL_OWNER_SOURCE,
        "pub(super) enum ForAwaitIteratorSymbol {",
        "}",
    );
    let variants = symbols
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["AsyncIterator,", "Iterator,"]);
    assert_eq!(
        SYMBOL_OWNER_SOURCE
            .matches("pub(super) enum ForAwaitIteratorSymbol {")
            .count(),
        1
    );
    for visibility in ["pub enum", "pub(crate) enum"] {
        assert!(!SYMBOL_OWNER_SOURCE.contains(&format!("{visibility} ForAwaitIteratorSymbol")));
    }
    assert!(!SYMBOL_OWNER_SOURCE.contains("impl Default for ForAwaitIteratorSymbol"));
    assert!(!SYMBOL_OWNER_SOURCE.contains("impl From<&str> for ForAwaitIteratorSymbol"));

    let projection = normalized(bounded(
        SYMBOL_OWNER_SOURCE,
        "impl ForAwaitIteratorSymbol {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    for mapping in [
        "Self::AsyncIterator=>\"Symbol.asyncIterator\"",
        "Self::Iterator=>\"Symbol.iterator\"",
    ] {
        assert_eq!(
            projection.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }
    assert_eq!(projection.matches("=>").count(), 2);
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
}

#[test]
fn well_known_symbol_reader_accepts_only_the_closed_domain() {
    let signature = bounded(
        SYMBOL_OWNER_SOURCE,
        "pub(super) fn emit_for_await_well_known_symbol_read(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("symbol: ForAwaitIteratorSymbol,"));
    assert!(!signature.contains("key: &str"));

    let reader = normalized(bounded(
        SYMBOL_OWNER_SOURCE,
        "pub(super) fn emit_for_await_well_known_symbol_read(",
        "\n    }\n}\n",
    ));
    assert_eq!(reader.matches("letkey=symbol.name();").count(), 1);
    assert!(reader.contains("ValueInfo::new(ValueKind::Symbol)"));
    assert!(reader.contains("PropertyKeyIr::StringExpr(Box::new(symbol_key))"));
    assert!(!reader.contains("debug_assert!"));
    assert!(!reader.contains("starts_with"));
    assert!(!reader.contains("_=>"));
    assert!(!reader.contains("unreachable!"));
}

#[test]
fn async_symbol_precedes_the_nullish_sync_fallback() {
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_async_for_of_iterator(",
        "pub(crate) fn compile_async_disposable_for_of_iterator(",
    );
    assert_eq!(
        owner
            .matches("self.emit_for_await_well_known_symbol_read(")
            .count(),
        2
    );
    assert_eq!(
        owner
            .matches("ForAwaitIteratorSymbol::AsyncIterator,")
            .count(),
        1
    );
    assert_eq!(
        owner.matches("ForAwaitIteratorSymbol::Iterator,").count(),
        1
    );
    assert!(!owner.contains("\"Symbol.asyncIterator\""));
    assert!(!owner.contains("\"Symbol.iterator\""));

    let async_iterator = owner
        .find("ForAwaitIteratorSymbol::AsyncIterator,")
        .expect("missing async-iterator acquisition");
    let nullish = owner
        .find("self.compile_nullish_tagged_i32(method_tag_local, function)?;")
        .expect("missing nullish fallback gate");
    let iterator = owner
        .find("ForAwaitIteratorSymbol::Iterator,")
        .expect("missing sync-iterator fallback");
    assert!(async_iterator < nullish);
    assert!(nullish < iterator);

    let fallback = bounded(
        owner,
        "if !iterable_is_statically_nullish {",
        "self.emit_propagate_current_completion_if_throw(function);",
    );
    assert_eq!(
        fallback
            .matches("ForAwaitIteratorSymbol::Iterator,")
            .count(),
        1
    );
    assert!(!fallback.contains("ForAwaitIteratorSymbol::AsyncIterator,"));
}
