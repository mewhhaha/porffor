use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/intl-date-time-format-mode-capability.md");
const TASK: &str = include_str!("../../../tasks/23-intl402.md");

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

fn normalized_without_line_comments(source: &str) -> String {
    normalized(
        &source
            .lines()
            .filter(|line| !line.trim_start().starts_with("///"))
            .collect::<String>(),
    )
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
fn format_mode_keeps_only_its_required_copy_capability() {
    let declaration_region = bounded(
        DTF_SOURCE,
        "const INTL_DTF_WEEKDAYS_NARROW: [&str; 7] = [\"S\", \"M\", \"T\", \"W\", \"T\", \"F\", \"S\"];",
        "enum IntlDateTimeFormatReceiverOperation {",
    );
    assert_eq!(declaration_region.matches("#[").count(), 1);
    assert!(declaration_region.contains("#[derive(Clone, Copy)]\nenum DtfFormatMode {"));
    assert_eq!(
        normalized_without_line_comments(bounded(
            declaration_region,
            "enum DtfFormatMode {",
            "\n}",
        )),
        "String,Parts,"
    );
    assert!(!DTF_SOURCE.contains("pub(crate) enum DtfFormatMode"));

    for forbidden in [
        "PartialEq for DtfFormatMode",
        "Eq for DtfFormatMode",
        "Debug for DtfFormatMode",
        "Default for DtfFormatMode",
    ] {
        assert!(!DTF_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn parts_allocation_is_an_exact_exhaustive_mode_projection() {
    let allocation = normalized(bounded(
        DTF_SOURCE,
        "        self.emit_dtf_set_const(sink.length_local, 0, function);",
        "        self.emit_dtf_set_const(body_last_local, 0, function);",
    ));
    assert_eq!(
        allocation,
        concat!(
            "matchmode{",
            "DtfFormatMode::String=>{}",
            "DtfFormatMode::Parts=>{",
            "self.emit_dtf_set_const(sink.scratch_local,match&range{",
            "None=>INTL_DTF_MAX_PARTS,",
            "Some(_)=>INTL_DTF_MAX_RANGE_PARTS,",
            "},function,);",
            "self.emit_alloc_array_payload_with_length(",
            "sink.scratch_local,sink.array_local,function,)?;",
            "self.load_i64_to_local_from_offset(",
            "sink.array_local,HEAP_PTR_OFFSET,sink.buffer_local,function,);",
            "}",
            "}"
        )
    );
    for forbidden in ["_=>", "mode==", "mode!=", ".eq(&DtfFormatMode"] {
        assert!(!allocation.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn formatter_policy_has_one_complete_recursive_owner_census() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(DTF_SOURCE.matches("DtfFormatMode").count(), 20);
    assert_eq!(count_in_rust_sources(&source_root, "DtfFormatMode"), 20);
    assert_eq!(DTF_SOURCE.matches("DtfFormatMode::String").count(), 7);
    assert_eq!(DTF_SOURCE.matches("DtfFormatMode::Parts").count(), 8);
    assert_eq!(DTF_SOURCE.matches("DtfFormatTimes").count(), 7);
    assert_eq!(count_in_rust_sources(&source_root, "DtfFormatTimes"), 7);
    assert!(DTF_SOURCE.contains("\nstruct DtfFormatTimes {"));
    assert!(!DTF_SOURCE.contains("pub(crate) struct DtfFormatTimes"));
    let times = bounded(
        DTF_SOURCE,
        "struct DtfFormatTimes {",
        "/// A `format`/`formatRange` argument",
    );
    assert!(!times.contains("pub("));

    assert_eq!(
        DTF_SOURCE
            .matches("    fn emit_intl_dtf_build_format_with_kind(")
            .count(),
        1
    );
    assert!(!DTF_SOURCE.contains("pub(crate) fn emit_intl_dtf_build_format_with_kind("));
    assert_eq!(
        DTF_SOURCE
            .matches("self.emit_intl_dtf_build_format_with_kind(")
            .count(),
        3
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_intl_dtf_build_format_with_kind"),
        9
    );

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `DtfFormatMode`"));
        assert!(evidence.contains("owner-private `DtfFormatTimes`"));
        assert!(
            evidence.contains("2a3472c7bf2f6ea58fdca16c0e3aa06afb5e8d6dcbaaab9f96f5991939d8ab70")
        );
        assert!(
            evidence.contains("672ab8ac8cb31e3e580d22705dadd2707e3b06f4c0d7e567d56ec08c1c42cafb")
        );
        assert!(
            evidence.contains("76bff3098cbab0ded80001f3b2a4687a927045c5992f8cd31ac3f9976a471ae8")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(TASK.contains("intl-date-time-format-mode-capability.md"));
    assert!(CONTRACT.contains("does not close T23"));
}
