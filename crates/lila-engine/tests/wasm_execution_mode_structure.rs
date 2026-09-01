use std::fs;
use std::path::Path;

const ENGINE_SOURCE: &str = include_str!("../src/lib.rs");

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
fn wasm_execution_mode_is_the_exact_private_no_capability_domain() {
    assert_eq!(
        ENGINE_SOURCE
            .matches(
                "pub struct Engine {\n    realm: Realm,\n}\n\nenum WasmExecutionMode {\n    Legacy,\n    Structured,\n}\n\nenum WasmExecutionOutcome {"
            )
            .count(),
        1
    );
    let declaration = bounded(
        ENGINE_SOURCE,
        "enum WasmExecutionMode {",
        "enum WasmExecutionOutcome {",
    );
    assert_eq!(normalized(declaration), "Legacy,Structured,}");
    assert!(!declaration.contains("#[derive"));

    let production = ENGINE_SOURCE
        .split_once("\n#[cfg(test)]\nmod tests {")
        .expect("engine unit-test boundary")
        .0;
    assert_eq!(production.matches("WasmExecutionMode").count(), 14);
    assert_eq!(ENGINE_SOURCE.matches("WasmExecutionMode").count(), 16);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "WasmExecutionMode"), 16);
    for forbidden in [
        "pub enum WasmExecutionMode",
        "impl WasmExecutionMode",
        "Default for WasmExecutionMode",
        "Clone for WasmExecutionMode",
        "Copy for WasmExecutionMode",
        "Debug for WasmExecutionMode",
        "PartialEq for WasmExecutionMode",
        "Eq for WasmExecutionMode",
        "== WasmExecutionMode::",
        "!= WasmExecutionMode::",
        "matches!(mode",
    ] {
        assert!(!production.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn five_entry_points_fix_their_exact_execution_mode() {
    let entries = [
        (
            "fn run_source_with_cached_wasm(",
            "fn observe_source_with_cached_wasm(",
            "Legacy",
        ),
        (
            "fn observe_source_with_cached_wasm(",
            "fn run_source_with_cached_wasm_on_current_thread(",
            "Structured",
        ),
        (
            "fn run_source_with_cached_wasm_on_current_thread(",
            "fn execute_source_with_cached_wasm_on_current_thread(",
            "Legacy",
        ),
        (
            "fn run_with_wasm_bytes_inner(",
            "fn execute_with_wasm_bytes_inner(",
            "Legacy",
        ),
        (
            "fn run_with_wasm_bytes_inner_with_agents(",
            "fn execute_with_wasm_bytes_inner_with_agents(",
            "Legacy",
        ),
    ];
    for (start, end, variant) in entries {
        let entry = bounded(ENGINE_SOURCE, start, end);
        assert_eq!(entry.matches("WasmExecutionMode::").count(), 1, "{start}");
        assert!(
            entry.contains(&format!("&WasmExecutionMode::{variant},")),
            "{start} must select {variant}"
        );
    }

    let production = ENGINE_SOURCE
        .split_once("\n#[cfg(test)]\nmod tests {")
        .expect("engine unit-test boundary")
        .0;
    assert_eq!(production.matches("mode: &WasmExecutionMode,").count(), 3);
}

#[test]
fn both_consumers_exhaustively_project_output_ownership_and_result_shape() {
    let output_projection = bounded(
        ENGINE_SOURCE,
        "fn for_mode(mode: &WasmExecutionMode) -> Self {",
        "fn record(&self, text: &str)",
    );
    assert_eq!(
        normalized(output_projection),
        "matchmode{WasmExecutionMode::Legacy=>Self::DelegateOnly,WasmExecutionMode::Structured=>Self::Capture(Arc::new(Mutex::new(Vec::new()))),}}"
    );

    let execution = bounded(
        ENGINE_SOURCE,
        "fn execute_with_wasm_bytes_inner_with_agents(",
        "enum WasmtimeExportedMemory {",
    );
    assert!(execution.contains("output_events: WasmOutputEvents::for_mode(mode),"));
    let result_projection = bounded(execution, "match mode {", "\n    }\n}");
    assert_eq!(
        result_projection
            .matches("WasmExecutionMode::Legacy =>")
            .count(),
        1
    );
    assert_eq!(
        result_projection
            .matches("WasmExecutionMode::Structured =>")
            .count(),
        1
    );
    assert_eq!(
        result_projection
            .matches("Ok(WasmExecutionOutcome::Legacy(")
            .count(),
        1
    );
    assert_eq!(
        result_projection
            .matches("Ok(WasmExecutionOutcome::Structured(")
            .count(),
        1
    );
    let legacy_arm = bounded(
        result_projection,
        "WasmExecutionMode::Legacy => {",
        "WasmExecutionMode::Structured => {",
    );
    assert_eq!(
        legacy_arm
            .matches("Ok(WasmExecutionOutcome::Legacy(")
            .count(),
        1
    );
    assert!(!legacy_arm.contains("WasmExecutionOutcome::Structured"));
    let structured_arm = result_projection
        .split_once("WasmExecutionMode::Structured => {")
        .expect("structured result arm")
        .1;
    assert_eq!(
        structured_arm
            .matches("Ok(WasmExecutionOutcome::Structured(")
            .count(),
        1
    );
    assert!(!structured_arm.contains("WasmExecutionOutcome::Legacy"));
    assert!(!result_projection.contains("_ =>"));
    assert!(execution.find("WasmOutputEvents::for_mode(mode)") < execution.find("match mode {"));
}
