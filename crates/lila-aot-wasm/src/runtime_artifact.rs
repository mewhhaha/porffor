//! The program-independent runtime module (R) of a linked Wasm artifact.
//!
//! R holds every builtin, runtime helper and compiler-owned datum. It is
//! byte-identical for every program compiled by one compiler build and Intl
//! data selection, so hosts compile it once and link each small program
//! module (P) against it. R exports every function and global it defines
//! under a name derived from its index; P imports each of them from
//! [`RUNTIME_IMPORT_NAMESPACE`] in index order, so the index spaces of R and P
//! together equal the one monolithic index space and P's bodies address
//! runtime functions and globals by the same numbers R uses.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Mutex, OnceLock};

use sha2::{Digest, Sha256};
use wasm_encoder::{EntityType, ExportKind, ExportSection, GlobalType, ImportSection};

use crate::data::{PoolBoundary, StringPool};
use crate::EmitError;

mod build_package;
mod cache;
pub use build_package::{build_runtime_artifact, RuntimeArtifactInputs, RuntimeBuildArtifact};
pub use cache::{RuntimeArtifactCache, RuntimeArtifactCacheKey};

/// Module namespace under which a program module imports runtime exports.
pub const RUNTIME_IMPORT_NAMESPACE: &str = "lila_runtime";

/// Bumped when the physical or semantic R/P contract changes.
// Version 3 admits whole-Completion tail calls, Identifier PutValue and native ByteArray allocation.
const LAYOUT_VERSION: u32 = 3;

fn function_export_name(index: u32) -> String {
    format!("f{index}")
}

fn global_export_name(index: u32) -> String {
    format!("g{index}")
}

/// Identity of one runtime module: its bytes, which a compiler build and an
/// Intl data selection fully determine. Equal keys mean byte-identical modules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeArtifactKey([u8; 32]);

impl RuntimeArtifactKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// What a program module must import from the runtime, and the data boundary
/// its own pool continues from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RuntimeLayout {
    imported_functions: u32,
    function_types: Vec<u32>,
    globals: Vec<GlobalType>,
    pool: PoolBoundary,
}

impl RuntimeLayout {
    pub(crate) fn new(
        imported_functions: u32,
        function_types: Vec<u32>,
        globals: Vec<GlobalType>,
        pool: PoolBoundary,
    ) -> Self {
        Self {
            imported_functions,
            function_types,
            globals,
            pool,
        }
    }

    /// A program's pool must continue exactly where the runtime's ended.
    pub(crate) fn require_pool(&self, pool: &StringPool) -> Result<(), EmitError> {
        if pool.compiler_owned_boundary() == self.pool {
            Ok(())
        } else {
            Err(EmitError::unsupported(
                "compiler invariant violated: the program's compiler-owned pool differs from the runtime module's",
            ))
        }
    }

    /// Declares every runtime function, then every runtime global, as imports
    /// in index order. `host_imports` is the count of host functions already
    /// imported, which the runtime's own index space begins after.
    pub(crate) fn emit_runtime_imports(
        &self,
        host_imports: u32,
        imports: &mut ImportSection,
    ) -> Result<(), EmitError> {
        if host_imports != self.imported_functions {
            return Err(EmitError::unsupported(
                "compiler invariant violated: the program imports a different host function set than the runtime",
            ));
        }
        for (position, type_index) in self.function_types.iter().enumerate() {
            let index = host_imports + position as u32;
            imports.import(
                RUNTIME_IMPORT_NAMESPACE,
                &function_export_name(index),
                EntityType::Function(*type_index),
            );
        }
        for (index, global_type) in self.globals.iter().enumerate() {
            imports.import(
                RUNTIME_IMPORT_NAMESPACE,
                &global_export_name(index as u32),
                EntityType::Global(*global_type),
            );
        }
        Ok(())
    }
}

/// Exports the runtime's functions and globals under their index names.
pub(crate) fn export_runtime_symbols(
    exports: &mut ExportSection,
    functions: Range<u32>,
    globals: u32,
) {
    for index in functions {
        exports.export(&function_export_name(index), ExportKind::Func, index);
    }
    for index in 0..globals {
        exports.export(&global_export_name(index), ExportKind::Global, index);
    }
}

/// Which module of a linked artifact an emission produces.
#[derive(Clone, Copy)]
pub(crate) enum ModuleKind<'r> {
    /// A program without a heap: one self-contained module.
    Standalone,
    /// The program-independent runtime module.
    Runtime,
    /// A program module that links against exactly this runtime.
    Program(&'r Arc<RuntimeArtifact>),
}

impl ModuleKind<'_> {
    pub(crate) const fn uses_heap(&self) -> bool {
        match self {
            Self::Standalone => false,
            Self::Runtime | Self::Program(_) => true,
        }
    }

    /// Whether this module defines the runtime half of the function space.
    pub(crate) const fn compiles_runtime(&self) -> bool {
        match self {
            Self::Runtime => true,
            Self::Standalone | Self::Program(_) => false,
        }
    }

    /// Whether this module defines `main` and the program's own bodies.
    pub(crate) const fn compiles_program(&self) -> bool {
        match self {
            Self::Standalone | Self::Program(_) => true,
            Self::Runtime => false,
        }
    }

    /// Whether the module defines the globals the host reads and carries the
    /// Intl data images.
    pub(crate) const fn owns_globals(&self) -> bool {
        match self {
            Self::Standalone | Self::Runtime => true,
            Self::Program(_) => false,
        }
    }
}

/// One emitted module and, for the runtime module, what programs import.
pub(crate) struct EmittedModule {
    pub(crate) artifact: crate::WasmArtifact,
    pub(crate) runtime_layout: Option<RuntimeLayout>,
}

pub struct RuntimeArtifact {
    key: RuntimeArtifactKey,
    bytes: Arc<[u8]>,
    layout: RuntimeLayout,
}

impl RuntimeArtifact {
    pub fn key(&self) -> RuntimeArtifactKey {
        self.key
    }

    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }

    pub(crate) fn layout(&self) -> &RuntimeLayout {
        &self.layout
    }
}

impl core::fmt::Debug for RuntimeArtifact {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RuntimeArtifact")
            .field("key", &self.key)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

impl PartialEq for RuntimeArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for RuntimeArtifact {}

type RuntimeSlot = Arc<OnceLock<Result<Arc<RuntimeArtifact>, EmitError>>>;

/// The runtime module for `selection`, emitted once per process and Intl data
/// selection.
pub fn runtime_artifact(
    selection: &lila_intl::IntlDataSelection,
) -> Result<Arc<RuntimeArtifact>, EmitError> {
    runtime_artifact_with_cache(selection, None)
}

/// Reuses a validated raw runtime artifact before falling back to emission.
/// Intl selection/admission always precedes either memory or disk cache lookup.
pub fn runtime_artifact_with_cache(
    selection: &lila_intl::IntlDataSelection,
    cache: Option<&dyn RuntimeArtifactCache>,
) -> Result<Arc<RuntimeArtifact>, EmitError> {
    runtime_artifact_with_inputs(selection, RuntimeArtifactInputs::new(cache))
}

/// Admit the current Intl selection before consulting either an untrusted raw
/// build-package candidate or the ordinary compiler/executable-bound cache.
pub fn runtime_artifact_with_inputs(
    selection: &lila_intl::IntlDataSelection,
    inputs: RuntimeArtifactInputs<'_>,
) -> Result<Arc<RuntimeArtifact>, EmitError> {
    static RUNTIMES: OnceLock<Mutex<HashMap<[u8; 32], RuntimeSlot>>> = OnceLock::new();
    let admission_started = std::time::Instant::now();
    let identity = cache::RuntimeIdentity::admit(selection)?;
    if std::env::var_os("LILA_WASM_TRACE").is_some() {
        eprintln!(
            "lila wasm trace: runtime-data admission: {:?}",
            admission_started.elapsed()
        );
    }
    let slot = Arc::clone(
        RUNTIMES
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(identity.digest())
            .or_default(),
    );
    slot.get_or_init(|| inputs.load_or_emit(&identity, || emit_runtime_artifact(selection)))
        .clone()
}

fn emit_runtime_artifact(
    selection: &lila_intl::IntlDataSelection,
) -> Result<Arc<RuntimeArtifact>, EmitError> {
    let parsed = lila_front::parse(";", lila_front::ParseOptions::script())
        .map_err(|_| EmitError::unsupported("the empty runtime script must parse"))?;
    let script = lila_ir::lower(&parsed)
        .script
        .ok_or_else(|| EmitError::unsupported("the empty runtime script must lower"))?;
    let module = crate::emit::emit_runtime_module(&script, selection)?;
    let layout = module
        .runtime_layout
        .ok_or_else(|| EmitError::unsupported("the runtime module must describe its layout"))?;
    let bytes: Arc<[u8]> = Arc::from(module.artifact.bytes);
    Ok(Arc::new(RuntimeArtifact {
        key: runtime_key(&bytes),
        bytes,
        layout,
    }))
}

fn runtime_key(bytes: &[u8]) -> RuntimeArtifactKey {
    let mut hasher = Sha256::new();
    hasher.update(LAYOUT_VERSION.to_le_bytes());
    hasher.update(bytes);
    RuntimeArtifactKey(hasher.finalize().into())
}
