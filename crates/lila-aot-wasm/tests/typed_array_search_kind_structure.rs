const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/typed-array-search-kind.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn search_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "    fn compile_typed_array_search_builtin(",
        "    pub(crate) fn compile_array_prototype_at_builtin(",
    )
}

#[test]
fn typed_array_search_kind_has_no_equality_projection() {
    let declaration = bounded(
        ARRAY_SOURCE,
        "enum TypedArraySearchKind {",
        "pub(crate) enum ArrayAtReceiverPolicy",
    );
    let variants = declaration
        .split_once('}')
        .expect("TypedArray search kind end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Includes,", "IndexOf,", "LastIndexOf,"]);

    let authority_header = bounded(
        ARRAY_SOURCE,
        "enum TypedArrayQuantifierKind {",
        "pub(crate) enum ArrayAtReceiverPolicy",
    );
    assert!(!authority_header.contains("#[derive"));
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
        assert!(
            !ARRAY_SOURCE.contains(&format!("impl {capability} for TypedArraySearchKind")),
            "TypedArray search kind must not implement {capability}"
        );
    }

    let body = search_body();
    assert_eq!(body.matches("match &search_kind").count(), 12);
    for forbidden in [
        "match search_kind",
        "search_kind.clone()",
        "search_kind ==",
        "search_kind !=",
        "is_includes",
        "is_index_of",
        "is_last_index_of",
    ] {
        assert!(
            !body.contains(forbidden),
            "TypedArray search semantics must not collapse to {forbidden}"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("capability-free `TypedArraySearchKind`"));
    }
}

#[test]
fn typed_array_search_producers_are_exact() {
    let wrappers = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_typed_array_prototype_includes_builtin(",
        "    fn compile_typed_array_search_builtin(",
    );
    let normalized_wrappers = without_whitespace(wrappers);
    for (method, variant) in [
        ("includes", "Includes"),
        ("index_of", "IndexOf"),
        ("last_index_of", "LastIndexOf"),
    ] {
        let call =
            format!("compile_typed_array_search_builtin(TypedArraySearchKind::{variant},function)");
        assert_eq!(
            normalized_wrappers.matches(&call).count(),
            1,
            "TypedArray {method} must produce exactly its matching search kind"
        );
    }
    assert_eq!(
        normalized_wrappers
            .matches("compile_typed_array_search_builtin(")
            .count(),
        3
    );

    let normalized_standard = without_whitespace(STANDARD_SOURCE).replace(",)", ")");
    for (builtin, compiler) in [
        ("TypedArrayPrototypeIncludes", "includes"),
        ("TypedArrayPrototypeIndexOf", "index_of"),
        ("TypedArrayPrototypeLastIndexOf", "last_index_of"),
    ] {
        let mapping = format!(
            "StandardBuiltinId::{builtin}=>{{self.compile_typed_array_prototype_{compiler}_builtin(function)?;}}"
        );
        assert_eq!(
            normalized_standard.matches(&mapping).count(),
            1,
            "{builtin} must map exactly once to its matching wrapper"
        );
    }
}

#[test]
fn typed_array_search_semantic_pairings_are_exhaustive() {
    let body = search_body();
    assert_eq!(
        body.matches("TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf")
            .count(),
        4,
        "the two forward searches must share only default-fromIndex, normalization, loop bound and advance"
    );
    assert_eq!(
        body.matches("TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf")
            .count(),
        6,
        "the two index searches must share only result initialization, invalid-index suppression, strict comparison, index projection and branch framing"
    );
    assert_eq!(
        body.matches("emit_tagged_payload_same_value_zero_i32(")
            .count(),
        1
    );
    assert_eq!(body.matches("emit_tagged_payload_equality_i32(").count(), 1);
    assert_eq!(
        body.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert_eq!(
        body.matches("TypedArrayWitnessUse::IntegerIndexedProperty")
            .count(),
        1
    );

    let normalized = without_whitespace(body);
    for sentinel in [
        "TypedArraySearchKind::Includes=>2,TypedArraySearchKind::IndexOf|TypedArraySearchKind::LastIndexOf=>3",
        "TypedArraySearchKind::Includes|TypedArraySearchKind::IndexOf=>{function.instruction(&Instruction::I64Add);}",
        "TypedArraySearchKind::LastIndexOf=>{function.instruction(&Instruction::I64Sub);}",
    ] {
        assert_eq!(
            normalized.matches(sentinel).count(),
            1,
            "missing exact TypedArray search projection: {sentinel}"
        );
    }
}
