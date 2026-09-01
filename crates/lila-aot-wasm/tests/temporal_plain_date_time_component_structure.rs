use std::fs;
use std::path::Path;

const METHODS: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
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
fn plain_date_time_component_is_a_non_copyable_two_variant_domain() {
    let domain = bounded(
        METHODS,
        "    With,\n}\n\n",
        "\n\nimpl TemporalDateTimeFieldKey",
    );
    let declaration = bounded(
        domain,
        "pub(super) enum TemporalPlainDateTimeComponent {",
        "\n}",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["PlainDate,", "PlainTime,"]);
    assert!(!domain.contains("#[derive("));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!declaration.contains(capability));
    }
    assert!(!METHODS.contains("pub enum TemporalPlainDateTimeComponent"));
    assert!(!METHODS.contains("pub(crate) enum TemporalPlainDateTimeComponent"));
}

#[test]
fn component_emitter_extracts_the_receiver_then_projects_once() {
    let emitter = bounded(
        METHODS,
        "    pub(super) fn emit_temporal_plain_date_time_to_component(",
        "    /// Temporal proposal 5.3.x `add` and `subtract`",
    );
    let extraction = emitter
        .find("self.emit_temporal_plain_date_time_fields_from_receiver(")
        .expect("missing receiver field extraction");
    let projection = emitter
        .find("match component {")
        .expect("missing component projection");

    assert!(emitter.contains("component: TemporalPlainDateTimeComponent,"));
    assert!(extraction < projection);
    assert_eq!(emitter.matches("match component {").count(), 1);
    assert_eq!(
        emitter
            .matches("TemporalPlainDateTimeComponent::PlainDate => {")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("TemporalPlainDateTimeComponent::PlainTime => {")
            .count(),
        1
    );

    let plain_date = bounded(
        emitter,
        "TemporalPlainDateTimeComponent::PlainDate => {",
        "            TemporalPlainDateTimeComponent::PlainTime => {",
    );
    assert!(plain_date.contains("TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX"));
    assert_eq!(
        plain_date
            .matches("let prototype_payload_local = self.reserve_temp_local();")
            .count(),
        1
    );
    assert!(plain_date.contains("self.emit_alloc_temporal_plain_date("));
    assert!(plain_date.contains("calendar_payload_local,"));
    assert_eq!(
        plain_date
            .matches("self.release_temp_local(prototype_payload_local);")
            .count(),
        1
    );
    assert!(!plain_date.contains("temporal_plain_date_time_time_locals"));
    assert!(!plain_date.contains("emit_alloc_temporal_plain_time"));

    let plain_time = bounded(
        emitter,
        "TemporalPlainDateTimeComponent::PlainTime => {",
        "        }\n\n        self.release_temporal_plain_date_time_field_locals",
    );
    assert!(plain_time.contains("Self::temporal_plain_date_time_time_locals(&field_locals)"));
    assert!(plain_time.contains("self.emit_alloc_temporal_plain_time("));
    assert!(!plain_time.contains("TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!plain_time.contains("emit_alloc_temporal_plain_date("));

    assert!(!emitter.contains("time: bool"));
    assert!(!emitter.contains("if time"));
    assert!(!emitter.contains("matches!(component"));
    assert!(!emitter.contains("_ =>"));
    assert!(!emitter.contains("unreachable!"));
}

#[test]
fn exactly_two_standard_producers_name_their_component() {
    let producers = bounded(
        STANDARD,
        "            StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate => {",
        "            StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime => {",
    );

    assert_eq!(
        STANDARD.matches("TemporalPlainDateTimeComponent,").count(),
        1,
        "standard.rs must import the component explicitly"
    );
    assert_eq!(
        producers
            .matches("self.emit_temporal_plain_date_time_to_component(")
            .count(),
        2
    );
    assert_eq!(
        producers
            .matches("TemporalPlainDateTimeComponent::PlainDate,")
            .count(),
        1
    );
    assert_eq!(
        producers
            .matches("TemporalPlainDateTimeComponent::PlainTime,")
            .count(),
        1
    );
    assert!(!producers.contains("false"));
    assert!(!producers.contains("true"));
    assert_eq!(
        STANDARD
            .matches("self.emit_temporal_plain_date_time_to_component(")
            .count(),
        2
    );
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_temporal_plain_date_time_to_component(",),
        3,
        "the component emitter definition and both standard producers must stay inventoried"
    );
}
