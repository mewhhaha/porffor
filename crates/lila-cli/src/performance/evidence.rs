//! Aggregate existing admitted evidence without invoking a benchmark or exporter.
use super::{digest, read_bounded, write_new};
use lila_engine::{ExecutionBackend, SelectedIntlDataBundle};
use lila_test262::{CompilerProvenance, ConformanceRunner, SuiteConfig};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

struct ConformanceOptions {
    config: SuiteConfig,
    snapshot: String,
    backend: ExecutionBackend,
    output: PathBuf,
}
fn parse_conformance(arguments: Vec<String>) -> Result<ConformanceOptions, String> {
    let mut config = SuiteConfig::default();
    let mut snapshot = None;
    let mut output = None;
    let mut backend = None;
    let mut root_seen = false;
    let mut snapshots_seen = false;
    let mut args = arguments.into_iter().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--suite-root" if !root_seen => { config.suite_root = PathBuf::from(value); root_seen = true; }
            "--snapshot-dir" if !snapshots_seen => { config.snapshot_dir = PathBuf::from(value); snapshots_seen = true; }
            "--snapshot" if snapshot.is_none() => snapshot = Some(value),
            "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
            "--execution-backend" if backend.is_none() => backend = Some(crate::parse_execution_backend(&value)?),
            _ => return Err(format!("unknown or duplicate performance conformance option: {flag}")),
        }
    }
    let snapshot = snapshot.ok_or("performance conformance needs --snapshot NAME")?;
    if snapshot.trim().is_empty() { return Err("snapshot name must not be blank".into()); }
    Ok(ConformanceOptions { config, snapshot, backend: backend.unwrap_or(ExecutionBackend::WasmAot),
        output: output.ok_or("performance conformance needs --output-dir PATH")? })
}
pub(super) fn conformance(arguments: Vec<String>) -> Result<PathBuf, String> {
    let options = parse_conformance(arguments)?;
    let report = ConformanceRunner::with_config(options.config)
        .load_performance_evidence(&options.snapshot, options.backend)?;
    let bytes = report.to_pretty_json()?;
    fs::create_dir(&options.output).map_err(|error| format!("performance output must be a fresh directory: {error}"))?;
    let path = options.output.join("report.json");
    write_new(&path, bytes.as_bytes())?;
    Ok(path)
}

struct DataOptions { bundles: Vec<PathBuf>, output: PathBuf }
fn parse_data(arguments: Vec<String>) -> Result<DataOptions, String> {
    let mut bundles = Vec::new();
    let mut output = None;
    let mut args = arguments.into_iter().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--bundle" if bundles.len() < 16 => bundles.push(PathBuf::from(value)),
            "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown, duplicate or excessive performance data option: {flag}")),
        }
    }
    if bundles.is_empty() { return Err("performance data needs 1..16 --bundle PATH inputs".into()); }
    Ok(DataOptions { bundles, output: output.ok_or("performance data needs --output-dir PATH")? })
}
#[derive(Debug, Serialize)]
struct ComponentFootprint {
    code: u16,
    section: &'static str,
    bytes: usize,
    sha256: String,
}
#[derive(Debug, Serialize)]
struct DataVersions {
    icu4x: &'static str,
    cldr: &'static str,
    unicode: &'static str,
    icu_data_tag: &'static str,
    segmenter_lstm: &'static str,
    tzdb: &'static str,
}
#[derive(Debug, Serialize)]
struct BundleFootprint {
    input: PathBuf,
    retained_bundle: String,
    sha256: String,
    identity: String,
    versions: DataVersions,
    bundle_bytes: usize,
    component_bytes: usize,
    framing_and_identity_bytes: usize,
    components: Vec<ComponentFootprint>,
}
fn inspect_bundle(input: PathBuf, retained_bundle: String, bytes: &[u8]) -> Result<BundleFootprint, String> {
    let selected = SelectedIntlDataBundle::from_export_bytes(bytes).map_err(|error| format!("cannot admit Intl footprint input {}: {error}", input.display()))?;
    let identity = String::from_utf8(selected.identity().artifact_identity().as_bytes().to_vec()).map_err(|error| error.to_string())?;
    let versions = selected.identity().versions();
    let components = selected.component_images().into_iter().map(|(component, image)| ComponentFootprint {
        code: component.code(), section: component.section_name(), bytes: image.len(), sha256: digest(&image),
    }).collect::<Vec<_>>();
    let component_bytes = components.iter().try_fold(0usize, |sum, component| sum.checked_add(component.bytes))
        .ok_or("Intl component byte sum overflow")?;
    let framing_and_identity_bytes = bytes.len().checked_sub(component_bytes).ok_or("Intl component sizes exceed admitted export")?;
    Ok(BundleFootprint { input, retained_bundle, sha256: digest(bytes), identity,
        versions: DataVersions { icu4x: versions.icu4x, cldr: versions.cldr, unicode: versions.unicode,
            icu_data_tag: versions.icu_data_tag, segmenter_lstm: versions.segmenter_lstm, tzdb: versions.tzdb },
        bundle_bytes: bytes.len(), component_bytes, framing_and_identity_bytes, components })
}
#[derive(Debug, Serialize)]
struct DataReport {
    version: u32,
    compiler: CompilerProvenance,
    complete: bool,
    scope: &'static str,
    unicode_accounting: &'static str,
    native_code_and_heap_accounting: &'static str,
    bundles: Vec<BundleFootprint>,
}
pub(super) fn data(arguments: Vec<String>) -> Result<PathBuf, String> {
    let options = parse_data(arguments)?;
    let mut report = DataReport { version: 1, compiler: CompilerProvenance::current()?, complete: false,
        scope: "actual-admitted-Intl-component-images;includes-selected-named-time-zone-and-zone-name-images;no-data-generation",
        unicode_accounting: "Unicode versions and bytes are reported per consuming component; shared Unicode data has no invented disjoint total",
        native_code_and_heap_accounting: "unavailable;this report measures serialized bundle frames, not native compiler tables, linked code, live heap or process peak",
        bundles: Vec::new() };
    fs::create_dir(&options.output).map_err(|error| format!("performance output must be a fresh directory: {error}"))?;
    let path = options.output.join("report.json");
    // On refusal the retained prefix stays explicitly incomplete; report.json
    // is only created after every requested bundle has passed original admission.
    write_new(&options.output.join("incomplete.json"), &serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?)?;
    for (index, input) in options.bundles.into_iter().enumerate() {
        let bytes = read_bounded(&input, SelectedIntlDataBundle::MAX_EXPORT_BYTES as u64)?;
        let retained = format!("bundle-{index:03}.lila-intl");
        let footprint = inspect_bundle(input, retained.clone(), &bytes)?;
        write_new(&options.output.join(retained), &bytes)?;
        report.bundles.push(footprint);
    }
    report.complete = true;
    write_new(&path, &serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?)?;
    fs::remove_file(options.output.join("incomplete.json")).map_err(|error| error.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> { values.iter().map(|value| (*value).into()).collect() }
    #[test]
    fn evidence_commands_do_not_accept_benchmark_or_ignored_options() {
        for values in [vec!["conformance", "--snapshot", "latest", "--output-dir", "x", "--idle-machine"],
            vec!["conformance", "--snapshot", "latest", "--output-dir", "x", "--samples", "3"],
            vec!["conformance", "--snapshot", "x", "--snapshot", "y", "--output-dir", "z"],
            vec!["conformance", "--snapshot", " ", "--output-dir", "x"]] {
            assert!(parse_conformance(args(&values)).is_err());
        }
        for values in [vec!["data", "--output-dir", "x"], vec!["data", "--bundle", "x", "--samples", "3"],
            vec!["data", "--bundle", "x", "--output-dir", "a", "--output-dir", "b"]] {
            assert!(parse_data(args(&values)).is_err());
        }
        assert!(inspect_bundle("bad".into(), "bundle-000.lila-intl".into(), b"unadmitted").is_err());
    }

    #[test]
    fn sparse_footprint_measures_original_frames_and_refuses_corrupt_bundle() {
        use lila_engine::{CustomIntlProfile, CustomProfileId, IntlCompilationProfile, IntlDataSelection};
        let profile = CustomIntlProfile::for_services(CustomProfileId::parse("footprint-list").unwrap(), &["ListFormat"]).unwrap();
        let selection = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(profile));
        let selected = selection.selected().unwrap();
        let bytes = selected.export_bytes().unwrap();
        let report = inspect_bundle("list.bundle".into(), "bundle-000.lila-intl".into(), &bytes).unwrap();
        assert_eq!(report.components.iter().map(|component| component.code).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(report.bundle_bytes, bytes.len());
        assert_eq!(report.framing_and_identity_bytes, 16 + report.identity.len() + 8 * 2);
        assert_eq!(report.component_bytes + report.framing_and_identity_bytes, report.bundle_bytes);
        assert!(report.components.iter().all(|component| component.bytes > 0 && component.sha256.len() == 64));
        let mut corrupt = bytes.to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(inspect_bundle("bad.bundle".into(), "bad".into(), &corrupt).is_err());
    }
}
