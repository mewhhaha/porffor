use std::fs;
use std::path::Path;

const CLI_LIBRARY_SOURCE: &str = include_str!("../src/lib.rs");

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
fn cli_output_ending_is_the_exact_private_no_capability_domain() {
    assert_eq!(
        CLI_LIBRARY_SOURCE.matches("enum CliOutputEnding {").count(),
        1
    );
    assert_eq!(
        CLI_LIBRARY_SOURCE
            .matches(
                "}\n\nenum CliOutputEnding {\n    None,\n    Newline,\n}\n\nfn write_cli_stdout("
            )
            .count(),
        1,
        "the private domain must be adjacent to its sinks with no attributes or visibility"
    );
    let variants = bounded(
        CLI_LIBRARY_SOURCE,
        "enum CliOutputEnding {",
        "\n}\n\nfn write_cli_stdout(",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(variants, ["None,", "Newline,"]);

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "CliOutputEnding"),
        12,
        "the declaration, two typed sinks, four exhaustive arms and five macro producers must own every mention"
    );
}

#[test]
fn stdout_and_stderr_exhaustively_project_the_same_two_byte_endings() {
    for (start, end, stream) in [
        ("fn write_cli_stdout(", "fn write_cli_stderr(", "stdout"),
        ("fn write_cli_stderr(", "macro_rules! print", "stderr"),
    ] {
        let sink = bounded(CLI_LIBRARY_SOURCE, start, end);
        assert!(sink.contains("ending: CliOutputEnding"));
        assert_eq!(sink.matches("match ending {").count(), 1);
        assert_eq!(sink.matches("CliOutputEnding::None =>").count(), 1);
        assert_eq!(sink.matches("CliOutputEnding::Newline =>").count(), 1);
        assert!(normalized(sink).contains(&format!(
            "matchending{{CliOutputEnding::None=>{{write!(output,\"{{arguments}}\").expect(\"writingCLI{stream}shouldsucceed\");}}CliOutputEnding::Newline=>{{writeln!(output,\"{{arguments}}\").expect(\"writingCLI{stream}shouldsucceed\");}}}}"
        )));
        let lock = sink
            .find(".lock()")
            .expect("sink must acquire its output lock");
        let decision = sink
            .find("match ending {")
            .expect("sink must project the output ending");
        assert!(
            lock < decision,
            "the sink must retain lock-before-write order"
        );
        for forbidden in ["newline: bool", "if ending", "_ =>", "==", "!=", "matches!"] {
            assert!(!sink.contains(forbidden), "found `{forbidden}` in {stream}");
        }
    }
}

#[test]
fn print_macros_are_the_exact_five_output_ending_producers() {
    let macros = bounded(
        CLI_LIBRARY_SOURCE,
        "macro_rules! print {",
        "include!(\"main.rs\");",
    );
    assert_eq!(
        normalized(macros),
        r#"($($arg:tt)*)=>{{$crate::write_cli_stdout(format_args!($($arg)*),$crate::CliOutputEnding::None)}};}macro_rules!println{()=>{{$crate::write_cli_stdout(format_args!(""),$crate::CliOutputEnding::Newline)}};($($arg:tt)*)=>{{$crate::write_cli_stdout(format_args!($($arg)*),$crate::CliOutputEnding::Newline)}};}macro_rules!eprintln{()=>{{$crate::write_cli_stderr(format_args!(""),$crate::CliOutputEnding::Newline)}};($($arg:tt)*)=>{{$crate::write_cli_stderr(format_args!($($arg)*),$crate::CliOutputEnding::Newline)}};}"#,
        "print, println and eprintln must retain their exact stream and ending policies"
    );
}
