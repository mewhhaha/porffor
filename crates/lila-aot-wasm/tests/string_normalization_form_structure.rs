const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");

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
fn normalization_form_owns_the_complete_domain_and_exhaustive_projections() {
    let declaration_start = STRING_SOURCE
        .find("pub(crate) enum StringNormalizationForm {")
        .expect("missing normalization-form declaration");
    let preceding_declaration = STRING_SOURCE[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing declaration before normalization form");
    assert_eq!(preceding_declaration.trim(), "};");

    let declaration = bounded(
        STRING_SOURCE,
        "pub(crate) enum StringNormalizationForm {",
        "\n}\n\nimpl StringNormalizationForm {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Nfc,", "Nfd,", "Nfkc,", "Nfkd,"]);

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!STRING_SOURCE.contains(&format!("impl {capability} for StringNormalizationForm")));
    }

    let projections = normalized(bounded(
        STRING_SOURCE,
        "impl StringNormalizationForm {",
        "mod empty_string_split_units {",
    ));
    assert!(projections.contains("constALL:[Self;4]=[Self::Nfc,Self::Nfd,Self::Nfkc,Self::Nfkd];"));
    assert!(projections.contains("constfnspelling(&self)->&'staticstr{"));
    assert!(projections.contains("constfnruntime_kind(self)->i64{"));
    assert!(projections.contains("fndecomposition_table(&self,strings:&StringPool)->(u32,u32){"));
    assert!(projections.contains("constfncomposes(self)->bool{"));
    for mapping in [
        "Self::Nfc=>\"NFC\"",
        "Self::Nfd=>\"NFD\"",
        "Self::Nfkc=>\"NFKC\"",
        "Self::Nfkd=>\"NFKD\"",
        "Self::Nfc=>0",
        "Self::Nfd=>1",
        "Self::Nfkc=>2",
        "Self::Nfkd=>3",
        "Self::Nfc|Self::Nfd=>(strings.canonical_decomposition_table_ptr,strings.canonical_decomposition_count,)",
        "Self::Nfkc|Self::Nfkd=>(strings.compatibility_decomposition_table_ptr,strings.compatibility_decomposition_count,)",
        "Self::Nfc|Self::Nfkc=>true",
        "Self::Nfd|Self::Nfkd=>false",
    ] {
        assert_eq!(
            projections.matches(mapping).count(),
            1,
            "normalization projection `{mapping}`"
        );
    }
    assert!(!projections.contains("_=>"));
    assert!(!projections.contains("unreachable!"));
}

#[test]
fn normalization_emitters_accept_only_the_closed_form() {
    let emitter_signature = bounded(
        STRING_SOURCE,
        "pub(super) fn emit_normalized_string_payload_from_local(",
        ") -> Result<(), EmitError> {",
    );
    assert!(emitter_signature.contains("form: StringNormalizationForm,"));
    assert!(!emitter_signature.contains(": bool"));

    let lookup_signature = bounded(
        STRING_SOURCE,
        "fn emit_normalization_decomposition_lookup(",
        ") {",
    );
    assert!(lookup_signature.contains("form: &StringNormalizationForm,"));
    assert!(!lookup_signature.contains(": bool"));

    let emitter = bounded(
        STRING_SOURCE,
        "pub(super) fn emit_normalized_string_payload_from_local(",
        "fn emit_normalization_decomposition_lookup(",
    );
    assert_eq!(emitter.matches("&form,").count(), 2);
    assert_eq!(emitter.matches("form,").count(), 2);
    assert_eq!(emitter.matches("if form.composes() {").count(), 1);
    assert!(!emitter.contains("compatibility"));
    assert!(!emitter.contains("if compose"));

    let lookup = bounded(
        STRING_SOURCE,
        "fn emit_normalization_decomposition_lookup(",
        "fn emit_normalization_combining_class_lookup(",
    );
    assert_eq!(
        lookup
            .matches("form.decomposition_table(self.strings)")
            .count(),
        1
    );
    assert!(!lookup.contains("if compatibility"));
}

#[test]
fn normalize_preserves_validation_and_dispatch_order() {
    let normalize = normalized(bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeNormalize => {",
        "StandardBuiltinId::StringPrototypeLocaleCompare => {",
    ));
    for required in [
        "forforminStringNormalizationForm::ALL{",
        "self.strings.payload(form.spelling())",
        "Instruction::I64Const(form.runtime_kind())",
        "StringNormalizationForm::Nfc.runtime_kind()",
        "StringNormalizationForm::Nfd.runtime_kind()",
        "StringNormalizationForm::Nfkc.runtime_kind()",
    ] {
        assert!(
            normalize.contains(required),
            "missing normalize step `{required}`"
        );
    }
    assert_eq!(
        normalize
            .matches("emit_normalized_string_payload_from_local(")
            .count(),
        4
    );

    let argument = normalize.find("emit_builtin_arg_to_locals(0,").unwrap();
    let undefined_check = normalize.find("ValueKind::Undefined.tag()").unwrap();
    let form_coercion = normalize
        .find("emit_value_to_string_payload(form_payload_local,form_tag_local,function)")
        .unwrap();
    let form_loop = normalize
        .find("forforminStringNormalizationForm::ALL{")
        .unwrap();
    let invalid_guard = normalize
        .find("Instruction::LocalGet(valid_form_local)")
        .unwrap();
    let range_error = normalize
        .find("emit_throw_current_function_realm_range_error(")
        .unwrap();
    let dispatch = normalize
        .find("emit_normalized_string_payload_from_local(string_local,StringNormalizationForm::Nfc")
        .unwrap();
    assert!(argument < undefined_check);
    assert!(undefined_check < form_coercion);
    assert!(form_coercion < form_loop);
    assert!(form_loop < invalid_guard);
    assert!(invalid_guard < range_error);
    assert!(range_error < dispatch);

    let mut previous = 0;
    for form in ["Nfc", "Nfd", "Nfkc", "Nfkd"] {
        let call = normalize
            .find(&format!(
                "emit_normalized_string_payload_from_local(string_local,StringNormalizationForm::{form}"
            ))
            .unwrap_or_else(|| panic!("missing `{form}` normalization dispatch"));
        assert!(previous < call, "`{form}` dispatch moved out of form order");
        previous = call;
    }
}

#[test]
fn locale_compare_and_the_string_pool_share_the_form_authority() {
    assert_eq!(
        BUILTINS_SOURCE
            .matches("pub(crate) use string::StringNormalizationForm;")
            .count(),
        1
    );

    let locale_compare = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::StringPrototypeLocaleCompare => {",
        "StandardBuiltinId::StringPrototypeIterator => {",
    );
    assert_eq!(
        locale_compare
            .matches("emit_normalized_string_payload_from_local(")
            .count(),
        2
    );
    assert_eq!(
        locale_compare
            .matches("StringNormalizationForm::Nfc")
            .count(),
        2
    );
    for form in ["Nfd", "Nfkc", "Nfkd"] {
        assert!(!locale_compare.contains(&format!("StringNormalizationForm::{form}")));
    }

    let pool = normalized(bounded(
        DATA_SOURCE,
        "if compiled_standard_builtins.contains(&StandardBuiltinId::StringPrototypeNormalize)",
        "fn append_normalization_tables(&mut self)",
    ));
    assert_eq!(
        pool.matches("forforminStringNormalizationForm::ALL{pool.intern_string(form.spelling());}")
            .count(),
        1
    );
    for spelling in ["NFC", "NFD", "NFKC", "NFKD"] {
        assert_eq!(
            STRING_SOURCE.matches(&format!("\"{spelling}\"")).count(),
            1,
            "`{spelling}` must be owned by the form spelling projection"
        );
        assert_eq!(
            STANDARD_SOURCE.matches(&format!("\"{spelling}\"")).count(),
            0
        );
        assert_eq!(DATA_SOURCE.matches(&format!("\"{spelling}\"")).count(), 0);
    }
}
