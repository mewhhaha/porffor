//! Fake-suite publication counts require complete product execution evidence.
use super::PublishedStatusCount;
use lila_engine::ExecutionBackend;
use lila_test262::{
    ConformanceRunVerdict, ConformanceRunner, LocalHarnessSource, RunConfig, RunSummary,
    SuiteConfig, SuiteManifest, TestExecutionId,
};
use std::collections::BTreeSet;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Only a complete, nonempty passing Wasm-AOT run can construct these counts.
#[derive(Debug)]
pub(super) struct VerifiedFakeSuiteCounts {
    full: PublishedStatusCount,
    wasm_safe: PublishedStatusCount,
}

impl VerifiedFakeSuiteCounts {
    pub(super) fn measure(mut config: SuiteConfig) -> Result<Self, String> {
        static NEXT_RUN: AtomicU64 = AtomicU64::new(0);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("cannot name fake-suite publication run: {err}"))?
            .as_nanos();
        let directory = config.snapshot_dir.join(format!(
            "{}-{timestamp}-{}",
            std::process::id(),
            NEXT_RUN.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&config.snapshot_dir)
            .map_err(|err| format!("cannot create fake-suite publication scratch root: {err}"))?;
        fs::create_dir(&directory).map_err(|err| {
            format!("cannot create fresh fake-suite publication scratch run: {err}")
        })?;
        config.snapshot_dir = directory;
        config.local_harness = LocalHarnessSource::EmbeddedWasmAot;
        config.worker_count = 2;
        let runner = ConformanceRunner::with_config(config);
        let full = runner.discover_suite(None)?;
        let wasm_safe = runner.discover_suite(Some("language/wasm/pass"))?;
        if full.cases.is_empty() || wasm_safe.cases.is_empty() {
            return Err(
                "fake-suite publication requires nonempty full and wasm-safe selections".into(),
            );
        }
        let measured = runner.run_full(RunConfig {
            execution_backend: ExecutionBackend::WasmAot,
            snapshot_name: "publication-fake-measured".into(),
            resume: false,
            ..RunConfig::default()
        })?;
        Self::from_measured_run(&full, &wasm_safe, measured)
    }

    fn from_measured_run(
        full: &SuiteManifest,
        wasm_safe: &SuiteManifest,
        measured: RunSummary,
    ) -> Result<Self, String> {
        match measured.verdict()? {
            ConformanceRunVerdict::NoEvidence => {
                return Err("fake-suite publication measured no executions".into());
            }
            ConformanceRunVerdict::Failed { total, failed } => {
                let detail = measured
                    .failures
                    .first()
                    .map(|failure| format!("; {}: {}", failure.test_id, failure.detail))
                    .unwrap_or_default();
                return Err(format!(
                    "fake-suite publication failed: {failed}/{total} executions failed{detail}"
                ));
            }
            ConformanceRunVerdict::Passed { .. } => {}
        }
        let full_ids: BTreeSet<_> = full.cases.iter().map(|case| case.execution_id()).collect();
        let wasm_safe_ids: BTreeSet<_> = wasm_safe
            .cases
            .iter()
            .map(|case| case.execution_id())
            .collect();
        let completed: BTreeSet<&TestExecutionId> = measured.completed_test_ids.iter().collect();
        if full_ids.len() != full.cases.len()
            || completed.len() != measured.completed_test_ids.len()
            || measured.total != full_ids.len()
            || completed != full_ids
            || wasm_safe_ids.is_empty()
            || wasm_safe_ids.len() != wasm_safe.cases.len()
            || !wasm_safe_ids.is_subset(&full_ids)
        {
            return Err(
                "fake-suite publication execution evidence is incomplete or inconsistent".into(),
            );
        }
        let wasm_safe_passed = completed.intersection(&wasm_safe_ids).count();
        Ok(Self {
            full: PublishedStatusCount {
                passed: measured.passed,
                total: measured.total,
            },
            wasm_safe: PublishedStatusCount {
                passed: wasm_safe_passed,
                total: wasm_safe_ids.len(),
            },
        })
    }

    pub(super) fn full(&self) -> &PublishedStatusCount {
        &self.full
    }

    pub(super) fn wasm_safe(&self) -> &PublishedStatusCount {
        &self.wasm_safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_config(name: &str, extra: Option<(&str, &str)>) -> SuiteConfig {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lila-publication-fake-{name}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let suite = root.join("vendor/test262");
        let safe = suite.join("test/language/wasm/pass");
        fs::create_dir_all(&safe).unwrap();
        fs::write(safe.join("safe.js"), "/*---\nflags: [raw]\n---*/\n1 + 2;\n").unwrap();
        if let Some((path, source)) = extra {
            let path = suite.join("test").join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        let mut config = SuiteConfig::default();
        config.suite_root = suite;
        config.snapshot_dir = root.join("scratch");
        config.local_harness = LocalHarnessSource::EmbeddedWasmAot;
        config.worker_count = 2;
        config
    }

    fn fixture_root(config: &SuiteConfig) -> PathBuf {
        config.suite_root.parent().unwrap().parent().unwrap().into()
    }

    fn measure_fixture_worker_entries(
        config: SuiteConfig,
    ) -> Result<VerifiedFakeSuiteCounts, String> {
        let runner = ConformanceRunner::with_config(config.clone());
        let full = runner.discover_suite(None)?;
        let safe = runner.discover_suite(Some("language/wasm/pass"))?;
        let mut measured = RunSummary {
            total: 0,
            passed: 0,
            counts_per_kind: Default::default(),
            counts_per_outcome: Default::default(),
            failures: Vec::new(),
            timeouts: Vec::new(),
            slowest_tests: Vec::new(),
            completed_test_ids: Vec::new(),
        };
        for (index, case) in full.cases.iter().enumerate() {
            let mut worker = config.clone();
            worker.worker_count = 1;
            let result = lila_test262::run_case_worker(
                worker,
                RunConfig {
                    filter: Some(case.execution_id().wire_key()),
                    execution_backend: ExecutionBackend::WasmAot,
                    snapshot_name: format!("finite-fixture-worker-{index}"),
                    ..RunConfig::default()
                },
            )?;
            measured.total += result.total;
            measured.passed += result.passed;
            measured.failures.extend(result.failures);
            measured
                .completed_test_ids
                .extend(result.completed_test_ids);
        }
        VerifiedFakeSuiteCounts::from_measured_run(&full, &safe, measured)
    }

    #[test]
    fn product_measurement_counts_modes_and_rejects_missing_completed_evidence() {
        let config = fixture_config(
            "passing",
            Some((
                "language/pass/two-modes.js",
                "/*---\n---*/\nassert(true);\n",
            )),
        );
        let verified = measure_fixture_worker_entries(config.clone()).unwrap();
        assert_eq!(verified.full().passed, 3);
        assert_eq!(verified.full().total, 3);
        assert_eq!(verified.wasm_safe().passed, 1);
        assert_eq!(verified.wasm_safe().total, 1);

        let runner = ConformanceRunner::with_config(config.clone());
        let full = runner.discover_suite(None).unwrap();
        let safe = runner.discover_suite(Some("language/wasm/pass")).unwrap();
        let measured = RunSummary {
            total: 3,
            passed: 3,
            counts_per_kind: Default::default(),
            counts_per_outcome: Default::default(),
            failures: Vec::new(),
            timeouts: Vec::new(),
            slowest_tests: Vec::new(),
            completed_test_ids: full
                .cases
                .iter()
                .skip(1)
                .map(|case| case.execution_id().clone())
                .collect(),
        };
        let error = VerifiedFakeSuiteCounts::from_measured_run(&full, &safe, measured).unwrap_err();
        assert!(error.contains("incomplete or inconsistent"), "{error}");
        fs::remove_dir_all(fixture_root(&config)).unwrap();
    }

    #[test]
    fn discovered_failure_outside_wasm_safe_subset_cannot_publish_green() {
        let config = fixture_config(
            "failing",
            Some((
                "language/pass/discovered-failure.js",
                "/*---\nflags: [raw]\n---*/\nthrow new Error('publication failure control');\n",
            )),
        );
        let runner = ConformanceRunner::with_config(config.clone());
        assert_eq!(runner.discover_suite(None).unwrap().cases.len(), 2);
        let error = measure_fixture_worker_entries(config.clone()).unwrap_err();
        assert!(error.contains("1/2 executions failed"), "{error}");
        assert!(error.contains("discovered-failure.js"), "{error}");
        fs::remove_dir_all(fixture_root(&config)).unwrap();
    }

    #[test]
    fn empty_fake_selection_cannot_publish_green() {
        let config = fixture_config("empty", None);
        fs::remove_dir_all(config.suite_root.join("test/language")).unwrap();
        let error = VerifiedFakeSuiteCounts::measure(config.clone()).unwrap_err();
        assert!(error.contains("nonempty"), "{error}");
        fs::remove_dir_all(fixture_root(&config)).unwrap();
    }
}
