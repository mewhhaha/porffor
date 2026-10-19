//! Native R authority is limited to this Cargo build's embedded output.
//!
//! Raw package bytes retain complete identity, metadata, data and layout
//! admission. Immutable Cargo output reuses the producer's full body validation.
//! No native constructor accepts a caller slice/path/cache blob, and no Module
//! is retained here.

#[cfg(test)]
#[path = "embedded_runtime/compile.rs"]
mod build_compile;
mod manifest;
use crate::wasmtime_config::{WasmNativeCompilationMode, CONFIGURATION_SCHEMA};
use lila_aot_wasm::{
    RuntimeArtifact, RuntimeArtifactCache, RuntimeArtifactInputs, RuntimeSourceCompilationProfile,
};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use wasmtime::{Engine, Module};

// The leaf crate owns the large initializers. Only non-inline accessor calls
// enter Engine metadata; source and native admission remain local here.
use lila_runtime_image::{manifest_bytes, native, package};

fn expected_source() -> &'static [u8; 32] {
    static SOURCE: OnceLock<[u8; 32]> = OnceLock::new();
    SOURCE.get_or_init(|| {
        manifest::source_identity(env!("LILA_COMPILER_FINGERPRINT"))
            .expect("Cargo supplied a SHA-256 compiler source identity")
    })
}

/// Construct only a lazy raw candidate. Scalar-only programs do not hash/admit
/// the package or native image and do not resolve any Intl component here.
pub(super) fn inputs(
    cache: Option<&dyn RuntimeArtifactCache>,
    source_compilation_profile: RuntimeSourceCompilationProfile,
) -> RuntimeArtifactInputs<'_> {
    let inputs = RuntimeArtifactInputs::new(cache)
        .with_source_compilation_profile(source_compilation_profile)
        .with_admission_jobs(
            std::num::NonZeroUsize::new(crate::configured_compilation_jobs().unwrap_or(1))
                .expect("explicit compilation jobs are positive"),
        );
    // The embedded native bundle is never a candidate for exported products.
    if source_compilation_profile == RuntimeSourceCompilationProfile::Portable {
        return inputs;
    }
    if package().is_empty() {
        inputs
    } else {
        // SAFETY: package() returns the immutable Cargo-owned array from
        // lila-runtime-image. Its sole producer round-trips these exact package
        // bytes through complete AOT body/layout validation before native
        // compilation and publication. expected_source() is immutable and
        // build-bound. No external slice, path or cache reaches this boundary.
        // AOT still checks source/profile/Intl identity, checksums, sections,
        // data and the reconstructed layout before using the candidate.
        unsafe { inputs.with_immutable_build_package(package(), expected_source()) }
    }
}

struct EmbeddedNativeRuntime {
    manifest: manifest::Manifest,
}

impl EmbeddedNativeRuntime {
    fn admit() -> Option<Self> {
        Self::admit_manifest(manifest::Manifest::decode(manifest_bytes())?)
    }

    fn admit_manifest(manifest: manifest::Manifest) -> Option<Self> {
        if &manifest.source != expected_source()
            || manifest.target != env!("LILA_RUNTIME_BUILD_TARGET")
            || manifest.configuration != CONFIGURATION_SCHEMA
            || manifest.modes != manifest::COMPILATION_MODES
            || manifest.package_digest != <[u8; 32]>::from(Sha256::digest(package()))
            || manifest.native_digest != <[u8; 32]>::from(Sha256::digest(native()))
        {
            return None;
        }
        Some(Self { manifest })
    }

    fn matches(&self, runtime: &RuntimeArtifact, mode: WasmNativeCompilationMode) -> bool {
        self.manifest.modes.execution.contains(mode)
            && runtime.key().as_bytes() == &self.manifest.runtime_key
    }

    fn load(
        &self,
        engine: &Engine,
        runtime: &RuntimeArtifact,
        mode: WasmNativeCompilationMode,
    ) -> Option<Module> {
        if !self.matches(runtime, mode) {
            return None;
        }
        // SAFETY: native() is exclusively the immutable Cargo output of
        // lila-runtime-image build.rs, whose sole producer compiles the freshly
        // admitted RuntimeBuildArtifact::wasm() with pinned Wasmtime. Native-host
        // builds use Wasmtime's exact-Wasm compilation cache and serialize the
        // resulting temporary Module; cross-target/cache-unavailable builds use
        // precompile_module. That owner first round-trips the exact raw R through
        // full identity/layout validation. Wasmtime owns cache compatibility.
        // The admitted manifest binds source, target, config, native bytes and
        // this actual RuntimeArtifact key. No caller-supplied native bytes reach
        // this call. deserialize owns its copied mapping in the supplied
        // execution Engine. The admitted modes explicitly allow optimized R
        // with Fast or SizeOptimized P; no build Engine survives in the cache.
        // Wasmtime still checks version/features/collector/tunables/ISA.
        match unsafe { Module::deserialize(engine, native()) } {
            Ok(module) => {
                if std::env::var_os("LILA_WASM_TRACE").is_some() {
                    eprintln!(
                        "lila wasm trace: runtime-native-bundle hit: {} bytes (runtime: {}, execution: {})",
                        native().len(),
                        self.manifest.modes.runtime.as_str(),
                        mode.as_str()
                    );
                }
                Some(module)
            }
            Err(error) => {
                if std::env::var_os("LILA_WASM_TRACE").is_some() {
                    eprintln!("lila wasm trace: runtime-native-bundle incompatible: {error:#}; compiling normally");
                }
                None
            }
        }
    }
}

/// Called only by the ordinary shared module-cache factory on an R miss or a
/// bypass. Every caller receives its own Wasmtime mapping; retained mappings
/// are owned/accounted solely by the existing bounded R/P cache. Its mode key
/// remains the execution Engine's mode, never the R image's optimization mode.
pub(super) fn load_native(
    engine: &Engine,
    runtime: &RuntimeArtifact,
    mode: WasmNativeCompilationMode,
) -> Option<Module> {
    // Unadmitted modes and different raw R keys do not touch the large
    // embedded images. This cheap metadata check can only reject a candidate;
    // the private authority still validates all manifest fields and digests.
    let candidate = manifest::Manifest::decode(manifest_bytes())?;
    if !candidate.modes.execution.contains(mode)
        || runtime.key().as_bytes() != &candidate.runtime_key
    {
        return None;
    }
    static BUNDLE: OnceLock<Option<EmbeddedNativeRuntime>> = OnceLock::new();
    BUNDLE
        .get_or_init(EmbeddedNativeRuntime::admit)
        .as_ref()?
        .load(engine, runtime, mode)
}

#[cfg(test)]
mod tests;
