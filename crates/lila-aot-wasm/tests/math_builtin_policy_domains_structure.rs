const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/math-builtin-policy-domains.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

fn enum_variants(source: &'static str, name: &str) -> Vec<&'static str> {
    source
        .split_once(&format!("enum {name} {{"))
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

fn path_variant_count(source: &str, path: &str, variant: &str) -> usize {
    let needle = format!("{path}::{variant}");
    source
        .match_indices(&needle)
        .filter(|(offset, _)| {
            !source[offset + needle.len()..]
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        })
        .count()
}

#[test]
fn math_builtin_domains_are_exact_and_capability_free() {
    assert!(MATH_SOURCE.contains("\nenum MathBuiltin {"));
    assert!(MATH_SOURCE.contains("\nenum MathUnaryBuiltin {"));
    assert!(!MATH_SOURCE.contains("pub(super) enum MathBuiltin"));
    assert!(!MATH_SOURCE.contains("pub(super) enum MathUnaryBuiltin"));
    assert_eq!(
        enum_variants(MATH_SOURCE, "MathBuiltin"),
        [
            "Unary(MathUnaryBuiltin),",
            "Atan2,",
            "Hypot,",
            "Imul,",
            "Max,",
            "Min,",
            "Pow,",
            "Random,",
            "SumPrecise,",
        ]
    );
    assert_eq!(
        enum_variants(MATH_SOURCE, "MathUnaryBuiltin"),
        [
            "Abs,",
            "Acos,",
            "Acosh,",
            "Asin,",
            "Asinh,",
            "Atan,",
            "Atanh,",
            "Cbrt,",
            "Ceil,",
            "Clz32,",
            "Cos,",
            "Cosh,",
            "Exp,",
            "Expm1,",
            "F16Round,",
            "Floor,",
            "Fround,",
            "Log,",
            "Log10,",
            "Log1p,",
            "Log2,",
            "Round,",
            "Sign,",
            "Sin,",
            "Sinh,",
            "Sqrt,",
            "Tan,",
            "Tanh,",
            "Trunc,",
        ]
    );

    let policy_declarations = MATH_SOURCE
        .split_once("enum MathExtremum {")
        .expect("Math policy declaration boundary")
        .0;
    assert!(!policy_declarations.contains("#[derive"));

    for domain in ["MathBuiltin", "MathUnaryBuiltin"] {
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
            assert!(!MATH_SOURCE.contains(&format!("impl {capability} for {domain}")));
        }
    }
}

#[test]
fn standard_dispatch_reaches_math_only_through_fixed_operation_entries() {
    assert!(!STANDARD_SOURCE.contains("MathBuiltin"));
    assert!(!STANDARD_SOURCE.contains("MathUnaryBuiltin"));
    assert!(!STANDARD_SOURCE.contains("MathFn"));
    assert!(!STANDARD_SOURCE.contains("UnaryMathFn"));
    assert!(!STANDARD_SOURCE.contains("self.emit_math("));
    assert_eq!(STANDARD_SOURCE.matches("self.emit_math_").count(), 37);

    for (entry, variant) in [
        ("abs", "Abs"),
        ("acos", "Acos"),
        ("acosh", "Acosh"),
        ("asin", "Asin"),
        ("asinh", "Asinh"),
        ("atan", "Atan"),
        ("atanh", "Atanh"),
        ("cbrt", "Cbrt"),
        ("ceil", "Ceil"),
        ("clz32", "Clz32"),
        ("cos", "Cos"),
        ("cosh", "Cosh"),
        ("exp", "Exp"),
        ("expm1", "Expm1"),
        ("f16round", "F16Round"),
        ("floor", "Floor"),
        ("fround", "Fround"),
        ("log", "Log"),
        ("log10", "Log10"),
        ("log1p", "Log1p"),
        ("log2", "Log2"),
        ("round", "Round"),
        ("sign", "Sign"),
        ("sin", "Sin"),
        ("sinh", "Sinh"),
        ("sqrt", "Sqrt"),
        ("tan", "Tan"),
        ("tanh", "Tanh"),
        ("trunc", "Trunc"),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("self.emit_math_{entry}_builtin(function)?"))
                .count(),
            1,
            "unary Math route `{entry}`"
        );
        assert_eq!(
            MATH_SOURCE
                .matches(&format!(
                    "self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::{variant}), function)"
                ))
                .count(),
            1,
            "unary Math producer `{entry}`"
        );
    }
    for (entry, variant) in [
        ("atan2", "Atan2"),
        ("hypot", "Hypot"),
        ("imul", "Imul"),
        ("max", "Max"),
        ("min", "Min"),
        ("pow", "Pow"),
        ("random", "Random"),
        ("sum_precise", "SumPrecise"),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("self.emit_math_{entry}_builtin(function)?"))
                .count(),
            1,
            "non-unary Math route `{entry}`"
        );
        assert_eq!(
            MATH_SOURCE
                .matches(&format!("self.emit_math(MathBuiltin::{variant}, function)"))
                .count(),
            1,
            "non-unary Math producer `{entry}`"
        );
    }
}

#[test]
fn math_emitter_consumes_one_top_level_policy_and_one_restricted_unary_policy() {
    let emitter = MATH_SOURCE
        .split_once("    fn emit_math(")
        .expect("Math emitter")
        .1;
    assert!(emitter.contains("builtin: MathBuiltin,"));
    assert_eq!(emitter.matches("match builtin").count(), 1);
    assert_eq!(emitter.matches("MathBuiltin::").count(), 9);
    assert_eq!(emitter.matches("MathBuiltin::Unary(unary)").count(), 1);
    assert_eq!(emitter.matches("match unary").count(), 1);
    assert_eq!(emitter.matches("MathUnaryBuiltin::").count(), 29);

    for variant in [
        "Abs", "Acos", "Acosh", "Asin", "Asinh", "Atan", "Atanh", "Cbrt", "Ceil", "Clz32", "Cos",
        "Cosh", "Exp", "Expm1", "F16Round", "Floor", "Fround", "Log", "Log10", "Log1p", "Log2",
        "Round", "Sign", "Sin", "Sinh", "Sqrt", "Tan", "Tanh", "Trunc",
    ] {
        assert_eq!(
            path_variant_count(emitter, "MathUnaryBuiltin", variant),
            1,
            "unary Math consumer `{variant}`"
        );
    }
    for forbidden in [
        "non-unary Math builtin reached unary dispatch",
        "_ =>",
        "builtin ==",
        "builtin !=",
        "unary ==",
        "unary !=",
        ".clone()",
    ] {
        assert!(
            !emitter.contains(forbidden),
            "Math policy contains `{forbidden}`"
        );
    }
}

#[test]
fn contract_and_task_record_the_nested_math_dispatch_invariant() {
    for evidence in [CONTRACT, TASK] {
        let words = evidence.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("capability-free `MathBuiltin`"));
        assert!(words.contains("capability-free `MathUnaryBuiltin`"));
        assert!(words.contains("Batch AK"));
        assert!(words.contains("Batch AW"));
        assert!(words.contains("37 fixed Math entries"));
        assert!(words.contains("source-equivalent"));
        assert!(words.contains("no new Math behavior"));
    }
}
