use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../../src/runtime_helpers.rs");
const EXEC_SOURCE: &str = include_str!("../../src/builtins/string/regexp_exec.rs");
const STRING_SOURCE: &str = include_str!("../../src/builtins/string.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
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
fn regexp_matcher_failure_route_is_the_exact_crate_private_no_capability_domain() {
    let declaration_region = bounded(
        OWNER_SOURCE,
        "/// How a failed RegExp matcher result becomes a JavaScript throw.",
        "/// Declares the complete RegExp matcher status ABI",
    );
    assert!(!declaration_region.contains("#["));
    let declaration = bounded(
        declaration_region,
        "pub(crate) enum RegExpMatcherFailureRoute {",
        "\n}",
    );
    assert_eq!(
        normalized(declaration),
        "GenericError,CurrentFunctionRealmRangeError,"
    );
    assert!(!OWNER_SOURCE.contains("impl RegExpMatcherFailureRoute"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(OWNER_SOURCE.matches("RegExpMatcherFailureRoute").count(), 5);
    assert_eq!(EXEC_SOURCE.matches("RegExpMatcherFailureRoute").count(), 2);
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpMatcherFailureRoute"),
        7,
        "the owner and sole product consumer must own every route mention"
    );
}

#[test]
fn regexp_matcher_failure_rows_own_the_exact_error_routes() {
    let rows = bounded(
        OWNER_SOURCE,
        "regexp_matcher_status_domain! {",
        "const fn regexp_matcher_status_words_are_unique()",
    );
    assert_eq!(
        normalized(rows),
        "CorruptProgram=>{word:1,route:GenericError,message:REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,},ResourceExhausted=>{word:2,route:CurrentFunctionRealmRangeError,message:REGEXP_MATCHER_SCRATCH_ARENA_EXCEEDS_THE_ENGINE_ADDRESSABLE_RESOURCE_LIMIT,},}"
    );

    let route_projection = bounded(
        OWNER_SOURCE,
        "pub(crate) const fn route(self) -> RegExpMatcherFailureRoute {",
        "pub(crate) const fn message(self) -> RuntimeErrorMessage {",
    );
    assert_eq!(route_projection.matches("match self {").count(), 1);
    assert!(route_projection.contains("$( Self::$failure => RegExpMatcherFailureRoute::$route, )+"));
    assert!(!route_projection.contains("_ =>"));

    let owner_unit = bounded(
        OWNER_SOURCE,
        "fn regexp_matcher_status_domain_owns_words_routes_and_messages() {",
        "fn indexes_are_dense_from_the_base()",
    );
    assert_eq!(owner_unit.matches("match failure.route() {").count(), 1);
    assert_eq!(
        owner_unit
            .matches("RegExpMatcherFailureRoute::GenericError => \"generic-error\"")
            .count(),
        1
    );
    assert_eq!(
        owner_unit
            .matches("RegExpMatcherFailureRoute::CurrentFunctionRealmRangeError =>")
            .count(),
        1
    );
    assert!(!owner_unit.contains("_ =>"));
}

#[test]
fn regexp_matcher_failure_route_has_one_exact_exhaustive_product_consumer() {
    let consumer = bounded(
        EXEC_SOURCE,
        "fn emit_native_regexp_builtin_exec(",
        "pub(super) fn emit_native_string_result_array(",
    );
    let route_arms = bounded(consumer, "let kind=match failure.route() {", "};");
    assert_eq!(normalized(route_arms), concat!(
        "crate::runtime_helpers::RegExpMatcherFailureRoute::GenericError=>NativeErrorKind::Error,",
        "crate::runtime_helpers::RegExpMatcherFailureRoute::CurrentFunctionRealmRangeError=>NativeErrorKind::RangeError,"
    ));
    assert_eq!(consumer.matches("match failure.route() {").count(), 1);
    assert_eq!(
        consumer
            .matches("self.emit_native_string_error_if(failure.message(), kind, result, exit, f)?;")
            .count(),
        1
    );
    assert_before(
        consumer,
        "scratch.finish(self, f);",
        "match failure.route() {",
    );
    assert_before(
        consumer,
        "match failure.route() {",
        "self.emit_native_string_error_if(failure.message(), kind, result, exit, f)?;",
    );
    let error = bounded(
        STRING_SOURCE,
        "fn emit_native_string_error_if(",
        "fn emit_native_string_require_coercible(",
    );
    assert!(error.contains("kind: NativeErrorKind,"));
    assert!(
        error.contains("self.emit_throw_current_function_realm_error(kind, message, output, f)?;")
    );
    assert!(error.contains("self.emit_branch_to_target(exit, f);"));
}
