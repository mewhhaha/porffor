//! Native R authority is limited to this Cargo build's embedded output.
//!
//! Raw package bytes still pass AOT's complete admission. No native constructor
//! accepts a caller slice/path/cache blob, and no Module is retained here.

mod manifest;
use crate::wasmtime_config::{WasmNativeCompilationMode, CONFIGURATION_SCHEMA};
use lila_aot_wasm::{RuntimeArtifact, RuntimeArtifactCache, RuntimeArtifactInputs};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use wasmtime::{Engine, Module};

// One immutable allocation per build output. Const byte slices instead expand
// these large payloads into the compiler metadata at their individual uses.
static PACKAGE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lila-runtime-package.bin"));
static NATIVE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lila-runtime-native.bin"));
static MANIFEST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lila-runtime-manifest.bin"));

fn expected_source() -> &'static [u8; 32] {
    static SOURCE: OnceLock<[u8; 32]> = OnceLock::new();
    SOURCE.get_or_init(|| {
        manifest::source_identity(env!("LILA_COMPILER_FINGERPRINT"))
            .expect("Cargo supplied a SHA-256 compiler source identity")
    })
}

/// Construct only a lazy raw candidate. Scalar-only programs do not hash/admit
/// the package or native image and do not resolve any Intl component here.
pub(super) fn inputs(cache: Option<&dyn RuntimeArtifactCache>) -> RuntimeArtifactInputs<'_> {
    let inputs = RuntimeArtifactInputs::new(cache);
    if PACKAGE.is_empty() {
        inputs
    } else {
        inputs.with_build_package(PACKAGE, expected_source())
    }
}

struct EmbeddedNativeRuntime {
    manifest: manifest::Manifest,
}

impl EmbeddedNativeRuntime {
    fn admit() -> Option<Self> {
        Self::admit_manifest(manifest::Manifest::decode(MANIFEST)?)
    }

    fn admit_manifest(manifest: manifest::Manifest) -> Option<Self> {
        if &manifest.source != expected_source()
            || manifest.target != env!("LILA_RUNTIME_BUILD_TARGET")
            || manifest.configuration != CONFIGURATION_SCHEMA
            || manifest.modes != manifest::COMPILATION_MODES
            || manifest.package_digest != <[u8; 32]>::from(Sha256::digest(PACKAGE))
            || manifest.native_digest != <[u8; 32]>::from(Sha256::digest(NATIVE))
        {
            return None;
        }
        Some(Self { manifest })
    }

    fn matches(&self, runtime: &RuntimeArtifact, mode: WasmNativeCompilationMode) -> bool {
        self.manifest.modes.execution == mode
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
        // SAFETY: NATIVE is exclusively the immutable include_bytes output of
        // our build.rs, whose sole producer calls pinned Wasmtime's
        // precompile_module on RuntimeBuildArtifact::wasm(). That owner first
        // round-trips the exact raw R through full identity/layout validation.
        // The admitted manifest binds source, target, config, native bytes and
        // this actual RuntimeArtifact key. No external native bytes can reach
        // this call. deserialize owns its copied mapping in the supplied
        // execution Engine. The admitted modes explicitly allow optimized R
        // with Fast P; no separate build Engine survives or reaches the cache.
        // Wasmtime still checks version/features/collector/tunables/ISA.
        match unsafe { Module::deserialize(engine, NATIVE) } {
            Ok(module) => {
                if std::env::var_os("LILA_WASM_TRACE").is_some() {
                    eprintln!(
                        "lila wasm trace: runtime-native-bundle hit: {} bytes (runtime: {}, execution: {})",
                        NATIVE.len(),
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
    // Unsupported modes and different raw R keys do not touch the large
    // embedded images. This cheap metadata check can only reject a candidate;
    // the private authority still validates all manifest fields and digests.
    let candidate = manifest::Manifest::decode(MANIFEST)?;
    if candidate.modes.execution != mode || runtime.key().as_bytes() != &candidate.runtime_key {
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
