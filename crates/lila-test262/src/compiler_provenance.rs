//! The snapshot version and its compiler binding are one checked identity.

use lila_engine::{CompilerCommitId, CompilerDigest, CompilerIdentity, CompilerSourceRevision};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The mandatory serialized compiler contract used by execution artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerProvenance(CompilerIdentity);

impl CompilerProvenance {
    pub fn current() -> Result<Self, String> {
        CompilerIdentity::current().cloned().map(Self)
    }

    pub fn identity(&self) -> &CompilerIdentity {
        &self.0
    }

    pub fn require_running(&self, operation: &str) -> Result<&Self, String> {
        if self.identity() != CompilerIdentity::current()? {
            return Err(format!("{operation} refuses evidence from a different compiler build or executable; retain it and use a fresh snapshot name"));
        }
        Ok(self)
    }
}

/// Old evidence has an explicit schema and cannot acquire a current compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacySnapshotVersion {
    V4,
    V5,
    PathOnlyV6,
    ExecutionIdentityV7,
}

impl LegacySnapshotVersion {
    pub const fn version(self) -> u32 {
        match self {
            Self::V4 => 4,
            Self::V5 => 5,
            Self::PathOnlyV6 => 6,
            Self::ExecutionIdentityV7 => 7,
        }
    }
}

/// A current snapshot cannot be constructed without its compiler identity.
/// Keeping the schema here prevents a bare version from contradicting it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotProvenance {
    Current(CompilerProvenance),
    LegacyUnbound(LegacySnapshotVersion),
}

impl SnapshotProvenance {
    pub fn running() -> Result<Self, String> {
        CompilerProvenance::current().map(Self::Current)
    }

    pub const fn version(&self) -> u32 {
        match self {
            Self::Current(_) => super::SNAPSHOT_VERSION,
            Self::LegacyUnbound(version) => version.version(),
        }
    }

    pub fn require_current(&self, operation: &str) -> Result<&CompilerProvenance, String> {
        match self {
            Self::Current(identity) => Ok(identity),
            Self::LegacyUnbound(version) => Err(format!(
                "{operation} requires compiler-bound snapshot version {}; version {} is read-only unbound evidence",
                super::SNAPSHOT_VERSION, version.version()
            )),
        }
    }

    pub fn require_running(&self, operation: &str) -> Result<&CompilerProvenance, String> {
        let identity = self.require_current(operation)?;
        identity.require_running(operation)
    }

    pub(super) fn to_wire(&self) -> WireCompilerIdentity {
        match self {
            Self::Current(identity) => WireCompilerIdentity::Present(Some(identity.clone())),
            Self::LegacyUnbound(_) => WireCompilerIdentity::Absent,
        }
    }
}

/// Wire presence is retained only until the schema admission constructor.
/// Legacy data may omit the field, but may not smuggle it through as null.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) enum WireCompilerIdentity {
    #[default]
    Absent,
    Present(Option<CompilerProvenance>),
}

impl WireCompilerIdentity {
    pub(super) fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }

    pub(super) fn admit(
        self,
        artifact: super::SnapshotArtifactKind,
    ) -> Result<SnapshotProvenance, String> {
        use super::SnapshotArtifactKind;
        let legacy = match artifact {
            SnapshotArtifactKind::CurrentLilaV8 => {
                return match self {
                    Self::Present(Some(identity)) => Ok(SnapshotProvenance::Current(identity)),
                    Self::Absent => {
                        Err("current snapshot is missing mandatory compiler_identity".into())
                    }
                    Self::Present(None) => {
                        Err("current snapshot has null compiler_identity".into())
                    }
                };
            }
            SnapshotArtifactKind::LegacyV4 => LegacySnapshotVersion::V4,
            SnapshotArtifactKind::LegacyV5 => LegacySnapshotVersion::V5,
            SnapshotArtifactKind::LegacyPathOnlyV6 => LegacySnapshotVersion::PathOnlyV6,
            SnapshotArtifactKind::LegacyExecutionIdentityV7 => {
                LegacySnapshotVersion::ExecutionIdentityV7
            }
        };
        match self {
            Self::Absent => Ok(SnapshotProvenance::LegacyUnbound(legacy)),
            Self::Present(_) => Err(format!(
                "legacy snapshot version {} must omit compiler_identity",
                legacy.version()
            )),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompilerIdentityWire {
    source_fingerprint_scheme: String,
    source_fingerprint: String,
    source_revision: SourceRevisionWire,
    executable_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum SourceRevisionWire {
    GitCommit { commit: String },
    UnversionedArchive,
}

impl CompilerIdentityWire {
    fn from_identity(identity: &CompilerIdentity) -> Self {
        Self {
            source_fingerprint_scheme: CompilerIdentity::source_fingerprint_scheme().into(),
            source_fingerprint: identity.source_fingerprint().to_string(),
            source_revision: match identity.source_revision() {
                CompilerSourceRevision::GitCommit(commit) => SourceRevisionWire::GitCommit {
                    commit: commit.as_str().into(),
                },
                CompilerSourceRevision::UnversionedArchive => {
                    SourceRevisionWire::UnversionedArchive
                }
            },
            executable_sha256: identity.executable_sha256().to_string(),
        }
    }

    fn into_identity(self) -> Result<CompilerIdentity, String> {
        if self.source_fingerprint_scheme != CompilerIdentity::source_fingerprint_scheme() {
            return Err(format!(
                "unsupported compiler source fingerprint scheme {}",
                self.source_fingerprint_scheme
            ));
        }
        let source_revision = match self.source_revision {
            SourceRevisionWire::GitCommit { commit } => {
                CompilerSourceRevision::GitCommit(CompilerCommitId::parse(&commit)?)
            }
            SourceRevisionWire::UnversionedArchive => CompilerSourceRevision::UnversionedArchive,
        };
        Ok(CompilerIdentity::from_parts(
            CompilerDigest::parse(&self.source_fingerprint)?,
            source_revision,
            CompilerDigest::parse(&self.executable_sha256)?,
        ))
    }
}

impl Serialize for WireCompilerIdentity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Absent | Self::Present(None) => serializer.serialize_none(),
            Self::Present(Some(identity)) => identity.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for WireCompilerIdentity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<CompilerProvenance>::deserialize(deserializer).map(Self::Present)
    }
}

impl Serialize for CompilerProvenance {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        CompilerIdentityWire::from_identity(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CompilerProvenance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CompilerIdentityWire::deserialize(deserializer)?
            .into_identity()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> CompilerProvenance {
        CompilerProvenance(CompilerIdentity::from_parts(
            CompilerDigest::parse(&"1".repeat(64)).unwrap(),
            CompilerSourceRevision::GitCommit(CompilerCommitId::parse(&"2".repeat(40)).unwrap()),
            CompilerDigest::parse(&"3".repeat(64)).unwrap(),
        ))
    }

    #[test]
    fn current_wire_requires_complete_canonical_identity_and_legacy_rejects_any_presence() {
        use super::super::SnapshotArtifactKind as Kind;
        for wire in [
            WireCompilerIdentity::Absent,
            WireCompilerIdentity::Present(None),
        ] {
            assert!(wire.admit(Kind::CurrentLilaV8).is_err());
        }
        for kind in [
            Kind::LegacyV4,
            Kind::LegacyV5,
            Kind::LegacyPathOnlyV6,
            Kind::LegacyExecutionIdentityV7,
        ] {
            assert!(WireCompilerIdentity::Present(None).admit(kind).is_err());
            assert!(WireCompilerIdentity::Present(Some(identity()))
                .admit(kind)
                .is_err());
            assert!(matches!(
                WireCompilerIdentity::Absent.admit(kind),
                Ok(SnapshotProvenance::LegacyUnbound(_))
            ));
        }
        let wire = WireCompilerIdentity::Present(Some(identity()));
        let encoded = serde_json::to_value(&wire).unwrap();
        let decoded: WireCompilerIdentity = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(wire, decoded);
        for key in [
            "source_fingerprint_scheme",
            "source_fingerprint",
            "source_revision",
            "executable_sha256",
        ] {
            let mut missing = encoded.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<WireCompilerIdentity>(missing).is_err(),
                "{key}"
            );
            let mut null = encoded.clone();
            null[key] = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<WireCompilerIdentity>(null).is_err(),
                "{key}"
            );
        }
    }

    #[test]
    fn decoded_evidence_keeps_its_producer_and_cannot_be_rebound_to_running_compiler() {
        use super::super::SnapshotArtifactKind as Kind;
        let running = CompilerIdentity::current().expect("the test compiler image is readable");
        assert!(
            SnapshotProvenance::Current(CompilerProvenance(running.clone()))
                .require_running("resume")
                .is_ok()
        );
        let producer = identity();
        assert_ne!(
            producer.identity(),
            running,
            "the mismatch witness differs from the real compiler"
        );
        let provenance = WireCompilerIdentity::Present(Some(producer.clone()))
            .admit(Kind::CurrentLilaV8)
            .unwrap();
        assert_eq!(provenance.require_current("compare").unwrap(), &producer);
        assert!(provenance.require_running("resume").is_err());
        assert!(
            SnapshotProvenance::LegacyUnbound(LegacySnapshotVersion::ExecutionIdentityV7)
                .require_current("publish")
                .is_err()
        );
    }
}
