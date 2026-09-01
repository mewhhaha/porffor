use std::fs;
use std::path::Path;

const CLI_SOURCE: &str = include_str!("../src/main.rs");

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
fn test262_verdict_command_is_the_exact_private_no_capability_domain() {
    let declaration_marker = "enum Test262VerdictCommand {";
    assert_eq!(CLI_SOURCE.matches(declaration_marker).count(), 1);
    assert_eq!(
        CLI_SOURCE
            .matches(
                "fn is_identifier_continue(ch: char) -> bool {\n    is_identifier_start(ch) || ch.is_ascii_digit()\n}\n\nenum Test262VerdictCommand {\n    Run,\n    Shard,\n}\n\nimpl Test262VerdictCommand {"
            )
            .count(),
        1,
        "the private command domain must be adjacent to its owner with no attributes, visibility or intervening text"
    );
    let variants = bounded(
        CLI_SOURCE,
        "enum Test262VerdictCommand {",
        "\n}\n\nimpl Test262VerdictCommand",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(variants, ["Run,", "Shard,"]);

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "Test262VerdictCommand"),
        5,
        "the declaration, impl, typed consumer and two producers must own every mention"
    );
}

#[test]
fn test262_verdict_command_has_one_exact_exhaustive_spelling_projection() {
    let implementation = bounded(
        CLI_SOURCE,
        "impl Test262VerdictCommand {",
        "fn require_passing_test262_verdict(",
    );
    assert_eq!(
        normalized(implementation),
        "constfnas_str(self)->&'staticstr{matchself{Self::Run=>\"run\",Self::Shard=>\"shard\",}}}"
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("Self::Run =>").count(), 1);
    assert_eq!(implementation.matches("Self::Shard =>").count(), 1);
    for forbidden in ["_ =>", "==", "!=", "matches!", "Default"] {
        assert!(!implementation.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn test262_run_and_shard_are_the_only_ordered_verdict_command_producers() {
    let consumer = bounded(
        CLI_SOURCE,
        "fn require_passing_test262_verdict(",
        "fn handle_test262_command(",
    );
    assert!(consumer.contains("command: Test262VerdictCommand,"));
    assert!(consumer.contains("verdict: ConformanceRunVerdict,"));
    assert_eq!(consumer.matches("command.as_str()").count(), 1);
    assert_eq!(consumer.matches("ConformanceRunVerdict::").count(), 3);
    assert_eq!(
        consumer
            .matches("ConformanceRunVerdict::NoEvidence =>")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("ConformanceRunVerdict::Passed { .. } =>")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("ConformanceRunVerdict::Failed { total, failed } =>")
            .count(),
        1
    );
    assert!(consumer.contains("test262 {command} produced no verdict: zero cases were selected"));
    assert!(consumer.contains("test262 {command} failed: {} of {} cases did not pass"));
    assert_before(consumer, "command.as_str()", "match verdict {");
    for forbidden in ["_ =>", "unwrap_or", "unwrap_or_else"] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "require_passing_test262_verdict("),
        3,
        "the definition and exact run/shard calls must own every invocation"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "Test262VerdictCommand::Run"),
        1
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "Test262VerdictCommand::Shard"),
        1
    );

    let command_dispatch = bounded(
        CLI_SOURCE,
        "fn handle_test262_command(",
        "fn parse_test262_args(",
    );
    for (start, end, variant) in [
        ("        \"run\" => {", "        \"report\" => {", "Run"),
        ("        \"shard\" => {", "        _ => Err(", "Shard"),
    ] {
        let command_arm = bounded(command_dispatch, start, end);
        assert_before(command_arm, "let summary =", "summary.verdict()?");
        assert_before(command_arm, "println!(\"passed:", "summary.verdict()?");
        assert_eq!(
            command_arm
                .matches("require_passing_test262_verdict(")
                .count(),
            1
        );
        let normalized_arm = normalized(command_arm);
        assert!(normalized_arm.ends_with(&format!(
            "require_passing_test262_verdict(Test262VerdictCommand::{variant},summary.verdict()?)}}"
        )));
    }
}
