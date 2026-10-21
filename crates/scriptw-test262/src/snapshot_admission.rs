//! Presence-aware wire decoding precedes all stateful/historical use policies.
//! Only this constructor can mint the immutable admitted envelope. A wire value
//! used by writers or corruption tests cannot enter readers without admission.
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct AdmittedSnapshot {
    file: SnapshotFile,
    artifact_kind: SnapshotArtifactKind,
    provenance: SnapshotProvenance,
}

impl std::ops::Deref for AdmittedSnapshot {
    type Target = SnapshotFile;
    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl AdmittedSnapshot {
    pub(super) fn into_parts(self) -> (SnapshotFile, SnapshotArtifactKind, SnapshotProvenance) {
        (self.file, self.artifact_kind, self.provenance)
    }
    pub(super) fn provenance(&self) -> &SnapshotProvenance {
        &self.provenance
    }
    pub(super) fn require_current(&self, path: &Path, operation: &str) -> Result<(), String> {
        self.provenance
            .require_current(operation)
            .map(|_| ())
            .map_err(|error| format!("{error} in {}", path.display()))
    }
    pub(super) fn require_running(&self, path: &Path, operation: &str) -> Result<(), String> {
        self.provenance
            .require_running(operation)
            .map(|_| ())
            .map_err(|error| format!("{error} in {}", path.display()))
    }
    pub(super) fn require_compiler(
        &self,
        expected: &CompilerProvenance,
        path: &Path,
    ) -> Result<(), String> {
        let actual = self.provenance.require_current("matrix node evidence")?;
        if actual != expected {
            return Err(format!("node snapshot {} has different compiler_identity from its aggregate or resume context", path.display()));
        }
        Ok(())
    }
}

pub(super) fn read_snapshot_file(path: &Path) -> Result<AdmittedSnapshot, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read snapshot {}: {err}", path.display()))?;
    decode_snapshot_bytes(raw.as_bytes(), path)
}

/// Robustness and file readers use this same schema/provenance admission.
pub(crate) fn decode_snapshot_bytes(raw: &[u8], path: &Path) -> Result<AdmittedSnapshot, String> {
    let file: SnapshotFile = serde_json::from_slice(raw)
        .map_err(|err| format!("failed to parse snapshot {}: {err}", path.display()))?;
    admit(file, path)
}

pub(super) fn admit(file: SnapshotFile, path: &Path) -> Result<AdmittedSnapshot, String> {
    let artifact_kind = SnapshotArtifactKind::decode(file.snapshot_version, file.producer)
        .map_err(|err| format!("invalid snapshot schema in {}: {err}", path.display()))?;
    let provenance = file
        .compiler_identity
        .clone()
        .admit(artifact_kind)
        .map_err(|err| {
            format!(
                "invalid snapshot compiler identity in {}: {err}",
                path.display()
            )
        })?;
    validate_snapshot_checkpoint_identity(&file, artifact_kind)
        .map_err(|err| format!("invalid snapshot schema in {}: {err}", path.display()))?;
    if artifact_kind.has_execution_identity() {
        parse_snapshot_execution_backend(&file.execution_backend)
            .map_err(|err| format!("invalid snapshot schema in {}: {err}", path.display()))?;
    }
    Ok(AdmittedSnapshot {
        file,
        artifact_kind,
        provenance,
    })
}
