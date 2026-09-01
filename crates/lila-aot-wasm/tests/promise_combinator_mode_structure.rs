use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_COMBINATOR_REACTION_PAIR_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_combinator_reaction_pair.rs");
const PROMISE_KEYED_COMBINATOR_MODE_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_keyed_combinator_mode.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect()
}

#[test]
fn standard_and_keyed_combinator_modes_are_separate_closed_domains() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_keyed_combinator_mode;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_keyed_combinator_mode;"));
    assert!(!PROMISE_SOURCE.contains("promise_keyed_combinator_mode::"));
    assert!(!PROMISE_SOURCE.contains("PromiseKeyedCombinatorMode"));
    assert!(PROMISE_KEYED_COMBINATOR_MODE_SOURCE.lines().count() <= 675);

    let standard_variants = bounded(
        PROMISE_SOURCE,
        "#[derive(Clone, Copy)]\nenum PromiseCombinatorMode {",
        "\n}\n\nimpl PromiseCombinatorMode {",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        standard_variants,
        ["Values,", "SettledRecords,", "FirstFulfillment,"]
    );

    let keyed_declaration = bounded(
        PROMISE_KEYED_COMBINATOR_MODE_SOURCE,
        "#[derive(Clone, Copy)]\nenum PromiseKeyedCombinatorMode {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let keyed_variants = keyed_declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(keyed_variants, ["Values,", "SettledRecords,"]);

    let standard_declaration = bounded(
        PROMISE_SOURCE,
        "#[derive(Clone, Copy)]\nenum PromiseCombinatorMode {",
        "impl PromiseCombinatorMode {",
    );
    for forbidden in ["PartialEq", "Eq", "bool"] {
        assert!(
            !standard_declaration.contains(forbidden),
            "combinator mode domains must not expose `{forbidden}` projection"
        );
        assert!(
            !keyed_declaration.contains(forbidden),
            "keyed combinator mode must not expose `{forbidden}` projection"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_sources = rust_sources(&source_root);
    assert_eq!(
        all_sources.matches("PromiseKeyedCombinatorMode").count(),
        10
    );
    assert!(!all_sources.contains("promise_keyed_combinator_mode::"));
}

#[test]
fn keyed_lowering_accepts_only_the_restricted_mode() {
    let wrappers = bounded(
        PROMISE_KEYED_COMBINATOR_MODE_SOURCE,
        "    pub(crate) fn emit_promise_all_keyed(",
        "    fn emit_promise_keyed(",
    );
    assert_eq!(
        wrappers
            .matches("PromiseKeyedCombinatorMode::Values")
            .count(),
        1
    );
    assert_eq!(
        wrappers
            .matches("PromiseKeyedCombinatorMode::SettledRecords")
            .count(),
        1
    );
    assert!(!wrappers.contains("PromiseCombinatorMode"));

    let keyed = bounded(
        PROMISE_KEYED_COMBINATOR_MODE_SOURCE,
        "    fn emit_promise_keyed(",
        "\n}",
    );
    assert!(keyed.contains("mode: PromiseKeyedCombinatorMode,"));
    assert_eq!(keyed.matches("match mode {").count(), 3);
    assert_eq!(
        keyed
            .matches("PromiseKeyedCombinatorMode::Values =>")
            .count(),
        3
    );
    assert_eq!(
        keyed
            .matches("PromiseKeyedCombinatorMode::SettledRecords =>")
            .count(),
        3
    );
    for forbidden in [
        "PromiseCombinatorMode",
        "FirstFulfillment",
        "mode ==",
        "mode !=",
        "matches!(mode",
    ] {
        assert!(
            !keyed.contains(forbidden),
            "keyed combinator selection must not contain `{forbidden}`"
        );
    }
}

#[test]
fn every_standard_combinator_policy_is_an_exhaustive_projection() {
    let wrappers = bounded(
        PROMISE_SOURCE,
        "    pub(crate) fn emit_promise_all(",
        "    fn emit_promise_combinator(",
    );
    for variant in ["Values", "SettledRecords", "FirstFulfillment"] {
        assert_eq!(
            wrappers
                .matches(&format!("PromiseCombinatorMode::{variant}"))
                .count(),
            1,
            "standard wrapper producer for `{variant}`"
        );
    }

    let standard = bounded(
        PROMISE_SOURCE,
        "    fn emit_promise_combinator(",
        "    pub(crate) fn emit_promise_resolving_function(",
    );
    assert_eq!(standard.matches("match mode {").count(), 4);
    assert_eq!(
        PROMISE_COMBINATOR_REACTION_PAIR_SOURCE
            .matches("match mode {")
            .count(),
        1,
    );
    for variant in ["Values", "SettledRecords", "FirstFulfillment"] {
        assert_eq!(
            standard
                .matches(&format!("PromiseCombinatorMode::{variant}"))
                .count(),
            4,
            "the four parent-owned standard policies must name `{variant}`"
        );
        assert_eq!(
            PROMISE_COMBINATOR_REACTION_PAIR_SOURCE
                .matches(&format!("PromiseCombinatorMode::{variant}"))
                .count(),
            1,
            "the child-owned reaction policy must name `{variant}`"
        );
    }

    let builtin_name = bounded(
        PROMISE_SOURCE,
        "impl PromiseCombinatorMode {",
        "impl AsyncAwaitContinuation",
    );
    assert_eq!(builtin_name.matches("match self {").count(), 1);
    for variant in ["Values", "SettledRecords", "FirstFulfillment"] {
        assert_eq!(
            builtin_name.matches(&format!("Self::{variant} =>")).count(),
            1,
            "builtin name policy for `{variant}`"
        );
    }

    for forbidden in ["mode ==", "mode !=", "matches!(mode", "_ =>"] {
        assert!(
            !standard.contains(forbidden),
            "standard combinator selection must not contain `{forbidden}`"
        );
    }
}
