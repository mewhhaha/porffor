//! Build-owned raw R packages use the same Wasm/layout admission as disk R.
//!
//! The package is a candidate, never authority for unchecked layout or native
//! code. Its source identity deliberately excludes the eventual executable.

use super::{
    cache, emit_runtime_artifact, CurrentRuntimeLayoutVersion, RuntimeArtifact,
    RuntimeArtifactCache, RuntimeArtifactKey, RuntimeSourceCompilationProfile, LAYOUT_VERSION,
};
use crate::EmitError;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const MAGIC: &[u8; 8] = b"LILARB01";
const DIGEST_BYTES: usize = 32;

/// An exact emitted runtime paired with its fully checked raw package. There
/// is no constructor accepting a caller-supplied layout or native image.
pub struct RuntimeBuildArtifact {
    runtime: Arc<RuntimeArtifact>,
    package: Vec<u8>,
}

impl RuntimeBuildArtifact {
    pub fn wasm(&self) -> &[u8] {
        self.runtime.bytes()
    }
    pub fn key(&self) -> RuntimeArtifactKey {
        self.runtime.key()
    }
    pub fn package_bytes(&self) -> &[u8] {
        &self.package
    }
}

/// Emit and round-trip one raw R for a build producer before it precompiles
/// these exact Wasm bytes. The normal executable-bound cache is not consulted.
pub fn build_runtime_artifact(
    selection: &lila_intl::IntlDataSelection,
    compiler_source: &[u8; DIGEST_BYTES],
) -> Result<RuntimeBuildArtifact, EmitError> {
    build_runtime_artifact_with_source_compilation_profile(
        selection,
        compiler_source,
        RuntimeSourceCompilationProfile::Portable,
    )
}

/// A native-image producer explicitly selects native source compilation.
/// Portable artifacts use the ordinary checked wrapper above.
pub fn build_runtime_artifact_with_source_compilation_profile(
    selection: &lila_intl::IntlDataSelection,
    compiler_source: &[u8; DIGEST_BYTES],
    source_compilation_profile: RuntimeSourceCompilationProfile,
) -> Result<RuntimeBuildArtifact, EmitError> {
    let identity = cache::RuntimeIdentity::admit(selection, source_compilation_profile)?;
    let runtime = emit_runtime_artifact(selection, source_compilation_profile)?;
    let package = encode(&runtime, &identity, compiler_source);
    let admitted = decode(&package, &identity, compiler_source).ok_or_else(|| {
        EmitError::unsupported(
            "compiler invariant: emitted build runtime failed its canonical raw admission",
        )
    })?;
    if admitted.key() != runtime.key() || admitted.layout() != runtime.layout() {
        return Err(EmitError::unsupported(
            "compiler invariant: build runtime round-trip changed its key or typed layout",
        ));
    }
    Ok(RuntimeBuildArtifact { runtime, package })
}

#[derive(Clone, Copy)]
struct Candidate<'a> {
    bytes: &'a [u8],
    expected_source: &'a [u8; DIGEST_BYTES],
}

/// Optional inputs to the one lazy runtime resolver. A package can be supplied
/// by any caller: full identity/Wasm/layout admission precedes every use.
#[derive(Clone, Copy)]
pub struct RuntimeArtifactInputs<'a> {
    cache: Option<&'a dyn RuntimeArtifactCache>,
    package: Option<Candidate<'a>>,
    source_compilation_profile: RuntimeSourceCompilationProfile,
    admission_jobs: std::num::NonZeroUsize,
}

impl<'a> RuntimeArtifactInputs<'a> {
    pub const fn new(cache: Option<&'a dyn RuntimeArtifactCache>) -> Self {
        Self {
            cache,
            package: None,
            source_compilation_profile: RuntimeSourceCompilationProfile::Portable,
            admission_jobs: std::num::NonZeroUsize::MIN,
        }
    }

    pub const fn with_source_compilation_profile(
        mut self,
        profile: RuntimeSourceCompilationProfile,
    ) -> Self {
        self.source_compilation_profile = profile;
        self
    }

    pub const fn source_compilation_profile(self) -> RuntimeSourceCompilationProfile {
        self.source_compilation_profile
    }

    /// Bound concurrent admission work by this explicit limit and the visible
    /// CPU capacity. Every identity, Wasm body and layout check still runs.
    pub const fn with_admission_jobs(mut self, jobs: std::num::NonZeroUsize) -> Self {
        self.admission_jobs = jobs;
        self
    }

    pub(super) const fn admission_jobs(self) -> std::num::NonZeroUsize {
        self.admission_jobs
    }

    pub const fn with_build_package(
        mut self,
        bytes: &'a [u8],
        expected_source: &'a [u8; DIGEST_BYTES],
    ) -> Self {
        self.package = Some(Candidate {
            bytes,
            expected_source,
        });
        self
    }

    pub(super) fn load_or_emit(
        self,
        identity: &cache::RuntimeIdentity,
        emit: impl FnOnce() -> Result<Arc<RuntimeArtifact>, EmitError>,
    ) -> Result<Arc<RuntimeArtifact>, EmitError> {
        if let Some(candidate) = self.package {
            let started = std::time::Instant::now();
            if let Some(runtime) = decode_with_jobs(
                candidate.bytes,
                identity,
                candidate.expected_source,
                self.admission_jobs,
            ) {
                if std::env::var_os("LILA_WASM_TRACE").is_some() {
                    eprintln!(
                        "lila wasm trace: runtime-build-package hit: {} bytes in {:?}",
                        runtime.bytes().len(),
                        started.elapsed()
                    );
                }
                return Ok(Arc::new(runtime));
            }
            if std::env::var_os("LILA_WASM_TRACE").is_some() {
                eprintln!("lila wasm trace: runtime-build-package miss: identity or layout rejected in {:?}", started.elapsed());
            }
        }
        if self.admission_jobs == std::num::NonZeroUsize::MIN {
            cache::load_or_emit(identity, self.cache, emit)
        } else {
            cache::load_or_emit_with_jobs(identity, self.cache, self.admission_jobs, emit)
        }
    }
}

fn encode(
    runtime: &RuntimeArtifact,
    identity: &cache::RuntimeIdentity,
    compiler_source: &[u8; DIGEST_BYTES],
) -> Vec<u8> {
    let mut package = Vec::with_capacity(runtime.bytes().len() + 168);
    package.extend_from_slice(MAGIC);
    package.extend_from_slice(&LAYOUT_VERSION.to_le_bytes());
    package.extend_from_slice(compiler_source);
    package.extend_from_slice(&identity.digest());
    package.extend_from_slice(runtime.key().as_bytes());
    cache::encode_payload(runtime, &mut package);
    let digest = Sha256::digest(&package);
    package.extend_from_slice(&digest);
    package
}

fn decode(
    package: &[u8],
    identity: &cache::RuntimeIdentity,
    expected_source: &[u8; DIGEST_BYTES],
) -> Option<RuntimeArtifact> {
    decode_with_jobs(
        package,
        identity,
        expected_source,
        std::num::NonZeroUsize::MIN,
    )
}

fn decode_with_jobs(
    package: &[u8],
    identity: &cache::RuntimeIdentity,
    expected_source: &[u8; DIGEST_BYTES],
    jobs: std::num::NonZeroUsize,
) -> Option<RuntimeArtifact> {
    let digest_offset = package.len().checked_sub(DIGEST_BYTES)?;
    let (framed, digest) = package.split_at(digest_offset);
    let mut payload = framed;
    // Reject unsupported source/profiles before hashing the large raw image.
    if &cache::take::<8>(&mut payload)? != MAGIC {
        return None;
    }
    let version =
        CurrentRuntimeLayoutVersion::admit(u32::from_le_bytes(cache::take(&mut payload)?))?;
    if &cache::take::<DIGEST_BYTES>(&mut payload)? != expected_source
        || cache::take::<DIGEST_BYTES>(&mut payload)? != identity.digest()
    {
        return None;
    }
    let expected_runtime = cache::take::<DIGEST_BYTES>(&mut payload)?;
    let actual_digest: [u8; DIGEST_BYTES] = Sha256::digest(framed).into();
    if actual_digest.as_slice() != digest {
        return None;
    }
    let runtime = cache::decode_payload(payload, identity, version, jobs)?;
    (runtime.key().as_bytes() == &expected_runtime).then_some(runtime)
}

#[cfg(test)]
mod tests;
