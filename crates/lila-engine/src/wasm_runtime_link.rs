//! Engine side of the runtime/program module split.
//!
//! A heap-using program is a small program module `P` that imports every
//! export it needs from the program-independent runtime module `R`, under
//! [`RUNTIME_IMPORT_NAMESPACE`]. `R` is identical for every program with the
//! same Intl profile, so it is compiled once per process and native mode and
//! served from Wasmtime's module cache across processes. `R` and `P` must be
//! compiled by the same `wasmtime::Engine`, so `R` is always requested for the
//! native compilation mode that was chosen for the execution.

use super::*;
use lila_aot_wasm::{RuntimeArtifact, RuntimeArtifactKey};

/// What the engine carries from an emitted artifact to execution.
#[derive(Clone, Copy)]
pub(super) struct WasmProgramRef<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) runtime: Option<&'a Arc<RuntimeArtifact>>,
}

impl<'a> WasmProgramRef<'a> {
    /// A standalone module: no runtime module to link against.
    pub(super) fn standalone(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            runtime: None,
        }
    }

    pub(super) fn new(bytes: &'a [u8], runtime: Option<&'a Arc<RuntimeArtifact>>) -> Self {
        Self { bytes, runtime }
    }

    /// The module that owns the host imports, memory imports and Intl images.
    pub(super) fn host_bytes(&self) -> &'a [u8] {
        match self.runtime {
            Some(runtime) => runtime.bytes(),
            None => self.bytes,
        }
    }
}

/// The runtime reference an [`Artifact`] carries. Equality is by key: the key
/// fingerprints the compiler build and Intl profile that fix the bytes.
#[derive(Clone)]
pub struct LinkedRuntime(pub(super) Arc<RuntimeArtifact>);

impl LinkedRuntime {
    pub fn key(&self) -> RuntimeArtifactKey {
        self.0.key()
    }

    pub fn bytes(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl core::fmt::Debug for LinkedRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LinkedRuntime").field(&self.0.key()).finish()
    }
}

impl PartialEq for LinkedRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.0.key() == other.0.key()
    }
}

impl Eq for LinkedRuntime {}

// Program-bytes disk cache entries: a tag, then for linked programs the key of
// the runtime the program was emitted against, then the program bytes.
const ENTRY_STANDALONE: u8 = 1;
const ENTRY_LINKED: u8 = 2;
const KEY_BYTES: usize = 32;

pub(super) fn encode_cache_entry(program: WasmProgramRef<'_>) -> Vec<u8> {
    let mut entry = Vec::with_capacity(1 + KEY_BYTES + program.bytes.len());
    match program.runtime {
        Some(runtime) => {
            entry.push(ENTRY_LINKED);
            entry.extend_from_slice(runtime.key().as_bytes());
        }
        None => entry.push(ENTRY_STANDALONE),
    }
    entry.extend_from_slice(program.bytes);
    entry
}

/// Decodes a standalone entry. A linked entry is a miss here: the caller has no
/// way to name the runtime it was emitted against.
pub(super) fn decode_standalone_cache_entry(entry: &[u8]) -> Option<&[u8]> {
    match entry.split_first()? {
        (&ENTRY_STANDALONE, program) => Some(program),
        _ => None,
    }
}

/// Decodes an entry and reattaches the runtime for a linked program. An entry
/// recorded against a different runtime key than the current one is a miss.
pub(super) fn decode_cache_entry(
    entry: &[u8],
    intl_profile: &IntlCompilationProfile,
) -> Option<(Arc<[u8]>, Option<Arc<RuntimeArtifact>>)> {
    match entry.split_first()? {
        (&ENTRY_STANDALONE, program) => Some((Arc::from(program), None)),
        (&ENTRY_LINKED, rest) => {
            let (recorded, program) = rest.split_at_checked(KEY_BYTES)?;
            let selection = IntlDataSelection::new(intl_profile.clone());
            let runtime = lila_aot_wasm::runtime_artifact(&selection).ok()?;
            (runtime.key().as_bytes().as_slice() == recorded)
                .then(|| (Arc::from(program), Some(runtime)))
        }
        _ => None,
    }
}

type RuntimeModuleSlot = Arc<OnceLock<Result<WasmtimeModule, EngineError>>>;

/// Compiled runtime modules for this process. A `wasmtime::Module` belongs to
/// one `Engine`, and each native compilation mode has exactly one process-wide
/// engine, so (key, mode) identifies the module.
fn runtime_modules(
) -> &'static Mutex<HashMap<(RuntimeArtifactKey, WasmNativeCompilationMode), RuntimeModuleSlot>> {
    static MODULES: OnceLock<
        Mutex<HashMap<(RuntimeArtifactKey, WasmNativeCompilationMode), RuntimeModuleSlot>>,
    > = OnceLock::new();
    MODULES.get_or_init(Default::default)
}

/// `engine` must be the process-wide engine for `mode`. Compiles through
/// `compile_wasm_module`, so Wasmtime's on-disk module cache applies.
pub(super) fn runtime_wasm_module(
    engine: &WasmtimeEngine,
    runtime: &RuntimeArtifact,
    mode: WasmNativeCompilationMode,
) -> Result<WasmtimeModule, EngineError> {
    let slot = Arc::clone(
        runtime_modules()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry((runtime.key(), mode))
            .or_default(),
    );
    slot.get_or_init(|| compile_wasm_module(engine, runtime.bytes()))
        .clone()
}

/// The native mode the runtime alone requires, scanned once per runtime.
fn runtime_native_compilation_mode(runtime: &RuntimeArtifact) -> WasmNativeCompilationMode {
    static MODES: OnceLock<Mutex<HashMap<RuntimeArtifactKey, WasmNativeCompilationMode>>> =
        OnceLock::new();
    let mut modes = MODES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *modes
        .entry(runtime.key())
        .or_insert_with(|| plan_wasm_native_compilation(runtime.bytes()).mode)
}

/// Chooses the one native mode for both modules: the program's own plan, raised
/// to size-optimized when the runtime needs it.
pub(super) fn plan_native_compilation(program: WasmProgramRef<'_>) -> WasmNativeCompilationPlan {
    let mut plan = plan_wasm_native_compilation(program.bytes);
    if let Some(runtime) = program.runtime {
        if runtime_native_compilation_mode(runtime) == WasmNativeCompilationMode::SizeOptimized {
            plan.mode = WasmNativeCompilationMode::SizeOptimized;
        }
    }
    plan
}

/// Intl kernel of the module that carries the images, shared across runs of the
/// same runtime so the large module is not re-scanned per run.
pub(super) fn intl_kernel_for(
    program: WasmProgramRef<'_>,
) -> Result<Arc<IntlKernel<EmbeddedIntlProvider>>, EngineError> {
    let Some(runtime) = program.runtime else {
        return intl_data_images::kernel_for_artifact(program.bytes);
    };
    static KERNELS: OnceLock<
        Mutex<HashMap<RuntimeArtifactKey, Arc<IntlKernel<EmbeddedIntlProvider>>>>,
    > = OnceLock::new();
    let kernels = KERNELS.get_or_init(Default::default);
    if let Some(kernel) = kernels
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&runtime.key())
    {
        return Ok(Arc::clone(kernel));
    }
    let kernel = intl_data_images::kernel_for_artifact(runtime.bytes())?;
    Ok(Arc::clone(
        kernels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(runtime.key())
            .or_insert(kernel),
    ))
}

/// The program module, plus the runtime module when linked, both for `mode`.
pub(super) struct WasmModules {
    pub(super) program: WasmtimeModule,
    pub(super) runtime: Option<WasmtimeModule>,
    pub(super) memory_cache_outcome: WasmModuleMemoryCacheOutcome,
}

impl WasmModules {
    /// The module that owns the host imports and memory imports.
    pub(super) fn host(&self) -> &WasmtimeModule {
        self.runtime.as_ref().unwrap_or(&self.program)
    }
}

/// `bypass_retention` overrides `memory_cache_policy` (agent workers never
/// retain). Only the program module goes through the in-memory LRU.
pub(super) fn compile_modules(
    engine: &WasmtimeEngine,
    program: WasmProgramRef<'_>,
    memory_cache_policy: WasmModuleMemoryCachePolicy,
    bypass_retention: bool,
    mode: WasmNativeCompilationMode,
) -> Result<WasmModules, EngineError> {
    let runtime = program
        .runtime
        .map(|runtime| runtime_wasm_module(engine, runtime, mode))
        .transpose()?;
    let (module, memory_cache_outcome) = if bypass_retention {
        (
            compile_wasm_module(engine, program.bytes)?,
            WasmModuleMemoryCacheOutcome::Bypassed,
        )
    } else {
        wasm_module_for_execution(engine, program.bytes, memory_cache_policy, mode)?
    };
    Ok(WasmModules {
        program: module,
        runtime,
        memory_cache_outcome,
    })
}

/// Instantiates the runtime module when present, publishes its exports to the
/// program under [`lila_aot_wasm::RUNTIME_IMPORT_NAMESPACE`], then instantiates
/// the program. Returns (program, instance that owns diagnostics and roots).
pub(super) fn instantiate(
    linker: &mut WasmtimeLinker<WasmHostState>,
    store: &mut WasmtimeStore<WasmHostState>,
    modules: &WasmModules,
) -> Result<(wasmtime::Instance, wasmtime::Instance), EngineError> {
    let runtime = match &modules.runtime {
        Some(runtime_module) => {
            let runtime = linker
                .instantiate(&mut *store, runtime_module)
                .map_err(|err| {
                    EngineError::new(format!("wasmtime runtime instantiate failed: {err:#}"))
                })?;
            linker
                .instance(
                    &mut *store,
                    lila_aot_wasm::RUNTIME_IMPORT_NAMESPACE,
                    runtime,
                )
                .map_err(|err| EngineError::new(format!("wasmtime linker setup failed: {err}")))?;
            Some(runtime)
        }
        None => None,
    };
    let program = linker
        .instantiate(&mut *store, &modules.program)
        .map_err(|err| EngineError::new(format!("wasmtime instantiate failed: {err:#}")))?;
    Ok((program, runtime.unwrap_or(program)))
}
