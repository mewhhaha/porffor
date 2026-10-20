//! Complete original harness loading/materialization inside an owned fixture.
use super::sandbox::{relative_fixture_path, Sandbox};
use crate::{
    LocalHarnessSource, MaterializedTest, PreludeOrigin, PreludeStore, SuiteConfig, TestCase,
    TestExecutionMode,
};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Profile {
    None,
    CustomMerged,
    EmbeddedWasmAot,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    contents: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u32,
    source: String,
    execution_mode: TestExecutionMode,
    harness_profile: Profile,
    merged_harness: Option<String>,
    files: Vec<File>,
    #[serde(default)]
    overrides: Vec<File>,
}
pub(super) struct PreludeInput {
    wire: Wire,
    case: TestCase,
}
pub(super) struct LoadedPrelude {
    sandbox: Sandbox,
    case: TestCase,
    store: PreludeStore,
}
#[derive(Debug)]
pub(super) enum PreludeLoadError {
    Fixture(String),
    Rejected(String),
}
impl PreludeInput {
    pub(super) fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > 192 * 1024 {
            return Err("prelude probe exceeds 192 KiB".into());
        }
        let wire: Wire = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if wire.schema_version != 1
            || wire.source.len() > 16 * 1024
            || wire.files.len() + wire.overrides.len() > 16
        {
            return Err("prelude schema 1 requires at most 16 files and 16 KiB case source".into());
        }
        match (&wire.harness_profile, &wire.merged_harness) {
            (Profile::CustomMerged, Some(source)) if source.len() <= 16 * 1024 => {}
            (Profile::None | Profile::EmbeddedWasmAot, None) => {}
            _ => return Err("only custom_merged owns a bounded merged_harness operand".into()),
        }
        let mut bytes = 0usize;
        for files in [&wire.files, &wire.overrides] {
            let mut names = BTreeSet::new();
            for file in files {
                relative_fixture_path(&file.name)?;
                if !names.insert(file.name.as_str()) {
                    return Err("duplicate prelude fixture name".into());
                }
                bytes = bytes
                    .checked_add(file.contents.len())
                    .ok_or("prelude byte count overflow")?;
            }
        }
        if bytes > 64 * 1024 {
            return Err("prelude fixture contents exceed 64 KiB".into());
        }
        let mut cases = crate::parse_test_executions(
            "robustness/prelude.js".into(),
            "robustness/prelude.js".into(),
            wire.source.clone(),
        )?
        .into_iter()
        .filter(|case| case.execution_mode() == wire.execution_mode);
        let case = cases
            .next()
            .ok_or("frontmatter does not admit the selected execution mode")?;
        if cases.next().is_some() {
            return Err("prelude probe must select exactly one execution mode".into());
        }
        Ok(Self { wire, case })
    }
    pub(super) fn load(self, parent: &Path) -> Result<LoadedPrelude, PreludeLoadError> {
        let sandbox = Sandbox::new_in(parent).map_err(PreludeLoadError::Fixture)?;
        let io = |error: std::io::Error| PreludeLoadError::Fixture(error.to_string());
        let suite = sandbox.root().join("suite");
        let harness = suite.join("harness");
        std::fs::create_dir_all(&harness).map_err(io)?;
        for file in &self.wire.files {
            let path = harness.join(&file.name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(io)?;
            }
            std::fs::write(path, &file.contents).map_err(io)?;
        }
        let profile = match self.wire.harness_profile {
            Profile::None => LocalHarnessSource::None,
            Profile::EmbeddedWasmAot => LocalHarnessSource::EmbeddedWasmAot,
            Profile::CustomMerged => {
                let path = sandbox.root().join("merged.js");
                std::fs::write(
                    &path,
                    self.wire
                        .merged_harness
                        .as_ref()
                        .expect("checked merged operand"),
                )
                .map_err(io)?;
                LocalHarnessSource::File(path)
            }
        };
        let mut config = SuiteConfig::default();
        config.suite_root = suite;
        config.local_harness = profile;
        config.snapshot_dir = sandbox.root().join("unused-snapshots");
        config.worker_count = 1;
        let mut store = crate::load_preludes(&config).map_err(PreludeLoadError::Rejected)?;
        for file in self.wire.overrides {
            // The original insertion invalidates real host ownership when an
            // assertion/sta override breaks its completed embedded authority.
            store.insert(file.name, file.contents, PreludeOrigin::LocalMerged);
        }
        Ok(LoadedPrelude {
            sandbox,
            case: self.case,
            store,
        })
    }
}
impl LoadedPrelude {
    pub(super) fn materialize(&self) -> Result<MaterializedTest, String> {
        let materialized = crate::materialize_test(&self.case, &self.store)?;
        let bytes = materialized
            .source
            .len()
            .checked_add(materialized.module_prelude.as_ref().map_or(0, String::len))
            .and_then(|bytes| {
                bytes.checked_add(materialized.agent_prelude.as_ref().map_or(0, String::len))
            })
            .ok_or("materialized prelude length overflow")?;
        if bytes > 512 * 1024 {
            return Err("materialized probe exceeds 512 KiB".into());
        }
        Ok(materialized)
    }
    pub(super) fn finish(self) -> Result<(), String> {
        self.sandbox.finish()
    }
}
