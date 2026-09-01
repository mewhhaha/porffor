const INTL_SOURCE: &str = include_str!("../src/builtins/intl.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/intl-locale-string-slot-dispatch.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T23: &str = include_str!("../../../tasks/23-intl402.md");

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
fn locale_string_slot_projects_offset_and_optionality_exhaustively() {
    let slot_domain = normalized(bounded(
        INTL_SOURCE,
        "enum IntlLocaleStringSlot {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    let declaration_offset = INTL_SOURCE
        .find("enum IntlLocaleStringSlot {")
        .expect("Intl.Locale string-slot declaration");
    let declaration_prefix = INTL_SOURCE[..declaration_offset]
        .rsplit_once("\n\n")
        .expect("Intl.Locale string-slot declaration prefix")
        .1;
    assert!(!declaration_prefix.contains("#[derive("));
    assert!(!INTL_SOURCE.contains("pub(crate) enum IntlLocaleStringSlot"));
    assert!(!INTL_SOURCE.contains("pub(super) enum IntlLocaleStringSlot"));
    assert_eq!(slot_domain.matches("constfnoffset(&self)->u64").count(), 1);
    assert_eq!(
        slot_domain
            .matches("constfnis_optional(&self)->bool")
            .count(),
        1
    );
    assert!(!slot_domain.contains("constfnoffset(self)"));
    assert!(!slot_domain.contains("constfnis_optional(self)"));
    for variant in ["Tag", "Language", "Script", "Region", "BaseName"] {
        assert_eq!(
            slot_domain.matches(&format!("Self::{variant}=>")).count(),
            2,
            "variant `{variant}` must project both offset and optionality"
        );
    }
    for mapping in [
        "Self::Tag=>HEAP_INTL_LOCALE_TAG_OFFSET",
        "Self::Language=>HEAP_INTL_LOCALE_LANGUAGE_OFFSET",
        "Self::Script=>HEAP_INTL_LOCALE_SCRIPT_OFFSET",
        "Self::Region=>HEAP_INTL_LOCALE_REGION_OFFSET",
        "Self::BaseName=>HEAP_INTL_LOCALE_BASE_NAME_OFFSET",
        "Self::Tag=>false",
        "Self::Language=>false",
        "Self::Script=>true",
        "Self::Region=>true",
        "Self::BaseName=>false",
    ] {
        assert_eq!(
            slot_domain.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }
    assert_eq!(slot_domain.matches("=>").count(), 10);
    assert!(!slot_domain.contains("_=>"));
    assert!(!slot_domain.contains("unreachable!"));
}

#[test]
fn locale_string_slot_emitter_accepts_only_the_closed_domain() {
    let signature = bounded(
        INTL_SOURCE,
        "fn emit_intl_locale_string_slot(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("slot: IntlLocaleStringSlot,"));
    assert!(!signature.contains("slot_offset: u64"));
    assert!(!signature.contains("optional: bool"));

    let emitter = bounded(
        INTL_SOURCE,
        "fn emit_intl_locale_string_slot(",
        "pub(crate) fn emit_intl_get_canonical_locales(",
    );
    assert_eq!(emitter.matches("slot.offset()").count(), 1);
    assert_eq!(emitter.matches("slot.is_optional()").count(), 1);
}

#[test]
fn locale_dispatch_maps_each_builtin_to_one_named_slot() {
    assert!(!STANDARD_SOURCE.contains("IntlLocaleStringSlot"));
    assert!(!STANDARD_SOURCE.contains("emit_intl_locale_string_slot("));

    let dispatch = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::IntlLocaleConstructor => {",
        "StandardBuiltinId::DateConstructor => {",
    ));
    for (builtin, method, variant) in [
        ("LanguageGetter", "language_getter", "Language"),
        ("ScriptGetter", "script_getter", "Script"),
        ("RegionGetter", "region_getter", "Region"),
        ("BaseNameGetter", "base_name_getter", "BaseName"),
        ("ToString", "to_string", "Tag"),
    ] {
        let fixed_entry = format!("pub(super) fn emit_intl_locale_{method}_builtin(");
        let fixed_body = INTL_SOURCE
            .split_once(&fixed_entry)
            .unwrap_or_else(|| panic!("missing fixed Intl.Locale {method} entry"))
            .1
            .split_once("\n    }")
            .expect("fixed Intl.Locale entry end")
            .0;
        assert_eq!(INTL_SOURCE.matches(&fixed_entry).count(), 1);
        assert_eq!(
            fixed_body
                .matches(&format!("IntlLocaleStringSlot::{variant}"))
                .count(),
            1
        );
        let mapping = format!(
            "IntlLocalePrototype{builtin}=>{{self.emit_intl_locale_{method}_builtin(function)?;}}"
        );
        assert_eq!(dispatch.matches(&mapping).count(), 1, "mapping `{mapping}`");
    }
}

#[test]
fn locale_string_slot_contract_records_exact_witnesses_and_nonclaims() {
    for marker in [
        "private, non-derived domain",
        "five fixed entries",
        "00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a",
        "4b346dcd2c819c503603ed7c08842e577d4b893dc98aa1f33c5f7d2c864cd134",
        "no new Intl behavior",
        "does not close T23",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
    for task in [TASK_T02, TASK_T23] {
        assert!(task.contains("intl-locale-string-slot-dispatch.md"));
        assert!(task.contains("00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a"));
        assert!(task.contains("4/4"));
        assert!(task.contains("no new Intl behavior"));
    }
}
