use lila_test262::differential::DifferentialCorpus;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lila-corpus-library-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn json_names(root: &Path, directory: &Path, names: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            json_names(root, &path, names);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            names.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        }
    }
}

#[test]
fn compiled_inventory_contains_every_current_json_fixture_once() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/differential");
    let mut actual = Vec::new();
    json_names(&root, &root, &mut actual);
    actual.sort();
    let compiled = DifferentialCorpus::compiled();
    assert!(!compiled.is_empty());
    assert_eq!(compiled.len(), 12);
    assert_eq!(
        compiled.names().collect::<Vec<_>>(),
        actual.iter().map(String::as_str).collect::<Vec<_>>()
    );
    let directory = DifferentialCorpus::from_directory(&root).unwrap();
    assert_eq!(
        directory.names().collect::<Vec<_>>(),
        compiled.names().collect::<Vec<_>>()
    );
}

#[test]
fn empty_and_over_limit_inventories_cannot_become_green_subsets() {
    let directory = Directory::new();
    assert!(DifferentialCorpus::from_directory(&directory.0).is_err());
    for index in 0..=lila_test262::differential::MAX_CORPUS_ENTRIES {
        std::fs::write(directory.0.join(format!("{index:03}.json")), "{}").unwrap();
    }
    assert!(DifferentialCorpus::from_directory(&directory.0).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_entries_and_roots_are_rejected_instead_of_aliasing_cases() {
    let directory = Directory::new();
    let root = directory.0.join("cases");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("one.json"), "{}").unwrap();
    let alias = directory.0.join("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    assert!(DifferentialCorpus::from_directory(&alias).is_err());
    std::os::unix::fs::symlink(root.join("one.json"), root.join("alias.json")).unwrap();
    assert!(DifferentialCorpus::from_directory(&root).is_err());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn compiled_corpus_replays_every_entry_through_the_named_actual_worker() {
    use lila_test262::differential::{replay_corpus, DifferentialWorkerRunner, SpecExecOracle};
    let directory = Directory::new();
    let output = directory.0.join("reports");
    let runner =
        DifferentialWorkerRunner::new(env!("CARGO_BIN_EXE_lila-differential-worker")).unwrap();
    let report = replay_corpus(
        DifferentialCorpus::compiled(),
        SpecExecOracle::explicitly_enabled(),
        &runner,
        &output,
    )
    .unwrap();
    assert_eq!(
        (
            report.total(),
            report.completed(),
            report.matched(),
            report.failed()
        ),
        (12, 12, 12, 0)
    );
    assert!(report.is_green());
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["semantic_equivalence"], "not_established");
    for ordinal in 0..12 {
        let row: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.join(format!("case-{ordinal:03}.report.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(row["status"], "compared");
        assert_eq!(
            row["report"]["wasm_aot"]["worker_identity"],
            row["report"]["spec_exec"]["worker_identity"]
        );
        for backend in ["wasm_aot", "spec_exec"] {
            let worker: lila_test262::CompilerProvenance =
                serde_json::from_value(row["report"][backend]["worker_identity"].clone()).unwrap();
            assert_eq!(
                worker.identity().source_fingerprint(),
                lila_test262::CompilerProvenance::current()
                    .unwrap()
                    .identity()
                    .source_fingerprint()
            );
        }
    }
    assert!(replay_corpus(
        DifferentialCorpus::compiled(),
        SpecExecOracle::explicitly_enabled(),
        &runner,
        &output
    )
    .is_err());
}
