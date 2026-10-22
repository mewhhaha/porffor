//! Per-Engine native admission and a transient private Cargo-byte lease.
use super::*;

impl EmbeddedNativeRuntime {
    pub(super) fn accepts_native_image(
        &self,
        image: &scriptw_runtime_image::NativeRuntimeBytes,
    ) -> bool {
        image.digest() == &self.manifest.native_digest
    }

    pub(super) fn load(
        &self,
        engine: &Engine,
        runtime: &RuntimeArtifact,
        mode: WasmNativeCompilationMode,
    ) -> Option<Module> {
        if !self.matches(runtime, mode) {
            return None;
        }
        // The weak lease shares only concurrent loads, never a mutable Store
        // or a Module. Its private constructor hashes the exact decoded Box;
        // the independent v4 manifest must admit that digest before deserialize.
        let acquiring = std::env::var_os("SCRIPTW_WASM_TRACE").map(|_| std::time::Instant::now());
        let lease = self.images.native()?;
        if !self.accepts_native_image(&lease) {
            return None;
        }
        if let Some(started) = acquiring {
            eprintln!(
                "scriptw wasm trace: runtime-native-bundle storage admission: {:?} (source: {:?})",
                started.elapsed(),
                self.images.profile
            );
        }
        let bytes = lease.bytes();
        // SAFETY: bytes are exclusively the immutable Cargo output of the
        // checked raw/native runtime producer. Each profile's exact decoded
        // digest is independently admitted above on every actual native load.
        // Source/target/config/modes/raw RuntimeArtifact key have
        // already matched. No caller slice/path/cache blob enters this seam.
        // Wasmtime47 Module::deserialize copies this slice into its owned mmap
        // (engine.rs load_code_bytes / MmapVec::from_slice_with_alignment).
        // deserialize_raw, which would borrow it, is deliberately not used.
        // Compatibility is checked by the actual destination Engine; no build
        // Engine or Module is cached here. Dropping the lease after this call
        // cannot invalidate the Module's separate Arc<CodeMemory> mapping.
        let started = std::env::var_os("SCRIPTW_WASM_TRACE").map(|_| std::time::Instant::now());
        let native_length = bytes.len();
        let loaded = unsafe { Module::deserialize(engine, bytes) };
        drop(lease);
        if let Some(started) = started {
            eprintln!(
                "scriptw wasm trace: runtime-native-bundle deserialization: {:?}",
                started.elapsed()
            );
        }
        match loaded {
            Ok(module) => {
                if std::env::var_os("SCRIPTW_WASM_TRACE").is_some() {
                    eprintln!(
                        "scriptw wasm trace: runtime-native-bundle hit: {} bytes (runtime: {}, execution: {}, source: {:?})",
                        native_length,
                        self.manifest.modes.runtime.as_str(),
                        mode.as_str(),
                        self.images.profile
                    );
                }
                Some(module)
            }
            Err(error) => {
                if std::env::var_os("SCRIPTW_WASM_TRACE").is_some() {
                    eprintln!("scriptw wasm trace: runtime-native-bundle incompatible: {error:#}; compiling normally");
                }
                None
            }
        }
    }
}
