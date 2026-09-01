use std::fs;
use std::path::Path;

const NUMBER_SOURCE: &str = include_str!("../src/builtins/number.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const NUMBER_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_number_builtin_family.js");
const NUMERICS_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/number-builtin-policy-domains.md");
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T20: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn enum_variants(name: &str) -> Vec<&'static str> {
    let marker = format!("enum {name} {{");
    NUMBER_SOURCE
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing enum `{name}`"))
        .1
        .split_once('}')
        .unwrap_or_else(|| panic!("missing end of enum `{name}`"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
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
fn number_builtin_domains_are_exact_and_capability_free() {
    assert_eq!(
        enum_variants("NumberBuiltin"),
        [
            "Constructor,",
            "IsInteger,",
            "IsSafeInteger,",
            "IsFinite,",
            "IsNaN,",
            "PrototypeToExponential,",
            "PrototypeToFixed,",
            "PrototypeToPrecision,",
            "PrototypeToString,",
            "PrototypeToLocaleString,",
            "PrototypeValueOf,",
        ]
    );
    assert_eq!(
        enum_variants("NumberPrototypeOperation"),
        [
            "ToExponential,",
            "ToFixed,",
            "ToPrecision,",
            "ToString,",
            "ToLocaleString,",
            "ValueOf,",
        ]
    );
    let declarations = bounded(
        NUMBER_SOURCE,
        "use super::super::*;",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(!declarations.contains("#[derive"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "NumberBuiltin"), 24);
    assert_eq!(
        count_in_rust_sources(&source_root, "NumberPrototypeOperation"),
        14
    );
    for domain in ["NumberBuiltin", "NumberPrototypeOperation"] {
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
            assert_eq!(
                count_in_rust_sources(&source_root, &format!("impl {capability} for {domain}")),
                0,
                "{domain} must not implement {capability}"
            );
        }
    }

    assert!(NUMBER_SOURCE.contains("enum NumberBuiltin {"));
    assert!(!NUMBER_SOURCE.contains("pub(super) enum NumberBuiltin"));
    for evidence in [CONTRACT, T02, T20] {
        assert!(evidence.contains("private `NumberBuiltin`"));
        assert!(evidence.contains("fixed Number entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new Number behavior"));
    }
}

#[test]
fn standard_dispatch_uses_all_eleven_fixed_number_entries() {
    assert!(!STANDARD_SOURCE.contains("NumberBuiltin"));
    assert!(!STANDARD_SOURCE.contains("emit_number_builtin("));
    for (standard_builtin, entry, variant) in [
        (
            "NumberConstructor",
            "emit_number_constructor_builtin",
            "Constructor",
        ),
        (
            "NumberIsInteger",
            "emit_number_is_integer_builtin",
            "IsInteger",
        ),
        (
            "NumberIsSafeInteger",
            "emit_number_is_safe_integer_builtin",
            "IsSafeInteger",
        ),
        (
            "NumberIsFinite",
            "emit_number_is_finite_builtin",
            "IsFinite",
        ),
        ("NumberIsNaN", "emit_number_is_nan_builtin", "IsNaN"),
        (
            "NumberPrototypeToExponential",
            "emit_number_prototype_to_exponential_builtin",
            "PrototypeToExponential",
        ),
        (
            "NumberPrototypeToFixed",
            "emit_number_prototype_to_fixed_builtin",
            "PrototypeToFixed",
        ),
        (
            "NumberPrototypeToPrecision",
            "emit_number_prototype_to_precision_builtin",
            "PrototypeToPrecision",
        ),
        (
            "NumberPrototypeToString",
            "emit_number_prototype_to_string_builtin",
            "PrototypeToString",
        ),
        (
            "NumberPrototypeToLocaleString",
            "emit_number_prototype_to_locale_string_builtin",
            "PrototypeToLocaleString",
        ),
        (
            "NumberPrototypeValueOf",
            "emit_number_prototype_value_of_builtin",
            "PrototypeValueOf",
        ),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("StandardBuiltinId::{standard_builtin} =>"))
                .count(),
            1,
            "standard route `{standard_builtin}`"
        );
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("self.{entry}(function)?"))
                .count(),
            1,
            "standard entry `{entry}`"
        );
        assert_eq!(
            NUMBER_SOURCE
                .matches(&format!(
                    "self.emit_number_builtin(NumberBuiltin::{variant}, function)"
                ))
                .count(),
            1,
            "fixed producer `{variant}`"
        );
    }
}

#[test]
fn number_builtin_matches_are_exhaustive_and_prototype_domain_is_restricted() {
    let prototype = bounded(
        NUMBER_SOURCE,
        "    fn emit_number_prototype_builtin(",
        "    pub(super) fn emit_number_constructor_builtin(",
    );
    assert!(prototype.contains("operation: NumberPrototypeOperation,"));
    assert_eq!(prototype.matches("match operation").count(), 1);
    for operation in [
        "ToExponential",
        "ToFixed",
        "ToPrecision",
        "ToString",
        "ToLocaleString",
        "ValueOf",
    ] {
        assert_eq!(
            prototype
                .matches(&format!("NumberPrototypeOperation::{operation}"))
                .count(),
            1,
            "prototype consumer `NumberPrototypeOperation::{operation}`"
        );
    }

    let builtin = NUMBER_SOURCE
        .split_once("    fn emit_number_builtin(")
        .expect("Number builtin dispatcher")
        .1;
    assert!(builtin.contains("builtin: NumberBuiltin,"));
    assert_eq!(builtin.matches("match builtin").count(), 1);
    for variant in [
        "Constructor",
        "IsInteger",
        "IsSafeInteger",
        "IsFinite",
        "IsNaN",
        "PrototypeToExponential",
        "PrototypeToFixed",
        "PrototypeToPrecision",
        "PrototypeToString",
        "PrototypeToLocaleString",
        "PrototypeValueOf",
    ] {
        assert_eq!(
            builtin
                .matches(&format!("NumberBuiltin::{variant}"))
                .count(),
            1,
            "Number builtin consumer `NumberBuiltin::{variant}`"
        );
    }
    assert_eq!(builtin.matches("emit_number_prototype_builtin(").count(), 6);
    for operation in [
        "ToExponential",
        "ToFixed",
        "ToPrecision",
        "ToString",
        "ToLocaleString",
        "ValueOf",
    ] {
        assert_eq!(
            builtin
                .matches(&format!("NumberPrototypeOperation::{operation}"))
                .count(),
            1,
            "restricted prototype producer `NumberPrototypeOperation::{operation}`"
        );
    }

    for body in [prototype, builtin] {
        for forbidden in [
            "operation ==",
            "operation !=",
            "builtin ==",
            "builtin !=",
            "_ =>",
            "unreachable!",
            "debug_assert!",
        ] {
            assert!(
                !body.contains(forbidden),
                "Number policy contains `{forbidden}`"
            );
        }
    }
}

#[test]
fn number_builtin_family_fixture_witnesses_every_operation() {
    assert_eq!(
        NUMERICS_CLI_TESTS
            .matches("fn run_wasm_backend_succeeds_for_number_builtin_family_fixture()")
            .count(),
        1
    );
    assert_eq!(
        NUMERICS_CLI_TESTS
            .matches("fixture_path(\"wasm_number_builtin_family.js\")")
            .count(),
        1
    );
    for witness in [
        "return Number(value);",
        "check(Number() === 0",
        "new Number(12)",
        "Number.isInteger(",
        "Number.isSafeInteger(",
        "Number.isFinite(",
        "Number.isNaN(",
        ".toExponential(",
        ".toFixed(",
        ".toPrecision(",
        ".toString(",
        ".toLocaleString(",
        "boxed.valueOf()",
        "Number.prototype.valueOf.call({})",
    ] {
        assert!(
            NUMBER_FIXTURE.contains(witness),
            "missing fixture witness `{witness}`"
        );
    }
}
