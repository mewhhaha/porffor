use std::fs;
use std::path::Path;

const DURATION_SOURCE: &str = include_str!("../src/builtins/temporal_duration.rs");
const PLAIN_DATE_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_time.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-field-offset-table-privacy.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

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
fn field_offset_tables_are_owner_private() {
    for (source, declaration) in [
        (
            DURATION_SOURCE,
            "const TEMPORAL_DURATION_FIELD_OFFSETS: [u64; 10] = [",
        ),
        (
            PLAIN_DATE_TIME_SOURCE,
            "const TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS: [u64; 9] = [",
        ),
    ] {
        assert_eq!(source.matches(declaration).count(), 1, "`{declaration}`");
        assert!(!source.contains(&format!("pub(crate) {declaration}")));
    }
}

#[test]
fn each_table_keeps_declaration_order_and_two_fixed_consumers() {
    let duration = normalized(bounded(
        DURATION_SOURCE,
        "const TEMPORAL_DURATION_FIELD_OFFSETS: [u64; 10] = [",
        "/// Declaration order: the constructor argument order",
    ));
    assert_eq!(
        duration,
        "HEAP_TEMPORAL_DURATION_YEARS_OFFSET,HEAP_TEMPORAL_DURATION_MONTHS_OFFSET,HEAP_TEMPORAL_DURATION_WEEKS_OFFSET,HEAP_TEMPORAL_DURATION_DAYS_OFFSET,HEAP_TEMPORAL_DURATION_HOURS_OFFSET,HEAP_TEMPORAL_DURATION_MINUTES_OFFSET,HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET,HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET,];"
    );
    assert_eq!(
        DURATION_SOURCE
            .matches("TEMPORAL_DURATION_FIELD_OFFSETS")
            .count(),
        3
    );
    assert!(DURATION_SOURCE.contains("TEMPORAL_DURATION_FIELD_OFFSETS[index],"));

    let plain_date_time = normalized(bounded(
        PLAIN_DATE_TIME_SOURCE,
        "const TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS: [u64; 9] = [",
        "impl<'a> FunctionBuilder<'a>",
    ));
    assert_eq!(
        plain_date_time,
        "HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,];"
    );
    assert_eq!(
        PLAIN_DATE_TIME_SOURCE
            .matches("TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS")
            .count(),
        3
    );
    assert_eq!(
        PLAIN_DATE_TIME_SOURCE
            .matches(".zip(field_locals.iter())")
            .count(),
        2
    );
}

#[test]
fn field_offset_tables_have_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for name in [
        "TEMPORAL_DURATION_FIELD_OFFSETS",
        "TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            3,
            "recursive `{name}`"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TEMPORAL_DURATION_FIELD_OFFSETS`"));
        assert!(evidence.contains("owner-private `TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS`"));
        assert!(
            evidence.contains("b47f9d79e4e1dc65b91a4ac7a2663a20b54cb5b6aea099266b381e6380e06ab1")
        );
        assert!(
            evidence.contains("f7047424c3fe0d3837f3d5db310d41d2c7a61740badcb97ec606c89c65746123")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
