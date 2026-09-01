use std::fs;
use std::path::Path;

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/numeric-conversion-realm-projection-capability.md"
);
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

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

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn numeric_realm_projections_are_exact_non_capability_domains() {
    let declarations = bounded(
        OPERATIONS_SOURCE,
        "enum UnaryNumericKind {\n    Number,\n    BigInt,\n}\n",
        "fn numeric_conversion_realm_access(",
    );
    assert!(!declarations.contains("#["));
    let declaration_code = declarations
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        normalized(&declaration_code),
        concat!(
            "enumNumericConversionRealmAccess{",
            "TrustedCurrentEnvironment,MainRealmFallback,}"
        )
    );

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(
            !OPERATIONS_SOURCE.contains(&format!("{capability} for NumericConversionRealmAccess"))
        );
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&src, "NumericConversionRealmAccess"),
        14
    );
    assert_eq!(
        count_in_rust_sources(
            &src,
            "NumericConversionRealmAccess::TrustedCurrentEnvironment"
        ),
        6
    );
    assert_eq!(
        count_in_rust_sources(&src, "NumericConversionRealmAccess::MainRealmFallback"),
        6
    );
    for retired_domain in [
        "OutlinedNumericRealmArgument",
        "NumericConversionErrorRealm",
    ] {
        assert_eq!(count_in_rust_sources(&src, retired_domain), 0);
    }
}

#[test]
fn all_three_source_rows_project_once_for_both_consumers() {
    let projections = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn numeric_conversion_realm_access(",
        "fn spec_operation_property_key_operand(",
    ));
    assert_eq!(
        projections,
        concat!(
            "source:NumericErrorRealmSource,)->NumericConversionRealmAccess{",
            "matchsource{",
            "NumericErrorRealmSource::GlobalFallback=>",
            "NumericConversionRealmAccess::MainRealmFallback,",
            "NumericErrorRealmSource::StandardBuiltinEnvironment|",
            "NumericErrorRealmSource::NumericConversionHelperArgument=>{",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment}}}"
        )
    );

    let unit = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn numeric_realm_projections_keep_ordinary_lexical_environments_out() {",
        "\n    }\n}",
    ));
    for expected in [
        concat!(
            "matchnumeric_conversion_realm_access(source){",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment=>{}",
            "NumericConversionRealmAccess::MainRealmFallback=>{",
            "panic!(\"trustednumericsourcelostitsRealmaccess\")}}"
        ),
        concat!(
            "matchnumeric_conversion_realm_access(NumericErrorRealmSource::GlobalFallback){",
            "NumericConversionRealmAccess::MainRealmFallback=>{}",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment=>{",
            "panic!(\"globalfallbackexposedalexicalenvironmentasnumericRealmstate\")}}"
        ),
    ] {
        assert_eq!(unit.matches(expected).count(), 1, "unit match `{expected}`");
    }
    for forbidden in [
        "assert_eq!(outlined_numeric_realm_argument",
        "assert_eq!(numeric_conversion_error_realm",
        "fnoutlined_numeric_realm_argument",
        "fnnumeric_conversion_error_realm",
    ] {
        assert!(!unit.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn each_projection_consumer_keeps_its_exact_emission_policy() {
    let outlined_consumer = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn emit_outlined_numeric_realm_argument(&self, function: &mut Function) {",
        "\n    }\n\n    fn emit_numeric_conversion_type_error(",
    ));
    assert_eq!(
        outlined_consumer,
        concat!(
            "matchnumeric_conversion_realm_access(self.numeric_error_realm_source()){",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment=>{",
            "function.instruction(&Instruction::LocalGet(self.current_env_local));}",
            "NumericConversionRealmAccess::MainRealmFallback=>{",
            "function.instruction(&Instruction::I64Const(0));}}"
        )
    );

    let type_error_consumer = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn emit_numeric_conversion_type_error(",
        "\n    }\n\n    fn emit_numeric_conversion_range_error(",
    ));
    assert_eq!(
        type_error_consumer,
        concat!(
            "&mutself,message:&str,payload_local:u32,tag_local:u32,",
            "function:&mutFunction,)->Result<(),EmitError>{",
            "matchnumeric_conversion_realm_access(self.numeric_error_realm_source()){",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment=>self.",
            "emit_throw_current_function_realm_type_error(",
            "message,payload_local,tag_local,function,),",
            "NumericConversionRealmAccess::MainRealmFallback=>self.emit_throw_runtime_error(",
            "TYPE_ERROR_NAME,message,payload_local,tag_local,function,),}"
        )
    );

    let range_error_consumer = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn emit_numeric_conversion_range_error(",
        "\n    }\n\n    fn finish_may_throw_operation(",
    ));
    assert_eq!(
        range_error_consumer,
        concat!(
            "&mutself,message:&str,payload_local:u32,tag_local:u32,",
            "function:&mutFunction,)->Result<(),EmitError>{",
            "matchnumeric_conversion_realm_access(self.numeric_error_realm_source()){",
            "NumericConversionRealmAccess::TrustedCurrentEnvironment=>self.",
            "emit_throw_current_function_realm_range_error(",
            "message,payload_local,tag_local,function,),",
            "NumericConversionRealmAccess::MainRealmFallback=>self.emit_throw_runtime_error(",
            "RANGE_ERROR_NAME,message,payload_local,tag_local,function,),}"
        )
    );

    let consumers = format!("{outlined_consumer}{type_error_consumer}{range_error_consumer}");
    for forbidden in ["_=>", "==", "!=", "matches!(", "unreachable!"] {
        assert!(!consumers.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_t04_record_the_source_equivalent_capability_closure() {
    for marker in [
        "one private, non-derived",
        "helper ABI parameter 6",
        "does not claim a completion-ABI redesign",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("numeric-conversion-realm-projection-capability.md"));
}
